"""Schedule 8 study and training support loan withholding against the ATO's schedule.

Expected amounts are typed in from the ATO's Schedule 8 page (published 17 June
2026, read on 15 October 2026) or derived by hand from it; none is produced by
the engine's own data.
"""

from decimal import ROUND_HALF_UP
from decimal import Decimal as D

import pytest
from austaxcalc import calculations as c
from austaxcalc.metadata import (
    PAYG_WITHHOLDING_COEFFICIENTS,
    STUDY_LOAN_WITHHOLDING_COEFFICIENTS,
)

# Schedule 8, "Coefficients to work out the weekly amounts to withhold", scales 1
# and 2 with a study and training support loans debt: (limit, a, b), the row
# applying while x is below the limit. These combined coefficients are the
# ATO's other accepted method and are not used by the worksheet.
COMBINED = {
    1: ((188, "0.1500", "0.1500"), (371, "0.2084", "11.0185"), (515, "0.1790", "0.1066"),
        (932, "0.3227", "74.1674"), (987, "0.3200", "71.6508"), (2144, "0.4700", "219.7124"),
        (2246, "0.4900", "262.6035"), (2727, "0.5600", "419.8343"),
        (3303, "0.4900", "228.8816"), (None, "0.5700", "493.1893")),
    2: ((362, "0", "0"), (538, "0.1500", "54.3462"), (673, "0.2500", "108.2135"),
        (721, "0.1700", "54.3473"), (865, "0.1790", "60.8377"), (1282, "0.3227", "185.1935"),
        (1337, "0.3200", "181.7319"), (2494, "0.4700", "382.2935"),
        (2596, "0.4900", "432.1846"), (3577, "0.5600", "613.9154"),
        (3653, "0.4900", "363.4627"), (None, "0.5700", "655.7704")),
}


def _withhold(earnings, period, scale):
    return c.study_loan_withholding(D(earnings), period, scale, "2026-27", True)


@pytest.mark.parametrize("earnings,period,scale,component", [
    ("2608.36", "weekly", 2, "193.00"),        # example 1: tax-free threshold claimed
    ("4409.75", "fortnightly", 2, "260.00"),   # example 2: weekly 130 x 2
    ("10627.88", "monthly", 1, "979.00"),      # example 3: no tax-free threshold, 226 x 13 / 3
])
def test_ato_worked_examples(earnings, period, scale, component):
    assert _withhold(earnings, period, scale)["amounts"]["study_loan_component"] == component


def test_example_1_shows_the_coefficients_and_the_total():
    result = _withhold("2608.36", "weekly", 2)
    assert result["amounts"]["weekly_earnings_used"] == "2608.99"
    assert (result["rates"]["component_a"], result["rates"]["component_b"]) == ("0.17", "250.4527")
    # Schedule 1 scale 2 from $2,596: 0.39 x 2608.99 - 363.4627 is 654.04, so $654.
    assert result["amounts"]["schedule_1_withholding"] == "654.00"
    assert result["amounts"]["withholding"] == "847.00"


@pytest.mark.parametrize("earnings,period,scale", [
    ("1000.00", "weekly", 2), ("2608.36", "weekly", 1), ("4409.75", "fortnightly", 3),
    ("10627.88", "monthly", 5), ("7000.33", "monthly", 6),
])
def test_the_schedule_1_amount_is_the_payg_worksheet_amount(earnings, period, scale):
    result = _withhold(earnings, period, scale)
    payg = c.payg_withholding(D(earnings), period, scale, "2026-27", True)
    assert result["amounts"]["schedule_1_withholding"] == payg["amounts"]["withholding"]
    assert D(result["amounts"]["withholding"]) == (
        D(payg["amounts"]["withholding"]) + D(result["amounts"]["study_loan_component"]))


# Schedule 8 "When to work out the study and training support loans component":
# the earnings from which the component starts, per pay period.
@pytest.mark.parametrize("scale,below,at,period", [
    (2, "1336.99", "1337.00", "weekly"), (2, "2673.99", "2674.00", "fortnightly"),
    (2, "5793.66", "5793.67", "monthly"), (3, "1336.99", "1337.00", "weekly"),
    (1, "986.99", "987.00", "weekly"), (1, "1973.99", "1974.00", "fortnightly"),
    (1, "4276.99", "4277.00", "monthly"),
])
def test_published_starting_earnings_select_the_first_component_row(scale, below, at, period):
    assert _withhold(below, period, scale)["rates"]["component_a"] == "0"
    assert _withhold(at, period, scale)["rates"]["component_a"] == "0.15"
    assert _withhold(below, period, scale)["amounts"]["study_loan_component"] == "0.00"


@pytest.mark.parametrize("scale", [1, 2])
def test_components_agree_with_the_combined_coefficients_to_a_dollar(scale):
    # The ATO says the two methods can differ slightly through rounding and accepts
    # either. Every whole-dollar weekly amount to $4,000 stays within $1.
    differences = set()
    for dollars in range(0, 4001):
        x = D(dollars) + D("0.99")
        a, b = next((D(a), D(b)) for limit, a, b in COMBINED[scale] if limit is None or x < limit)
        combined = max(D(0), (a * x - b).quantize(D(1), rounding=ROUND_HALF_UP))
        worksheet = D(_withhold(str(dollars), "weekly", scale)["amounts"]["withholding"])
        differences.add(worksheet - combined)
    assert differences <= {D(-1), D(0), D(1)}
    assert D(0) in differences


def test_combined_rows_are_schedule_1_plus_the_component_rows():
    # Holds the component transcription to the schedule's other table: for scales 1
    # and 2, every combined row is the Schedule 1 row plus the component row that
    # applies over the same range, to the schedule's 4 decimal places.
    for scale, table in ((1, "no_threshold"), (2, "threshold_or_foreign")):
        for limit, a, b in COMBINED[scale]:
            x = D(limit) - 1 if limit else D(5000)
            pick = [next((D(ra), D(rb)) for top, ra, rb in rows if top is None or x < top)
                    for rows in (PAYG_WITHHOLDING_COEFFICIENTS["2026-27"][scale],
                                 STUDY_LOAN_WITHHOLDING_COEFFICIENTS["2026-27"][table])]
            assert pick[0][0] + pick[1][0] == D(a), (scale, limit)
            assert abs(pick[0][1] + pick[1][1] - D(b)) <= D("0.0001"), (scale, limit)


def test_every_component_year_has_schedule_1_coefficients():
    assert set(STUDY_LOAN_WITHHOLDING_COEFFICIENTS) <= set(PAYG_WITHHOLDING_COEFFICIENTS)


@pytest.mark.parametrize("scale", [0, 4, "2", True])
def test_unsupported_scales_are_refused(scale):
    with pytest.raises(ValueError, match="scale"):
        c.study_loan_withholding(D("1000"), "weekly", scale, "2026-27", True)


@pytest.mark.parametrize("period", ["quarterly", ""])
def test_unsupported_pay_periods_are_refused(period):
    with pytest.raises(ValueError, match="pay_period"):
        c.study_loan_withholding(D("1000"), period, 2, "2026-27", True)
