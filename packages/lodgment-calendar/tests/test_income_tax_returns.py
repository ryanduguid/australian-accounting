"""Income tax return due dates for the 2025-26 income year, for fictional entities.

Every entity named here is synthetic: Synthetic Grevillea (an individual), Synthetic
Wattle Trust, Synthetic Saltbush Partners, Synthetic Banksia Pty Ltd, Synthetic
Ironbark Holdings (a consolidated group's head company) and the Synthetic Mallee
Super Fund (an SMSF).
"""

import pytest
from lodgmentcalendar import Facts, due_dates

COVERED = {"on_agent_client_list_by_cutoff": True, "prior_returns_outstanding": False}


def rows(entity_type, channel, **facts):
    return due_dates("income_tax_return", entity_type, "2025-26", channel,
                     facts=Facts(**facts))["obligations"]


def brief(result_rows):
    return [(row["kind"], row["status"], row["nominal_due_date"], row["due_date"],
             row["rule_id"]) for row in result_rows]


def names(row):
    return [item.split(":")[0] for item in row["missing"]]


# Synthetic Grevillea --------------------------------------------------------------

def test_self_lodged_individual_return_rolls_to_monday_2_november():
    result = rows("individual", "self")
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2026-10-31", "2026-11-02", "ITR-SELF-IND"),
        ("payment", "UNKNOWN", "2026-11-21", None, "PAY-SELF-IND"),
    ]
    assert result[0]["payment"] == "separate"
    assert result[1]["missing"][0].startswith("the lodgement date and the assessment issue")
    assert "Lodged late: pay by 21 November" in result[1]["summary"]
    assert "payment" not in result[1]


def test_agent_individual_return_due_15_may_rolls_to_monday():
    result = rows("individual", "tax_agent", latest_tax_liability_20k_or_more=False, **COVERED)
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2027-05-15", "2027-05-17", "ITR-AGENT-IT-15MAY"),
        ("payment", "UNKNOWN", None, None, "PAY-IT-15MAY"),
        ("concession", "UNKNOWN", "2027-06-05", None, "CON-IT-5JUN"),
    ]
    assert "21 March 2027" in result[1]["summary"]
    assert "next-business-day rule extends this concession" in result[2]["missing"][0]
    assert result[0]["source_url"].endswith("/individuals-and-trusts")


def test_agent_individual_with_a_large_prior_liability_lodges_by_31_march():
    result = rows("individual", "tax_agent", latest_tax_liability_20k_or_more=True, **COVERED)
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2027-03-31", "2027-03-31", "ITR-AGENT-IT-31MAR"),
        ("payment", "UNKNOWN", None, None, "PAY-NOA"),
    ]


OVERDUE = {"on_agent_client_list_by_cutoff": True, "prior_returns_outstanding": True,
           "overdue_returns_lodged_by_31_october": False}


def test_agent_individual_with_overdue_returns_lodges_by_31_october():
    result = rows("individual", "tax_agent", **OVERDUE)
    assert brief(result)[0] == ("lodgment", "DETERMINED", "2026-10-31", "2026-11-02",
                                "ITR-AGENT-OVERDUE")
    assert result[1]["rule_id"] == "PAY-NOA"


def test_overdue_returns_lodged_by_31_october_restore_the_program_date():
    cleared = rows("individual", "tax_agent", latest_tax_liability_20k_or_more=False,
                   **{**OVERDUE, "overdue_returns_lodged_by_31_october": True})
    assert cleared[0]["rule_id"] == "ITR-AGENT-IT-15MAY"
    unsure = rows("individual", "tax_agent", latest_tax_liability_20k_or_more=False,
                  **{**OVERDUE, "overdue_returns_lodged_by_31_october": None})
    assert names(unsure[0]) == ["overdue_returns_lodged_by_31_october"]


@pytest.mark.parametrize("entity_type", ["individual", "trust"])
def test_a_new_registrant_needs_no_latest_return_test(entity_type):
    result = rows(entity_type, "tax_agent", new_registrant=True, large_or_medium=False,
                  **COVERED)
    assert (result[0]["rule_id"], result[0]["due_date"]) == ("ITR-AGENT-IT-15MAY", "2027-05-17")


