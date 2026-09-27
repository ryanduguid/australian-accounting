"""Period, input and example metadata owned by the worksheet engine."""

from typing import Any

# Resident basic tax scale per income year: (lower bound, upper bound, marginal rate)
# on taxable income above the lower bound, whole dollars; None is no upper bound.
# Income Tax Rates Act 1986, Schedule 7 Part I clause 1, as compiled for each year.
# A period is supported only when its scale is written here, so a new year never
# borrows another year's rates by falling through a default.
RESIDENT_TAX_SCALES: dict[str, tuple[tuple[int, int | None, str], ...]] = {
    "2024-25": ((18200, 45000, "0.16"), (45000, 135000, "0.30"),
                (135000, 190000, "0.37"), (190000, None, "0.45")),
    "2025-26": ((18200, 45000, "0.16"), (45000, 135000, "0.30"),
                (135000, 190000, "0.37"), (190000, None, "0.45")),
    "2026-27": ((18200, 45000, "0.15"), (45000, 135000, "0.30"),
                (135000, 190000, "0.37"), (190000, None, "0.45")),
}

# PAYG withholding coefficients per income year and scale: (weekly earnings
# upper limit, a, b) for y = a * x - b, where the row applies while weekly
# earnings x are below the limit; None is no upper limit. Transcribed from
# Schedule 1 (NAT 1004), "Coefficients to use in formulas for withholding from
# weekly payments", for payments made from 1 July 2026. The scale 2, 5 and 6
# rows at a = b = 0 are the ATO's nil rows below the tax-free threshold.
PAYG_WITHHOLDING_COEFFICIENTS: dict[str, dict[int, tuple[tuple[int | None, str, str], ...]]] = {
    "2026-27": {
        1: ((188, "0.1500", "0.1500"), (371, "0.2084", "11.0185"),
            (515, "0.1790", "0.1066"), (932, "0.3227", "74.1674"),
            (2246, "0.3200", "71.6508"), (3303, "0.3900", "228.8816"),
            (None, "0.4700", "493.1893")),
        2: ((362, "0", "0"), (538, "0.1500", "54.3462"), (673, "0.2500", "108.2135"),
            (721, "0.1700", "54.3473"), (865, "0.1790", "60.8377"),
            (1282, "0.3227", "185.1935"), (2596, "0.3200", "181.7319"),
            (3653, "0.3900", "363.4627"), (None, "0.4700", "655.7704")),
        3: ((2596, "0.3000", "0.3000"), (3653, "0.3700", "181.7308"),
            (None, "0.4500", "474.0385")),
        5: ((362, "0", "0"), (721, "0.1500", "54.3462"), (865, "0.1590", "60.8365"),
            (1282, "0.3027", "185.1923"), (2596, "0.3000", "181.7308"),
            (3653, "0.3700", "363.4615"), (None, "0.4500", "655.7692")),
        6: ((362, "0", "0"), (721, "0.1500", "54.3462"), (865, "0.1590", "60.8365"),
            (908, "0.3027", "185.1923"), (1135, "0.3527", "230.6135"),
            (1282, "0.3127", "185.1923"), (2596, "0.3100", "181.7308"),
            (3653, "0.3800", "363.4615"), (None, "0.4600", "655.7692")),
    },
}

# Per income year: (general concessional contributions cap, general transfer
# balance cap), whole dollars, from the ATO contributions caps and
# non-concessional contributions cap pages. The non-concessional cap is 4 times
# the concessional cap (ITAA 1997 s 292-85(2)) and the bring-forward bands are
# derived from both caps (s 292-85(5)), so neither is stored separately.
CONTRIBUTION_CAPS: dict[str, tuple[str, str]] = {
    "2024-25": ("30000", "1900000"),
    "2025-26": ("30000", "2000000"),
    "2026-27": ("32500", "2100000"),
}
# Study and training support loan compulsory repayment thresholds per income
# year, whole dollars, from the ATO page "Study and training loan repayment
# thresholds and rates" (marginal rates from 2025-26): the minimum repayment
# income, the top of the 15% band, the base amount the ATO prints for the 17%
# band, and the first repayment income at which the repayment is 10% of the
# whole. HESA 2003 ss 154-10 and 154-20 name the first two figures; the 2026-27
# amounts were notified by Gazette C2026G00249 on 20 April 2026. The printed
# base is the 15% band rounded to the dollar: 15% of ($129,717 - $69,528) is
# $9,028.35, which is what makes $186,051 the first income where 10% of the
# whole is the smaller amount. The worksheet computes the exact base and the
# crossover, and the tests hold both published figures to those derivations.
STUDY_LOAN_REPAYMENT_THRESHOLDS: dict[str, dict[str, str]] = {
    "2025-26": {"minimum": "67000", "step": "125000", "printed_base": "8700",
                "flat_from": "179286"},
    "2026-27": {"minimum": "69528", "step": "129717", "printed_base": "9028",
                "flat_from": "186051"},
}
STUDY_LOAN_REPAYMENT_RATES = {"lower": "0.15", "upper": "0.17", "flat": "0.10"}

