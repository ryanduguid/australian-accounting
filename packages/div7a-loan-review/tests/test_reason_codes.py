"""The reason-code vocabulary, across both engines that publish it.

A code nobody can produce is a promise to a caller that never arrives. A
reason produced under no code is the drift each engine's constructor guard
catches. This module owns the third property the guards cannot see on their
own: that the published vocabulary and the codes the engines actually emit are
the same set.
"""
from __future__ import annotations

from decimal import Decimal

import pytest
from div7aloan.gate import GateFacts, GateResult, complying_loan_gate
from div7aloan.myr import minimum_yearly_repayment
from div7aloan.verdicts import GateVerdict, ReasonCode
from div7aloan.years import parse_year

from test_gate import facts as gate_facts
from test_myr import MADE
from test_myr import facts as myr_facts

D = Decimal


GATE_SCENARIOS = {
    "GATE_WRITTEN_AGREEMENT_FAIL": {"written_agreement": False},
    "GATE_WRITTEN_AGREEMENT_UNKNOWN": {"written_agreement": None},
    "GATE_LODGMENT_DAY_FAIL": {"terms_in_place_before_lodgment_day": False},
    "GATE_LODGMENT_DAY_UNKNOWN": {"terms_in_place_before_lodgment_day": None},
    "GATE_INTEREST_FAIL": {"interest_rate_for_years_after_year_loan_made": D("0.01")},
    "GATE_INTEREST_UNKNOWN": {"interest_rate_for_years_after_year_loan_made": None},
    "GATE_TERM_FAIL": {"maximum_term_years": D("8")},
    "GATE_TERM_UNKNOWN": {"maximum_term_years": None},
}

MYR_SCENARIOS = {
    "REFUSED_GATE_RESULT_MISSING": {"gate_result": None},
    "REFUSED_YEAR_IS_YEAR_OF_LOAN": {"year_of_income": MADE},
    "REFUSED_YEAR_BEFORE_LOAN": {"year_of_income": parse_year("2021-22")},
    "REFUSED_REMAINING_TERM_NOT_POSITIVE": {"remaining_term_years": D("0")},
    "YEAR_OF_INCOME_UNKNOWN": {"year_of_income": None},
    "YEAR_LOAN_MADE_UNKNOWN": {"year_loan_made": None},
    "BENCHMARK_RATE_UNKNOWN": {"year_of_income": parse_year("2027-28")},
    "UNPAID_BALANCE_UNKNOWN": {"amalgamated_loan_unpaid_at_end_of_previous_year": None},
    "PAYMENTS_APPLIED_UNKNOWN": {"payments_applied_during_the_year": None},
    "REMAINING_TERM_UNKNOWN": {"remaining_term_years": None},
}

#: Emitted by scenarios that need more than one overridden fact, and covered
#: by their own tests in test_gate.py and test_myr.py.
COVERED_ELSEWHERE = frozenset({
    "GATE_NO_BENCHMARK_YEAR",
    "REFUSED_GATE_NOT_COMPLYING",
    "REFUSED_GATE_UNKNOWN",
    "REFUSED_GATE_BENCHMARK_YEAR_MISMATCH",
    "REFUSED_BENCHMARK_RATE_NOT_POSITIVE",
})


def test_every_gate_limb_that_does_not_pass_carries_its_code():
    for code, overrides in GATE_SCENARIOS.items():
        result = complying_loan_gate(gate_facts(**overrides))
        assert result.verdict is not GateVerdict.COMPLYING, code
        assert code in result.reason_codes, code
        assert code in result.to_json_dict()["reason_codes"], code


def test_a_gate_with_no_year_to_read_a_benchmark_for_carries_its_code():
    result = complying_loan_gate(
        GateFacts(loan_id="TEST", written_agreement=True, maximum_term_years=D("7"))
    )
    assert result.verdict is GateVerdict.UNKNOWN
    assert result.reason_codes == ("GATE_NO_BENCHMARK_YEAR",)


def test_a_complying_gate_carries_no_reason_codes():
    result = complying_loan_gate(gate_facts())
    assert result.verdict is GateVerdict.COMPLYING
    assert result.reason_codes == ()


def test_the_gate_codes_stay_in_step_with_its_prose():
    result = complying_loan_gate(
        gate_facts(written_agreement=False, maximum_term_years=None)
    )
    assert len(result.reason_codes) == len(result.reasons) == 2
    assert set(result.reason_codes) == {
        "GATE_WRITTEN_AGREEMENT_FAIL", "GATE_TERM_UNKNOWN",
    }


def test_a_gate_reason_without_a_code_is_refused_at_construction():
    with pytest.raises(ValueError, match="every reason needs its ReasonCode"):
        GateResult(verdict=GateVerdict.UNKNOWN, reasons=("no code for this one",))


def test_no_code_in_the_vocabulary_is_unreachable():
    emitted = set(COVERED_ELSEWHERE)
    for overrides in GATE_SCENARIOS.values():
        emitted.update(complying_loan_gate(gate_facts(**overrides)).reason_codes)
    for overrides in MYR_SCENARIOS.values():
        emitted.update(minimum_yearly_repayment(myr_facts(**overrides)).reason_codes)
    emitted.update(
        complying_loan_gate(
            GateFacts(loan_id="TEST", written_agreement=True, maximum_term_years=D("7"))
        ).reason_codes
    )
    assert emitted == {code.value for code in ReasonCode}


def test_no_scenario_here_emits_a_code_outside_the_vocabulary():
    published = {code.value for code in ReasonCode}
    for overrides in GATE_SCENARIOS.values():
        assert set(complying_loan_gate(gate_facts(**overrides)).reason_codes) <= published
    for overrides in MYR_SCENARIOS.values():
        result = minimum_yearly_repayment(myr_facts(**overrides))
        assert set(result.reason_codes) <= published