def test_agent_individual_with_no_facts_names_every_missing_condition():
    result = rows("individual", "tax_agent")
    assert brief(result) == [("lodgment", "UNKNOWN", None, None, "INDIVIDUAL-AGENT")]
    assert names(result[0]) == ["on_agent_client_list_by_cutoff", "prior_returns_outstanding",
                                "latest_tax_liability_20k_or_more"]
    assert result[0]["payment"] == "unknown"


def test_a_client_added_after_31_october_is_outside_the_program():
    result = rows("individual", "tax_agent", on_agent_client_list_by_cutoff=False)
    assert brief(result) == [("lodgment", "UNKNOWN", None, None, "ITR-AGENT-NOT-COVERED")]
    assert result[0]["source_url"].endswith("/october-2026")


# Synthetic Wattle Trust ------------------------------------------------------------

def test_self_lodged_trust_return():
    result = rows("trust", "self")
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2026-10-31", "2026-11-02", "ITR-SELF-TRUST"),
        ("payment", "UNKNOWN", None, None, "PAY-SELF-TRUST"),
    ]


@pytest.mark.parametrize("facts,rule,nominal,due", [
    ({"prior_year_return_taxable": True, "new_registrant": False},
     "ITR-AGENT-TRUST-LM-31JAN", "2027-01-31", "2027-02-01"),
    ({"prior_year_return_taxable": False, "new_registrant": False},
     "ITR-AGENT-TRUST-LM-28FEB", "2027-02-28", "2027-03-02"),
    ({"new_registrant": True}, "ITR-AGENT-TRUST-LM-NEW", "2027-02-28", "2027-03-02"),
])
def test_agent_large_or_medium_trust(facts, rule, nominal, due):
    result = rows("trust", "tax_agent", large_or_medium=True, **facts, **COVERED)
    assert brief(result)[0] == ("lodgment", "DETERMINED", nominal, due, rule)
    assert result[1]["rule_id"] == "PAY-NOA"


def test_agent_large_or_medium_trust_needs_its_registration_and_taxable_status():
    result = rows("trust", "tax_agent", large_or_medium=True, **COVERED)
    assert names(result[0]) == ["new_registrant", "prior_year_return_taxable"]
    partly = rows("trust", "tax_agent", large_or_medium=True, prior_year_return_taxable=True,
                  **COVERED)
    assert names(partly[0]) == ["new_registrant"]


def test_agent_trust_not_large_or_medium_follows_the_individual_dates():
    result = rows("trust", "tax_agent", large_or_medium=False,
                  latest_tax_liability_20k_or_more=False, **COVERED)
    assert [row["rule_id"] for row in result] == [
        "ITR-AGENT-IT-15MAY", "PAY-IT-15MAY", "CON-IT-5JUN"]


# Synthetic Saltbush Partners --------------------------------------------------------

def test_self_lodged_partnership_return_has_no_payment():
    result = rows("partnership", "self")
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2026-10-31", "2026-11-02", "ITR-SELF-PARTNERSHIP")]
    assert result[0]["payment"] == "none"


def test_agent_partnership_return_and_its_unsettled_concession():
    result = rows("partnership", "tax_agent", **COVERED)
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2027-05-15", "2027-05-17", "ITR-AGENT-PART-15MAY"),
        ("concession", "UNKNOWN", "2027-06-05", None, "CON-PART-5JUN"),
    ]
    assert "next-business-day rule extends this concession" in result[1]["missing"][0]
    assert result[1]["source_url"].endswith("/individuals-and-trusts")


def test_a_partnership_pays_no_tax_even_while_its_date_is_unknown():
    result = rows("partnership", "tax_agent")
    assert [(row["status"], row["payment"]) for row in result] == [("UNKNOWN", "none")]


def test_agent_partnership_with_overdue_returns():
    result = rows("partnership", "tax_agent", **OVERDUE)
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2026-10-31", "2026-11-02", "ITR-AGENT-OVERDUE")]
    assert result[0]["source_url"].endswith("/october-2026")


# Synthetic Banksia Pty Ltd ----------------------------------------------------------

BANKSIA = {"consolidated_head_company": False, **COVERED}


