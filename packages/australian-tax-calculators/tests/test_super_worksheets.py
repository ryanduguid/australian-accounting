"""Contribution caps and pension minimums against the ATO's published figures.

The expected values are the ATO's own tables, typed in here, not values derived
from the engine's data. Derivations are in docs/calculation-evidence.md.
"""

from decimal import Decimal as D

import pytest
from austaxcalc import calculations as c

# ATO "Bring-forward cap first year" tables for each year: (3-year band ceiling,
# 2-year band ceiling, general transfer balance cap, annual non-concessional cap).
BRING_FORWARD_TABLES = {
    "2024-25": ("1660000", "1780000", "1900000", "120000"),
    "2025-26": ("1760000", "1880000", "2000000", "120000"),
    "2026-27": ("1840000", "1970000", "2100000", "130000"),
}
CONCESSIONAL_CAPS = {"2024-25": "30000", "2025-26": "30000", "2026-27": "32500"}


def _caps(year, balance, concessional="0", unused="0", non_concessional="0", under_75=True):
    return c.contribution_caps(D(balance), D(concessional), D(unused), D(non_concessional),
                               under_75, year, True)


@pytest.mark.parametrize("year", BRING_FORWARD_TABLES)
def test_first_year_cap_matches_the_published_table_at_every_boundary(year):
    three, two, transfer_cap, annual = (D(value) for value in BRING_FORWARD_TABLES[year])
    cent = D("0.01")
    for balance, multiple in ((three - cent, 3), (three, 2), (two - cent, 2), (two, 1),
                              (transfer_cap - cent, 1), (transfer_cap, 0)):
        result = _caps(year, balance)
        assert result["amounts"]["non_concessional_available"] == f"{annual * multiple:.2f}"
        assert result["rates"]["non_concessional_cap_multiple"] == str(multiple)
    assert result["rates"]["non_concessional_cap"] == str(annual)
    assert result["rates"]["concessional_cap"] == CONCESSIONAL_CAPS[year]


@pytest.mark.parametrize("year", BRING_FORWARD_TABLES)
def test_75_or_older_all_year_keeps_only_the_annual_cap(year):
    annual = BRING_FORWARD_TABLES[year][3]
    result = _caps(year, "100000", non_concessional=annual + ".01", under_75=False)
    assert result["amounts"]["non_concessional_available"] == annual + ".00"
    assert result["amounts"]["excess_non_concessional"] == "0.01"
    assert result["rates"]["bring_forward_period_years"] == "0"


def test_bring_forward_triggers_only_above_the_annual_cap():
    at_cap = _caps("2026-27", "1000000", non_concessional="130000")
    above = _caps("2026-27", "1000000", non_concessional="130000.01")
    two_year = _caps("2026-27", "1900000", non_concessional="200000")
    assert at_cap["rates"]["bring_forward_period_years"] == "0"
    assert above["rates"]["bring_forward_period_years"] == "3"
    assert two_year["rates"]["bring_forward_period_years"] == "2"
    assert two_year["amounts"]["non_concessional_remaining"] == "60000.00"


def test_nil_cap_at_the_transfer_balance_cap_makes_every_contribution_excess():
    result = _caps("2025-26", "2000000", non_concessional="1000")
    assert result["amounts"]["non_concessional_available"] == "0.00"
    assert result["amounts"]["excess_non_concessional"] == "1000.00"


def test_carry_forward_needs_a_balance_strictly_below_500000():
    below = _caps("2025-26", "499999.99", concessional="45000", unused="20000")
    at = _caps("2025-26", "500000", concessional="45000", unused="20000")
    assert below["amounts"] == {**below["amounts"], "carry_forward_applied": "20000.00",
                                "concessional_available": "50000.00",
                                "concessional_remaining": "5000.00",
                                "excess_concessional": "0.00"}
    assert at["amounts"]["carry_forward_applied"] == "0.00"
    assert at["amounts"]["excess_concessional"] == "15000.00"


def test_caps_refuse_a_non_boolean_age_condition():
    with pytest.raises(ValueError, match="boolean"):
        c.contribution_caps(D("0"), D("0"), D("0"), D("0"), 1, "2025-26", True)


# SISR Schedule 7 table: the factor changes on these birthdays.
@pytest.mark.parametrize("age,factor", [
    (0, "0.04"), (64, "0.04"), (65, "0.05"), (74, "0.05"), (75, "0.06"), (79, "0.06"),
    (80, "0.07"), (84, "0.07"), (85, "0.09"), (89, "0.09"), (90, "0.11"), (94, "0.11"),
    (95, "0.14"), (150, "0.14"),
])
def test_pension_factor_by_age(age, factor):
    result = c.pension_minimum(D("100000"), age, 365, "2025-26", True)
    assert result["rates"]["percentage_factor"] == factor
    assert result["amounts"]["minimum_payment"] == f"{D(100000) * D(factor):.2f}"


@pytest.mark.parametrize("balance,minimum", [
    ("100125.00", "4010.00"),  # exactly $4,005: an exact $5 rounds up
    ("100124.75", "4000.00"),  # $4,004.99 rounds to the nearest $10
    ("100250.00", "4010.00"),  # $4,010 needs no rounding
])
def test_pension_rounds_to_the_nearest_10_dollars(balance, minimum):
    result = c.pension_minimum(D(balance), 60, 365, "2026-27", True)
    assert result["amounts"]["minimum_payment"] == minimum


def test_pension_is_pro_rated_from_commencement_and_nil_from_1_june():
    # 181 days: commenced 1 January 2025. 31 days: 31 May. 30 days: 1 June.
    january = c.pension_minimum(D("250000"), 66, 181, "2024-25", True)
    may = c.pension_minimum(D("250000"), 66, 31, "2024-25", True)
    june = c.pension_minimum(D("250000"), 66, 30, "2024-25", True)
    assert january["amounts"]["minimum_payment"] == "6200.00"
    assert may["amounts"]["minimum_payment"] == "1060.00"
    assert june["amounts"]["minimum_payment"] == "0.00"
    assert june["rates"]["days"] == "30"


@pytest.mark.parametrize("age,days", [(True, 365), (-1, 365), (60, 0), (60, 366), (60, "365")])
def test_pension_refuses_impossible_age_or_days(age, days):
    with pytest.raises(ValueError, match="must be an integer"):
        c.pension_minimum(D("100000"), age, days, "2025-26", True)
