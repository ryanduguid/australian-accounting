"""Explicitly bounded worksheets using Decimal and published official methods."""

from __future__ import annotations

from datetime import date
from decimal import ROUND_DOWN, ROUND_HALF_UP, Decimal, localcontext
from typing import Any

from . import __version__
from .metadata import (
    CARRY_FORWARD_BALANCE_LIMIT,
    CONTRIBUTION_CAPS,
    EXAMPLES,
    INPUT_UNITS,
    LIBRARY_EVIDENCE,
    PAYG_WITHHOLDING_COEFFICIENTS,
    PENSION_MINIMUM_FACTORS,
    RESIDENT_TAX_SCALES,
    SOURCE_REVIEWS,
    SUPPORTED_PERIODS,
    periods,
)

D = Decimal
# Kept for consumers of the older catalogue. Results use the per-rule record.
SOURCE_CHECKED = "2026-09-10"
SOURCES = {
    "gst": "https://smallbusiness.taxsuperandyou.gov.au/goods-and-services-tax/fast-facts",
    "resident_tax": "https://www.legislation.gov.au/C2004A03348/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
    "capital_gains": "https://www.ato.gov.au/forms-and-instructions/capital-gains-tax-guide-2014/part-b-completing-the-capital-gains-section-of-your-tax-return/step-6-applying-current-year-capital-losses",
    "fbt": "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/calculating-your-fbt",
    "depreciation": "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/general-depreciation-rules-capital-allowances/prime-cost-straight-line-and-diminishing-value-methods",
    "quarterly_sg": "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/super-guarantee",
    "payg_withholding": "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-1-statement-of-formulas-for-calculating-amounts-to-be-withheld",
    "contribution_caps": "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/contributions-caps",
    "pension_minimum": "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/self-managed-super-funds-smsf/paying-smsf-benefits/income-stream-pension-rules-and-payments",
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
           "year ended 31 March 2026. Excludes benefit valuation, exemptions, rebates "
           "and not-for-profit caps. Retains gross-up precision for the estimate, then "
           "presents the amounts to cents. The return_item figures follow the FBT return "
           "instead: items 14A and 14B in whole dollars with cents dropped (the "
           "instructions show whole dollars but do not say whether cents are dropped or "
           "rounded), item 15 as their sum and item 16 at 47%, before any optional "
           "rounding down to 5 cents.",
    "depreciation": "First year only, ordinary tangible Division 40 asset first held on or "
           "after 10 May 2006. Established cost, effective life and taxable-use proportion. "
           "Days run from first use or installation ready for use. No second-element costs, "
           "disposals, changed life, pooling, immediate write-offs, special assets or Division "
           "43 capital works. Does not determine an effective life from an asset description.",
    "quarterly_sg": "One eligible employee, one employer and one complete quarter of 2025-26. "
           "Established ordinary time earnings and qualifying contributions for that quarter. "
           "Excludes eligibility and earnings classification, defined benefits, contribution "
           "caps, Norfolk Island transitional rates, special certificates, salary-sacrifice "
           "offsets, award entitlements and SGC. "
           "Does not test timeliness. From July 2026 Payday rules require a separate calculation.",
    "payg_withholding": "One regular weekly, fortnightly or monthly payment of salary or wages "
           "made from 1 July 2026 to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the "
           "operator established from the payee's declarations. Earnings include allowances "
           "subject to withholding. Excludes scale 4, tax offsets, Medicare levy adjustments, "
           "study and training support loans, extra amounts for 53 or 27 pays, quarterly and "
           "bi-monthly pays, back payments, bonuses, termination and other schedules.",
    "contribution_caps": "One individual's contributions received by all their funds in the "
           "income year, already classified by the operator as concessional or "
           "non-concessional, with no bring-forward period started in either of the 2 "
           "previous years. Total super balance is at the 30 June before the year; unused "
           "concessional cap is the established total of unexpired amounts from the previous "
           "5 years. Excludes classification, fund acceptance and work-test rules, "
           "deductibility, the CGT cap, downsizer and other excluded contributions, excess "
           "concessional amounts counting as non-concessional, determinations, release "
           "elections, Division 293 tax and the transfer balance cap.",
    "pension_minimum": "One account-based pension that started on or after 1 July 2007 and "
           "pays under SISR Schedule 7. Established account balance on 1 July, or on the "
           "commencement day in the first year, or the withdrawal benefit if higher, and the "
           "member's age on that day. Excludes pensions under the pre-2007 schedules, "
           "market-linked, lifetime and life expectancy pensions, the transition to "
           "retirement 10% maximum, commutations, death and reversion, and whether payments "
           "made meet the standard.",
}