@pytest.mark.parametrize("facts,lodgment,payment", [
    ({"large_or_medium": True, "new_registrant": False, "prior_year_return_taxable": True},
     ("ITR-AGENT-CO-LM-31JAN", "2027-02-01"), ("PAY-1DEC-2026", "2026-12-01")),
    ({"large_or_medium": True, "new_registrant": False, "prior_year_return_taxable": False},
     ("ITR-AGENT-CO-LM-28FEB", "2027-03-02"), None),
    ({"large_or_medium": True, "new_registrant": True},
     ("ITR-AGENT-CO-LM-NEW", "2027-03-02"), None),
    ({"large_or_medium": False, "total_income_over_2m": True},
     ("ITR-AGENT-CO-31MAR", "2027-03-31"), None),
])
def test_agent_company_return(facts, lodgment, payment):
    result = rows("company", "tax_agent", **facts, **BANKSIA)
    assert (result[0]["rule_id"], result[0]["due_date"]) == lodgment
    if payment is None:
        assert len(result) == 1 and result[0]["payment"] == "with_lodgment"
    else:
        assert (result[1]["rule_id"], result[1]["due_date"]) == payment


def test_agent_company_with_overdue_returns_pays_by_1_december():
    result = rows("company", "tax_agent", consolidated_head_company=False, **OVERDUE)
    assert [(row["rule_id"], row["due_date"]) for row in result] == [
        ("ITR-AGENT-OVERDUE", "2026-11-02"), ("PAY-1DEC-2026", "2026-12-01")]
    assert result[1]["source_url"].endswith("/taxpayers-with-overdue-tax-returns")


def test_a_new_registrant_company_needs_no_income_threshold():
    result = rows("company", "tax_agent", large_or_medium=False, new_registrant=True,
                  prior_year_non_taxable_or_refund=False, **BANKSIA)
    assert [(row["rule_id"], row["due_date"]) for row in result] == [
        ("ITR-AGENT-CO-15MAY", "2027-05-17")]


def test_agent_company_due_15_may_with_the_refund_concession():
    result = rows("company", "tax_agent", large_or_medium=False, total_income_over_2m=False,
                  prior_year_non_taxable_or_refund=True, current_year_non_taxable_or_refund=True,
                  **BANKSIA)
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2027-05-15", "2027-05-17", "ITR-AGENT-CO-15MAY"),
        ("concession", "UNKNOWN", "2027-06-05", None, "CON-CO-5JUN"),
    ]


def test_agent_company_concession_drops_out_or_waits_on_its_facts():
    taxable = rows("company", "tax_agent", large_or_medium=False, total_income_over_2m=False,
                   prior_year_non_taxable_or_refund=True,
                   current_year_non_taxable_or_refund=False, **BANKSIA)
    assert [row["kind"] for row in taxable] == ["lodgment"]
    unsure = rows("company", "tax_agent", large_or_medium=False, total_income_over_2m=False,
                  **BANKSIA)
    assert names(unsure[1]) == ["prior_year_non_taxable_or_refund",
                                "current_year_non_taxable_or_refund"]


def test_self_lodged_company_28_february_is_left_unknown():
    # The page gives 28 February 'generally' for 'small companies' without defining them.
    result = rows("company", "self", consolidated_head_company=False,
                  prior_returns_outstanding=False)
    assert brief(result) == [("lodgment", "UNKNOWN", "2027-02-28", None, "ITR-SELF-CO")]
    assert "does not define" in result[0]["missing"][0]
    assert result[0]["payment"] == "unknown"


def test_self_lodged_head_company_date_is_not_published():
    result = rows("company", "self", consolidated_head_company=True)
    assert brief(result) == [("lodgment", "UNKNOWN", None, None, "ITR-SELF-CO-HEAD")]
    assert result[0]["payment"] == "unknown"


def test_self_lodged_company_with_overdue_returns():
    result = rows("company", "self", consolidated_head_company=False,
                  prior_returns_outstanding=True)
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2026-10-31", "2026-11-02", "ITR-SELF-CO-OVERDUE"),
        ("payment", "UNKNOWN", None, None, "PAY-SELF-CO-OVERDUE"),
    ]
    assert "applies to a self-lodger" in result[1]["missing"][0]


# Synthetic Ironbark Holdings, a head company ----------------------------------------

IRONBARK = {"consolidated_head_company": True, "first_consolidated_return": False, **COVERED}


SMALL_GROUP = {"member_large_or_medium": False, "large_or_medium": False}


