"""Australian accounting MCP server.

Statutory tools are facades over reviewed delegated engines. SBR payloads are synthetic.
"""

from __future__ import annotations

from decimal import Decimal
from importlib.metadata import PackageNotFoundError, version
from typing import Annotated, Literal, cast

from mcp.server.mcpserver import MCPServer
from mcp_types import ToolAnnotations
from pydantic import Field

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
from .outputs import (
    BenchmarkComparison,
    Div7aRate,
    Div7aReview,
    IndustryList,
    PaydayReview,
    ScopeRefusal,
    SyntheticFixture,
)

SERVER_INSTRUCTIONS = """Australian accounting review tools operating on operator-supplied facts.
- Start with list_ato_benchmark_industries to select an industry, then use
  get_ato_benchmarks to compare supplied buckets with the bundled ATO dataset.
  Use search and limit=20 for concise discovery; continue with next_offset as
  offset while has_more is true. Keep search unchanged and use the returned
  benchmark_year as year on subsequent pages.
  Supply established other_income for a ratio denominator; omitted buckets are
  unknown, not zero. Comparisons are not findings of wrongdoing.
- Use calc_payday_super_deadline for one contribution, with an explicit as_at
  date. Remittance does not establish fund receipt. Do not infer receipt dates
  or clearing-house latency, or report ON_TIME without evidence of receipt.
- Use get_div7a_benchmark_rate for rate-only queries and review_div7a_loan for
  the reviewed s 109N/s 109E facts of one operator-supplied amalgamated loan.
  Use refuse_div7a for unsupported matters. Do not form amalgamated loans,
  classify s 109R payments, or invent eligibility, rates or missing facts.
- generate_synthetic_sbr_fixture is only for fabricated integration tests.
  Never use its CTR/BAS output as a real calculation or lodgment.
Money and rates use decimal strings; dates use YYYY-MM-DD and income years
YYYY-YY. Preserve UNKNOWN, REFUSED, not_supplied and null outcomes. ok=true means
execution succeeded, not that a review passed. For Division 7A, summary is the
default; request response_detail="full" when the full audit trail is needed.
Retain engine versions, source/review dates, citations, warnings and caveats.
Bundled data is not a live lookup. These tools do not access the network, write
records or lodge. Results are review aids, not advice or determinations; obtain
human review before consequential accounting action.
"""

mcp = MCPServer("aus-accounting-mcp", version=_VERSION, instructions=SERVER_INSTRUCTIONS)

# These tools read bundled data and return results in memory. They never lodge,
# write records or contact external services; installation is a separate step.
LOCAL_READ_ONLY = ToolAnnotations(
    read_only_hint=True,
    destructive_hint=False,
    idempotent_hint=True,
    open_world_hint=False,
)

DIV7A_SCOPE_REFUSAL = (
    "The reviewed div7a-loan-review engine covers s 109N loan terms, s 109N(2) "
    "benchmark rates and s 109E minimum yearly repayments for an operator-supplied "
    "amalgamated loan. It does not form amalgamated loans, classify repayments under "
    "s 109R, model unpaid present entitlements, distributable surplus, interposed "
    "entities, debt forgiveness or the Commissioner's discretion. Those matters "
    "remain refused."
)