def _money(value: Decimal) -> Decimal:
    if (not isinstance(value, Decimal) or not value.is_finite() or value < 0
            or value > D("1e12") or int(value.as_tuple().exponent) < -2):
        raise ValueError("Amounts must be finite non-negative Decimals, at most 2dp and 1e12.")
    return value


def worksheet_catalogue() -> dict[str, Any]:
    """Return independent JSON-ready discovery data for the supported worksheets.

    Example money uses decimal strings. Library callers convert those strings
    to Decimal. The example facts carry ``scope_confirmed: True`` so the
    fabricated example runs, and that confirmation is fabricated with the rest
    of the example: for real facts the flag is the operator's own confirmation
    of the scope conditions, not a field to copy from the example.
    """
    result: dict[str, Any] = {}
    for kind, description in SCOPES.items():
        supported = periods(kind)
        period_field = "year_ended" if kind == "fbt" else "year"
        result[kind] = {
            "scope": description, "source": SOURCES[kind],
            "source_checked": SOURCE_REVIEWS[kind]["checked"],
            "source_passage": SOURCE_REVIEWS[kind]["passage"],
            "example_evidence": {
                **LIBRARY_EVIDENCE[kind],
                "note": f"docs/calculation-evidence.md#{kind.replace('_', '-')}",
            },
            "supported_periods": supported,
            "required_inputs": {
                "kind": kind, "scope_confirmed": "boolean true; establish every scope condition",
                period_field: "integer" if kind == "fbt" else "income-year string",
                **INPUT_UNITS[kind],
            },
            "money_format": "Non-negative finite AUD decimal strings, at most 2 decimal places "
                            "and 1000000000000.00. Resident taxable income must be whole dollars.",
            "methods": ["prime_cost", "diminishing_value"] if kind == "depreciation" else [],
            "example": {
                "synthetic": True,
                "facts": {"kind": kind, "scope_confirmed": True,
                          **supported[0]["arguments"], **EXAMPLES[kind]},
            },
        }
    return result


def _scope(confirmed: bool, year: str, kind: str) -> None:
    if confirmed is not True:
        raise ValueError("Establish all stated scope conditions before calculating.")
    supported = SUPPORTED_PERIODS[kind]
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
        "scope": SCOPES[kind], "sources": [SOURCES[kind]],
        "source_checked": SOURCE_REVIEWS[kind]["checked"],
        "warnings": [
            "Operator-confirmed facts and scope; eligibility is not independently verified.",
            "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
        ],
    }


def gst(amount: Decimal, gst_inclusive: bool, scope_confirmed: bool,
        year: str) -> dict[str, Any]:
    _scope(scope_confirmed, year, "gst")
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
    _scope(scope_confirmed, year, "resident_tax")
    income = _money(taxable_income)
    if income != income.to_integral_value():
        raise ValueError("Supply established whole-dollar taxable income.")
    scale = RESIDENT_TAX_SCALES[year]
    # Every worksheet computes in its own context, as gst and depreciation
    # already did: a caller that had lowered decimal precision rounded these
    # intermediate amounts before _result could quantise them, and a
    # different tax figure came back with ok=true.
    with localcontext() as context:
        context.prec = 40
        tax = D(0)
        for low, high, rate in scale:
            ceiling = income if high is None else min(income, D(high))
            tax += max(D(0), ceiling - low) * D(rate)
        return _result("resident_tax", year, {"taxable_income_used": income,
                                              "basic_income_tax": tax},
                       {f"over_{low}": rate for low, _, rate in scale})


