"""The s 109N(1) complying-loan gate."""
from __future__ import annotations

from decimal import Decimal

import pytest

from div7aloan.facts import FactError
from div7aloan.gate import GateFacts, LimbState, complying_loan_gate
from div7aloan.verdicts import GateVerdict
from div7aloan.years import parse_year

D = Decimal
YEAR = parse_year("2026-27")
BENCHMARK = D("0.0877")  # 2026-27


def facts(**overrides) -> GateFacts:
    """A loan that satisfies every limb, so a test can break exactly one."""
    base = dict(
        loan_id="TEST",
        written_agreement=True,
        terms_in_place_before_lodgment_day=True,
        maximum_term_years=D("7"),
        secured_by_registered_mortgage_over_real_property=False,
        security_coverage_at_first_made=None,
        interest_rate_for_years_after_year_loan_made=BENCHMARK,
        year_of_income_being_tested=YEAR,
    )
    base.update(overrides)
    return GateFacts(**base)


def limb(result, cite):
    return next(item for item in result.limbs if item.cite.startswith(cite))


def test_seven_year_unsecured_at_exactly_the_benchmark_is_complying():
    result = complying_loan_gate(facts())
    assert result.verdict is GateVerdict.COMPLYING
    assert result.maximum_term_years_allowed == D("7")
    assert all(item.state is LimbState.PASS for item in result.limbs)


def test_seven_year_unsecured_one_basis_point_under_is_not_complying():
    result = complying_loan_gate(
        facts(interest_rate_for_years_after_year_loan_made=BENCHMARK - D("0.0001"))
    )
    assert result.verdict is GateVerdict.NOT_COMPLYING
    assert limb(result, "s 109N(1)(b)").state is LimbState.FAIL
    assert "0.0876" in limb(result, "s 109N(1)(b)").finding


def test_twenty_five_year_loan_without_a_registered_mortgage_is_not_complying():
    result = complying_loan_gate(facts(maximum_term_years=D("25")))
    assert result.verdict is GateVerdict.NOT_COMPLYING
    assert result.maximum_term_years_allowed == D("7")
    assert "s 109N(3)(b)" in limb(result, "s 109N(1)(c)").finding


def test_twenty_five_year_loan_with_mortgage_but_109_per_cent_cover_is_not_complying():
    result = complying_loan_gate(
        facts(
            maximum_term_years=D("25"),
            secured_by_registered_mortgage_over_real_property=True,
            security_coverage_at_first_made=D("1.09"),
        )
    )
    assert result.verdict is GateVerdict.NOT_COMPLYING
    assert result.maximum_term_years_allowed == D("7")
    assert "109N(3)(a)(ii)" in limb(result, "s 109N(1)(c)").finding


def test_twenty_five_year_loan_with_mortgage_and_110_per_cent_cover_is_complying():
    result = complying_loan_gate(
        facts(
            maximum_term_years=D("25"),
            secured_by_registered_mortgage_over_real_property=True,
            security_coverage_at_first_made=D("1.10"),
        )
    )
    assert result.verdict is GateVerdict.COMPLYING
    assert result.maximum_term_years_allowed == D("25")


def test_the_110_per_cent_boundary_is_inclusive():
    """s 109N(3)(a)(ii) says "at least 110%", so 1.10 exactly passes and the
    cent below it does not."""
    just_under = complying_loan_gate(
        facts(
            maximum_term_years=D("25"),
            secured_by_registered_mortgage_over_real_property=True,
            security_coverage_at_first_made=D("1.0999"),
        )
    )
    assert just_under.verdict is GateVerdict.NOT_COMPLYING


def test_a_twenty_five_year_loan_carries_the_refinancing_caveat():
    result = complying_loan_gate(
        facts(
            maximum_term_years=D("25"),
            secured_by_registered_mortgage_over_real_property=True,
            security_coverage_at_first_made=D("1.50"),
        )
    )
    assert any("109N(3A)" in caveat for caveat in result.caveats)


@pytest.mark.parametrize(
    "field",
    [
        "written_agreement",
        "terms_in_place_before_lodgment_day",
        "maximum_term_years",
        "interest_rate_for_years_after_year_loan_made",
    ],
)
def test_an_unestablished_fact_gives_unknown_never_complying(field):
    result = complying_loan_gate(facts(**{field: None}))
    assert result.verdict is GateVerdict.UNKNOWN
    assert result.verdict is not GateVerdict.COMPLYING
    assert result.reasons


def test_written_agreement_unknown_does_not_become_false():
    """UNKNOWN and NOT_COMPLYING are different findings. Coercing the first
    into the second would report a breach on a loan whose paperwork simply
    has not been looked at."""
    result = complying_loan_gate(facts(written_agreement=None))
    assert result.verdict is GateVerdict.UNKNOWN
    assert result.verdict is not GateVerdict.NOT_COMPLYING


def test_written_agreement_false_is_not_complying():
    result = complying_loan_gate(facts(written_agreement=False))
    assert result.verdict is GateVerdict.NOT_COMPLYING


