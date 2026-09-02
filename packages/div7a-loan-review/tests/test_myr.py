"""The s 109E(5) and (6) minimum yearly repayment."""
from __future__ import annotations

from decimal import Decimal
from fractions import Fraction

import pytest

from div7aloan.gate import GateFacts, complying_loan_gate
from div7aloan.money import ROUNDING
from div7aloan.myr import (
    MyrFacts,
    minimum_yearly_repayment,
    minimum_yearly_repayment_amount,
    statutory_remaining_term,
)
from div7aloan.verdicts import GateVerdict, MyrVerdict
from div7aloan.years import parse_year

D = Decimal
YEAR = parse_year("2026-27")
MADE = parse_year("2022-23")
RATE_2026_27 = D("0.0877")


def _gate(verdict: GateVerdict):
    """A gate result carrying the verdict a test needs."""
    common = dict(
        loan_id="TEST",
        terms_in_place_before_lodgment_day=True,
        maximum_term_years=D("7"),
        secured_by_registered_mortgage_over_real_property=False,
        interest_rate_for_years_after_year_loan_made=D("0.0477"),
        year_loan_made=MADE,
    )
    written = {
        GateVerdict.COMPLYING: True,
        GateVerdict.NOT_COMPLYING: False,
        GateVerdict.UNKNOWN: None,
    }[verdict]
    result = complying_loan_gate(GateFacts(written_agreement=written, **common))
    assert result.verdict is verdict
    return result


COMPLYING_GATE = _gate(GateVerdict.COMPLYING)


def facts(**overrides) -> MyrFacts:
    base = dict(
        loan_id="TEST",
        year_of_income=YEAR,
        amalgamated_loan_unpaid_at_end_of_previous_year=D("100000.00"),
        remaining_term_years=D("5"),
        payments_applied_during_the_year=D("25556.00"),
        gate_result=COMPLYING_GATE,
        year_loan_made=MADE,
    )
    base.update(overrides)
    return MyrFacts(**base)


# --- the formula itself ------------------------------------------------


def _exact(principal: str, rate: str, term: int) -> Decimal:
    """The s 109E(6) formula in exact rational arithmetic, rounded half up to
    cents. An independent check on the Decimal implementation: if the two
    agree, the intermediate precision is not costing cents."""
    p, r = Fraction(principal), Fraction(rate)
    value = (p * r) / (1 - (Fraction(1) / (1 + r)) ** term)
    cents = value * 100
    whole = cents.numerator // cents.denominator
    if cents - whole >= Fraction(1, 2):
        whole += 1
    return Decimal(whole).scaleb(-2)


@pytest.mark.parametrize(
    "principal,rate,term,expected",
    [
        # The three hand-worked fixtures in evaluation/div7a_myr/README.md.
        ("100000.00", "0.0877", 5, "25556.00"),
        ("250000.00", "0.0827", 7, "48462.41"),
        ("48500.00", "0.0877", 3, "19081.67"),
        # The same third loan at the rate its agreement was written under.
        ("48500.00", "0.0452", 3, "17649.66"),
        # A long secured loan, and the shortest possible remaining term.
        ("1000000.00", "0.0837", 25, "96657.07"),
        ("10000.00", "0.0877", 1, "10877.00"),
    ],
)
def test_formula_regression(principal, rate, term, expected):
    assert minimum_yearly_repayment_amount(D(principal), D(rate), D(term)) == D(expected)


@pytest.mark.parametrize(
    "principal,rate,term",
    [
        ("100000.00", "0.0877", 5),
        ("250000.00", "0.0827", 7),
        ("48500.00", "0.0877", 3),
        ("1000000.00", "0.0837", 25),
        ("7.13", "0.0537", 2),
        ("999999999.99", "0.0452", 24),
    ],
)
def test_formula_agrees_with_exact_rational_arithmetic(principal, rate, term):
    assert minimum_yearly_repayment_amount(D(principal), D(rate), D(term)) == _exact(
        principal, rate, term
    )


def test_a_single_remaining_year_is_principal_plus_one_year_of_interest():
    """A closed-form check of the formula's shape: at n = 1 the denominator
    is r/(1+r), so the repayment is P x (1 + r)."""
    principal, rate = D("123456.78"), D("0.0877")
    expected = (principal * (1 + rate)).quantize(D("0.01"))
    assert minimum_yearly_repayment_amount(principal, rate, D(1)) == expected


