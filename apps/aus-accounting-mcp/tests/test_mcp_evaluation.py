"""Verify the evaluation answer key through real MCP calls, without a model."""

import asyncio
from pathlib import Path
import sys
import xml.etree.ElementTree as ET

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
import pytest


QUESTIONS = ET.parse(Path(__file__).resolve().parents[1] / "evaluation" / "questions.xml")
CASES = [(pair.attrib["id"], pair.findtext("answer")) for pair in QUESTIONS.findall("qa_pair")]


async def _answer(session, case):
    async def call(name, **arguments):
        result = await session.call_tool(name, arguments)
        assert not result.is_error, result.content
        assert result.structured_content is not None
        return result.structured_content

    def ratio(result, name):
        return next(row["status"] for row in result["ratios"] if row["ratio"] == name)

    if case == "catalogue-pages":
        first = await call("list_ato_benchmark_industries", search="shop", year="2023-24", limit=2)
        second = await call(
            "list_ato_benchmark_industries", search="shop", year=first["benchmark_year"],
            limit=2, offset=first["next_offset"],
        )
        return str(len({item["name"] for item in first["industries"] + second["industries"]}))

    if case in {"missing-income", "established-zero-income", "missing-rent"}:
        found = await call("list_ato_benchmark_industries", search="baker", year="2023-24", limit=20)
        industry = next(
            item["name"] for item in found["industries"]
            if item["name"] == "Bakeries and hot bread shops"
        )
        figures = {
            "industry": industry, "year": "2023-24",
            "turnover": "850000.00", "cost_of_sales": "270000.00",
        }
        if case == "missing-rent":
            missing = await call("get_ato_benchmarks", **figures, other_income="0.00")
            supplied = await call(
                "get_ato_benchmarks", **figures, other_income="0.00", rent="40000.00"
            )
            assert ratio(supplied, "rent_to_turnover") != "not_supplied"
            return ratio(missing, "rent_to_turnover")
        missing = await call("get_ato_benchmarks", **figures)
        if case == "missing-income":
            return ratio(missing, "cost_of_sales_to_turnover")
        supplied = await call("get_ato_benchmarks", **figures, other_income="0.00")
        assert ratio(missing, "cost_of_sales_to_turnover") == "not_supplied"
        return ratio(supplied, "cost_of_sales_to_turnover")

    if case == "receipt-evidence":
        facts = {
            "qe_day": "2027-07-01", "sg_amount": "120.00",
            "remitted": "2027-07-02", "as_at": "2027-08-01",
        }
        missing = await call("calc_payday_super_deadline", **facts)
        supplied = await call("calc_payday_super_deadline", **facts, received="2027-07-02")
        assert supplied["result"]["verdict"] == "ON_TIME"
        return missing["result"]["verdict"]

    if case in {"unknown-rate", "audit-consistency"}:
        year = "2027-28" if case == "unknown-rate" else "2025-26"
        summary = await call("get_div7a_benchmark_rate", year_of_income=year)
        full = await call(
            "get_div7a_benchmark_rate", year_of_income=year, response_detail="full"
        )
        key = "verdict" if case == "unknown-rate" else "benchmark_rate"
        assert summary[key] == full[key]
        if case == "unknown-rate":
            assert summary["benchmark_rate"] is full["benchmark_rate"] is None
        else:
            assert full["statutory_trace"]
        return summary[key]

    if case in {"missing-loan-facts", "unsupported-scope"}:
        rate = await call("get_div7a_benchmark_rate", year_of_income="2025-26")
        assert rate["verdict"] == "KNOWN"
        if case == "missing-loan-facts":
            review = await call(
                "review_div7a_loan", year_of_income="2025-26",
                interest_rate_for_years_after_year_loan_made=rate["benchmark_rate"],
            )
            return review["gate"]["verdict"]
        refusal = await call(
            "refuse_div7a", borrower_name="Synthetic Borrower",
            lender_entity_name="Synthetic Lender", loan_principal="50000.00",
        )
        assert refusal["available"] is False
        return refusal["code"]

    assert case == "synthetic-boundary"
    fixtures = [
        await call("generate_synthetic_sbr_fixture", form_type=form,
                   entity_name="Synthetic Evaluation Pty Ltd", revenue_or_sales="110000.00")
        for form in ("BAS", "CTR")
    ]
    assert all(fixture["synthetic"] for fixture in fixtures)
    return str(all(fixture["not_a_lodgment"] for fixture in fixtures)).lower()


async def _evaluate(case):
    parameters = StdioServerParameters(command=sys.executable, args=["-m", "aus_accounting_mcp.cli"])
    async with stdio_client(parameters) as (reader, writer):
        async with ClientSession(reader, writer) as session:
            await session.initialize()
            return await _answer(session, case)


@pytest.mark.parametrize("case,expected", CASES, ids=[case for case, _ in CASES])
def test_evaluation_answer_is_reproducible(case, expected):
    assert asyncio.run(_evaluate(case)) == expected
