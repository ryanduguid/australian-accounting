"""FBT return dates, by lodgment channel, with fabricated facts."""

from datetime import date

import pytest
from lodgmentcalendar import Facts, due_dates, to_ics
from lodgmentcalendar.holidays import COVERS_TO, HOLIDAYS

FBT_PAGE = "lodging-your-fbt-return-and-paying"
AGENT_PAGE = "due-dates-by-obligation-type/fbt-return"


def fbt(channel, facts=None, *, entity_type="company", year="2026-27"):
    return due_dates("fbt_return", entity_type, year, channel, facts=facts)


def only_row(result):
    assert len(result["obligations"]) == 1
    return result["obligations"][0]


def test_a_self_lodger_lodges_and_pays_by_21_may():
    row = only_row(fbt("self"))

    assert (row["status"], row["due_date"], row["rule_id"]) == (
        "DETERMINED", "2027-05-21", "FBT-21-MAY")
    assert row["payment"] == "with_lodgment"
    assert FBT_PAGE in row["source_url"]


def test_a_listed_client_lodged_electronically_by_a_tax_agent_gets_25_june():
    row = only_row(fbt("tax_agent", Facts(lodges_electronically=True,
                                          on_fbt_client_list_by_21_may=True)))

    assert (row["status"], row["due_date"], row["rule_id"]) == (
        "DETERMINED", "2027-06-25", "FBT-AGENT-ELECTRONIC")
    assert row["payment"] == "with_lodgment"
    assert AGENT_PAGE in row["source_url"]


@pytest.mark.parametrize("listed", [True, False, None])
def test_an_agent_lodging_on_paper_keeps_21_may_whatever_the_list(listed):
    row = only_row(fbt("tax_agent", Facts(lodges_electronically=False,
                                          on_fbt_client_list_by_21_may=listed)))

    assert (row["status"], row["due_date"], row["rule_id"]) == (
        "DETERMINED", "2027-05-21", "FBT-AGENT-PAPER")


@pytest.mark.parametrize("electronic", [True, None])
def test_a_client_not_listed_by_21_may_keeps_21_may(electronic):
    row = only_row(fbt("tax_agent", Facts(lodges_electronically=electronic,
                                          on_fbt_client_list_by_21_may=False)))

    assert (row["status"], row["due_date"], row["rule_id"]) == (
        "DETERMINED", "2027-05-21", "FBT-AGENT-NOT-LISTED")
    assert AGENT_PAGE in row["source_url"]


@pytest.mark.parametrize(
    "facts,missing",
    [(Facts(), ["lodges_electronically", "on_fbt_client_list_by_21_may"]),
     (Facts(lodges_electronically=True), ["on_fbt_client_list_by_21_may"]),
     (Facts(on_fbt_client_list_by_21_may=True), ["lodges_electronically"])],
)
def test_a_fact_on_the_path_not_supplied_is_unknown_and_named(facts, missing):
    row = only_row(fbt("tax_agent", facts))

    assert row["status"] == "UNKNOWN" and row["due_date"] is None
    assert [item.split(":")[0] for item in row["missing"]] == missing
    assert row["payment"] == "unknown"
    assert AGENT_PAGE in row["source_url"]


def test_a_bas_agent_is_unknown_because_the_pages_name_only_tax_agents():
    row = only_row(fbt("bas_agent", Facts(lodges_electronically=True,
                                          on_fbt_client_list_by_21_may=True)))

    assert (row["status"], row["due_date"], row["rule_id"]) == (
        "UNKNOWN", None, "FBT-BAS-AGENT")
    assert row["nominal_due_date"] == "2027-05-21"
    assert "BAS agent" in row["missing"][0]


@pytest.mark.parametrize("entity_type", ["individual", "trust", "partnership", "smsf"])
def test_the_dates_do_not_depend_on_the_entity(entity_type):
    assert only_row(fbt("self", entity_type=entity_type))["due_date"] == "2027-05-21"


@pytest.mark.parametrize("year", ["2025-26", "2027-28"])
def test_another_fbt_year_is_not_recorded(year):
    row = only_row(fbt("self", year=year))

    assert (row["status"], row["rule_id"]) == ("UNKNOWN", "YEAR-NOT-RECORDED")
    assert row["nominal_due_date"] is None


@pytest.mark.parametrize("day", [date(2027, 5, 21), date(2027, 6, 25)])
def test_both_dates_are_business_days_inside_the_tracked_holiday_table(day):
    # No separate end date to track: past COVERS_TO, check_rate_expiry.py already warns.
    assert day <= COVERS_TO
    assert day.weekday() < 5 and day not in HOLIDAYS


def test_a_cycle_is_refused():
    with pytest.raises(ValueError, match="reporting_cycle applies only to bas and ias"):
        due_dates("fbt_return", "company", "2026-27", "self", reporting_cycle="monthly")


def test_the_fact_must_be_a_boolean():
    with pytest.raises(ValueError, match="on_fbt_client_list_by_21_may"):
        fbt("tax_agent", Facts(on_fbt_client_list_by_21_may="yes"))  # type: ignore[arg-type]


def test_what_is_left_open_stays_visible():
    decided = " ".join(fbt("self")["not_decided"])

    assert "notice of non-lodgment" in decided
    assert "FBT instalments" in decided
    assert "extension" in decided


def test_the_calendar_file_holds_only_determined_fbt_dates():
    determined = to_ics(fbt("tax_agent", Facts(lodges_electronically=True,
                                               on_fbt_client_list_by_21_may=True)))
    assert "DTSTART;VALUE=DATE:20270625" in determined["ics"]
    assert determined["unknown"] == []

    for result in (fbt("tax_agent"), fbt("bas_agent")):
        unknown = to_ics(result)
        assert unknown["ics"] is None
        assert [row["status"] for row in unknown["unknown"]] == ["UNKNOWN"]
