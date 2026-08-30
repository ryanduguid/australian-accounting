"""Australian accounting MCP server.

Statutory tools are facades over payday-super-checker and ato-benchmark-compare.
Division 7A is refused until a reviewed engine exists. SBR payloads are synthetic.
"""

from __future__ import annotations

from decimal import Decimal
from importlib.metadata import PackageNotFoundError, version
from typing import Any

from mcp.server.mcpserver import MCPServer

try:
    _VERSION = version("aus-accounting-mcp")
except PackageNotFoundError:  # running from a source tree without installation
    _VERSION = "0.0.0.dev0"

from .adapters.benchmarks import compare_figures, list_industries
from .adapters.payday import review_contribution
from .fixtures.synthetic_sbr import (
    generate_synthetic_bas_payload,
    generate_synthetic_ctr_payload,
)
from .money import parse_amount

mcp = MCPServer("aus-accounting-mcp", version=_VERSION)

DIV7A_REFUSAL = (
    "Division 7A MYR, benchmark interest and franking-offset journals are not "
    "backed by a reviewed computational engine in this server. The previous "
    "MCP-local simulator has been removed so agents cannot treat it as statutory "
    "output. Wired engines: payday-super-checker and ato-benchmark-compare."
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
def refuse_div7a(
    borrower_name: str,
    lender_entity_name: str,
    loan_principal: str,
    start_fy: int = 2025,
    is_secured_25_year: bool = False,
) -> dict[str, Any]:
    """Refuse Division 7A calculations. No reviewed engine is wired."""
    parse_amount(loan_principal, "loan_principal")
    del borrower_name, lender_entity_name, start_fy, is_secured_25_year
    return {
        "ok": False,
        "available": False,
        "reviewed_engine": False,
        "code": "ERR_POLICY_DIV7A_REFUSED",
        "reason": DIV7A_REFUSAL,
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
    raise ValueError(f"Unknown form_type {form_type!r}. Supported: CTR, BAS.")


def run_stdio() -> None:
    """Run MCP server over stdio transport."""
    mcp.run(transport="stdio")


if __name__ == "__main__":
    run_stdio()
