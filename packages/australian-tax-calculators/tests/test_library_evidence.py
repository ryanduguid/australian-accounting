"""Frozen results derived in docs/calculation-evidence.md, without calling an oracle."""

from decimal import Decimal as D

import pytest
from austaxcalc import calculations as c


@pytest.mark.parametrize("function,arguments,expected", [
    pytest.param(c.gst, (D("22000"), True, True, "2025-26"),
                 {"gst": "2000.00", "exclusive": "20000.00", "inclusive": "22000.00"},
                 id="gst-library-12-020"),
    pytest.param(c.resident_tax, (D("144014"), "2024-25", True),
                 {"taxable_income_used": "144014.00", "basic_income_tax": "34623.18"},
                 id="resident-library-7-010-basic-tax-only"),
    pytest.param(c.capital_gains, (D("8000"), D("21435"), D("8660"), D("0"),
                                  True, "2025-26"),
                 {"losses_used": "8660.00", "losses_remaining": "0.00",
                  "net_capital_gain": "10387.50", "discount_applied": "10387.50"},
                 id="cgt-library-2-240-established-gains-only"),
    pytest.param(c.fbt, (D("8500"), D("0"), 2026, True),
                 {"type_one_grossed_up": "17681.70", "type_two_grossed_up": "0.00",
                  "fbt_estimate": "8310.40", "return_item_14a": "17681.00",
                  "return_item_14b": "0.00", "return_item_15": "17681.00",
                  "return_item_16": "8310.07"}, id="fbt-library-3-020-established-value-only"),
    pytest.param(c.fbt, (D("0"), D("10000"), 2026, True),
                 {"type_one_grossed_up": "0.00", "type_two_grossed_up": "18868.00",
                  "fbt_estimate": "8867.96", "return_item_14a": "0.00",
                  "return_item_14b": "18868.00", "return_item_15": "18868.00",
                  "return_item_16": "8867.96"}, id="fbt-library-3-000-established-value-only"),
    pytest.param(c.depreciation, (D("12400"), D("4"), 304, D("1"), "prime_cost",
                                 True, "2025-26"),
                 {"decline_in_value": "2581.92", "deduction": "2581.92",
                  "closing_adjustable_value": "9818.08"}, id="depreciation-adapted-6-000"),
    pytest.param(c.quarterly_sg, (D("62500"), D("7500"), "2025-26", 4, True),
                 {"earnings_used": "62500.00", "minimum_sg": "7500.00",
                  "additional_contribution": "0.00"}, id="sg-derived-18-600-and-18-620"),
    pytest.param(c.contribution_caps, (D("1500000"), D("0"), D("0"), D("240000"), True,
                                       "2024-25", True),
                 {"carry_forward_applied": "0.00", "concessional_available": "30000.00",
                  "concessional_remaining": "30000.00", "excess_concessional": "0.00",
                  "non_concessional_available": "360000.00",
                  "non_concessional_remaining": "120000.00", "excess_non_concessional": "0.00"},
                 id="caps-library-7-278-first-year-only"),
    pytest.param(c.pension_minimum, (D("250000"), 66, 181, "2024-25", True),
                 {"account_balance_used": "250000.00", "minimum_before_rounding": "6198.63",
                  "minimum_payment": "6200.00"}, id="pension-derived-18-500"),
])
def test_independently_derived_library_results(function, arguments, expected):
    result = function(*arguments)
    assert result["amounts"] == expected
    assert result["warnings"]