@mcp.tool(annotations=LOCAL_READ_ONLY)
def list_ato_benchmark_industries(
    search: Annotated[
        str | None,
        Field(description=(
            'Optional case-insensitive industry-name search; omit to list all industries in the '
            'selected shipped dataset.'
        )),
    ] = None,
    year: Annotated[
        str | None,
        Field(description=(
            'Benchmark dataset year in YYYY-YY form, e.g. "2023-24". Omit or null selects the '
            'latest shipped dataset, not a live ATO lookup.'
        )),
    ] = None,
    *,
    limit: Annotated[
        int | None,
        Field(
            strict=True, ge=1, le=100,
            description=(
                'Maximum industries returned, 1 to 100; use 20 for concise discovery. '
                'Omit or null returns all remaining matches for compatibility.'
            ),
        ),
    ] = None,
    offset: Annotated[
        int,
        Field(
            strict=True, ge=0,
            description=(
                'Zero-based position in the filtered results; start at 0, then pass '
                'next_offset while has_more is true. Keep search and year unchanged.'
            ),
        ),
    ] = 0,
) -> IndustryList:
    """List ATO small-business benchmark industries from ato-benchmark-compare.

    Pass search to filter by name. year is an optional benchmark year such as
    2023-24; omit it to use the latest shipped dataset. Use the returned name
    with get_ato_benchmarks. Optional limit/offset page the filtered results in
    dataset order. count is the page size; total_count is all matching industries.
    Continue with next_offset and the returned benchmark_year, keeping search
    unchanged, until has_more is false. Omit limit to retain the full-list result.
    Reads bundled data locally; no network or writes.
    """
    return cast(IndustryList, list_industries(search=search, year=year, limit=limit, offset=offset))


@mcp.tool(annotations=LOCAL_READ_ONLY)
def get_ato_benchmarks(
    industry: Annotated[
        str,
        Field(description=(
            'ATO business-type name returned by list_ato_benchmark_industries. Select an '
            'industry before comparing figures.'
        )),
    ],
    turnover: Annotated[
        str,
        Field(description=(
            'Sales of goods and services, excluding other income. AUD decimal string, e.g. '
            '"1000.00"; finite, at most 2 decimal places, absolute value at most '
            '1000000000000.00.'
        )),
    ],
    other_income: Annotated[
        str | None,
        Field(description=(
            'Non-sales business income, e.g. interest or grants. Required to establish any '
            'ratio denominator. AUD decimal string, e.g. "1000.00"; finite, at most 2 decimal '
            'places, absolute value at most 1000000000000.00. Omit or null means not supplied; '
            'use "0.00" only for an established zero.'
        )),
    ] = None,
    cost_of_sales: Annotated[
        str | None,
        Field(description=(
            'Cost of sales excluding salary and wages; put that labour in cost_of_sales_labour. '
            'AUD decimal string, e.g. "1000.00"; finite, at most 2 decimal places, absolute '
            'value at most 1000000000000.00. Omit or null means not supplied; use "0.00" only '
            'for an established zero.'
        )),
    ] = None,
    cost_of_sales_labour: Annotated[
        str | None,
        Field(description=(
            'Salary and wages within cost of sales, excluding separately bucketed payments to '
            'associated persons. AUD decimal string, e.g. "1000.00"; finite, at most 2 decimal '
            'places, absolute value at most 1000000000000.00. Omit or null means not supplied; '
            'use "0.00" only for an established zero.'
        )),
    ] = None,
    salary_wages: Annotated[
        str | None,
        Field(description=(
            'Salary and wages outside cost of sales, excluding separately bucketed payments to '
            'associated persons. AUD decimal string, e.g. "1000.00"; finite, at most 2 decimal '
            'places, absolute value at most 1000000000000.00. Omit or null means not supplied; '
            'use "0.00" only for an established zero.'
        )),
    ] = None,
    contractor_commission: Annotated[
        str | None,
        Field(description=(
            'Contractor, subcontractor and commission expenses. AUD decimal string, e.g. '
            '"1000.00"; finite, at most 2 decimal places, absolute value at most '
            '1000000000000.00. Omit or null means not supplied; use "0.00" only for an '
            'established zero.'
        )),
    ] = None,
    associated_persons: Annotated[
        str | None,
        Field(description=(
            'Payments to associated persons, kept separate from salary/wage buckets to avoid '
            'double counting. Needed for labour comparison when w1 is supplied. AUD decimal '
            'string, e.g. "1000.00"; finite, at most 2 decimal places, absolute value at most '
            '1000000000000.00. Omit or null means not supplied; use "0.00" only for an '
            'established zero.'
        )),
    ] = None,
    rent: Annotated[
        str | None,
        Field(description=(
            'Business rent expenses for the comparison period. AUD decimal string, e.g. '
            '"1000.00"; finite, at most 2 decimal places, absolute value at most '
            '1000000000000.00. Omit or null means not supplied; use "0.00" only for an '
            'established zero.'
        )),
    ] = None,
    motor_vehicle: Annotated[
        str | None,
        Field(description=(
            'Business motor vehicle expenses for the comparison period. AUD decimal string, '
            'e.g. "1000.00"; finite, at most 2 decimal places, absolute value at most '
            '1000000000000.00. Omit or null means not supplied; use "0.00" only for an '
            'established zero.'
        )),
    ] = None,
    other_expense: Annotated[
        str | None,
        Field(description=(
            'Other expenses, including superannuation and depreciation; exclude amounts already '
            'in another bucket and income tax expense. AUD decimal string, e.g. "1000.00"; '
            'finite, at most 2 decimal places, absolute value at most 1000000000000.00. Omit or '
            'null means not supplied; use "0.00" only for an established zero.'
        )),
    ] = None,
    w1: Annotated[
        str | None,
        Field(description=(
            'Activity statement W1 total for the same period; used by the engine when greater '
            'than the reconstructed salary and wages label. Supply associated_persons too. AUD '
            'decimal string, e.g. "1000.00"; finite, at most 2 decimal places, absolute value '
            'at most 1000000000000.00. Omit or null means not supplied; use "0.00" only for an '
            'established zero.'
        )),
    ] = None,
    year: Annotated[
        str | None,
        Field(description=(
            'Benchmark dataset year in YYYY-YY form, e.g. "2023-24". Omit or null selects the '
            'latest shipped dataset, not a live ATO lookup.'
        )),
    ] = None,
) -> BenchmarkComparison:
    """Compare bucket totals against ATO small-business benchmarks.

    Amounts are decimal strings. industry is an ATO business-type name
    (see list_ato_benchmark_industries). other_income is needed for any ratio:
    the ATO turnover rule reads it to choose the denominator, so without it
    every ratio is not_supplied. Pass 0 only where the operator established
    the business has no other income. This is a comparison, not a finding
    that anything is wrong. Supply at least one expense bucket; omitted
    buckets remain not_supplied, never evidenced zeros. Returns ratios,
    ranges, source citations and warnings from ato-benchmark-compare.
    Runs locally with no network, writes or lodgments. Not tax advice.
    """
    return cast(
        BenchmarkComparison,
        compare_figures(
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
        ),
    )


