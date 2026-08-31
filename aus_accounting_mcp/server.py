"""Australian accounting MCP server.

Statutory tools are facades over reviewed delegated engines. SBR payloads are synthetic.
"""

from __future__ import annotations

from decimal import Decimal
from importlib.metadata import PackageNotFoundError, version
from typing import Any, Literal

from mcp.server.mcpserver import MCPServer

try:
    _VERSION = version("aus-accounting-mcp")
except PackageNotFoundError:  # running from a source tree without installation
    _VERSION = "0.0.0.dev0"

from .adapters.benchmarks import compare_figures, list_industries
from .adapters.div7a import get_benchmark_rate, review_loan
from .adapters.payday import review_contribution
from .fixtures.synthetic_sbr import (
    generate_synthetic_bas_payload,
    generate_synthetic_ctr_payload,
)
from .errors import InputError
from .money import parse_amount

mcp = MCPServer("aus-accounting-mcp", version=_VERSION)

DIV7A_SCOPE_REFUSAL = (
    "The reviewed div7a-loan-review engine covers s 109N loan terms, s 109N(2) "
    "benchmark rates and s 109E minimum yearly repayments for an operator-supplied "
    "amalgamated loan. It does not form amalgamated loans, classify repayments under "
    "s 109R, model unpaid present entitlements, distributable surplus, interposed "
    "entities, debt forgiveness or the Commissioner's discretion. Those matters "
    "remain refused."
)


@mcp.tool()
def list_ato_benchmark_industries(
    search: str | None = None,
    year: str | None = None,
) -> dict[str, Any]:
    """List ATO small-business benchmark industries from ato-benchmark-compare.

    Pass search to filter by name. year is an optional benchmark year such as
    2023-24; omit it to use the latest shipped dataset.
    """
    return list_industries(search=search, year=year)


@mcp.tool()
def get_ato_benchmarks(
    industry: str,
    turnover: str,
    other_income: str | None = None,
    cost_of_sales: str | None = None,
    cost_of_sales_labour: str | None = None,
    salary_wages: str | None = None,
    contractor_commission: str | None = None,
    associated_persons: str | None = None,
    rent: str | None = None,
    motor_vehicle: str | None = None,
    other_expense: str | None = None,
    w1: str | None = None,
    year: str | None = None,
) -> dict[str, Any]:
    """Compare bucket totals against ATO small-business benchmarks.

    Amounts are decimal strings. industry is an ATO business-type name
    (see list_ato_benchmark_industries). other_income is needed for any ratio:
    the ATO turnover rule reads it to choose the denominator, so without it
    every ratio is not_supplied. Pass 0 only where the operator established
    the business has no other income. This is a comparison, not a finding
    that anything is wrong. Uses ato-benchmark-compare.
    """
    return compare_figures(
        industry=industry,
        turnover=turnover,
        year=year,
        other_income=other_income,
        cost_of_sales=cost_of_sales,
        cost_of_sales_labour=cost_of_sales_labour,
        salary_wages=salary_wages,
        contractor_commission=contractor_commission,
        associated_persons=associated_persons,
        rent=rent,
        motor_vehicle=motor_vehicle,
        other_expense=other_expense,
        w1=w1,
    )


@mcp.tool()
def calc_payday_super_deadline(
    qe_day: str,
    sg_amount: str,
    as_at: str,
    remitted: str | None = None,
    received: str | None = None,
    employee_id: str = "mcp-1",
    first_to_fund: bool = False,
    out_of_cycle: bool = False,
    next_standard_qe_day: str | None = None,
    db_interest: bool = False,
) -> dict[str, Any]:
    """Review one contribution against payday-super-checker.

    qe_day is the qualifying-earnings (payday) date. as_at is required.
    received is fund receipt. remitted is the day money was sent. This tool
    does not invent clearing-house latency and cannot confirm LCR 2026/1
    transition allocation. Without a fund-receipt date the statutory test
    cannot return ON_TIME. Dates are ISO-8601. Amounts are decimal strings.
    """
    return review_contribution(
        qe_day=qe_day,
        sg_amount=sg_amount,
        as_at=as_at,
        remitted=remitted,
        received=received,
        employee_id=employee_id,
        first_to_fund=first_to_fund,
        out_of_cycle=out_of_cycle,
        next_standard_qe_day=next_standard_qe_day,
        db_interest=db_interest,
    )


