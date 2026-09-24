"""Schedule 1 withholding, checked against every value in the ATO's own sample data."""

import csv
from decimal import Decimal as D
from pathlib import Path

import pytest
from austaxcalc import calculations as c

SAMPLE = Path(__file__).with_name("payg_withholding_sample_2026_27.csv")
SCALES = (1, 2, 3, 5, 6)
ROWS = list(csv.DictReader(SAMPLE.open(encoding="utf-8", newline="")))


def test_the_sample_is_complete():
    # 48 earnings points for each of 3 pay periods, as published on 17 June 2026.
    assert len(ROWS) == 144
    assert {row["period"] for row in ROWS} == {"weekly", "fortnightly", "monthly"}


@pytest.mark.parametrize("row", ROWS, ids=lambda row: f"{row['period']}-{row['earnings']}")
def test_every_scale_matches_the_ato_sample(row):
    for scale in SCALES:
        result = c.payg_withholding(D(row["earnings"]), row["period"], scale, "2026-27", True)
        assert result["amounts"]["withholding"] == f"{D(row[f'scale{scale}']):.2f}", scale


def test_weekly_earnings_ignore_cents_then_add_99_cents():
    # Schedule 1 "Working out the weekly earnings": $467.59 plus a $9.50 allowance.
    result = c.payg_withholding(D("477.09"), "weekly", 2, "2026-27", True)
    assert result["amounts"]["weekly_earnings_used"] == "477.99"


def test_a_monthly_amount_ending_in_33_cents_gains_a_cent():
    # 1564.33 x 3 / 13 is 360.9992, so without the cent x would be 360.99.
    # 1564.34 x 3 / 13 is 361.0015, so with it x is 361.99.
    with_cent = c.payg_withholding(D("1564.33"), "monthly", 1, "2026-27", True)
    assert with_cent["amounts"]["weekly_earnings_used"] == "361.99"


def test_the_result_names_the_coefficients_used():
    result = c.payg_withholding(D("1000"), "weekly", 2, "2026-27", True)
    assert result["rates"] == {"scale": "2", "a": "0.3227", "b": "185.1935"}
    assert result["period"] == "2026-27 weekly"


@pytest.mark.parametrize("scale", [0, 4, 7, "2", True, 2.0])
def test_unsupported_scales_are_refused(scale):
    with pytest.raises(ValueError, match="scale"):
        c.payg_withholding(D("1000"), "weekly", scale, "2026-27", True)


@pytest.mark.parametrize("period", ["quarterly", "bi-monthly", "Weekly", ""])
def test_unsupported_pay_periods_are_refused(period):
    with pytest.raises(ValueError, match="pay_period"):
        c.payg_withholding(D("1000"), period, 2, "2026-27", True)


@pytest.mark.parametrize("earnings", [D("-1"), D("1.001"), D("NaN"), D("1e13")])
def test_invalid_earnings_are_refused(earnings):
    with pytest.raises(ValueError):
        c.payg_withholding(earnings, "weekly", 2, "2026-27", True)


def test_a_year_without_its_own_coefficients_is_refused():
    with pytest.raises(ValueError, match="Unsupported period"):
        c.payg_withholding(D("1000"), "weekly", 2, "2025-26", True)
