"""Next-business-day roll under the ATO lodgment program's public holiday rule.

The ATO's "Lodgment and payment dates on weekends or public holidays" page for the
2026–27 program says a due date that is not a business day moves to the next business
day, where a business day is not a Saturday, a Sunday or a day that is a public
holiday for the whole of any state or territory. A holiday in one state therefore
moves the date for taxpayers in every state. That page's Tables 1 and 2 are the
calendar used here, copied as published; no other holiday source is read.
"""

from __future__ import annotations

from datetime import date, timedelta

HOLIDAY_SOURCE = (
    "https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/"
    "prepare-and-lodge/registered-agent-lodgment-program/"
    "lodgment-and-payment-dates-on-weekends-or-public-holidays"
)
HOLIDAY_PROVISION = (
    "Approach to weekends and public holidays; Table 1: Australian public holidays in "
    "2026; Table 2: Australian public holidays in 2027 (page last updated 1 July 2026)"
)

# Tables 1 and 2 cover the program year: Table 1 starts at 3 August 2026, so no
# whole-of-state holiday falls in July 2026, and Table 2 ends at 14 June 2027.
# Outside these dates the ATO has published no table for the program, so a weekday
# there cannot be shown to be a business day.
COVERS_FROM = date(2026, 7, 1)
COVERS_TO = date(2027, 6, 30)

HOLIDAYS: dict[date, str] = {
    date(2026, 8, 3): "Picnic Day (NT)",
    date(2026, 9, 25): "Friday before AFL Grand Final (VIC)",
    date(2026, 9, 28): "King's Birthday (WA)",
    date(2026, 10, 5): "Labour Day (ACT, NSW, SA); King's Birthday (QLD)",
    date(2026, 11, 3): "Melbourne Cup (VIC)",
    date(2026, 12, 24): "Christmas Eve (QLD, NT, SA)",
    date(2026, 12, 25): "Christmas Day",
    date(2026, 12, 28): "Additional public holiday for Boxing Day (ACT, NSW, NT, QLD, TAS, "
                        "VIC, WA); additional public holiday for Proclamation Day (SA)",
    date(2026, 12, 31): "New Year's Eve (NT, SA)",
    date(2027, 1, 1): "New Year's Day",
    date(2027, 1, 26): "Australia Day",
    date(2027, 3, 1): "Labour Day (WA)",
    date(2027, 3, 8): "Eight Hours Day (TAS); Labour Day (VIC); Adelaide Cup Day (SA); "
                      "Canberra Day (ACT)",
    date(2027, 3, 26): "Good Friday",
    date(2027, 3, 29): "Easter Monday",
    date(2027, 4, 26): "Additional public holiday for ANZAC Day (ACT, NSW, NT, QLD, WA)",
    date(2027, 5, 3): "May Day (NT); Labour Day (QLD)",
    date(2027, 5, 31): "Reconciliation Day (ACT)",
    date(2027, 6, 7): "Western Australia Day",
    date(2027, 6, 14): "King's Birthday (excluding WA and QLD)",
}


def roll(day: date) -> tuple[date | None, list[dict[str, str]]]:
    """The first business day on or after ``day``, and the days passed over.

    Returns ``None`` for the date when a weekday on the way falls outside the
    published tables, because it may be a holiday the ATO has not yet listed.
    """
    skipped: list[dict[str, str]] = []
    while True:
        reason = {5: "Saturday", 6: "Sunday"}.get(day.weekday())
        if reason is None:
            if not COVERS_FROM <= day <= COVERS_TO:
                return None, skipped
            reason = HOLIDAYS.get(day)
            if reason is None:
                return day, skipped
        skipped.append({"date": day.isoformat(), "reason": reason})
        day += timedelta(days=1)
