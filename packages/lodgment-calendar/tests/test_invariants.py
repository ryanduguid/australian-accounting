"""Properties every result must hold, over every request shape and many fact sets.

The fact sets are a fixed pseudo-random sample, so a failure always reproduces.
"""

import random
from dataclasses import fields
from datetime import date, timedelta

from lodgmentcalendar import Facts, due_dates
from lodgmentcalendar.due_dates import CHANNELS, CYCLES, ENTITY_TYPES
from lodgmentcalendar.holidays import COVERS_FROM, HOLIDAYS

FACT_NAMES = [field.name for field in fields(Facts)]
SAMPLES = 60
CALLER_HOLIDAY = date(2027, 7, 28)
COMPLETE = date(2027, 9, 30)


def requests():
    for entity_type in ENTITY_TYPES:
        for channel in CHANNELS:
            for obligation in ("bas", "ias"):
                for cycle in CYCLES:
                    yield obligation, entity_type, "2026-27", channel, cycle
            if channel != "bas_agent":
                yield "income_tax_return", entity_type, "2025-26", channel, None


def fact_sets():
    rng = random.Random(20261020)
    yield Facts()
    for _ in range(SAMPLES):
        yield Facts(**{name: rng.choice((True, False, None)) for name in FACT_NAMES})


def check(result, known_holidays, covers_to):
    rows = result["obligations"]
    assert rows
    assert result["counts"]["DETERMINED"] + result["counts"]["UNKNOWN"] == len(rows)
    for index, row in enumerate(rows):
        assert row["source_url"].startswith("https://www.ato.gov.au/")
        if row["status"] == "DETERMINED":
            assert row["missing"] == []
            due = date.fromisoformat(row["due_date"])
            nominal = date.fromisoformat(row["nominal_due_date"])
            assert due.weekday() < 5 and due not in known_holidays
            assert COVERS_FROM <= due <= covers_to
            # The roll passes over exactly the days from the nominal date to the day before
            # the due date, each one a weekend or known holiday: the first business day wins.
            skipped = [date.fromisoformat(day["date"])
                       for day in (row["holiday_roll"] or {}).get("skipped", [])]
            assert skipped == [nominal + timedelta(days=n) for n in range((due - nominal).days)]
            assert all(day.weekday() >= 5 or day in known_holidays for day in skipped)
            assert date(2026, 9, 25) not in skipped
        else:
            assert row["status"] == "UNKNOWN" and row["due_date"] is None and row["missing"]
        if row["kind"] == "lodgment":
            assert row["payment"] in {"with_lodgment", "separate", "none", "unknown"}
            follows = rows[index + 1] if index + 1 < len(rows) else None
            has_payment_row = follows is not None and follows["kind"] == "payment"
            assert has_payment_row == (row["payment"] == "separate")
        else:
            assert "payment" not in row
        if row["kind"] == "concession":
            assert result["request"]["obligation"] == "income_tax_return"


def test_every_result_holds_its_properties_with_the_ato_table():
    determined = rolled = 0
    for request in requests():
        obligation, entity_type, year, channel, cycle = request
        for facts in fact_sets():
            result = due_dates(obligation, entity_type, year, channel, reporting_cycle=cycle,
                               facts=facts)
            check(result, HOLIDAYS, date(2027, 6, 30))
            settled = [row for row in result["obligations"] if row["status"] == "DETERMINED"]
            determined += len(settled)
            rolled += sum(1 for row in settled if row["due_date"] != row["nominal_due_date"])
    # The sample must reach settled and rolled dates, or the checks above prove nothing.
    assert determined > 1000 and rolled > 100


def test_every_result_holds_its_properties_with_a_caller_calendar():
    known = {**HOLIDAYS, CALLER_HOLIDAY: "Synthetic test holiday"}
    past_table = past_caller_holiday = 0
    for request in requests():
        obligation, entity_type, year, channel, cycle = request
        for facts in fact_sets():
            result = due_dates(obligation, entity_type, year, channel, reporting_cycle=cycle,
                               facts=facts,
                               extra_holidays={CALLER_HOLIDAY: "Synthetic test holiday"},
                               holidays_complete_until=COMPLETE)
            check(result, known, COMPLETE)
            for row in result["obligations"]:
                if row["status"] == "DETERMINED":
                    past_table += row["due_date"] > "2027-06-30"
                    past_caller_holiday += row["nominal_due_date"] == CALLER_HOLIDAY.isoformat()
    # The caller's calendar must settle dates after the ATO table and roll past its holiday.
    assert past_table > 100 and past_caller_holiday > 10


def test_a_fact_the_caller_supplied_is_never_listed_as_missing():
    for request in requests():
        obligation, entity_type, year, channel, cycle = request
        for facts in fact_sets():
            result = due_dates(obligation, entity_type, year, channel, reporting_cycle=cycle,
                               facts=facts)
            for row in result["obligations"]:
                asked = {item.split(":")[0] for item in row["missing"]}
                assert all(getattr(facts, name) is None
                           for name in asked & set(FACT_NAMES))
