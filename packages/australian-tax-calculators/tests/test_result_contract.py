"""The result envelope, refusal messages and input bounds each worksheet promises.

Mutation testing showed these were run but not checked: a worksheet could drop its
period or rates, flip ``ok`` or reword a refusal and every other test still passed.
"""

import re
from decimal import Decimal as D

import pytest
from austaxcalc import __version__
from austaxcalc import calculations as c

WARNINGS = [
    "Operator-confirmed facts and scope; eligibility is not independently verified.",
    "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
]

# One call per worksheet with the period label and rates it must report.
CASES = {
    "gst": (lambda: c.gst(D("1100.00"), True, True, "2025-26"),
            "2025-26", {"gst_rate": "0.10"}),
    "resident_tax": (lambda: c.resident_tax(D("45000"), "2024-25", True), "2024-25",
                     {"over_18200": "0.16", "over_45000": "0.30", "over_135000": "0.37",
                      "over_190000": "0.45"}),
    "capital_gains": (lambda: c.capital_gains(D("100"), D("1000"), D("200"), D("300"), True,
                                              "2025-26"), "2025-26", {"discount": "0.50"}),
    "fbt": (lambda: c.fbt(D("1000"), D("1000"), 2026, True), "year ended 31 March 2026",
            {"type_one": "2.0802", "type_two": "1.8868", "fbt_rate": "0.47"}),
    "depreciation": (lambda: c.depreciation(D("3000"), D("4"), 146, D("0.4"), "prime_cost",
                                            True, "2025-26"), "2025-26", {}),
    "quarterly_sg": (lambda: c.quarterly_sg(D("80000"), D("2000"), "2025-26", 1, True),
                     "2025-26 Q1", {"sg_rate": "0.12", "maximum_quarterly_base": "62500.00"}),
    "payg_withholding": (lambda: c.payg_withholding(D("1000"), "weekly", 2, "2026-27", True),
                         "2026-27 weekly", {"scale": "2", "a": "0.3227", "b": "185.1935"}),
    "contribution_caps": (lambda: c.contribution_caps(D("450000"), D("40000"), D("15000"),
                                                      D("150000"), True, "2024-25", True),
                          "2024-25", {"concessional_cap": "30000",
                                      "non_concessional_cap": "120000",
                                      "general_transfer_balance_cap": "1900000",
                                      "carry_forward_balance_limit": "500000",
                                      "non_concessional_cap_multiple": "3",
                                      "bring_forward_period_years": "3"}),
    "pension_minimum": (lambda: c.pension_minimum(D("600000"), 67, 365, "2024-25", True),
                        "2024-25", {"percentage_factor": "0.05", "days": "365",
                                    "days_in_year": "365"}),
    "study_loan_repayment": (lambda: c.study_loan_repayment(D("86380"), "2025-26", True),
                             "2025-26", {"rate_applied": "0.15",
                                         "minimum_repayment_income": "67000",
                                         "lower_rate": "0.15", "lower_band_top": "125000",
                                         "upper_band_base": "8700.00",
                                         "upper_band_base_as_printed": "8700",
                                         "upper_rate": "0.17", "flat_rate": "0.10",
                                         "flat_from_as_printed": "179286"}),
    "study_loan_withholding": (lambda: c.study_loan_withholding(D("2000"), "weekly", 2,
                                                                "2026-27", True),
                               "2026-27 weekly", {"scale": "2", "a": "0.3200",
                                                  "b": "181.7319", "component_a": "0.15",
                                                  "component_b": "200.5615"}),
    "medicare_levy": (lambda: c.medicare_levy(D("30000"), False, "2024-25", True), "2024-25",
                      {"levy_rate": "0.02", "shade_in_rate": "0.10",
                       "threshold_amount": "27222", "phase_in_limit": "34027"}),
    "low_income_tax_offset": (lambda: c.low_income_tax_offset(D("40000"), "2024-25", True),
                              "2024-25", {"maximum": "700", "first_step": "37500",
                                          "first_rate": "0.05", "second_step": "45000",
                                          "second_amount": "325", "second_rate": "0.015",
                                          "cut_out": "66667"}),
    "division_293": (lambda: c.division_293(D("240000"), D("20000"), "2024-25", True),
                     "2024-25", {"threshold": "250000", "rate": "0.15"}),
}


def test_every_worksheet_has_a_contract_case():
    assert set(CASES) == set(c.SCOPES)


@pytest.mark.parametrize("kind", CASES)
def test_result_envelope(kind):
    call, period, rates = CASES[kind]
    result = call()
    assert result["ok"] is True
    assert result["engine"] == "australian-tax-calculators"
    assert result["engine_version"] == __version__
    assert result["calculation"] == kind
    assert result["period"] == period
    assert result["rates"] == rates
    assert result["warnings"] == WARNINGS


def test_result_amount_keys():
    assert set(CASES["payg_withholding"][0]()["amounts"]) == {
        "weekly_earnings_used", "weekly_withholding", "withholding"}
    assert set(CASES["medicare_levy"][0]()["amounts"]) == {
        "taxable_income_used", "levy_at_full_rate", "low_income_reduction", "medicare_levy"}
    assert set(CASES["low_income_tax_offset"][0]()["amounts"]) == {
        "taxable_income_used", "low_income_tax_offset"}
    assert set(CASES["division_293"][0]()["amounts"]) == {
        "combined_income_and_contributions", "excess_over_threshold",
        "taxable_contributions", "division_293_tax"}