@mcp.tool()
def get_div7a_benchmark_rate(
    year_of_income: str,
    response_detail: Literal["summary", "full"] = "summary",
) -> dict[str, Any]:
    """Return the reviewed s 109N(2) rate for a year, or UNKNOWN.

    Years use the YYYY-YY form, such as 2026-27. The delegated engine fails
    closed outside its reviewed frozen table and does not read the network.
    response_detail defaults to summary; pass full for the complete provenance
    and statutory trace.
    """
    return get_benchmark_rate(year_of_income, response_detail=response_detail)


@mcp.tool()
def review_div7a_loan(
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
    """Review one operator-supplied amalgamated Division 7A loan.

    The tool runs the s 109N gate and then the s 109E minimum yearly repayment.
    Unknown facts may be omitted or passed as null; they remain UNKNOWN and are
    never coerced to false or zero. Amounts and rates are decimal strings.
    response_detail defaults to summary; pass full for the complete engine audit.
    """
    return review_loan(
        year_of_income=year_of_income,
        year_loan_made=year_loan_made,
        written_agreement=written_agreement,
        terms_in_place_before_lodgment_day=terms_in_place_before_lodgment_day,
        maximum_term_years=maximum_term_years,
        secured_by_registered_mortgage_over_real_property=(
            secured_by_registered_mortgage_over_real_property
        ),
        security_coverage_at_first_made=security_coverage_at_first_made,
        interest_rate_for_years_after_year_loan_made=(
            interest_rate_for_years_after_year_loan_made
        ),
        amalgamated_loan_unpaid_at_end_of_previous_year=(
            amalgamated_loan_unpaid_at_end_of_previous_year
        ),
        remaining_term_years=remaining_term_years,
        payments_applied_during_the_year=payments_applied_during_the_year,
        loan_id=loan_id,
        response_detail=response_detail,
    )


@mcp.tool()
def refuse_div7a(
    borrower_name: str,
    lender_entity_name: str,
    loan_principal: str,
    start_fy: int = 2025,
    is_secured_25_year: bool = False,
) -> dict[str, Any]:
    """Refuse Division 7A matters outside the reviewed loan/MYR scope."""
    parse_amount(loan_principal, "loan_principal")
    del borrower_name, lender_entity_name, start_fy, is_secured_25_year
    return {
        "ok": False,
        "available": False,
        "reviewed_engine": True,
        "code": "ERR_POLICY_DIV7A_SCOPE_REFUSED",
        "reason": DIV7A_SCOPE_REFUSAL,
    }


@mcp.tool()
def generate_synthetic_sbr_fixture(
    form_type: str,
    entity_name: str = "Synthetix Pty Ltd",
    revenue_or_sales: str = "1000000.00",
) -> dict[str, Any]:
    """Generate a synthetic CTR or BAS fixture. Not a lodgment and not statutory advice."""
    amount = parse_amount(revenue_or_sales, "revenue_or_sales")
    kind = form_type.strip().upper()
    if kind == "CTR":
        return generate_synthetic_ctr_payload(
            company_name=entity_name,
            gross_revenue=amount,
            cost_of_sales=(amount * Decimal("0.4")).quantize(Decimal("0.01")),
            deductible_operating_expenses=(amount * Decimal("0.3")).quantize(Decimal("0.01")),
        )
    if kind == "BAS":
        return generate_synthetic_bas_payload(
            entity_name=entity_name,
            total_sales_g1=amount,
            capital_purchases_g10=Decimal("11000.00"),
            non_capital_purchases_g11=(amount * Decimal("0.4")).quantize(Decimal("0.01")),
        )
    raise InputError(f"Unknown form_type {form_type!r}. Supported: CTR, BAS.")


def run_stdio() -> None:
    """Run MCP server over stdio transport."""
    mcp.run(transport="stdio")


if __name__ == "__main__":
    run_stdio()
