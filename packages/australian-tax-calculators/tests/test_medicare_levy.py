"""Medicare levy for a single against the Act and the ATO's published thresholds.

Expected amounts are typed in from the ATO's pages or derived by hand from the
Medicare Levy Act 1986 thresholds; none is produced by the engine's own data.
"""

from decimal import Decimal as D

import pytest
from austaxcalc import calculations as c
from austaxcalc.metadata import MEDICARE_LOW_INCOME_THRESHOLDS

# ATO single thresholds for a person not entitled to SAPTO: (lower, upper).
PUBLISHED = {"2024-25": ("27222", "34027"), "2025-26": ("28011", "35013")}


def _levy(year, income):
    return c.medicare_levy(D(income), False, year, True)["amounts"]


def test_ato_example_angie():
    # $29,000 in 2025-26, single, no SAPTO, no exemption: levy reduced to $98.90.
    assert _levy("2025-26", "29000")["medicare_levy"] == "98.90"


@pytest.mark.parametrize("year,income,expected", [
    ("2025-26", "0", "0.00"),
    ("2025-26", "28011", "0.00"),        # s 7(1): at the threshold amount, no levy
    ("2025-26", "28012", "0.10"),        # s 7(2): 10% of the $1 excess
    ("2025-26", "35013", "700.20"),      # phase-in limit: 10% of $7,002, below 2%
    ("2025-26", "35014", "700.28"),      # above the limit: 2% of the whole
    ("2025-26", "100000", "2000.00"),
    ("2024-25", "27222", "0.00"),
    ("2024-25", "30000", "277.80"),
    ("2024-25", "34027", "680.50"),
    ("2024-25", "34028", "680.56"),
])
def test_threshold_and_phase_in_boundaries(year, income, expected):
    assert _levy(year, income)["medicare_levy"] == expected


def test_reduction_is_the_difference_from_the_full_rate():
    amounts = _levy("2025-26", "29000")
    assert amounts["levy_at_full_rate"] == "580.00"
    assert amounts["low_income_reduction"] == "481.10"
    assert _levy("2025-26", "50000")["low_income_reduction"] == "0.00"


@pytest.mark.parametrize("year", PUBLISHED)
def test_thresholds_match_the_published_table(year):
    assert MEDICARE_LOW_INCOME_THRESHOLDS[year] == PUBLISHED[year]


@pytest.mark.parametrize("year", PUBLISHED)
def test_phase_in_limit_is_where_ten_percent_of_the_excess_meets_two_percent(year):
    # 10% x (income - threshold) <= 2% x income up to income = 1.25 x threshold;
    # the published upper threshold is that point rounded down to the dollar.
    lower, upper = (D(value) for value in PUBLISHED[year])
    assert upper == (lower * D("1.25")).to_integral_value(rounding="ROUND_FLOOR")


@pytest.mark.parametrize("year", PUBLISHED)
def test_levy_never_falls_as_income_rises(year):
    lower, upper = (int(value) for value in PUBLISHED[year])
    previous = D(0)
    for income in range(lower - 2, upper + 3):
        levy = D(_levy(year, str(income))["medicare_levy"])
        assert previous <= levy <= D(income) * D("0.02")
        previous = levy


def test_unpublished_year_and_cents_are_refused():
    with pytest.raises(ValueError, match="Unsupported period"):
        c.medicare_levy(D("50000"), False, "2026-27", True)
    with pytest.raises(ValueError, match="whole-dollar"):
        c.medicare_levy(D("50000.50"), False, "2025-26", True)


def test_result_names_the_rates_applied():
    rates = c.medicare_levy(D("30000"), False, "2025-26", True)["rates"]
    assert rates == {"levy_rate": "0.02", "shade_in_rate": "0.10",
                     "threshold_amount": "28011", "phase_in_limit": "35013"}


def test_section_9a_lump_sum_in_arrears_is_refused():
    # 2025-26: $40,000 taxable income includes $12,000 of eligible arrears from two
    # earlier years, so normal taxable income is $28,000. Under s 9A(2) the levy is
    # worked out on $28,000, below the $28,011 threshold, so the law gives nil; the
    # worksheet refuses rather than return $800.
    with pytest.raises(ValueError, match="s 9A") as refused:
        c.medicare_levy(D("40000"), True, "2025-26", True)
    assert "lump-sum-payment-in-arrears" in str(refused.value)


def test_without_section_9a_the_standard_levy_applies():
    assert _levy("2025-26", "40000")["medicare_levy"] == "800.00"


def test_section_9a_flag_is_required_and_boolean():
    with pytest.raises(TypeError):
        c.medicare_levy(D("40000"), year="2025-26", scope_confirmed=True)
    for flag in (None, 0, "false"):
        with pytest.raises(ValueError, match="section_9a_adjustment_applies"):
            c.medicare_levy(D("40000"), flag, "2025-26", True)