def test_the_result_is_a_decimal_quantised_to_cents():
    value = minimum_yearly_repayment_amount(D("100000.00"), RATE_2026_27, D(5))
    assert isinstance(value, Decimal)
    assert value.as_tuple().exponent == -2


def test_the_formula_is_refused_at_a_nil_rate():
    with pytest.raises(ValueError, match="nil benchmark rate"):
        minimum_yearly_repayment_amount(D("100000.00"), D("0"), D(5))


def test_the_formula_is_refused_at_a_nil_term():
    with pytest.raises(ValueError, match="nil remaining term"):
        minimum_yearly_repayment_amount(D("100000.00"), RATE_2026_27, D(0))


@pytest.mark.parametrize(
    "supplied,expected",
    [("5", "5"), ("4.2", "5"), ("4.0001", "5"), ("0.1", "1"), ("7", "7")],
)
def test_remaining_term_rounds_up_per_the_closing_words_of_109E_6(supplied, expected):
    assert statutory_remaining_term(D(supplied)) == D(expected)


# --- verdicts ----------------------------------------------------------


def test_repayment_met_exactly_to_the_cent():
    result = minimum_yearly_repayment(facts())
    assert result.verdict is MyrVerdict.MYR_MET
    assert result.myr_required == D("25556.00")
    assert result.shortfall == D("0.00")
    assert result.experimental_deemed_dividend_exposure is None


def test_repayment_short_by_one_dollar():
    result = minimum_yearly_repayment(
        facts(payments_applied_during_the_year=D("25555.00"))
    )
    assert result.verdict is MyrVerdict.MYR_SHORT
    assert result.shortfall == D("1.00")
    assert result.experimental_deemed_dividend_exposure == D("1.00")


def test_repayment_short_by_one_cent_is_still_short():
    result = minimum_yearly_repayment(
        facts(payments_applied_during_the_year=D("25555.99"))
    )
    assert result.verdict is MyrVerdict.MYR_SHORT
    assert result.shortfall == D("0.01")


def test_an_overpayment_does_not_produce_a_negative_shortfall():
    result = minimum_yearly_repayment(
        facts(payments_applied_during_the_year=D("40000.00"))
    )
    assert result.verdict is MyrVerdict.MYR_MET
    assert result.shortfall == D("0.00")
    assert result.shortfall >= 0


def test_a_nil_remaining_term_is_refused_without_dividing_by_zero():
    result = minimum_yearly_repayment(facts(remaining_term_years=D("0")))
    assert result.verdict is MyrVerdict.REFUSED
    assert result.myr_required is None
    assert "109E(6)" in result.reasons[0]


def test_a_negative_remaining_term_is_refused():
    result = minimum_yearly_repayment(facts(remaining_term_years=D("-1")))
    assert result.verdict is MyrVerdict.REFUSED
    assert result.myr_required is None


def test_the_year_of_the_loan_is_refused():
    """s 109E(1)(a) reaches a loan made in an earlier year of income, and
    s 109P puts an amalgamated loan outside s 109D in the year it is made."""
    result = minimum_yearly_repayment(
        facts(year_of_income=MADE, year_loan_made=MADE)
    )
    assert result.verdict is MyrVerdict.REFUSED
    assert result.myr_required is None
    assert any("109P" in reason for reason in result.reasons)


def test_a_year_before_the_loan_was_made_is_refused():
    result = minimum_yearly_repayment(
        facts(year_of_income=parse_year("2021-22"), year_loan_made=MADE)
    )
    assert result.verdict is MyrVerdict.REFUSED


def test_a_non_complying_gate_refuses_the_repayment_figure():
    result = minimum_yearly_repayment(facts(gate_result=_gate(GateVerdict.NOT_COMPLYING)))
    assert result.verdict is MyrVerdict.REFUSED
    assert result.myr_required is None
    assert result.shortfall is None
    assert any("NOT_COMPLYING" in reason for reason in result.reasons)


def test_an_unknown_gate_refuses_the_repayment_figure():
    result = minimum_yearly_repayment(facts(gate_result=_gate(GateVerdict.UNKNOWN)))
    assert result.verdict is MyrVerdict.REFUSED
    assert result.myr_required is None


