"""BAS and IAS due dates for 2026-27 periods, for fictional entities.

Synthetic Wattle Trust lodges its own statements; Synthetic Banksia Pty Ltd uses a
registered agent. Neither exists.
"""

import pytest
from lodgmentcalendar import Facts, due_dates

AGENT_READY = Facts(lodges_electronically=True, previous_statement_lodged_electronically=True)
BANKSIA = Facts(consolidated_head_company=False, lodges_electronically=True,
                previous_statement_lodged_electronically=True)
DECEMBER_READY = Facts(lodges_electronically=True, previous_statement_lodged_electronically=True,
                       annual_turnover_up_to_10m=True, deferred_gst_registered=False)


def rows(*args, **kwargs):
    return due_dates(*args, **kwargs)["obligations"]


def summary(result_rows):
    return [(row["period"], row["status"], row["nominal_due_date"], row["due_date"],
             row["rule_id"]) for row in result_rows]


def test_quarterly_bas_lodged_online_by_the_trust_itself():
    result = rows("bas", "trust", "2026-27", "self", reporting_cycle="quarterly",
                  facts=Facts(lodges_electronically=True))
    assert summary(result) == [
        ("2026-27 Q1", "DETERMINED", "2026-11-11", "2026-11-11", "AS-Q-ONLINE"),
        ("2026-27 Q2", "DETERMINED", "2027-02-28", "2027-03-02", "AS-Q-28"),
        ("2026-27 Q3", "DETERMINED", "2027-05-12", "2027-05-12", "AS-Q-ONLINE"),
        ("2026-27 Q4", "UNKNOWN", "2027-08-11", None, "AS-Q-ONLINE"),
    ]
    assert result[0]["obligation"] == "BAS quarter 1 2026-27 (July to September 2026)"
    assert all(row["kind"] == "lodgment" and row["payment"] == "with_lodgment" for row in result)
    assert result[1]["holiday_roll"]["skipped"] == [
        {"date": "2027-02-28", "reason": "Sunday"},
        {"date": "2027-03-01", "reason": "Labour Day (WA)"},
    ]
    assert result[3]["missing"] == [
        "whether 2027-08-11 is a public holiday for the whole of any state or territory: "
        "the ATO's table covers 2026-07-01 to 2027-06-30"]
    assert result[0]["source_url"].endswith("two-week-lodgment-concession-terms-and-conditions")


def test_quarterly_bas_on_paper_by_the_trust_itself_keeps_the_original_dates():
    result = rows("bas", "trust", "2026-27", "self", reporting_cycle="quarterly",
                  facts=Facts(lodges_electronically=False))
    assert [row["due_date"] for row in result] == [
        "2026-10-28", "2027-03-02", "2027-04-28", None]
    assert {row["rule_id"] for row in result} == {"AS-Q-28"}


def test_quarterly_bas_without_the_lodgment_method_is_unknown_except_quarter_2():
    result = rows("bas", "individual", "2026-27", "self", reporting_cycle="quarterly")
    assert [row["status"] for row in result] == ["UNKNOWN", "DETERMINED", "UNKNOWN", "UNKNOWN"]
    assert result[0]["missing"] == ["lodges_electronically: the statement is lodged "
                                    "electronically: online for a self-lodger, through Online "
                                    "services for agents or the practitioner lodgment service "
                                    "for an agent"]
    assert result[0]["nominal_due_date"] is None and result[0]["payment"] == "unknown"


@pytest.mark.parametrize("channel", ["tax_agent", "bas_agent"])
def test_quarterly_bas_lodged_by_an_agent_gets_the_program_dates(channel):
    result = rows("bas", "company", "2026-27", channel, reporting_cycle="quarterly",
                  facts=BANKSIA)
    assert summary(result) == [
        ("2026-27 Q1", "DETERMINED", "2026-11-25", "2026-11-25", "AS-Q-AGENT"),
        ("2026-27 Q2", "DETERMINED", "2027-02-28", "2027-03-02", "AS-Q-28"),
        ("2026-27 Q3", "DETERMINED", "2027-05-26", "2027-05-26", "AS-Q-AGENT"),
        ("2026-27 Q4", "UNKNOWN", "2027-08-25", None, "AS-Q-AGENT"),
    ]
    assert "to be confirmed" in result[3]["missing"][0]
    assert result[0]["source_url"].endswith("bas-agent-lodgment-program-2026-27")


def test_agent_quarterly_bas_on_paper_keeps_the_original_dates():
    result = rows("bas", "partnership", "2026-27", "bas_agent", reporting_cycle="quarterly",
                  facts=Facts(lodges_electronically=False))
    assert [(row["rule_id"], row["due_date"]) for row in result] == [
        ("AS-Q-AGENT-PAPER", "2026-10-28"), ("AS-Q-28", "2027-03-02"),
        ("AS-Q-AGENT-PAPER", "2027-04-28"), ("AS-Q-AGENT-PAPER", None)]


def test_agent_first_or_after_paper_statement_is_unknown_with_the_original_date():
    result = rows("ias", "smsf", "2026-27", "tax_agent", reporting_cycle="quarterly",
                  facts=Facts(lodges_electronically=True,
                              previous_statement_lodged_electronically=False))
    first = result[0]
    assert (first["status"], first["nominal_due_date"], first["rule_id"]) == (
        "UNKNOWN", "2026-10-28", "AS-Q-AGENT-FIRST-OR-PAPER")
    assert "earlier date" in first["missing"][0]


