"""Study loan repayments against the ATO's published tables, examples and Gazette.

The expected values are typed in from the ATO page and derived by hand from
its thresholds; none is produced by the engine's own data. Derivations are in
docs/calculation-evidence.md.
"""

from decimal import Decimal as D

import pytest
from austaxcalc import calculations as c
from austaxcalc.metadata import STUDY_LOAN_REPAYMENT_THRESHOLDS

# ATO table 1 (2026-27) and table 2 (2025-26): minimum repayment income, top of
# the 15% band, printed base of the 17% band, first income repaying 10% of the whole.
PUBLISHED = {
    "2025-26": ("67000", "125000", "8700", "179286"),
    "2026-27": ("69528", "129717", "9028", "186051"),
}


def _repay(year, income):
    return c.study_loan_repayment(D(income), year, True)


@pytest.mark.parametrize("income,expected", [
    ("86380", "2527.80"),    # example 1: 15c for each $1 over $69,528
    ("254780", "25478.00"),  # example 3: 10% of the whole repayment income
])
def test_ato_worked_examples_for_2026_27(income, expected):
    assert _repay("2026-27", income)["amounts"]["compulsory_repayment"] == expected


def test_ato_example_2_differs_by_the_rounded_base_it_prints():
    """Example 2 adds $9,028 to 17% of $7,347 and reaches $10,276.99. The exact
    15% band is $9,028.35, and the worksheet applies it."""
    result = _repay("2026-27", "137064")
    assert result["amounts"]["compulsory_repayment"] == "10277.34"
    assert result["rates"]["upper_band_base"] == "9028.35"
    assert result["rates"]["upper_band_base_as_printed"] == "9028"
    assert D(result["amounts"]["compulsory_repayment"]) - D("10276.99") == D("0.35")


@pytest.mark.parametrize("year", PUBLISHED)
def test_every_band_boundary_matches_the_published_table(year):
    minimum, step, printed_base, flat_from = (D(value) for value in PUBLISHED[year])
    one = D(1)
    assert _repay(year, minimum)["amounts"]["compulsory_repayment"] == "0.00"
    assert _repay(year, minimum)["rates"]["rate_applied"] == "0"
    assert _repay(year, minimum + one)["amounts"]["compulsory_repayment"] == "0.15"
    at_step = _repay(year, step)
    assert at_step["amounts"]["compulsory_repayment"] == f"{(step - minimum) * D('0.15'):.2f}"
    assert at_step["rates"]["rate_applied"] == "0.15"
    above_step = _repay(year, step + one)
    assert D(above_step["amounts"]["compulsory_repayment"]) == (
        (step - minimum) * D("0.15") + D("0.17"))
    assert above_step["rates"]["rate_applied"] == "0.17"
    # The printed base is the exact 15% band rounded to the dollar.
    assert printed_base == ((step - minimum) * D("0.15")).quantize(one)
    # Below the crossover the marginal amount applies; from it, 10% of the whole.
    below = _repay(year, flat_from - one)
    assert below["rates"]["rate_applied"] == "0.17"
    assert D(below["amounts"]["compulsory_repayment"]) < D(
        below["amounts"]["ten_percent_of_income"])
    at_flat = _repay(year, flat_from)
    assert at_flat["rates"]["rate_applied"] == "0.10"
    assert at_flat["amounts"]["compulsory_repayment"] == f"{flat_from * D('0.10'):.2f}"


@pytest.mark.parametrize("year", PUBLISHED)
def test_the_published_crossover_is_derived_from_the_exact_base(year):
    minimum, step, _, flat_from = (D(value) for value in PUBLISHED[year])
    base = (step - minimum) * D("0.15")
    crossovers = [income for income in range(int(step), int(flat_from) + 2)
                  if D(income) * D("0.10") <= base + (D(income) - step) * D("0.17")]
    assert crossovers[0] == flat_from
    assert STUDY_LOAN_REPAYMENT_THRESHOLDS[year]["flat_from"] == str(flat_from)


def test_amounts_and_facts_travel_with_the_result():
    result = _repay("2025-26", "100000")
    assert result["amounts"] == {
        "repayment_income_used": "100000.00", "marginal_amount": "4950.00",
        "ten_percent_of_income": "10000.00", "compulsory_repayment": "4950.00",
    }
    assert result["rates"]["minimum_repayment_income"] == "67000"
    assert result["rates"]["flat_from_as_printed"] == "179286"
    assert result["sources"][1] == "https://www.legislation.gov.au/C2026G00249/latest/text"
    assert result["source_checked"] == "2026-09-27"
    # A stale minimum would give $4,950.00 for 2026-27 as well; the year's own does not.
    assert _repay("2026-27", "100000")["amounts"]["compulsory_repayment"] == "4570.80"


def test_refusals():
    with pytest.raises(ValueError, match="Unsupported period"):
        _repay("2024-25", "100000")
    with pytest.raises(ValueError, match="whole-dollar"):
        _repay("2026-27", "100000.50")
    with pytest.raises(ValueError, match="scope conditions"):
        c.study_loan_repayment(D("100000"), "2026-27", False)
    with pytest.raises(ValueError, match="non-negative"):
        _repay("2026-27", "-1")
