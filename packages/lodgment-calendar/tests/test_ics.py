"""The iCalendar export: every determined date is an event, and only those."""

from datetime import date, timedelta

import pytest
from lodgmentcalendar import Facts, due_dates, to_ics

QUARTERLY_AGENT = Facts(consolidated_head_company=False, lodges_electronically=True,
                        previous_statement_lodged_electronically=True,
                        includes_monthly_payg_obligation=False)
START = "DTSTART;VALUE=DATE"
END = "DTEND;VALUE=DATE"


def quarterly():
    return due_dates("bas", "company", "2026-27", "bas_agent", reporting_cycle="quarterly",
                     facts=QUARTERLY_AGENT)


def unfold(text):
    """Split into properties, checking the line rules on the way."""
    assert text.endswith("\r\n")
    for line in text[:-2].split("\r\n"):
        assert len(line.encode()) <= 75
    return text[:-2].replace("\r\n ", "").split("\r\n")


def events(text):
    found, current = [], {}
    for line in unfold(text):
        if line == "BEGIN:VEVENT":
            current = {}
        elif line == "END:VEVENT":
            found.append(current)
        else:
            name, value = line.split(":", 1)
            current[name] = value
    return found


def test_each_determined_date_is_one_all_day_event():
    result = quarterly()
    found = events(to_ics(result)["ics"])
    determined = [row for row in result["obligations"] if row["status"] == "DETERMINED"]
    assert [event[START] for event in found] == [
        row["due_date"].replace("-", "") for row in determined]
    # Quarter 2 is rolled from Sunday 28 February over Labour Day (WA) to 2 March.
    assert "20270302" in [event[START] for event in found]
    for event, row in zip(found, determined, strict=True):
        next_day = date.fromisoformat(row["due_date"]) + timedelta(days=1)
        assert event[END] == f"{next_day:%Y%m%d}"


def test_rule_source_and_roll_are_in_the_description():
    rolled = next(event for event in events(to_ics(quarterly())["ics"])
                  if event[START] == "20270302")
    description = rolled["DESCRIPTION"]
    assert "Rule: AS-Q-28" in description
    assert "before the next-business-day roll: 2027-02-28" in description
    assert "https://www.ato.gov.au" in description
    assert "Source checked: 2026-10-28" in description
    assert "guide only" in description
    assert rolled["SUMMARY"].endswith(": lodgment")


def test_an_unknown_row_is_returned_apart_and_gets_no_event():
    output = to_ics(quarterly())
    assert [row["period"] for row in output["unknown"]] == ["2026-27 Q4"]
    assert output["unknown"][0]["due_date"] is None
    found = events(output["ics"])
    assert "20270825" not in [event[START] for event in found]
    assert not any("quarter 4" in event["SUMMARY"] for event in found)
    assert "UNKNOWN and not in this calendar: BAS quarter 4" in "".join(unfold(output["ics"]))


def test_no_determined_row_gives_no_calendar():
    result = due_dates("bas", "company", "2026-27", "bas_agent", reporting_cycle="quarterly")
    output = to_ics(result)
    assert output["ics"] is None
    assert len(output["unknown"]) == len(result["obligations"])


def test_all_determined_rows_add_no_unknown_note():
    output = to_ics(due_dates("ias", "trust", "2026-27", "self", reporting_cycle="monthly",
                             holidays_complete_until=date(2027, 9, 30)))
    assert output["unknown"] == []
    assert "UNKNOWN" not in output["ics"]


def test_the_same_result_exports_the_same_bytes():
    assert to_ics(quarterly())["ics"] == to_ics(quarterly())["ics"]


def test_a_uid_survives_a_changed_date_and_differs_by_scope():
    first = events(to_ics(quarterly())["ics"])[0]
    paper = due_dates("bas", "company", "2026-27", "bas_agent", reporting_cycle="quarterly",
                      facts=Facts(consolidated_head_company=False, lodges_electronically=False,
                                  includes_monthly_payg_obligation=False))
    moved = events(to_ics(paper)["ics"])[0]
    assert moved[START] != first[START]
    assert moved["UID"] == first["UID"]
    scoped = events(to_ics(quarterly(), uid_scope="H1")["ics"])[0]
    assert scoped["UID"] != first["UID"]
    assert "H1" not in scoped["UID"]


def test_uid_scope_must_be_text():
    with pytest.raises(ValueError, match="uid_scope"):
        to_ics(quarterly(), uid_scope=1)  # type: ignore[arg-type]


def test_text_is_escaped_and_a_multibyte_character_is_never_split():
    result = quarterly()
    row = next(row for row in result["obligations"] if row["status"] == "DETERMINED")
    row["summary"] = "a,b;c\\d\ne " + "–" * 60
    joined = "".join(unfold(to_ics(result)["ics"]))
    assert "a\\,b\\;c\\\\d\\ne " + "–" * 60 in joined


def test_every_request_shape_keeps_rows_and_uids_distinct():
    facts = Facts(lodges_electronically=True, previous_statement_lodged_electronically=True,
                  includes_monthly_payg_obligation=False, consolidated_head_company=False,
                  prior_returns_outstanding=False, on_agent_client_list_by_cutoff=True,
                  large_or_medium=False, new_registrant=False,
                  prior_year_return_taxable=True, current_year_return_taxable=True,
                  latest_tax_liability_20k_or_more=False, total_income_over_2m=False,
                  annual_turnover_up_to_10m=True, deferred_gst_registered=False)
    uids = set()
    for entity_type in ("individual", "trust", "partnership", "company", "smsf"):
        for channel in ("self", "tax_agent", "bas_agent"):
            shapes = [(obligation, "2026-27", cycle) for obligation in ("bas", "ias")
                      for cycle in ("monthly", "quarterly")]
            if channel != "bas_agent":
                shapes.append(("income_tax_return", "2025-26", None))
            for obligation, year, cycle in shapes:
                result = due_dates(obligation, entity_type, year, channel,
                                   reporting_cycle=cycle, facts=facts)
                output = to_ics(result)
                assert len(output["unknown"]) == result["counts"]["UNKNOWN"]
                found = events(output["ics"]) if output["ics"] else []
                assert len(found) == result["counts"]["DETERMINED"]
                new = {event["UID"] for event in found}
                assert len(new) == len(found)
                assert not new & uids
                uids |= new


def test_a_carriage_return_never_reaches_a_content_line():
    result = quarterly()
    row = next(row for row in result["obligations"] if row["status"] == "DETERMINED")
    row["summary"] = "one\r\ntwo\rthree"
    text = to_ics(result)["ics"]
    assert "\r" not in text.replace("\r\n", "")
    assert "one\\ntwo\\nthree" in "".join(unfold(text))


def test_a_changed_holiday_input_moves_the_date_and_keeps_the_uid():
    def export(**holidays):
        return events(to_ics(due_dates(
            "bas", "trust", "2026-27", "self", reporting_cycle="quarterly",
            facts=Facts(lodges_electronically=True),
            holidays_complete_until=date(2027, 8, 31), **holidays))["ics"])

    plain, moved = export(), export(extra_holidays={date(2027, 8, 11): "Test holiday"})
    assert (plain[3][START], moved[3][START]) == ("20270811", "20270812")
    assert [event["UID"] for event in plain] == [event["UID"] for event in moved]

