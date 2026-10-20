"""The next-business-day roll against the ATO's own published table."""

from datetime import date

import pytest
from lodgmentcalendar import holidays
from lodgmentcalendar.holidays import CONFLICTS, COVERS_FROM, COVERS_TO, HOLIDAYS, roll

# Each row of the ATO's Tables 1 and 2: the holiday and its "First business day".
# The 25 September 2026 row is left out: the table gives 28 September, a day it
# also lists as King's Birthday (WA). See the primary-source review.
ATO_FIRST_BUSINESS_DAY = {
    date(2026, 8, 3): date(2026, 8, 4),
    date(2026, 9, 28): date(2026, 9, 29),
    date(2026, 10, 5): date(2026, 10, 6),
    date(2026, 11, 3): date(2026, 11, 4),
    date(2026, 12, 24): date(2026, 12, 29),
    date(2026, 12, 25): date(2026, 12, 29),
    date(2026, 12, 28): date(2026, 12, 29),
    date(2026, 12, 31): date(2027, 1, 4),
    date(2027, 1, 1): date(2027, 1, 4),
    date(2027, 1, 26): date(2027, 1, 27),
    date(2027, 3, 1): date(2027, 3, 2),
    date(2027, 3, 8): date(2027, 3, 9),
    date(2027, 3, 26): date(2027, 3, 30),
    date(2027, 3, 29): date(2027, 3, 30),
    date(2027, 4, 26): date(2027, 4, 27),
    date(2027, 5, 3): date(2027, 5, 4),
    date(2027, 5, 31): date(2027, 6, 1),
    date(2027, 6, 7): date(2027, 6, 8),
    date(2027, 6, 14): date(2027, 6, 15),
}


@pytest.mark.parametrize("holiday,first", ATO_FIRST_BUSINESS_DAY.items(),
                         ids=lambda value: value.isoformat())
def test_roll_reproduces_the_ato_first_business_day_column(holiday, first):
    rolled, skipped, open_question = roll(holiday)
    assert (rolled, open_question) == (first, None)
    assert skipped[0] == {"date": holiday.isoformat(), "reason": HOLIDAYS[holiday]}


def test_every_listed_holiday_is_checked_against_the_table_bar_one():
    assert set(HOLIDAYS) - set(ATO_FIRST_BUSINESS_DAY) == {date(2026, 9, 25)}


def test_a_business_day_stays_put():
    assert roll(date(2026, 10, 28)) == (date(2026, 10, 28), [], None)


def test_a_weekend_moves_to_monday():
    assert roll(date(2026, 10, 31)) == (date(2026, 11, 2), [
        {"date": "2026-10-31", "reason": "Saturday"},
        {"date": "2026-11-01", "reason": "Sunday"},
    ], None)


def test_a_weekend_then_a_holiday_in_one_state_moves_every_state():
    # 28 February 2027 is a Sunday and 1 March 2027 is Labour Day in WA only.
    rolled, skipped, _ = roll(date(2027, 2, 28))
    assert rolled == date(2027, 3, 2)
    assert [day["reason"] for day in skipped] == ["Sunday", "Labour Day (WA)"]


def test_a_weekday_outside_the_published_tables_cannot_be_rolled():
    assert roll(date(2027, 7, 28))[:2] == (None, [])
    rolled, skipped, open_question = roll(date(2027, 7, 31))
    assert (rolled, [day["reason"] for day in skipped]) == (None, ["Saturday", "Sunday"])
    assert open_question.startswith("whether 2027-08-02 is a public holiday")
    assert roll(date(2026, 6, 30))[:2] == (None, [])


def test_the_conflicting_25_september_row_cannot_be_rolled():
    # The table gives 28 September 2026 for this row and lists 28 September as a holiday.
    assert roll(date(2026, 9, 25)) == (None, [], CONFLICTS[date(2026, 9, 25)])
    rolled, skipped, open_question = roll(date(2026, 9, 26))
    assert (rolled, len(skipped)) == (date(2026, 9, 29), 3)
    assert open_question is None


def test_coverage_is_the_program_year():
    assert (COVERS_FROM, COVERS_TO) == (date(2026, 7, 1), date(2027, 6, 30))
    assert all(COVERS_FROM <= day <= COVERS_TO for day in HOLIDAYS)
    assert holidays.HOLIDAY_SOURCE.startswith("https://www.ato.gov.au/")