def capital_gains(other_gains: Decimal, discount_gains: Decimal, current_losses: Decimal,
                  prior_losses: Decimal, scope_confirmed: bool, year: str) -> dict[str, Any]:
    _scope(scope_confirmed, year, "capital_gains")
    for value in (other_gains, discount_gains, current_losses, prior_losses):
        _money(value)
    with localcontext() as context:
        context.prec = 40
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
    _scope(scope_confirmed, str(year_ended), "fbt")
    with localcontext() as context:
        context.prec = 40
        first = _money(type_one_value) * D("2.0802")
        second = _money(type_two_value) * D("1.8868")
        item_14a = first.quantize(D(1), rounding=ROUND_DOWN)
        item_14b = second.quantize(D(1), rounding=ROUND_DOWN)
        return _result("fbt", "year ended 31 March 2026", {
            "type_one_grossed_up": first, "type_two_grossed_up": second,
            "fbt_estimate": (first + second) * D("0.47"),
            "return_item_14a": item_14a, "return_item_14b": item_14b,
            "return_item_15": item_14a + item_14b,
            "return_item_16": (item_14a + item_14b) * D("0.47"),
        }, {"type_one": "2.0802", "type_two": "1.8868", "fbt_rate": "0.47"})


def depreciation(cost: Decimal, effective_life: Decimal, days: int,
                 taxable_use: Decimal, method: str, scope_confirmed: bool,
                 year: str) -> dict[str, Any]:
    _scope(scope_confirmed, year, "depreciation")
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
    _scope(scope_confirmed, year, "quarterly_sg")
    if type(quarter) is not int or not 1 <= quarter <= 4:
        raise ValueError("quarter must be 1 (Jul-Sep), 2, 3 or 4 (Apr-Jun).")
    with localcontext() as context:
        context.prec = 40
        earnings = min(_money(ordinary_time_earnings), D("62500"))
        paid = _money(qualifying_contributions)
        minimum = earnings * D("0.12")
        return _result("quarterly_sg", f"{year} Q{quarter}", {
            "earnings_used": earnings, "minimum_sg": minimum,
            "additional_contribution": max(D(0), minimum - paid),
        }, {"sg_rate": "0.12", "maximum_quarterly_base": "62500.00"})


def payg_withholding(earnings: Decimal, pay_period: str, scale: int, year: str,
                     scope_confirmed: bool) -> dict[str, Any]:
    """Schedule 1 formula y = a * x - b on the weekly equivalent of earnings."""
    _scope(scope_confirmed, year, "payg_withholding")
    _money(earnings)
    if pay_period not in ("weekly", "fortnightly", "monthly"):
        raise ValueError("pay_period must be weekly, fortnightly or monthly.")
    coefficients = PAYG_WITHHOLDING_COEFFICIENTS[year]
    if type(scale) is not int or scale not in coefficients:
        raise ValueError("scale must be 1, 2, 3, 5 or 6; scale 4 is not supported.")
    whole = D(1)
    with localcontext() as context:
        context.prec = 40
        if pay_period == "weekly":
            weekly = earnings
        elif pay_period == "fortnightly":
            weekly = earnings / 2
        else:
            # Schedule 1: a monthly amount ending in 33 cents gains a cent first.
            if earnings % 1 == D("0.33"):
                earnings += D("0.01")
            weekly = earnings * 3 / 13
        x = weekly.quantize(whole, rounding=ROUND_DOWN) + D("0.99")
        a, b = next((D(a), D(b)) for limit, a, b in coefficients[scale]
                    if limit is None or x < limit)
        # Rounded straight to the dollar, 50 cents up, with no cent rounding first.
        weekly_amount = max(D(0), (a * x - b).quantize(whole, rounding=ROUND_HALF_UP))
        amount = {
            "weekly": weekly_amount,
            "fortnightly": weekly_amount * 2,
            "monthly": (weekly_amount * 13 / 3).quantize(whole, rounding=ROUND_HALF_UP),
        }[pay_period]
        return _result("payg_withholding", f"{year} {pay_period}", {
            "weekly_earnings_used": x, "weekly_withholding": weekly_amount,
            "withholding": amount,
        }, {"scale": str(scale), "a": str(a), "b": str(b)})


