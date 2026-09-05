"""Check the schemas and lossless JSON that MCP clients actually receive."""

import asyncio
from copy import deepcopy
import json
import sys

from jsonschema import Draft202012Validator, FormatChecker, ValidationError
from pydantic import TypeAdapter, ValidationError as ModelValidationError
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
import pytest

from aus_accounting_mcp import server


LOAN = {
    "year_of_income": "2026-27",
    "year_loan_made": "2025-26",
    "written_agreement": True,
    "terms_in_place_before_lodgment_day": True,
    "maximum_term_years": "7",
    "secured_by_registered_mortgage_over_real_property": False,
    "interest_rate_for_years_after_year_loan_made": "0.0837",
    "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
    "remaining_term_years": "1",
    "payments_applied_during_the_year": "108770.00",
    "loan_id": "synthetic-schema-loan",
}
PAYDAY = {"qe_day": "2027-07-01", "sg_amount": "120.00", "as_at": "2027-08-01"}
BENCHMARK = {
    "industry": "Bakeries and hot bread shops",
    "turnover": "100000.00",
    "cost_of_sales": "35000.00",
}
REFUSAL = {
    "borrower_name": "Synthetic Borrower",
    "lender_entity_name": "Synthetic Lender",
    "loan_principal": "50000.00",
}
CASES = [
    ("list_ato_benchmark_industries", {}),
    ("list_ato_benchmark_industries", {"search": "synthetic-no-such-industry"}),
    ("get_ato_benchmarks", BENCHMARK),
    ("get_ato_benchmarks", {**BENCHMARK, "other_income": "0.00"}),
    ("get_ato_benchmarks", {**BENCHMARK, "other_income": "0.00", "turnover": "1000000000.00"}),
    ("calc_payday_super_deadline", PAYDAY),
    ("calc_payday_super_deadline", {**PAYDAY, "received": "2027-07-02"}),
    ("calc_payday_super_deadline", {**PAYDAY, "received": "2027-07-30"}),
    ("calc_payday_super_deadline", {**PAYDAY, "db_interest": True}),
    ("refuse_div7a", REFUSAL),
    ("generate_synthetic_sbr_fixture", {"form_type": "BAS"}),
    ("generate_synthetic_sbr_fixture", {"form_type": "CTR"}),
]
for detail in ("summary", "full"):
    for year in ("2025-26", "2027-28"):
        CASES.append(
            (
                "get_div7a_benchmark_rate",
                {
                    "year_of_income": year,
                    "response_detail": detail,
                },
            )
        )
    for overrides in (
        {},
        {"payments_applied_during_the_year": "100000.00"},
        {"written_agreement": None},
        {"written_agreement": False},
        {"remaining_term_years": None},
    ):
        CASES.append(("review_div7a_loan", {**LOAN, **overrides, "response_detail": detail}))

SCHEMAS = {tool.name: tool.output_schema for tool in asyncio.run(server.mcp.list_tools())}


@pytest.mark.parametrize("name,arguments", CASES)
def test_output_schema_validates_without_changing_the_tool_payload(name, arguments):
    schema = SCHEMAS[name]
    assert schema and (schema.get("properties") or schema.get("anyOf")), (
        f"{name} publishes no result fields"
    )
    Draft202012Validator.check_schema(schema)
    direct = getattr(server, name)(**arguments)
    result = asyncio.run(server.mcp.call_tool(name, arguments))
    assert not result.is_error
    assert result.structured_content == direct  # No dropped audit fields or inserted defaults.
    assert json.loads(result.content[0].text) == direct
    Draft202012Validator(schema).validate(result.structured_content)


@pytest.mark.parametrize(
    "name,arguments,path,bad_value",
    [
        ("get_div7a_benchmark_rate", {"year_of_income": "2027-28"}, ["benchmark_rate"], 0),
        ("review_div7a_loan", LOAN, ["minimum_yearly_repayment", "myr_required"], 108770),
        ("calc_payday_super_deadline", PAYDAY, ["result", "sg_amount"], 120),
        ("generate_synthetic_sbr_fixture", {"form_type": "BAS"}, ["synthetic"], False),
        ("refuse_div7a", REFUSAL, ["ok"], True),
    ],
)
def test_schema_rejects_misleading_money_and_safety_markers(name, arguments, path, bad_value):
    payload = deepcopy(getattr(server, name)(**arguments))
    target = payload
    for key in path[:-1]:
        target = target[key]
    target[path[-1]] = bad_value
    with pytest.raises(ValidationError):
        Draft202012Validator(SCHEMAS[name]).validate(payload)


@pytest.mark.parametrize(
    "name,arguments,path",
    [
        ("generate_synthetic_sbr_fixture", {"form_type": "CTR"}, ["income_statement"]),
        ("generate_synthetic_sbr_fixture", {"form_type": "BAS"}, ["gst_labels"]),
        ("get_div7a_benchmark_rate", {"year_of_income": "2025-26"}, ["source"]),
        (
            "get_div7a_benchmark_rate",
            {"year_of_income": "2025-26", "response_detail": "full"},
            ["provenance"],
        ),
        (
            "get_div7a_benchmark_rate",
            {"year_of_income": "2025-26", "response_detail": "full"},
            ["statutory_trace"],
        ),
        ("review_div7a_loan", LOAN, ["source"]),
        ("review_div7a_loan", {**LOAN, "response_detail": "full"}, ["gate", "limbs"]),
        (
            "review_div7a_loan",
            {**LOAN, "response_detail": "full"},
            ["minimum_yearly_repayment", "statutory_trace"],
        ),
    ],
)
def test_output_variants_require_their_sections_and_audit_fields(name, arguments, path):
    payload = deepcopy(getattr(server, name)(**arguments))
    target = payload
    for key in path[:-1]:
        target = target[key]
    del target[path[-1]]
    with pytest.raises(ValidationError):
        Draft202012Validator(SCHEMAS[name]).validate(payload)
    from typing import get_type_hints

    with pytest.raises(ModelValidationError):
        TypeAdapter(get_type_hints(getattr(server, name))["return"]).validate_python(payload)


