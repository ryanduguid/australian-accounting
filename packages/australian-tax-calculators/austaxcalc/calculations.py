"""Explicitly bounded worksheets using Decimal and published official methods."""

from __future__ import annotations

from decimal import ROUND_HALF_UP, Decimal, localcontext
from typing import Any

from . import __version__

D = Decimal
SOURCE_CHECKED = "2026-09-10"
SOURCES = {
    "gst": "https://smallbusiness.taxsuperandyou.gov.au/goods-and-services-tax/fast-facts",
    "resident_tax": "https://www.legislation.gov.au/C2004A03348/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
    "capital_gains": "https://www.ato.gov.au/forms-and-instructions/capital-gains-tax-guide-2014/part-b-completing-the-capital-gains-section-of-your-tax-return/step-6-applying-current-year-capital-losses",
    "fbt": "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/calculating-your-fbt",
    "depreciation": "https://www.ato.gov.au/law/view/document?DocNum=0215000005&FullDocument=true&PiT=99991231235958",
    "quarterly_sg": "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/super-guarantee",
}
SCOPES = {
    "gst": "One ordinary taxable supply at 10%, already classified by the operator. "
           "Excludes mixed supplies, exemptions, margin schemes, adjustments, tax invoice "
           "rounding across line items, input-credit entitlement and BAS preparation.",
    "resident_tax": "Full-year Australian resident individual, ordinary taxable income only. "
           "Excludes minors' special rates, working holiday makers, offsets, Medicare levy, "
           "surcharge, HELP, PAYG credits and tax payable/refundable. Supply established "
           "whole-dollar taxable income; this is basic tax before offsets and levies.",
    "capital_gains": "Australian resident individual with established ordinary gains and "
           "eligible 50% discount gains. Losses apply to non-discount gains first, then "
           "discount gains. Excludes asset classification, cost bases, eligibility decisions, "
           "collectables, personal-use assets, indexation, exemptions, rollovers, foreign "
           "residency and small-business concessions. The result is a net gain, not CGT payable.",
    "fbt": "Ordinary taxable employer, established type 1 and type 2 taxable values for the "
           "year ended 31 March 2026. Excludes benefit valuation, exemptions, rebates, "
           "not-for-profit caps and return preparation. Rounds each type's grossed-up "
           "total to the nearest dollar, then the tax estimate to cents.",
    "depreciation": "First year only, ordinary tangible Division 40 asset first held on or "
           "after 10 May 2006. Established cost, effective life and taxable-use proportion. "
           "Days run from first use or installation ready for use. No second-element costs, "
           "disposals, changed life, pooling, immediate write-offs, special assets or Division "
           "43 capital works. Does not determine an effective life from an asset description.",
    "quarterly_sg": "One eligible employee, one employer and one complete quarter of 2025-26. "
           "Established ordinary time earnings and qualifying contributions for that quarter. "
           "Excludes eligibility and earnings classification, defined benefits, contribution "
           "caps, special certificates, salary-sacrifice offsets, award entitlements and SGC. "
           "Does not test timeliness. From July 2026 Payday rules require a separate calculation.",
}


def _money(value: Decimal) -> Decimal:
    if (not isinstance(value, Decimal) or not value.is_finite() or value < 0
            or value > D("1e12") or int(value.as_tuple().exponent) < -2):
        raise ValueError("Amounts must be finite non-negative Decimals, at most 2dp and 1e12.")
    return value


def _scope(confirmed: bool, year: str, supported: tuple[str, ...] = ("2025-26",)) -> None:
    if confirmed is not True:
        raise ValueError("Establish all stated scope conditions before calculating.")
    if year not in supported:
        raise ValueError(f"Unsupported period {year!r}; supported: {', '.join(supported)}.")


def _result(kind: str, period: str, amounts: dict[str, Decimal],
            rates: dict[str, str] | None = None) -> dict[str, Any]:
    with localcontext() as context:
        context.prec = 40
        rendered = {key: str(value.quantize(D("0.01"), rounding=ROUND_HALF_UP))
                    for key, value in amounts.items()}
    return {
        "ok": True, "engine": "australian-tax-calculators", "engine_version": __version__,
        "calculation": kind, "period": period, "amounts": rendered, "rates": rates or {},
        "scope": SCOPES[kind], "sources": [SOURCES[kind]], "source_checked": SOURCE_CHECKED,
        "warnings": [
            "Operator-confirmed facts and scope; eligibility is not independently verified.",
            "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
        ],
    }