@mcp.tool(annotations=LOCAL_READ_ONLY)
def calc_payday_super_deadline(
    qe_day: Annotated[
        str,
        Field(description=(
            'Qualifying-earnings payment date (payday), YYYY-MM-DD. This engine rejects dates '
            'before its Payday Super regime.'
        )),
    ],
    sg_amount: Annotated[
        str,
        Field(description=(
            'Superannuation guarantee contribution amount for this employee and '
            'qualifying-earnings payment. AUD decimal string, e.g. "1000.00"; finite, at most 2 '
            'decimal places, absolute value at most 1000000000000.00.'
        )),
    ],
    as_at: Annotated[
        str,
        Field(description=(
            'Required assessment date, YYYY-MM-DD. Supply explicitly; the tool does not assume '
            'today.'
        )),
    ],
    remitted: Annotated[
        str | None,
        Field(description=(
            'Date money was sent, YYYY-MM-DD. Optional; does not prove receipt by the fund or '
            'establish ON_TIME.'
        )),
    ] = None,
    received: Annotated[
        str | None,
        Field(description=(
            'Actual fund-receipt date, YYYY-MM-DD. Omit or null if unknown; required before the '
            'statutory test can return ON_TIME.'
        )),
    ] = None,
    employee_id: Annotated[
        str,
        Field(description=(
            'Operator reference echoed in the result; defaults to "mcp-1". No employee record '
            'is looked up or written.'
        )),
    ] = "mcp-1",
    first_to_fund: Annotated[
        bool,
        Field(description=(
            'Whether this is the first eligible contribution to this fund under the engine '
            'first-contribution rule. Defaults to false; establish eligibility before setting '
            'true.'
        )),
    ] = False,
    out_of_cycle: Annotated[
        bool,
        Field(description=(
            'Whether the payment qualifies for the out-of-cycle pathway. Defaults to false; '
            'true requires next_standard_qe_day for an actual subsequent standard QE payment.'
        )),
    ] = False,
    next_standard_qe_day: Annotated[
        str | None,
        Field(description=(
            'Subsequent schedule-consistent non-out-of-cycle QE payment date, YYYY-MM-DD; must '
            'be after qe_day when out_of_cycle is true. Not an assumed future payday.'
        )),
    ] = None,
    db_interest: Annotated[
        bool,
        Field(description=(
            'Whether this is a defined-benefit interest. Defaults to false; true selects the '
            'engine pathway that skips lateness testing.'
        )),
    ] = False,
) -> PaydayReview:
    """Review one contribution against payday-super-checker.

    qe_day is the qualifying-earnings (payday) date. as_at is required.
    received is fund receipt. remitted is the day money was sent. This tool
    does not invent clearing-house latency and cannot confirm LCR 2026/1
    transition allocation. Without a fund-receipt date the statutory test
    cannot return ON_TIME. Returns a deadline, pathway, verdict and caveats;
    experimental review only, not a compliance determination. Runs locally
    without network access, remitting contributions or changing records.
    """
    return cast(
        PaydayReview,
        review_contribution(
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
        ),
    )