@pytest.mark.parametrize("form,other_section", [("CTR", "gst_labels"), ("BAS", "reconciliation")])
def test_fixture_schema_rejects_sections_from_the_other_form(form, other_section):
    from aus_accounting_mcp.outputs import SyntheticFixture

    payload = server.generate_synthetic_sbr_fixture(form)
    payload[other_section] = {}
    with pytest.raises(ValidationError):
        Draft202012Validator(SCHEMAS["generate_synthetic_sbr_fixture"]).validate(payload)
    with pytest.raises(ModelValidationError):
        TypeAdapter(SyntheticFixture).validate_python(payload)


@pytest.mark.parametrize(
    "path,bad_value",
    [
        (["result", "sg_amount"], "abc"),
        (["result", "sg_amount"], "NaN"),
        (["as_at"], "not-a-date"),
        (["as_at"], "2027-13-01"),
        (["as_at"], "2027-02-30"),
        (["result", "received"], "not-a-date"),
    ],
)
def test_output_schema_rejects_malformed_decimal_and_date_text(path, bad_value):
    from aus_accounting_mcp.outputs import PaydayReview

    payload = server.calc_payday_super_deadline(**PAYDAY)
    target = payload
    for key in path[:-1]:
        target = target[key]
    target[path[-1]] = bad_value
    with pytest.raises(ValidationError):
        Draft202012Validator(
            SCHEMAS["calc_payday_super_deadline"],
            format_checker=FormatChecker(),
        ).validate(payload)
    with pytest.raises(ModelValidationError):
        TypeAdapter(PaydayReview).validate_python(payload)


def test_output_schema_rejects_malformed_income_year():
    from aus_accounting_mcp.outputs import Div7aRate

    payload = server.get_div7a_benchmark_rate("2025-26")
    payload["year_of_income"] = "2027"
    with pytest.raises(ValidationError):
        Draft202012Validator(SCHEMAS["get_div7a_benchmark_rate"]).validate(payload)
    with pytest.raises(ModelValidationError):
        TypeAdapter(Div7aRate).validate_python(payload)


@pytest.mark.parametrize(
    "name,arguments",
    [
        ("get_div7a_benchmark_rate", {"year_of_income": "2025-26", "response_detail": "full"}),
        ("review_div7a_loan", {**LOAN, "response_detail": "full"}),
        ("generate_synthetic_sbr_fixture", {"form_type": "CTR"}),
        ("generate_synthetic_sbr_fixture", {"form_type": "BAS"}),
    ],
)
def test_variant_validation_retains_unknown_extension_fields(name, arguments):
    from typing import get_type_hints

    payload = getattr(server, name)(**arguments)
    payload["future_engine_audit"] = {"reference": "synthetic", "missing": None}
    Draft202012Validator(SCHEMAS[name]).validate(payload)
    adapter = TypeAdapter(get_type_hints(getattr(server, name))["return"])
    assert adapter.dump_python(adapter.validate_python(payload), mode="json") == payload


async def _inspect_stdio():
    parameters = StdioServerParameters(
        command=sys.executable,
        args=["-m", "aus_accounting_mcp.cli"],
    )
    async with stdio_client(parameters) as (read_stream, write_stream):
        async with ClientSession(read_stream, write_stream) as session:
            initialized = await session.initialize()
            assert initialized.instructions
            tools = (await session.list_tools()).tools
            assert all(tool.name in initialized.instructions for tool in tools)
            assert all(
                tool.output_schema.get("properties") or tool.output_schema.get("anyOf")
                for tool in tools
            )
            result = await session.call_tool(
                "review_div7a_loan",
                {
                    **LOAN,
                    "written_agreement": None,
                    "response_detail": "full",
                },
            )
            assert not result.is_error
            assert result.structured_content["gate"]["verdict"] == "UNKNOWN"
            assert result.structured_content["minimum_yearly_repayment"]["myr_required"] is None
            assert result.structured_content["gate"]["statutory_trace"]
            # Expected input failures remain tool errors, not schema-validation crashes.
            for name, arguments, message in (
                ("get_ato_benchmarks", {**BENCHMARK, "turnover": "0.00"}, "turnover"),
                ("calc_payday_super_deadline", {**PAYDAY, "qe_day": "2026-06-15"}, "1 Jul 2026"),
                ("generate_synthetic_sbr_fixture", {"form_type": "INVALID"}, "Supported: CTR, BAS"),
            ):
                error = await session.call_tool(name, arguments)
                assert error.is_error
                assert error.structured_content is None
                assert message in error.content[0].text


def test_stdio_initialization_publishes_guidance_and_preserves_full_unknown_results():
    asyncio.run(_inspect_stdio())