def gst(amount: Decimal, gst_inclusive: bool, scope_confirmed: bool,
        year: str) -> dict[str, Any]:
    _scope(scope_confirmed, year)
    _money(amount)
    if type(gst_inclusive) is not bool:
        raise ValueError("gst_inclusive must be a boolean.")
    with localcontext() as context:
        context.prec = 40
        tax = amount / 11 if gst_inclusive else amount / 10
        exclusive = amount - tax if gst_inclusive else amount
        return _result("gst", year, {"gst": tax, "exclusive": exclusive,
                                     "inclusive": exclusive + tax}, {"gst_rate": "0.10"})


def resident_tax(taxable_income: Decimal, year: str, scope_confirmed: bool) -> dict[str, Any]:
    _scope(scope_confirmed, year, ("2024-25", "2025-26", "2026-27"))
    income = _money(taxable_income)
    if income != income.to_integral_value():
        raise ValueError("Supply established whole-dollar taxable income.")
    first = D("0.15") if year == "2026-27" else D("0.16")
    tax = D(0)
    for low, high, rate in [(18200, 45000, first), (45000, 135000, D("0.30")),
                            (135000, 190000, D("0.37")), (190000, 10**12, D("0.45"))]:
        tax += max(D(0), min(income, D(high)) - low) * rate
    return _result("resident_tax", year, {"taxable_income_used": income,
                                          "basic_income_tax": tax})


def capital_gains(other_gains: Decimal, discount_gains: Decimal, current_losses: Decimal,
                  prior_losses: Decimal, scope_confirmed: bool, year: str) -> dict[str, Any]:
    _scope(scope_confirmed, year)
    for value in (other_gains, discount_gains, current_losses, prior_losses):
        _money(value)
    losses = current_losses + prior_losses
    other_offset = min(other_gains, losses)
    discount_offset = min(discount_gains, losses - other_offset)
    discounted = (discount_gains - discount_offset) / 2
    return _result("capital_gains", year, {
        "losses_used": other_offset + discount_offset,
        "losses_remaining": losses - other_offset - discount_offset,
        "net_capital_gain": other_gains - other_offset + discounted,
        "discount_applied": discounted,
    }, {"discount": "0.50"})


def fbt(type_one_value: Decimal, type_two_value: Decimal, year_ended: int,
        scope_confirmed: bool) -> dict[str, Any]:
    _scope(scope_confirmed, str(year_ended), ("2026",))
    first = (_money(type_one_value) * D("2.0802")).quantize(D(1), rounding=ROUND_HALF_UP)
    second = (_money(type_two_value) * D("1.8868")).quantize(D(1), rounding=ROUND_HALF_UP)
    return _result("fbt", "year ended 31 March 2026", {
        "type_one_grossed_up": first, "type_two_grossed_up": second,
        "fbt_estimate": (first + second) * D("0.47"),
    }, {"type_one": "2.0802", "type_two": "1.8868", "fbt_rate": "0.47"})


def depreciation(cost: Decimal, effective_life: Decimal, days: int,
                 taxable_use: Decimal, method: str, scope_confirmed: bool,
                 year: str) -> dict[str, Any]:
    _scope(scope_confirmed, year)
    _money(cost)
    if (not isinstance(effective_life, Decimal) or not effective_life.is_finite()
            or not D("0.01") <= effective_life <= 1000):
        raise ValueError("effective_life must be a finite Decimal between 0.01 and 1000 years.")
    if (not isinstance(taxable_use, Decimal) or not taxable_use.is_finite()
            or not 0 <= taxable_use <= 1):
        raise ValueError("taxable_use must be a finite Decimal fraction from 0 to 1.")
    if type(days) is not int or not 0 <= days <= 365:
        raise ValueError("days must be an integer from 0 to 365 for 2025-26.")
    if method not in ("prime_cost", "diminishing_value"):
        raise ValueError("method must be prime_cost or diminishing_value.")
    with localcontext() as context:
        context.prec = 40
        factor = D(1) if method == "prime_cost" else D(2)
        decline = min(cost, cost * D(days) / 365 * factor / effective_life)
        return _result("depreciation", year, {"decline_in_value": decline,
            "deduction": decline * taxable_use, "closing_adjustable_value": cost - decline})


def quarterly_sg(ordinary_time_earnings: Decimal, qualifying_contributions: Decimal,
                 year: str, quarter: int, scope_confirmed: bool) -> dict[str, Any]:
    _scope(scope_confirmed, year)
    if type(quarter) is not int or not 1 <= quarter <= 4:
        raise ValueError("quarter must be 1 (Jul-Sep), 2, 3 or 4 (Apr-Jun).")
    earnings = min(_money(ordinary_time_earnings), D("62500"))
    paid = _money(qualifying_contributions)
    minimum = earnings * D("0.12")
    return _result("quarterly_sg", f"{year} Q{quarter}", {
        "earnings_used": earnings, "minimum_sg": minimum,
        "additional_contribution": max(D(0), minimum - paid),
    }, {"sg_rate": "0.12", "maximum_quarterly_base": "62500.00"})
