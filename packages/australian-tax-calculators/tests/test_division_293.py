"""Division 293 tax against ITAA 1997 s 293-20 and the ATO's worked example.

Expected amounts are typed in from the ATO page or derived by hand from the
provision; none is produced by the engine's own data.
"""

from decimal import Decimal as D

import pytest
from austaxcalc import calculations as c


def _tax(income, contributions, year="2025-26"):
    return c.division_293(D(income), D(contributions), year, True)["amounts"]


def test_ato_example_jan():
    # $240,000 income and $15,000 contributions: the $5,000 excess is the lesser.
    amounts = _tax("240000", "15000")
    assert amounts["combined_income_and_contributions"] == "255000.00"
    assert amounts["taxable_contributions"] == "5000.00"
    assert amounts["division_293_tax"] == "750.00"


@pytest.mark.parametrize("income,contributions,taxable,tax", [
    ("230000.00", "20000.00", "0.00", "0.00"),       # exactly $250,000 does not exceed it
    ("230000.00", "20000.10", "0.10", "0.02"),       # 15% of 10 cents, rounded half up
    ("300000.00", "27500.00", "27500.00", "4125.00"),  # contributions are the lesser
    ("400000.00", "0.00", "0.00", "0.00"),           # s 293-20(2): nil contributions
    ("0.00", "260000.00", "10000.00", "1500.00"),
])
def test_lesser_of_contributions_and_excess(income, contributions, taxable, tax):
    amounts = _tax(income, contributions)
    assert (amounts["taxable_contributions"], amounts["division_293_tax"]) == (taxable, tax)


@pytest.mark.parametrize("year", ["2024-25", "2025-26", "2026-27"])
def test_each_supported_year_uses_the_250000_threshold(year):
    result = c.division_293(D("245000"), D("30000"), year, True)
    assert result["amounts"]["division_293_tax"] == "3750.00"
    assert result["rates"] == {"threshold": "250000", "rate": "0.15"}


def test_tax_never_exceeds_15_percent_of_contributions():
    for income in range(200000, 300001, 2500):
        for contributions in ("0", "1000", "25000", "60000"):
            tax = D(_tax(str(income), contributions)["division_293_tax"])
            assert D(0) <= tax <= D(contributions) * D("0.15")


def test_negative_and_sub_cent_amounts_are_refused():
    with pytest.raises(ValueError, match="Amounts"):
        c.division_293(D("-1"), D("1000"), "2025-26", True)
    with pytest.raises(ValueError, match="Amounts"):
        c.division_293(D("260000"), D("1000.001"), "2025-26", True)