def test_no_gate_result_at_all_is_refused():
    result = minimum_yearly_repayment(facts(gate_result=None))
    assert result.verdict is MyrVerdict.REFUSED
    assert "Run the gate first" in result.reasons[0]


def test_a_missing_unpaid_balance_is_unknown():
    result = minimum_yearly_repayment(
        facts(amalgamated_loan_unpaid_at_end_of_previous_year=None)
    )
    assert result.verdict is MyrVerdict.UNKNOWN
    assert result.myr_required is None
    assert any("109E(3)" in reason for reason in result.reasons)


def test_missing_payments_are_unknown_not_nil():
    """Treating no assertion as a nil payment would report the whole minimum
    yearly repayment as exposure."""
    result = minimum_yearly_repayment(facts(payments_applied_during_the_year=None))
    assert result.verdict is MyrVerdict.UNKNOWN
    assert result.shortfall is None
    assert any("109R" in reason for reason in result.reasons)


def test_a_missing_remaining_term_is_unknown():
    result = minimum_yearly_repayment(facts(remaining_term_years=None))
    assert result.verdict is MyrVerdict.UNKNOWN


def test_an_unreviewed_rate_year_is_unknown():
    result = minimum_yearly_repayment(facts(year_of_income=parse_year("2027-28")))
    assert result.verdict is MyrVerdict.UNKNOWN
    assert result.myr_required is None


def test_a_refusal_outranks_an_unknown():
    """A question that is outside the engine stays refused even when a fact is
    also missing: the operator should be told the question is wrong, not that
    they need to go and find a number that will not help."""
    result = minimum_yearly_repayment(
        facts(gate_result=_gate(GateVerdict.NOT_COMPLYING), payments_applied_during_the_year=None)
    )
    assert result.verdict is MyrVerdict.REFUSED


def test_a_fractional_remaining_term_is_rounded_up_and_noted():
    result = minimum_yearly_repayment(facts(remaining_term_years=D("4.2")))
    assert result.remaining_term_years_used == D("5")
    assert any("rounded up" in caveat for caveat in result.caveats)


# --- what every result must carry --------------------------------------


def test_every_result_carries_the_genuine_repayment_caveat():
    for result in (
        minimum_yearly_repayment(facts()),
        minimum_yearly_repayment(facts(payments_applied_during_the_year=D("1.00"))),
        minimum_yearly_repayment(facts(remaining_term_years=None)),
    ):
        assert any("109R" in caveat for caveat in result.caveats)


def test_every_result_says_it_is_not_a_determination():
    result = minimum_yearly_repayment(facts(payments_applied_during_the_year=D("1.00")))
    joined = " ".join(result.caveats)
    assert "not an ATO assessment" in joined
    assert "109Y" in joined
    assert "109Q" in joined


def test_the_statutory_trace_shows_the_values_plugged_in():
    """A reviewer has to be able to re-perform the sum on paper from the
    trace alone."""
    result = minimum_yearly_repayment(facts())
    trace = " ".join(result.statutory_trace)
    assert "s 109E(6)" in trace
    assert "100000.00" in trace
    assert "0.0877" in trace
    assert "remaining term" in trace.lower()
    assert "25556.00" in trace
    assert ROUNDING in trace
    assert "FILRHLBVS" in trace


def test_the_benchmark_provenance_reaches_the_result():
    result = minimum_yearly_repayment(facts())
    assert result.benchmark.rba_month == "2026-05"
    assert result.benchmark.rba_series == "FILRHLBVS"


def test_a_missing_year_loan_made_is_noted_rather_than_assumed():
    result = minimum_yearly_repayment(facts(year_loan_made=None))
    assert result.verdict is MyrVerdict.MYR_MET
    assert any("109E(1)(a)" in caveat for caveat in result.caveats)


def test_every_money_field_is_a_decimal():
    result = minimum_yearly_repayment(facts(payments_applied_during_the_year=D("1.00")))
    for value in (
        result.myr_required,
        result.payments_applied,
        result.shortfall,
        result.experimental_deemed_dividend_exposure,
        result.amalgamated_loan_unpaid_at_end_of_previous_year,
    ):
        assert isinstance(value, Decimal)