# Unused concessional cap carries forward only below this total super balance
# on the previous 30 June (ITAA 1997 s 291-20(3)).
CARRY_FORWARD_BALANCE_LIMIT = "500000"

# SISR Schedule 7 percentage factors as (lowest age, factor), oldest band first.
# The 50% reductions ended with 2022-23; these apply from 2023-24.
PENSION_MINIMUM_FACTORS: tuple[tuple[int, str], ...] = (
    (95, "0.14"), (90, "0.11"), (85, "0.09"), (80, "0.07"),
    (75, "0.06"), (65, "0.05"), (0, "0.04"),
)

SUPPORTED_PERIODS = {
    "gst": ("2025-26",),
    "resident_tax": tuple(RESIDENT_TAX_SCALES),
    "capital_gains": ("2025-26",),
    "fbt": ("2026",),
    "depreciation": ("2025-26",),
    "quarterly_sg": ("2025-26",),
    "payg_withholding": tuple(PAYG_WITHHOLDING_COEFFICIENTS),
    "contribution_caps": tuple(CONTRIBUTION_CAPS),
    "pension_minimum": ("2024-25", "2025-26", "2026-27"),
    "study_loan_repayment": tuple(STUDY_LOAN_REPAYMENT_THRESHOLDS),
}

# Preserve the existing rule-review baseline. A Library example check does not
# revalidate every supported period. See docs/calculation-evidence.md.
SOURCE_REVIEWS = {
    "gst": {"checked": "2026-09-10", "passage": "GST fast facts: Charging GST"},
    "resident_tax": {
        "checked": "2026-09-10",
        "passage": "Income Tax Rates Act 1986, Schedule 7 Part I clause 1",
    },
    "capital_gains": {
        "checked": "2026-09-10",
        "passage": "Capital gains tax guide 2014, step 6: Applying current year capital losses",
    },
    "fbt": {
        "checked": "2026-09-27",
        "passage": "Calculating your FBT, steps 3 to 7 and the MantCo example; FBT return "
                   "2026 instructions, items 14A to 16",
    },
    "depreciation": {
        "checked": "2026-09-24",
        "passage": "Decline in value: diminishing value and prime cost methods; taxable use",
    },
    "quarterly_sg": {
        "checked": "2026-09-10",
        "passage": "Super guarantee, tables 21 and 24, 2025-26 rows",
    },
    "payg_withholding": {
        "checked": "2026-09-24",
        "passage": "Schedule 1 (NAT 1004) for payments from 1 July 2026: weekly coefficients, "
                   "using a formula, working out the weekly earnings",
    },
    "contribution_caps": {
        "checked": "2026-09-24",
        "passage": "Contributions caps, tables 1.1 and 4 and unused concessional cap carry "
                   "forward; Non-concessional contributions cap, bring-forward arrangement",
    },
    "pension_minimum": {
        "checked": "2026-09-24",
        "passage": "Income stream (pension) rules and payments: how to calculate the minimum "
                   "annual payment; SISR Schedule 7 clauses 1 to 5",
    },
    "study_loan_repayment": {
        "checked": "2026-09-27",
        "passage": "Study and training loan repayment thresholds and rates: tables 1 and 2, "
                   "repayment income, examples 1 to 3; Gazette C2026G00249 for the 2026-27 "
                   "minimum repayment income and replacement indexable amount",
    },
}

# "document" is the Library document id; each paragraph is an extract at
# _reference/<document id>/p<paragraph>-01.md. See docs/calculation-evidence.md.
LIBRARY_EVIDENCE = {
    "gst": {
        "document": "tax-examples-gst-and-other-indirect-taxes",
        "paragraphs": "12-020", "reviewed": "2026-06-30", "checked": "2026-09-15",
    },
    "resident_tax": {
        "document": "tax-examples-individuals", "paragraphs": "7-010",
        "reviewed": "2025-06-30", "checked": "2026-09-15",
    },
    "capital_gains": {
        "document": "tax-examples-capital-gains-tax-cgt", "paragraphs": "2-040, 2-240",
        "reviewed": "2026-06-30", "checked": "2026-09-15",
    },
    "fbt": {
        "document": "tax-examples-fringe-benefits-tax-fbt", "paragraphs": "3-000, 3-020",
        "reviewed": "2026-06-30", "checked": "2026-09-15",
    },
    "depreciation": {
        "document": "tax-examples-depreciation", "paragraphs": "6-000, 6-020",
        "reviewed": "2025-06-30", "checked": "2026-09-15",
    },
    "quarterly_sg": {
        "document": "superannuation-instant-reference-rates-thresholds-and-checklists",
        "paragraphs": "18-600, 18-620", "reviewed": "2026-06-30", "checked": "2026-09-15",
    },
    # Not a Library document: the ATO publishes its own sample data for this
    # schedule, and tests/payg_withholding_sample_2026_27.csv carries all of it.
    "payg_withholding": {
        "document": "ATO Schedule 1 withholding amounts sample data",
        "paragraphs": "weekly, fortnightly and monthly tables",
        "reviewed": "2026-06-17", "checked": "2026-09-24",
    },
    "contribution_caps": {
        "document": "tax-examples-individuals", "paragraphs": "7-278",
        "reviewed": "2025-06-30", "checked": "2026-09-24",
    },
    "pension_minimum": {
        "document": "superannuation-instant-reference-rates-thresholds-and-checklists",
        "paragraphs": "18-500", "reviewed": "2026-06-30", "checked": "2026-09-24",
    },
    # Not a Library document: the ATO page carries its own worked examples, and
    # tests/test_study_loan.py holds the worksheet to them and to the Gazette.
    "study_loan_repayment": {
        "document": "ATO study and training loan repayment thresholds and rates",
        "paragraphs": "tables 1 and 2, examples 1 to 3",
        "reviewed": "2026-06-30", "checked": "2026-09-27",
    },
}