@mcp.tool(annotations=LOCAL_READ_ONLY)
def get_div7a_benchmark_rate(
    year_of_income: Annotated[
        str,
        Field(description=(
            'Income year being reviewed, YYYY-YY, e.g. "2025-26". Years outside the engine '
            'reviewed rate table return UNKNOWN.'
        )),
    ],
    response_detail: Annotated[
        Literal["summary", "full"],
        Field(description=(
            '"summary" (default) returns the concise result; "full" includes the complete '
            'engine audit and provenance.'
        )),
    ] = "summary",
) -> Div7aRate:
    """Return the reviewed s 109N(2) rate for a year, or UNKNOWN.

    Years use the YYYY-YY form, such as 2026-27. The delegated engine fails
    closed outside its reviewed frozen table and does not read the network.
    response_detail defaults to summary; pass full for the complete provenance
    and statutory trace. Use review_div7a_loan to review supplied loan facts.
    Returns cited data without writes; not advice or a live rate lookup.
    """
    return cast(Div7aRate, get_benchmark_rate(year_of_income, response_detail=response_detail))


@mcp.tool(annotations=LOCAL_READ_ONLY)
def review_div7a_loan(
    year_of_income: Annotated[
        str,
        Field(description=(
            'Income year being reviewed, YYYY-YY, e.g. "2025-26". Years outside the engine '
            'reviewed rate table return UNKNOWN.'
        )),
    ],
    year_loan_made: Annotated[
        str | None,
        Field(description=(
            'Income year the loan was made, YYYY-YY, e.g. "2024-25". Omit or null if unknown.'
        )),
    ] = None,
    written_agreement: Annotated[
        bool | None,
        Field(description=(
            'Whether the loan agreement is in writing. Omit or null means UNKNOWN, not false.'
        )),
    ] = None,
    terms_in_place_before_lodgment_day: Annotated[
        bool | None,
        Field(description=(
            'Operator assertion that terms were in place before the relevant lodgment day. The '
            'engine does not compute that day. Omit or null if unknown.'
        )),
    ] = None,
    maximum_term_years: Annotated[
        str | None,
        Field(description=(
            'Actual agreed loan term in years as a decimal string, e.g. "7"; not the statutory '
            'maximum. Omit or null if unknown.'
        )),
    ] = None,
    secured_by_registered_mortgage_over_real_property: Annotated[
        bool | None,
        Field(description=(
            'Whether the loan has a registered mortgage over real property. Omit or null means '
            'UNKNOWN, not false.'
        )),
    ] = None,
    security_coverage_at_first_made: Annotated[
        str | None,
        Field(description=(
            'Property market value less prior secured liabilities, divided by the loan at '
            'inception; decimal ratio, e.g. "1.10" means 110%. Omit or null if unknown.'
        )),
    ] = None,
    interest_rate_for_years_after_year_loan_made: Annotated[
        str | None,
        Field(description=(
            'Agreed interest rate as a decimal fraction, e.g. "0.08" means 8%, not "8". Omit or '
            'null if unknown; do not assume a current benchmark rate.'
        )),
    ] = None,
    amalgamated_loan_unpaid_at_end_of_previous_year: Annotated[
        str | None,
        Field(description=(
            'Operator-established amalgamated-loan balance at the end of the preceding income '
            'year. The tool does not form amalgamated loans. AUD decimal string, e.g. '
            '"1000.00"; finite, at most 2 decimal places, absolute value at most '
            '1000000000000.00. Omit or null means not supplied; use "0.00" only for an '
            'established zero.'
        )),
    ] = None,
    remaining_term_years: Annotated[
        str | None,
        Field(description=(
            'Remaining statutory term for the supplied amalgamated loan, as a decimal string. '
            'The engine rounds fractional years up. Omit or null if unknown.'
        )),
    ] = None,
    payments_applied_during_the_year: Annotated[
        str | None,
        Field(description=(
            'Amount the operator establishes as applied during the income year. The tool does '
            'not classify payments under s 109R; omit if not established. AUD decimal string, '
            'e.g. "1000.00"; finite, at most 2 decimal places, absolute value at most '
            '1000000000000.00. Omit or null means not supplied; use "0.00" only for an '
            'established zero.'
        )),
    ] = None,
    loan_id: Annotated[
        str,
        Field(description=(
            'Operator loan reference echoed in the result; defaults to "mcp-div7a-1". No loan '
            'record is looked up or written.'
        )),
    ] = "mcp-div7a-1",
    response_detail: Annotated[
        Literal["summary", "full"],
        Field(description=(
            '"summary" (default) returns the concise result; "full" includes the complete '
            'engine audit and provenance.'
        )),
    ] = "summary",
) -> Div7aReview:
    """Review one operator-supplied amalgamated Division 7A loan.

    The tool runs the s 109N gate and then the s 109E minimum yearly repayment.
    Unknown facts may be omitted or passed as null; they remain UNKNOWN and are
    never coerced to false or zero. Amounts and rates are decimal strings.
    response_detail defaults to summary; pass full for the complete engine audit.
    Returns gate and repayment verdicts, reasons and caveats. Use
    get_div7a_benchmark_rate for rate-only lookups; unsupported matters remain
    refused by refuse_div7a. This tool does not form amalgamated loans or
    classify payments under s 109R. Runs locally with no network, writes or
    lodgments. Experimental review aid, not a tax determination or advice.
    """
    return cast(
        Div7aReview,
        review_loan(
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
        ),
    )