@pytest.mark.parametrize("facts,rule,due,payment", [
    ({"member_large_or_medium": True, "large_or_medium": True,
      "current_year_return_taxable": True},
     "ITR-AGENT-HEAD-LM-31JAN", "2027-02-01", "separate"),
    ({"member_large_or_medium": True, "current_year_return_taxable": False},
     "ITR-AGENT-HEAD-LM-28FEB", "2027-03-02", "none"),
    ({**SMALL_GROUP, "new_registrant": False, "total_income_over_2m": True},
     "ITR-AGENT-HEAD-31MAR", "2027-03-31", "with_lodgment"),
    ({**SMALL_GROUP, "new_registrant": False, "total_income_over_2m": False},
     "ITR-AGENT-HEAD-15MAY", "2027-05-17", "with_lodgment"),
])
def test_agent_head_company_return(facts, rule, due, payment):
    result = rows("company", "tax_agent", **facts, **IRONBARK)
    assert (result[0]["rule_id"], result[0]["due_date"], result[0]["payment"]) == (
        rule, due, payment)
    assert all(row["kind"] != "concession" for row in result)


@pytest.mark.parametrize("facts,rule", [
    ({"first_consolidated_return": True}, "ITR-AGENT-HEAD-FIRST"),
    ({**SMALL_GROUP, "new_registrant": True}, "ITR-AGENT-HEAD-NEW"),
    ({"member_large_or_medium": True, "large_or_medium": False,
      "current_year_return_taxable": True}, "ITR-AGENT-HEAD-LM-SPLIT"),
    ({"member_large_or_medium": False, "large_or_medium": True}, "ITR-AGENT-HEAD-LM-SPLIT"),
])
def test_agent_head_company_dates_left_unknown(facts, rule):
    result = rows("company", "tax_agent", **{**IRONBARK, **facts})
    assert brief(result) == [("lodgment", "UNKNOWN", None, None, rule)]


def test_agent_head_company_needs_its_taxable_status_and_overdue_returns_come_first():
    unsure = rows("company", "tax_agent", member_large_or_medium=True, **IRONBARK)
    assert names(unsure[0]) == ["current_year_return_taxable", "large_or_medium"]
    group_only = rows("company", "tax_agent", large_or_medium=False, **IRONBARK)
    assert names(group_only[0])[0] == "member_large_or_medium"
    assert group_only[0]["status"] == "UNKNOWN"
    group_unsure = rows("company", "tax_agent", member_large_or_medium=True,
                        current_year_return_taxable=True, **IRONBARK)
    assert names(group_unsure[0]) == ["large_or_medium"]
    member_unsure = rows("company", "tax_agent", large_or_medium=True, **IRONBARK)
    assert names(member_unsure[0]) == ["member_large_or_medium"]
    overdue = rows("company", "tax_agent", **{**IRONBARK, **OVERDUE})
    assert [row["rule_id"] for row in overdue] == ["ITR-AGENT-OVERDUE", "PAY-1DEC-2026"]


# Synthetic Mallee Super Fund --------------------------------------------------------

MALLEE = COVERED


@pytest.mark.parametrize("facts,lodgment,payment", [
    ({"large_or_medium": True, "new_registrant": True}, ("ITR-AGENT-SMSF-NEW", "2027-03-02"),
     None),
    ({"large_or_medium": True, "new_registrant": False, "prior_year_return_taxable": True},
     ("ITR-AGENT-SMSF-LM-31JAN", "2027-02-01"), ("PAY-1DEC-2026", "2026-12-01")),
    ({"large_or_medium": True, "new_registrant": False, "prior_year_return_taxable": False},
     ("ITR-AGENT-SMSF-LM-28FEB", "2027-03-02"), None),
    ({"large_or_medium": False, "new_registrant": True}, ("ITR-AGENT-SMSF-NEW", "2027-03-02"),
     None),
    ({"large_or_medium": False, "new_registrant": False, "total_income_over_2m": True},
     ("ITR-AGENT-SMSF-31MAR", "2027-03-31"), None),
])
def test_agent_smsf_return(facts, lodgment, payment):
    result = rows("smsf", "tax_agent", **facts, **MALLEE)
    assert (result[0]["rule_id"], result[0]["due_date"]) == lodgment
    if payment is None:
        assert len(result) == 1
    else:
        assert (result[1]["rule_id"], result[1]["due_date"]) == payment


