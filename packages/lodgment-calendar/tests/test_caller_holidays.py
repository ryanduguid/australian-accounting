"""Caller-supplied holidays settle dates after the ATO table ends.

Every holiday here is fabricated for the test; none is a real proclamation.
"""

from datetime import date, datetime

import pytest
from lodgmentcalendar import Facts, due_dates

ONLINE = Facts(lodges_electronically=True)
PAPER = Facts(lodges_electronically=False)
COMPLETE = date(2027, 8, 31)


def quarter_4(facts, **calendar):
    return due_dates("bas", "trust", "2026-27", "self", reporting_cycle="quarterly",
                     facts=facts, **calendar)


def test_a_complete_calendar_settles_quarter_4():
    result = quarter_4(ONLINE, holidays_complete_until=COMPLETE)
    q4 = result["obligations"][3]
    assert (q4["status"], q4["due_date"], q4["holiday_roll"]) == (
        "DETERMINED", "2027-08-11", None)
    assert result["holiday_calendar"]["covers_to"] == "2027-08-31"
    assert result["holiday_calendar"]["caller_complete_until"] == "2027-08-31"
    assert result["holiday_calendar"]["caller_holidays"] == {}


def test_a_caller_holiday_moves_the_date_and_is_marked():
    result = quarter_4(PAPER, extra_holidays={date(2027, 7, 28): "Synthetic test holiday"},
                       holidays_complete_until=COMPLETE)
    q4 = result["obligations"][3]
    assert (q4["nominal_due_date"], q4["due_date"]) == ("2027-07-28", "2027-07-29")
    assert q4["holiday_roll"]["skipped"] == [
        {"date": "2027-07-28", "reason": "Synthetic test holiday (caller-supplied)"}]
    assert result["holiday_calendar"]["caller_holidays"] == {
        "2027-07-28": "Synthetic test holiday (caller-supplied)"}


def test_holidays_alone_do_not_extend_coverage():
    result = quarter_4(PAPER, extra_holidays={date(2027, 7, 28): "Synthetic test holiday"})
    q4 = result["obligations"][3]
    assert (q4["status"], q4["due_date"]) == ("UNKNOWN", None)
    assert q4["missing"] == ["whether 2027-07-29 is a public holiday for the whole of any "
                             "state or territory: the holidays known cover 2026-07-01 to "
                             "2027-06-30"]


@pytest.mark.parametrize("extra,complete,expected", [
    # The complete span must reach the day the roll lands on, not just the nominal date.
    ({date(2027, 7, 28): "Synthetic"}, date(2027, 7, 28), ("UNKNOWN", None, "2027-07-29")),
    ({date(2027, 7, 28): "Synthetic"}, date(2027, 7, 29), ("DETERMINED", "2027-07-29", None)),
    ({date(2027, 7, 28): "Synthetic", date(2027, 7, 29): "Synthetic"}, date(2027, 7, 28),
     ("UNKNOWN", None, "2027-07-30")),
])
def test_the_complete_span_must_cover_the_rolled_date(extra, complete, expected):
    q4 = quarter_4(PAPER, extra_holidays=extra, holidays_complete_until=complete)[
        "obligations"][3]
    asked = q4["missing"][0].split()[1] if q4["missing"] else None
    assert (q4["status"], q4["due_date"], asked) == expected


def test_rolling_into_the_table_conflict_stays_unknown():
    extra = {date(2026, 9, day): "Synthetic" for day in (21, 22, 23, 24)}
    result = due_dates("ias", "company", "2026-27", "self", reporting_cycle="monthly",
                       extra_holidays=extra, holidays_complete_until=COMPLETE)
    august = result["obligations"][1]
    assert (august["nominal_due_date"], august["status"]) == ("2026-09-21", "UNKNOWN")
    assert august["missing"][0].startswith("the next business day after 2026-09-25")


def test_an_earlier_complete_date_cannot_shrink_the_ato_table():
    result = quarter_4(PAPER, holidays_complete_until=date(2027, 1, 31))
    assert result["holiday_calendar"]["covers_to"] == "2027-06-30"
    assert result["obligations"][2]["due_date"] == "2027-04-28"


def test_a_complete_calendar_settles_the_june_monthly_statement():
    result = due_dates("ias", "company", "2026-27", "self", reporting_cycle="monthly",
                       holidays_complete_until=COMPLETE)
    assert result["obligations"][-1]["due_date"] == "2027-07-21"
    assert result["counts"] == {"DETERMINED": 12, "UNKNOWN": 0}


@pytest.mark.parametrize("calendar,message", [
    ({"extra_holidays": [date(2027, 7, 28)]}, "must map at most 50 dates"),
    ({"extra_holidays": {date(2027, 7, day): "Synthetic" for day in range(1, 32)}
      | {date(2027, 8, day): "Synthetic" for day in range(1, 21)}},
     "must map at most 50 dates"),
    ({"extra_holidays": {datetime(2027, 7, 28): "Synthetic"}}, "each key must be a date"),
    ({"extra_holidays": {"2027-07-28": "Synthetic"}}, "each key must be a date"),
    ({"extra_holidays": {date(2027, 7, 28): " "}}, "needs a holiday name"),
    ({"extra_holidays": {date(2027, 7, 28): "Synthetic\nholiday"}}, "needs a holiday name"),
    ({"extra_holidays": {date(2027, 7, 28): "Synthetic\x7fholiday"}}, "needs a holiday name"),
    ({"extra_holidays": {date(2027, 7, 28): "Synthetic\x85holiday"}}, "needs a holiday name"),
    ({"extra_holidays": {date(2027, 7, 28): "Synthetic\u202eholiday"}}, "needs a holiday name"),
    ({"extra_holidays": {date(2027, 7, 28): "x" * 121}}, "needs a holiday name"),
    ({"extra_holidays": {date(2027, 6, 7): "Synthetic"}}, "already in the ATO table"),
    ({"extra_holidays": {date(2026, 9, 25): "Synthetic"}}, "already in the ATO table"),
    ({"extra_holidays": {date(2027, 7, 31): "Synthetic"}}, "falls on a weekend"),
    ({"holidays_complete_until": "2027-08-31"}, "holidays_complete_until must be a date"),
    ({"holidays_complete_until": datetime(2027, 8, 31)}, "holidays_complete_until must be"),
])
def test_invalid_calendars_are_refused(calendar, message):
    with pytest.raises(ValueError, match=message):
        quarter_4(PAPER, **calendar)