def test_agent_quarterly_without_the_previous_lodgment_method_is_unknown():
    result = rows("ias", "trust", "2026-27", "bas_agent", reporting_cycle="quarterly",
                  facts=Facts(lodges_electronically=True))
    assert result[0]["missing"] == [
        "previous_statement_lodged_electronically: the client's previous activity statement "
        "was lodged electronically; false for a first statement or a paper one"]
    no_method = rows("ias", "trust", "2026-27", "bas_agent", reporting_cycle="quarterly")
    assert no_method[2]["missing"][0].startswith("lodges_electronically:")


def test_a_company_must_say_whether_it_is_a_head_company():
    result = rows("bas", "company", "2026-27", "tax_agent", reporting_cycle="quarterly",
                  facts=AGENT_READY)
    assert [row["status"] for row in result] == ["UNKNOWN"] * 4
    assert result[1]["missing"][0].startswith("consolidated_head_company:")


def test_head_company_quarterly_ias_is_due_on_the_21st():
    result = rows("ias", "company", "2026-27", "self", reporting_cycle="quarterly",
                  facts=Facts(consolidated_head_company=True))
    assert summary(result) == [
        ("2026-27 Q1", "DETERMINED", "2026-10-21", "2026-10-21", "AS-Q-HEAD"),
        ("2026-27 Q2", "DETERMINED", "2027-01-21", "2027-01-21", "AS-Q-HEAD"),
        ("2026-27 Q3", "DETERMINED", "2027-04-21", "2027-04-21", "AS-Q-HEAD"),
        ("2026-27 Q4", "UNKNOWN", "2027-07-21", None, "AS-Q-HEAD"),
    ]


def test_head_company_quarterly_bas_is_left_unknown():
    result = rows("bas", "company", "2026-27", "tax_agent", reporting_cycle="quarterly",
                  facts=Facts(consolidated_head_company=True))
    assert {(row["status"], row["rule_id"]) for row in result} == {("UNKNOWN", "AS-Q-HEAD-BAS")}
    assert result[0]["nominal_due_date"] is None


def test_monthly_bas_by_the_trust_itself():
    result = rows("bas", "trust", "2026-27", "self", reporting_cycle="monthly")
    assert [row["period"] for row in result] == [
        "2026-07", "2026-08", "2026-09", "2026-10", "2026-11", "2026-12",
        "2027-01", "2027-02", "2027-03", "2027-04", "2027-05", "2027-06"]
    assert [row["due_date"] for row in result] == [
        "2026-08-21", "2026-09-21", "2026-10-21", "2026-11-23", "2026-12-21", "2027-01-21",
        "2027-02-22", "2027-03-22", "2027-04-21", "2027-05-21", "2027-06-21", None]
    assert result[-1]["nominal_due_date"] == "2027-07-21"
    assert result[0]["obligation"] == "BAS July 2026"
    assert {row["rule_id"] for row in result} == {"AS-M-21"}


def test_monthly_ias_follows_the_same_dates_with_no_december_concession():
    result = rows("ias", "company", "2026-27", "tax_agent", reporting_cycle="monthly",
                  facts=DECEMBER_READY)
    assert result[5]["due_date"] == "2027-01-21"
    assert result[5]["source_url"].endswith("activity-statement-generate-dates")


def test_december_bas_concession_for_an_agent_client():
    result = rows("bas", "company", "2026-27", "bas_agent", reporting_cycle="monthly",
                  facts=DECEMBER_READY)
    december = result[5]
    assert (december["period"], december["rule_id"], december["nominal_due_date"],
            december["due_date"]) == ("2026-12", "AS-M-DEC-AGENT", "2027-02-21", "2027-02-22")
    assert sum(row["status"] == "DETERMINED" for row in result) == 11


@pytest.mark.parametrize("change", [
    {"annual_turnover_up_to_10m": False},
    {"deferred_gst_registered": True},
    {"lodges_electronically": False},
])
def test_december_bas_outside_the_concession_is_due_21_january(change):
    facts = Facts(**{**DECEMBER_READY.__dict__, **change})
    december = rows("bas", "trust", "2026-27", "tax_agent", reporting_cycle="monthly",
                    facts=facts)[5]
    assert (december["rule_id"], december["due_date"]) == ("AS-M-21", "2027-01-21")


def test_december_bas_after_a_paper_statement_is_unknown():
    facts = Facts(**{**DECEMBER_READY.__dict__, "previous_statement_lodged_electronically": False})
    december = rows("bas", "trust", "2026-27", "tax_agent", reporting_cycle="monthly",
                    facts=facts)[5]
    assert (december["status"], december["nominal_due_date"], december["rule_id"]) == (
        "UNKNOWN", "2027-01-21", "AS-M-DEC-AGENT-FIRST-OR-PAPER")


def test_december_bas_without_the_concession_facts_names_each_one():
    december = rows("bas", "trust", "2026-27", "tax_agent", reporting_cycle="monthly")[5]
    assert december["status"] == "UNKNOWN"
    assert [item.split(":")[0] for item in december["missing"]] == [
        "annual_turnover_up_to_10m", "deferred_gst_registered", "lodges_electronically",
        "previous_statement_lodged_electronically"]
    assert december["rule_id"] == "BAS-AGENT"


def test_a_self_lodger_gets_no_december_concession():
    december = rows("bas", "trust", "2026-27", "self", reporting_cycle="monthly",
                    facts=DECEMBER_READY)[5]
    assert december["due_date"] == "2027-01-21"


def test_periods_outside_2026_27_are_not_recorded():
    result = rows("bas", "trust", "2027-28", "self", reporting_cycle="quarterly")
    assert summary(result) == [("2027-28", "UNKNOWN", None, None, "YEAR-NOT-RECORDED")]
    assert result[0]["obligation"] == "BAS 2027-28"
    assert result[0]["missing"] == [
        "rules for 2027-28: this release records only 2026-27 for this obligation"]