def test_agent_smsf_due_15_may_with_its_concession_and_overdue_case():
    result = rows("smsf", "tax_agent", large_or_medium=False, new_registrant=False,
                  total_income_over_2m=False, prior_year_non_taxable_or_refund=True,
                  current_year_non_taxable_or_refund=True, **MALLEE)
    assert [row["rule_id"] for row in result] == ["ITR-AGENT-SMSF-15MAY", "CON-SMSF-5JUN"]
    overdue = rows("smsf", "tax_agent", **OVERDUE)
    assert [(row["rule_id"], row["due_date"]) for row in overdue] == [
        ("ITR-AGENT-OVERDUE", "2026-11-02"), ("PAY-1DEC-2026", "2026-12-01")]
    assert overdue[0]["source_url"].endswith("/super-lodgment")
    assert overdue[1]["source_url"].endswith("/taxpayers-with-overdue-tax-returns")


def test_a_new_registrant_smsf_needs_no_size_test():
    result = rows("smsf", "tax_agent", new_registrant=True, **MALLEE)
    assert [(row["rule_id"], row["due_date"]) for row in result] == [
        ("ITR-AGENT-SMSF-NEW", "2027-03-02")]


@pytest.mark.parametrize("facts", [{"new_registrant": True},
                                   {"new_registrant": False, "prior_returns_outstanding": True},
                                   {"prior_returns_outstanding": True}])
def test_self_preparing_smsf_takes_the_earliest_31_october_date(facts):
    result = rows("smsf", "self", **facts)
    assert brief(result) == [
        ("lodgment", "DETERMINED", "2026-10-31", "2026-11-02", "ITR-SELF-SMSF-31OCT"),
        ("payment", "DETERMINED", "2026-12-01", "2026-12-01", "PAY-SELF-SMSF-1DEC"),
    ]


@pytest.mark.parametrize("facts,rule,due", [
    ({"large_or_medium": True, "current_year_return_taxable": True}, "ITR-SELF-SMSF-31JAN",
     "2027-02-01"),
    ({"large_or_medium": True, "current_year_return_taxable": False}, "ITR-SELF-SMSF-28FEB",
     "2027-03-02"),
    ({"large_or_medium": False}, "ITR-SELF-SMSF-28FEB", "2027-03-02"),
])
def test_self_preparing_smsf_later_dates(facts, rule, due):
    result = rows("smsf", "self", new_registrant=False, prior_returns_outstanding=False, **facts)
    assert (result[0]["rule_id"], result[0]["due_date"]) == (rule, due)


def test_self_preparing_smsf_with_a_non_taxable_return_needs_no_size_test():
    result = rows("smsf", "self", new_registrant=False, prior_returns_outstanding=False,
                  current_year_return_taxable=False)
    assert (result[0]["rule_id"], result[0]["due_date"]) == ("ITR-SELF-SMSF-28FEB", "2027-03-02")


def test_self_preparing_smsf_with_facts_missing():
    assert names(rows("smsf", "self", new_registrant=False)[0]) == [
        "prior_returns_outstanding", "large_or_medium"]
    unsure = rows("smsf", "self", new_registrant=False, prior_returns_outstanding=False,
                  large_or_medium=True)
    assert names(unsure[0]) == ["current_year_return_taxable"]


# Financial-year boundaries and the result envelope ----------------------------------

@pytest.mark.parametrize("year", ["2024-25", "2026-27"])
def test_returns_outside_the_2025_26_income_year_are_not_recorded(year):
    result = due_dates("income_tax_return", "individual", year, "self")["obligations"]
    assert [(row["rule_id"], row["status"], row["period"]) for row in result] == [
        ("YEAR-NOT-RECORDED", "UNKNOWN", year)]
    assert "covers 2025-26" in result[0]["summary"]


def test_result_envelope_carries_provenance_and_counts():
    result = due_dates("income_tax_return", "individual", "2025-26", "self")
    assert result["ok"] is True and result["engine"] == "lodgment-calendar"
    assert result["program_year"] == "2026-27" and result["source_checked"] == "2026-10-31"
    assert result["counts"] == {"DETERMINED": 1, "UNKNOWN": 1}
    assert result["request"] == {"obligation": "income_tax_return", "entity_type": "individual",
                                 "financial_year": "2025-26", "channel": "self",
                                 "reporting_cycle": None}
    assert result["facts_used"]["prior_returns_outstanding"] is None
    assert result["holiday_calendar"]["covers_to"] == "2027-06-30"
    assert result["not_decided"] and result["warnings"]
    for row in result["obligations"]:
        assert row["source_url"].startswith("https://www.ato.gov.au/")
        assert row["source_provision"] and row["source_checked"] == "2026-10-31"
