"""Low income tax offset against ITAA 1997 s 61-115(1) and the ATO's worked example.

Expected amounts are typed in from the ATO page or derived by hand from the
provision's table; none is produced by the engine's own data.
"""

from decimal import ROUND_CEILING
from decimal import Decimal as D

import pytest
from austaxcalc import calculations as c
from austaxcalc.metadata import LOW_INCOME_TAX_OFFSET


def _offset(income, year="2025-26"):
    return c.low_income_tax_offset(D(income), year, True)["amounts"]["low_income_tax_offset"]


def test_ato_example_jeff():
    # $45,000: $700 less 5 cents for each dollar above $37,500 is $325.
    assert _offset("45000") == "325.00"


@pytest.mark.parametrize("income,expected", [
    ("0", "700.00"), ("37500", "700.00"),
    ("37501", "699.95"),      # item 2: 5% of the $1 excess
    ("40000", "575.00"),
    ("45001", "324.99"),      # item 3: 1.5% of $1 leaves 324.985, rounded half up
    ("50000", "250.00"),
    ("66666", "0.01"),        # 325 - 1.5% x 21,666 = 0.01
    ("66667", "0.00"),        # 325 - 1.5% x 21,667 is below nil
    ("66668", "0.00"), ("180000", "0.00"),
])
def test_table_items_and_boundaries(income, expected):
    assert _offset(income) == expected


def test_second_step_amount_and_cut_out_follow_from_the_table():
    rule = {key: D(value) for key, value in LOW_INCOME_TAX_OFFSET.items()}
    assert rule["second_amount"] == rule["maximum"] - rule["first_rate"] * (
        rule["second_step"] - rule["first_step"])
    # s 61-110(1)(b): $66,667 is the first whole dollar at which item 3 reaches nil.
    reaches_nil = rule["second_step"] + rule["second_amount"] / rule["second_rate"]
    assert rule["cut_out"] == reaches_nil.to_integral_value(rounding=ROUND_CEILING)


@pytest.mark.parametrize("year", ["2024-25", "2025-26", "2026-27"])
def test_each_supported_year_applies_the_same_table(year):
    assert _offset("41000", year) == "525.00"


def test_offset_never_rises_with_income():
    previous = D(700)
    for income in range(37000, 67001, 7):
        offset = D(_offset(str(income)))
        assert D(0) <= offset <= previous
        previous = offset


def test_cents_are_refused():
    with pytest.raises(ValueError, match="whole-dollar"):
        c.low_income_tax_offset(D("40000.10"), "2025-26", True)