def test_terms_not_in_place_before_lodgment_day_is_not_complying():
    result = complying_loan_gate(facts(terms_in_place_before_lodgment_day=False))
    assert result.verdict is GateVerdict.NOT_COMPLYING
    assert limb(result, "s 109N(1) chapeau").state is LimbState.FAIL


def test_a_failed_limb_beats_an_unknown_one():
    """s 109N(1) needs every limb. One that has definitely failed settles the
    question whatever else is outstanding."""
    result = complying_loan_gate(
        facts(written_agreement=None, interest_rate_for_years_after_year_loan_made=D("0.01"))
    )
    assert result.verdict is GateVerdict.NOT_COMPLYING


def test_unknown_security_does_not_make_a_short_term_unknown():
    """Below seven years both limbs of s 109N(3) permit the term, so the
    security facts cannot change the answer."""
    result = complying_loan_gate(
        facts(secured_by_registered_mortgage_over_real_property=None, maximum_term_years=D("7"))
    )
    assert result.verdict is GateVerdict.COMPLYING
    assert result.maximum_term_years_allowed == D("7")


def test_unknown_security_does_make_a_long_term_unknown():
    result = complying_loan_gate(
        facts(secured_by_registered_mortgage_over_real_property=None, maximum_term_years=D("25"))
    )
    assert result.verdict is GateVerdict.UNKNOWN
    assert result.maximum_term_years_allowed is None


def test_unknown_coverage_with_a_long_term_is_unknown():
    result = complying_loan_gate(
        facts(
            secured_by_registered_mortgage_over_real_property=True,
            security_coverage_at_first_made=None,
            maximum_term_years=D("25"),
        )
    )
    assert result.verdict is GateVerdict.UNKNOWN


def test_an_unknown_benchmark_year_makes_the_rate_limb_unknown():
    result = complying_loan_gate(facts(year_of_income_being_tested=parse_year("2027-28")))
    assert result.verdict is GateVerdict.UNKNOWN
    assert limb(result, "s 109N(1)(b)").state is LimbState.UNKNOWN


def test_no_year_at_all_is_unknown():
    result = complying_loan_gate(GateFacts(written_agreement=True))
    assert result.verdict is GateVerdict.UNKNOWN
    assert "year_loan_made" in result.reasons[0]


def test_the_floor_year_defaults_to_the_year_the_loan_was_made():
    """s 109N(1)(b) points at the benchmark rate for the year the loan was
    made, so that is the anchor when the operator nominates nothing else."""
    result = complying_loan_gate(
        GateFacts(
            written_agreement=True,
            terms_in_place_before_lodgment_day=True,
            maximum_term_years=D("7"),
            secured_by_registered_mortgage_over_real_property=False,
            interest_rate_for_years_after_year_loan_made=D("0.0477"),
            year_loan_made=parse_year("2022-23"),
        )
    )
    assert result.benchmark_year_used == "2022-23"
    assert result.verdict is GateVerdict.COMPLYING


def test_testing_a_later_year_carries_a_caveat_naming_the_divergence():
    result = complying_loan_gate(
        facts(year_loan_made=parse_year("2022-23"), year_of_income_being_tested=YEAR)
    )
    assert result.benchmark_year_used == "2026-27"
    assert any("s 109N(1)(b)" in caveat and "2022-23" in caveat for caveat in result.caveats)


def test_complying_always_carries_the_out_of_scope_caveat():
    result = complying_loan_gate(facts())
    assert any("Subdivision D" in caveat for caveat in result.caveats)


def test_from_mapping_reads_a_csv_row():
    row = {
        "loan_id": "L-1",
        "written_agreement": "true",
        "terms_in_place_before_lodgment_day": "yes",
        "maximum_term_years": "7",
        "secured_by_registered_mortgage_over_real_property": "no",
        "security_coverage_at_first_made": "unknown",
        "interest_rate_for_years_after_year_loan_made": "0.0877",
        "year_loan_made": "2026-27",
    }
    parsed = GateFacts.from_mapping(row)
    assert parsed.written_agreement is True
    assert parsed.terms_in_place_before_lodgment_day is True
    assert parsed.secured_by_registered_mortgage_over_real_property is False
    assert parsed.security_coverage_at_first_made is None
    assert parsed.maximum_term_years == D("7")
    assert complying_loan_gate(parsed).verdict is GateVerdict.COMPLYING


def test_a_blank_cell_is_unknown_not_false():
    parsed = GateFacts.from_mapping({"written_agreement": ""})
    assert parsed.written_agreement is None


def test_an_unrecognised_boolean_is_an_error_not_a_false():
    with pytest.raises(FactError, match="true, false, or unknown"):
        GateFacts.from_mapping({"written_agreement": "ture"})


def test_a_negative_interest_rate_is_refused():
    with pytest.raises(FactError):
        GateFacts.from_mapping({"interest_rate_for_years_after_year_loan_made": "-0.05"})


def test_a_nan_rate_is_refused():
    """NaN compares false against every threshold, so an unguarded one would
    sail through the benchmark test."""
    with pytest.raises(FactError, match="nan"):
        GateFacts.from_mapping({"interest_rate_for_years_after_year_loan_made": "nan"})
