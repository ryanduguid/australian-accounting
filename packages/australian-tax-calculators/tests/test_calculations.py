"""Synthetic examples and boundaries, independent of the MCP transport."""

from decimal import Decimal as D

import pytest
from austaxcalc import calculations as c


def test_gst_inclusive_and_exclusive_reconcile():
    inclusive = c.gst(D("1100"), True, True, "2025-26")
    exclusive = c.gst(D("1000"), False, True, "2025-26")
    assert inclusive["amounts"] == exclusive["amounts"]
    assert inclusive["amounts"]["gst"] == "100.00"


@pytest.mark.parametrize("year,income,tax", [
    ("2025-26", "18200", "0.00"), ("2025-26", "45000", "4288.00"),
    ("2025-26", "135000", "31288.00"), ("2025-26", "190000", "51638.00"),
    ("2025-26", "200000", "56138.00"), ("2026-27", "45000", "4020.00"),
])
def test_resident_brackets(year, income, tax):
    assert c.resident_tax(D(income), year, True)["amounts"]["basic_income_tax"] == tax


def test_losses_precede_discount_and_excess_carries_forward():
    r = c.capital_gains(D("100"), D("1000"), D("200"), D("300"), True, "2025-26")
    assert r["amounts"]["net_capital_gain"] == "300.00"
    r = c.capital_gains(D("100"), D("1000"), D("1200"), D("300"), True, "2025-26")
    assert r["amounts"]["net_capital_gain"] == "0.00"
    assert r["amounts"]["losses_remaining"] == "400.00"


def test_fbt_type_one_and_two():
    r = c.fbt(D("1000"), D("1000"), 2026, True)
    assert r["amounts"]["fbt_estimate"] == "1864.49"
    r = c.fbt(D("16500"), D("6000"), 2026, True)
    assert r["amounts"]["fbt_estimate"] == "21452.73"


def test_fbt_retains_gross_up_precision_until_final_presentation():
    r = c.fbt(D("1000"), D("0"), 2026, True)
    assert r["amounts"]["type_one_grossed_up"] == "2080.20"
    assert r["amounts"]["fbt_estimate"] == "977.69"


@pytest.mark.parametrize("method,decline,deduction", [
    ("prime_cost", "300.00", "120.00"), ("diminishing_value", "600.00", "240.00"),
])
def test_first_year_depreciation(method, decline, deduction):
    r = c.depreciation(D("3000"), D("4"), 146, D("0.4"), method, True, "2025-26")
    assert r["amounts"]["decline_in_value"] == decline
    assert r["amounts"]["deduction"] == deduction


def test_depreciation_cannot_exceed_cost():
    r = c.depreciation(D("100"), D("1"), 365, D("1"), "diminishing_value", True,
                       "2025-26")
    assert r["amounts"]["decline_in_value"] == "100.00"


def test_quarterly_sg_maximum_and_amount_still_needed():
    r = c.quarterly_sg(D("80000"), D("2000"), "2025-26", 1, True)
    assert r["amounts"]["minimum_sg"] == "7500.00"
    assert r["amounts"]["additional_contribution"] == "5500.00"


@pytest.mark.parametrize("amount", [D("NaN"), D("Infinity"), D("-1"),
                                    D("1E13"), D("0.001"), 1.0, True])
def test_bad_amounts_rejected(amount):
    with pytest.raises(ValueError):
        c.gst(amount, True, True, "2025-26")


@pytest.mark.parametrize("confirmation", [False, None, "true", 1])
def test_scope_must_be_established(confirmation):
    with pytest.raises(ValueError):
        c.resident_tax(D("45000"), "2025-26", confirmation)


def test_unknown_periods_and_non_boolean_flags_fail():
    with pytest.raises(ValueError):
        c.quarterly_sg(D("100"), D("0"), "2026-27", 1, True)
    with pytest.raises(ValueError):
        c.resident_tax(D("100"), "2030-31", True)
    with pytest.raises(ValueError):
        c.gst(D("100"), "false", True, "2025-26")
    with pytest.raises(ValueError):
        c.resident_tax(D("45000.01"), "2025-26", True)
    with pytest.raises(ValueError):
        c.quarterly_sg(D("100"), D("0"), "2025-26", True, True)


@pytest.mark.parametrize("field,value", [("effective_life", D("NaN")),
    ("effective_life", D("0")), ("taxable_use", D("1.01")), ("taxable_use", D("Infinity")),
    ("days", True), ("days", 366), ("method", "pool")])
def test_depreciation_rejects_invalid_scope_inputs(field, value):
    args = dict(cost=D("100"), effective_life=D("5"), days=365, taxable_use=D("1"),
                method="prime_cost", scope_confirmed=True, year="2025-26")
    args[field] = value
    with pytest.raises(ValueError):
        c.depreciation(**args)