def contribution_caps(total_super_balance: Decimal, concessional_contributions: Decimal,
                      unused_concessional_cap: Decimal, non_concessional_contributions: Decimal,
                      under_75_in_year: bool, year: str,
                      scope_confirmed: bool) -> dict[str, Any]:
    """Cap room and excess for one year's established contributions."""
    _scope(scope_confirmed, year, "contribution_caps")
    for value in (total_super_balance, concessional_contributions, unused_concessional_cap,
                  non_concessional_contributions):
        _money(value)
    if type(under_75_in_year) is not bool:
        raise ValueError("under_75_in_year must be a boolean.")
    concessional_cap, transfer_cap = (D(value) for value in CONTRIBUTION_CAPS[year])
    annual = concessional_cap * 4
    carried = unused_concessional_cap if total_super_balance < D(
        CARRY_FORWARD_BALANCE_LIMIT) else D(0)
    concessional_available = concessional_cap + carried
    # s 292-85: nil at or above the transfer balance cap; otherwise the balance
    # band sets how many annual caps the first year can bring forward.
    if total_super_balance >= transfer_cap:
        multiple = 0
    elif not under_75_in_year or total_super_balance >= transfer_cap - annual:
        multiple = 1
    elif total_super_balance >= transfer_cap - 2 * annual:
        multiple = 2
    else:
        multiple = 3
    available = annual * multiple
    triggered = multiple > 1 and non_concessional_contributions > annual
    return _result("contribution_caps", year, {
        "carry_forward_applied": carried,
        "concessional_available": concessional_available,
        "concessional_remaining": max(D(0), concessional_available - concessional_contributions),
        "excess_concessional": max(D(0), concessional_contributions - concessional_available),
        "non_concessional_available": available,
        "non_concessional_remaining": max(D(0), available - non_concessional_contributions),
        "excess_non_concessional": max(D(0), non_concessional_contributions - available),
    }, {
        "concessional_cap": str(concessional_cap), "non_concessional_cap": str(annual),
        "general_transfer_balance_cap": str(transfer_cap),
        "carry_forward_balance_limit": CARRY_FORWARD_BALANCE_LIMIT,
        "non_concessional_cap_multiple": str(multiple),
        "bring_forward_period_years": str(multiple if triggered else 0),
    })


def pension_minimum(account_balance: Decimal, age: int, days: int, year: str,
                    scope_confirmed: bool) -> dict[str, Any]:
    """SISR Schedule 7 minimum annual payment for one account-based pension."""
    _scope(scope_confirmed, year, "pension_minimum")
    _money(account_balance)
    if type(age) is not int or not 0 <= age <= 150:
        raise ValueError("age must be an integer number of years from 0 to 150.")
    start = int(year[:4])
    year_end = date(start + 1, 6, 30)
    days_in_year = (year_end - date(start, 7, 1)).days + 1
    if type(days) is not int or not 1 <= days <= days_in_year:
        raise ValueError(f"days must be an integer from 1 to {days_in_year} for {year}.")
    factor = next(D(rate) for lowest, rate in PENSION_MINIMUM_FACTORS if age >= lowest)
    commenced = date.fromordinal(year_end.toordinal() - days + 1)
    with localcontext() as context:
        context.prec = 40
        # cl 4: nothing is required for a pension commencing on or after 1 June.
        if commenced >= date(start + 1, 6, 1):
            exact = D(0)
        else:
            # cl 3 pro-rates the factor itself, so rounding happens once, under cl 5:
            # to the nearest $10, with an exact $5 rounding up.
            exact = account_balance * factor * days / days_in_year
        minimum = (exact / 10).quantize(D(1), rounding=ROUND_HALF_UP) * 10
        return _result("pension_minimum", year, {
            "account_balance_used": account_balance, "minimum_before_rounding": exact,
            "minimum_payment": minimum,
        }, {"percentage_factor": str(factor), "days": str(days),
            "days_in_year": str(days_in_year)})