def _refused(message, call, *args):
    with pytest.raises(ValueError, match=f"^{re.escape(message)}$"):
        call(*args)


@pytest.mark.parametrize("message,call,args", [
    ("Amounts must be finite non-negative Decimals, at most 2dp and 1e12.",
     c.gst, (D("-1"), True, True, "2025-26")),
    ("Establish all stated scope conditions before calculating.",
     c.gst, (D("1"), True, False, "2025-26")),
    ("Unsupported period '2027-28'; supported: 2024-25, 2025-26, 2026-27.",
     c.resident_tax, (D("1"), "2027-28", True)),
    ("gst_inclusive must be a boolean.", c.gst, (D("1"), 1, True, "2025-26")),
    ("Supply established whole-dollar taxable income.",
     c.resident_tax, (D("1.50"), "2025-26", True)),
    ("Supply established whole-dollar taxable income.",
     c.medicare_levy, (D("1.50"), False, "2024-25", True)),
    ("Supply established whole-dollar taxable income.",
     c.low_income_tax_offset, (D("1.50"), "2024-25", True)),
    ("Supply established whole-dollar repayment income.",
     c.study_loan_repayment, (D("1.50"), "2025-26", True)),
    ("effective_life must be a finite Decimal between 0.01 and 1000 years.",
     c.depreciation, (D("1"), D("0"), 1, D("1"), "prime_cost", True, "2025-26")),
    ("taxable_use must be a finite Decimal fraction from 0 to 1.",
     c.depreciation, (D("1"), D("1"), 1, D("2"), "prime_cost", True, "2025-26")),
    ("days must be an integer from 0 to 365 for 2025-26.",
     c.depreciation, (D("1"), D("1"), 366, D("1"), "prime_cost", True, "2025-26")),
    ("method must be prime_cost or diminishing_value.",
     c.depreciation, (D("1"), D("1"), 1, D("1"), "pooled", True, "2025-26")),
    ("quarter must be 1 (Jul-Sep), 2, 3 or 4 (Apr-Jun).",
     c.quarterly_sg, (D("1"), D("0"), "2025-26", 0, True)),
    ("pay_period must be weekly, fortnightly or monthly.",
     c.payg_withholding, (D("1"), "daily", 2, "2026-27", True)),
    ("scale must be 1, 2, 3, 5 or 6; scale 4 is not supported.",
     c.payg_withholding, (D("1"), "weekly", 4, "2026-27", True)),
    ("section_9a_adjustment_applies must be a boolean.",
     c.medicare_levy, (D("1"), 0, "2024-25", True)),
    ("Medicare Levy Act 1986 s 9A (lump sum payments in arrears) applies, and this "
     f"worksheet does not make that adjustment. See {c.EXTRA_SOURCES['medicare_levy'][2]}",
     c.medicare_levy, (D("1"), True, "2024-25", True)),
    ("under_75_in_year must be a boolean.",
     c.contribution_caps, (D("0"), D("0"), D("0"), D("0"), 1, "2024-25", True)),
    ("age must be an integer number of years from 0 to 150.",
     c.pension_minimum, (D("1"), 151, 1, "2024-25", True)),
])
def test_refusal_messages(message, call, args):
    _refused(message, call, *args)


def test_input_bounds_are_inclusive():
    assert c.gst(D("1000000000000.00"), False, True, "2025-26")["ok"] is True
    for life in (D("0.01"), D("1000")):
        assert c.depreciation(D("1"), life, 1, D("1"), "prime_cost", True, "2025-26")["ok"]
    nothing_used = c.depreciation(D("1000"), D("4"), 0, D("0"), "prime_cost", True, "2025-26")
    assert nothing_used["amounts"]["decline_in_value"] == "0.00"
    assert nothing_used["amounts"]["deduction"] == "0.00"


def test_values_just_outside_the_bounds_are_refused():
    with pytest.raises(ValueError, match="effective_life"):
        c.depreciation(D("1"), D("1000.5"), 1, D("1"), "prime_cost", True, "2025-26")
    with pytest.raises(ValueError, match="taxable_use"):
        c.depreciation(D("1"), D("1"), 1, 0.5, "prime_cost", True, "2025-26")
    with pytest.raises(ValueError, match="quarter"):
        c.quarterly_sg(D("1"), D("0"), "2025-26", 5, True)


def test_fbt_return_items_round_half_up_to_the_dollar():
    # 2,500.00 x 2.0802 = 5,200.50 and 1,250.00 x 1.8868 = 2,358.50: the ATO rounds
    # both up, where banker's rounding would give the even dollar below.
    amounts = c.fbt(D("2500.00"), D("1250.00"), 2026, True)["amounts"]
    assert amounts["return_item_14a"] == "5201.00"
    assert amounts["return_item_14b"] == "2359.00"


def test_pension_starting_in_june_needs_no_payment_before_rounding():
    result = c.pension_minimum(D("600000"), 67, 30, "2024-25", True)
    assert result["amounts"]["minimum_before_rounding"] == "0.00"
    assert result["amounts"]["minimum_payment"] == "0.00"


def test_study_loan_marginal_amount_is_nil_below_the_minimum():
    amounts = c.study_loan_repayment(D("50000"), "2025-26", True)["amounts"]
    assert amounts["marginal_amount"] == "0.00"
    assert amounts["compulsory_repayment"] == "0.00"
