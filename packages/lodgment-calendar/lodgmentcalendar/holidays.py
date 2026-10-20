"""Next-business-day roll under the ATO lodgment program's public holiday rule.

The ATO's "Lodgment and payment dates on weekends or public holidays" page for the
2026–27 program says a due date that is not a business day moves to the next business
day, where a business day is not a Saturday, a Sunday or a day that is a public
holiday for the whole of any state or territory. A holiday in one state therefore
moves the date for taxpayers in every state. That page's Tables 1 and 2 are the
calendar used here, copied as published; no other holiday source is read.
"""

from __future__ import annotations

from collections.abc import Mapping
from datetime import date, datetime, timedelta

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
MAX_EXTRA_HOLIDAYS = 50

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


# Days whose roll the table itself answers 2 ways.
CONFLICTS = {
    date(2026, 9, 25): "the next business day after 2026-09-25: the ATO table gives 28 "
    "September 2026, but also lists 28 September as King's Birthday (WA), which would make "
    "it 29 September",
}


def roll(day: date, holidays: Mapping[date, str] = HOLIDAYS,
         covers_to: date = COVERS_TO) -> tuple[date | None, list[dict[str, str]], str | None]:
    """The first business day on or after ``day``, the days passed over, and why not.

    Returns no date, with the reason, when a weekday on the way falls outside the
    holidays known to be complete (it may be a holiday nobody has listed) or the ATO
    table answers the roll 2 ways. ``holidays`` and ``covers_to`` default to the ATO
    table; ``calendar`` adds a caller's holidays to them.
    """
    skipped: list[dict[str, str]] = []
    while True:
        if day in CONFLICTS:
            return None, skipped, CONFLICTS[day]
        # A known holiday is not a business day, even past the span known to be complete.
        reason = {5: "Saturday", 6: "Sunday"}.get(day.weekday()) or holidays.get(day)
        if reason is None:
            if not COVERS_FROM <= day <= covers_to:
                return None, skipped, (
                    f"whether {day.isoformat()} is a public holiday for the whole of any "
                    f"state or territory: the holidays known cover {COVERS_FROM.isoformat()} "
                    f"to {covers_to.isoformat()}")
            return day, skipped, None
        skipped.append({"date": day.isoformat(), "reason": reason})
        day += timedelta(days=1)


def calendar(extra: Mapping[date, str] | None,
             complete_until: date | None) -> tuple[dict[date, str], date]:
    """The ATO table plus a caller's holidays, and the last day the result is complete to.

    A caller's holiday must be a public holiday for the whole of a state or territory
    that the ATO table does not list. ``complete_until`` is the caller's statement that
    the 2 together hold every such holiday to that date; adding holidays alone never
    extends coverage, and an earlier date cannot shrink the ATO table's own span.
    """
    extra = {} if extra is None else extra
    if not isinstance(extra, Mapping) or len(extra) > MAX_EXTRA_HOLIDAYS:
        raise ValueError(f"extra_holidays must map at most {MAX_EXTRA_HOLIDAYS} dates to "
                         "holiday names")
    merged = dict(HOLIDAYS)
    for day, name in extra.items():
        if not _is_date(day):
            raise ValueError(f"extra_holidays has {day!r}; each key must be a date")
        if (not isinstance(name, str) or not name.strip() or len(name) > 120
                or any(ord(char) < 32 for char in name)):
            raise ValueError(f"extra_holidays[{day.isoformat()}] needs a holiday name of at "
                             "most 120 printable characters")
        if day in HOLIDAYS or day in CONFLICTS:
            raise ValueError(f"extra_holidays[{day.isoformat()}] is already in the ATO table")
        if day.weekday() >= 5:
            raise ValueError(f"extra_holidays[{day.isoformat()}] falls on a weekend")
        merged[day] = f"{name.strip()} (caller-supplied)"
    if complete_until is not None and not _is_date(complete_until):
        raise ValueError(f"holidays_complete_until must be a date; got {complete_until!r}")
    return merged, max(COVERS_TO, complete_until or COVERS_TO)


def _is_date(value: object) -> bool:
    return isinstance(value, date) and not isinstance(value, datetime)