INPUT_UNITS = {
    "gst": {"amount": "AUD", "gst_inclusive": "boolean"},
    "resident_tax": {"taxable_income": "AUD, whole dollars"},
    "capital_gains": {
        "other_gains": "AUD", "discount_gains": "AUD",
        "current_losses": "AUD", "prior_losses": "AUD",
    },
    "fbt": {"type_one_value": "AUD", "type_two_value": "AUD"},
    "depreciation": {
        "cost": "AUD", "effective_life": "years, decimal string from 0.01 to 1000",
        "days": "integer from 0 to 365",
        "taxable_use": "fraction, decimal string from 0 to 1", "method": "method name",
    },
    "quarterly_sg": {
        "ordinary_time_earnings": "AUD", "qualifying_contributions": "AUD",
        "quarter": "integer from 1 to 4",
    },
    "payg_withholding": {
        "earnings": "AUD, gross earnings and allowances subject to withholding for the pay period",
        "pay_period": "weekly, fortnightly or monthly",
        "scale": "integer 1, 2, 3, 5 or 6",
    },
    "contribution_caps": {
        "total_super_balance": "AUD, total super balance at 30 June before the income year",
        "concessional_contributions": "AUD",
        "unused_concessional_cap": "AUD, unexpired unused amounts from the previous 5 years",
        "non_concessional_contributions": "AUD",
        "under_75_in_year": "boolean, under 75 at any time in the income year",
    },
    "pension_minimum": {
        "account_balance": "AUD, at 1 July or on the commencement day in the first year",
        "age": "integer years on the same day as the account balance",
        "days": "integer days in the income year from and including the commencement day, "
                "or all days in the year for a pension running on 1 July",
    },
    "study_loan_repayment": {
        "repayment_income": "AUD, whole dollars, established repayment income for the year",
    },
}

EXAMPLES: dict[str, dict[str, Any]] = {
    "gst": {"amount": "1100.00", "gst_inclusive": True},
    "resident_tax": {"taxable_income": "45000"},
    "capital_gains": {
        "other_gains": "100.00", "discount_gains": "1000.00",
        "current_losses": "200.00", "prior_losses": "300.00",
    },
    "fbt": {"type_one_value": "1000.00", "type_two_value": "1000.00"},
    "depreciation": {
        "cost": "3000.00", "effective_life": "4", "days": 146,
        "taxable_use": "0.4", "method": "prime_cost",
    },
    "quarterly_sg": {
        "ordinary_time_earnings": "80000.00", "qualifying_contributions": "2000.00",
        "quarter": 1,
    },
    "payg_withholding": {"earnings": "1000.00", "pay_period": "weekly", "scale": 2},
    "contribution_caps": {
        "total_super_balance": "450000.00", "concessional_contributions": "40000.00",
        "unused_concessional_cap": "15000.00", "non_concessional_contributions": "150000.00",
        "under_75_in_year": True,
    },
    "pension_minimum": {"account_balance": "600000.00", "age": 67, "days": 365},
    "study_loan_repayment": {"repayment_income": "86380"},
}


def periods(kind: str) -> list[dict[str, Any]]:
    """Describe accepted period arguments and inclusive calendar boundaries."""
    if kind == "fbt":
        return [{"id": year, "basis": "FBT year", "start": f"{int(year) - 1}-04-01",
                 "end": f"{year}-03-31", "arguments": {"year_ended": int(year)}}
                for year in SUPPORTED_PERIODS[kind]]
    result = []
    for year in SUPPORTED_PERIODS[kind]:
        start = int(year[:4])
        period: dict[str, Any] = {
            "id": year, "basis": "income year", "start": f"{start}-07-01",
            "end": f"{start + 1}-06-30", "arguments": {"year": year},
        }
        if kind == "quarterly_sg":
            period["basis"] = "complete quarter within income year"
            period["quarters"] = [
                {"quarter": 1, "start": f"{start}-07-01", "end": f"{start}-09-30"},
                {"quarter": 2, "start": f"{start}-10-01", "end": f"{start}-12-31"},
                {"quarter": 3, "start": f"{start + 1}-01-01", "end": f"{start + 1}-03-31"},
                {"quarter": 4, "start": f"{start + 1}-04-01", "end": f"{start + 1}-06-30"},
            ]
        result.append(period)
    return result
