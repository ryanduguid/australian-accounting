"""Period, input and example metadata owned by the worksheet engine."""

from typing import Any

SUPPORTED_PERIODS = {
    "gst": ("2025-26",),
    "resident_tax": ("2024-25", "2025-26", "2026-27"),
    "capital_gains": ("2025-26",),
    "fbt": ("2026",),
    "depreciation": ("2025-26",),
    "quarterly_sg": ("2025-26",),
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
    "fbt": {"checked": "2026-09-10", "passage": "Calculating your FBT, steps 3 to 7"},
    "depreciation": {
        "checked": "2026-09-10",
        "passage": "Decline in value: diminishing value and prime cost methods; taxable use",
    },
    "quarterly_sg": {
        "checked": "2026-09-10",
        "passage": "Super guarantee, tables 21 and 24, 2025-26 rows",
    },
}

LIBRARY_EVIDENCE = {
    "gst": {
        "document": "Tax/Goods and services tax (GST) and other indirect taxes.md",
        "paragraphs": "12-020", "reviewed": "2026-06-30", "checked": "2026-09-15",
    },
    "resident_tax": {
        "document": "Tax/Individuals.md", "paragraphs": "7-010",
        "reviewed": "2025-06-30", "checked": "2026-09-15",
    },
    "capital_gains": {
        "document": "Tax/Capital Gains Tax (CGT).md", "paragraphs": "2-040, 2-240",
        "reviewed": "2026-06-30", "checked": "2026-09-15",
    },
    "fbt": {
        "document": "Tax/Fringe Benefits Tax (FBT).md", "paragraphs": "3-000, 3-020",
        "reviewed": "2026-06-30", "checked": "2026-09-15",
    },
    "depreciation": {
        "document": "Tax/Depreciation.md", "paragraphs": "6-000, 6-020",
        "reviewed": "2025-06-30", "checked": "2026-09-15",
    },
    "quarterly_sg": {
        "document": "Superannuation/Instant Reference – Rates, Thresholds and Checklists.md",
        "paragraphs": "18-600, 18-620", "reviewed": "2026-06-30", "checked": "2026-09-15",
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