@mcp.tool(annotations=LOCAL_READ_ONLY)
def refuse_div7a(
    borrower_name: Annotated[
        str,
        Field(description=(
            'Legacy borrower label; ignored. This refusal tool does not look up a borrower or '
            'calculate a repayment.'
        )),
    ],
    lender_entity_name: Annotated[
        str,
        Field(description=(
            'Legacy lender label; ignored. No entity record is looked up or written.'
        )),
    ],
    loan_principal: Annotated[
        str,
        Field(description=(
            'Legacy principal, validated then discarded; this tool always refuses unsupported '
            'scope. AUD decimal string, e.g. "1000.00"; finite, at most 2 decimal places, '
            'absolute value at most 1000000000000.00.'
        )),
    ],
    start_fy: Annotated[
        int,
        Field(description=(
            'Legacy financial-year value; defaults to 2025 and is ignored. Use '
            'review_div7a_loan with explicit income years for supported reviews.'
        )),
    ] = 2025,
    is_secured_25_year: Annotated[
        bool,
        Field(description=(
            'Legacy secured-loan flag; defaults to false and is ignored. Does not establish '
            'eligibility or enable a calculation.'
        )),
    ] = False,
) -> ScopeRefusal:
    """Return an explicit refusal for unsupported Division 7A matters.

    Use review_div7a_loan for reviewed s 109N/s 109E loan facts, or
    get_div7a_benchmark_rate for a reviewed rate. This compatibility tool
    always returns ERR_POLICY_DIV7A_SCOPE_REFUSED with the scope explanation;
    it never calculates a repayment. Legacy inputs are ignored except for
    principal validation. No network, writes or lodgments.
    """
    parse_amount(loan_principal, "loan_principal")
    del borrower_name, lender_entity_name, start_fy, is_secured_25_year
    return {
        "ok": False,
        "available": False,
        "reviewed_engine": True,
        "code": "ERR_POLICY_DIV7A_SCOPE_REFUSED",
        "reason": DIV7A_SCOPE_REFUSAL,
    }


