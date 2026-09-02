"""Thin adapter over div7a-loan-review.

The delegated engine owns the statutory tests, rates, arithmetic, refusals and
serialization. This module only translates MCP arguments and applies the
facade's money boundary.
"""

from __future__ import annotations

from typing import Any, Literal

from div7aloan import (
    LAW_COMPILATION,
    LAW_CONTENT_DATE,
    GateFacts,
    MyrFacts,
    __version__ as DIV7A_VERSION,
    benchmark_rate,
    complying_loan_gate,
    minimum_yearly_repayment,
    parse_year,
)

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.money import parse_optional_amount

DISCLAIMER = (
    "Experimental review aid. Not a Division 7A determination, an ATO assessment "
    "or professional advice. The operator supplies the statutory facts. Verify the "
    "result against the compiled Act and obtain human review before acting."
)


def _validate_response_detail(response_detail: str) -> None:
    if response_detail not in {"summary", "full"}:
        raise InputError("response_detail must be 'summary' or 'full'")


def get_benchmark_rate(
    year_of_income: str,
    response_detail: Literal["summary", "full"] = "summary",
) -> dict[str, Any]:
    _validate_response_detail(response_detail)
    try:
        result = benchmark_rate(year_of_income)
    except ValueError as exc:
        raise InputError(str(exc)) from exc
    payload = result.to_json_dict()
    payload.update(
        {
            "ok": True,
            "engine": "div7a-loan-review",
            "engine_version": DIV7A_VERSION,
            "law_content_date": LAW_CONTENT_DATE,
            "law_compilation": LAW_COMPILATION,
            "disclaimer": DISCLAIMER,
        }
    )
    if response_detail == "full":
        return payload
    return {
        **{key: payload[key] for key in (
            "ok",
            "engine",
            "engine_version",
            "law_content_date",
            "law_compilation",
            "disclaimer",
            "year_of_income",
            "verdict",
            "benchmark_rate",
            "reason",
        )},
        "response_detail": "summary",
        "source": {"verify_at": payload["provenance"]["verify_at"]},
    }


def review_loan(
    *,
    year_of_income: str,
    year_loan_made: str | None = None,
    written_agreement: bool | None = None,
    terms_in_place_before_lodgment_day: bool | None = None,
    maximum_term_years: str | None = None,
    secured_by_registered_mortgage_over_real_property: bool | None = None,
    security_coverage_at_first_made: str | None = None,
    interest_rate_for_years_after_year_loan_made: str | None = None,
    amalgamated_loan_unpaid_at_end_of_previous_year: str | None = None,
    remaining_term_years: str | None = None,
    payments_applied_during_the_year: str | None = None,
    loan_id: str = "mcp-div7a-1",
    response_detail: Literal["summary", "full"] = "summary",
) -> dict[str, Any]:
    _validate_response_detail(response_detail)
    try:
        gate_facts = GateFacts.from_mapping(
            {
                "loan_id": loan_id,
                "year_loan_made": year_loan_made,
                "written_agreement": written_agreement,
                "terms_in_place_before_lodgment_day": terms_in_place_before_lodgment_day,
                "maximum_term_years": maximum_term_years,
                "secured_by_registered_mortgage_over_real_property": (
                    secured_by_registered_mortgage_over_real_property
                ),
                "security_coverage_at_first_made": security_coverage_at_first_made,
                "interest_rate_for_years_after_year_loan_made": (
                    interest_rate_for_years_after_year_loan_made
                ),
            },
            "MCP review_div7a_loan",
        )
        gate = complying_loan_gate(gate_facts)
        year = parse_year(year_of_income)
        parsed_myr = MyrFacts.from_mapping(
            {
                "loan_id": loan_id,
                "year_loan_made": year_loan_made,
                "remaining_term_years": remaining_term_years,
            },
            year,
            gate,
            "MCP review_div7a_loan",
        )
        myr = minimum_yearly_repayment(
            MyrFacts(
                loan_id=parsed_myr.loan_id,
                year_of_income=parsed_myr.year_of_income,
                year_loan_made=parsed_myr.year_loan_made,
                gate_result=gate,
                remaining_term_years=parsed_myr.remaining_term_years,
                amalgamated_loan_unpaid_at_end_of_previous_year=parse_optional_amount(
                    amalgamated_loan_unpaid_at_end_of_previous_year,
                    "amalgamated_loan_unpaid_at_end_of_previous_year",
                ),
                payments_applied_during_the_year=parse_optional_amount(
                    payments_applied_during_the_year,
                    "payments_applied_during_the_year",
                ),
            )
        )
    except ValueError as exc:
        raise InputError(str(exc)) from exc

    gate_payload: dict[str, Any] = gate.to_json_dict()
    repayment_payload: dict[str, Any] = myr.to_json_dict()
    payload: dict[str, Any] = {
        "ok": True,
        "engine": "div7a-loan-review",
        "engine_version": DIV7A_VERSION,
        "law_content_date": LAW_CONTENT_DATE,
        "law_compilation": LAW_COMPILATION,
        "disclaimer": DISCLAIMER,
        "gate": gate_payload,
        "minimum_yearly_repayment": repayment_payload,
    }
    if response_detail == "full":
        return payload

    provenance = (
        gate_payload["benchmark_provenance"]
        or repayment_payload["benchmark_provenance"]
        or {}
    )
    return {
        **{key: payload[key] for key in (
            "ok",
            "engine",
            "engine_version",
            "law_content_date",
            "law_compilation",
            "disclaimer",
        )},
        "response_detail": "summary",
        "source": {"verify_at": provenance.get("verify_at")},
        "gate": {key: gate_payload[key] for key in (
            "verdict",
            "loan_id",
            "benchmark_year_used",
            "benchmark_rate",
            "maximum_term_years_allowed",
            "reasons",
            "caveats",
        )},
        "minimum_yearly_repayment": {key: repayment_payload[key] for key in (
            "verdict",
            "loan_id",
            "year_of_income",
            "gate_verdict",
            "benchmark_rate",
            "amalgamated_loan_unpaid_at_end_of_previous_year",
            "remaining_term_years_used",
            "myr_required",
            "payments_applied",
            "shortfall",
            "experimental_deemed_dividend_exposure",
            "rounding",
            "reasons",
            "caveats",
        )},
    }