@mcp.tool(annotations=LOCAL_READ_ONLY)
def generate_synthetic_sbr_fixture(
    form_type: Annotated[
        str,
        Field(description=(
            'Fixture type: "CTR" (company tax return) or "BAS" (activity statement), '
            'case-insensitive. No other forms are supported.'
        )),
    ],
    entity_name: Annotated[
        str,
        Field(description=(
            'Fabricated entity label for test output; defaults to "Synthetix Pty Ltd". Do not '
            'supply real client data.'
        )),
    ] = "Synthetix Pty Ltd",
    revenue_or_sales: Annotated[
        str,
        Field(description=(
            'Fabricated gross revenue (CTR) or total sales G1 (BAS); defaults to "1000000.00". '
            'Other figures use fixed demonstration assumptions. AUD decimal string, e.g. '
            '"1000.00"; finite, at most 2 decimal places, absolute value at most '
            '1000000000000.00.'
        )),
    ] = "1000000.00",
) -> SyntheticFixture:
    """Generate fabricated CTR/BAS payloads for testing an agent integration.

    Use only with synthetic inputs. Fixed demonstration assumptions produce
    a payload marked synthetic=true and not_a_lodgment=true, not a real tax
    calculation or production SBR validation. Returns the fixture in memory;
    no file writes, network calls or lodgments. Not statutory advice.
    """
    amount = parse_amount(revenue_or_sales, "revenue_or_sales")
    kind = form_type.strip().upper()
    if kind == "CTR":
        return cast(
            SyntheticFixture,
            generate_synthetic_ctr_payload(
                company_name=entity_name,
                gross_revenue=amount,
                cost_of_sales=(amount * Decimal("0.4")).quantize(Decimal("0.01")),
                deductible_operating_expenses=(amount * Decimal("0.3")).quantize(Decimal("0.01")),
            ),
        )
    if kind == "BAS":
        return cast(
            SyntheticFixture,
            generate_synthetic_bas_payload(
                entity_name=entity_name,
                total_sales_g1=amount,
                capital_purchases_g10=Decimal("11000.00"),
                non_capital_purchases_g11=(amount * Decimal("0.4")).quantize(Decimal("0.01")),
            ),
        )
    raise InputError(f"Unknown form_type {form_type!r}. Supported: CTR, BAS.")


def run_stdio() -> None:
    """Run MCP server over stdio transport."""
    mcp.run(transport="stdio")


if __name__ == "__main__":
    run_stdio()
