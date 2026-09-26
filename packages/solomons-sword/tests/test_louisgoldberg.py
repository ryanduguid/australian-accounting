from datetime import date
from decimal import Decimal

import pytest
from louisgoldberg.division6 import (
    BeneficiaryEntitlement,
    TrustIncomeAssessment,
    calculate_proportionate_share,
)
from louisgoldberg.section99b import ForeignTrustReceipt, evaluate_section99b_liability
from louisgoldberg.section100a import Section100ARiskZone, evaluate_section100a_risk
from louisgoldberg.trust_resolution import TrustResolutionSchedule, validate_trust_resolution

# Every s 100A fact stated as False. A zone that depends on an unstated fact is
# refused, so a test about one fact states the other 6 rather than leaving them
# to a default the module no longer has.
ALL_FACTS_NEGATIVE = dict(
    beneficiary_is_adult_child=False,
    funds_retained_by_parents_without_loan=False,
    circular_flow_of_funds=False,
    corporate_beneficiary_unpaid_present_entitlement=False,
    beneficiary_actually_received_funds=False,
    funds_used_for_beneficiary_direct_benefit=False,
    commercial_loan_agreement_in_place=False,
    entitlement_applied_to_pre_18_expenses=False,
    received_within_two_years=False,
    retention_scenario_conditions_met=False,
    paragraph_32_exclusion_present=False,
)


def resident_adult(*args, **kwargs) -> BeneficiaryEntitlement:
    """A resident beneficiary not under a legal disability.

    Residency is a required input and legal disability is tristate, so both are
    stated here once rather than in every fixture below. The tests that turn on
    either fact construct the entitlement directly.
    """
    return BeneficiaryEntitlement(
        *args, is_resident=True, is_under_legal_disability=False, **kwargs
    )


def test_division6_proportionate_approach():
    # Trust with $100k accounting income, $120k s95 taxable net income (due to non-deductible adjustments)
    assessment = TrustIncomeAssessment(
        financial_year=2025,
        trust_name="Smith Family Trust",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("120000.00"),
        franking_credits=Decimal("15000.00"),
        beneficiaries=[
            resident_adult(
                beneficiary_name="Alice Smith",
                percentage_entitlement=Decimal("50.00"),
            ),
            resident_adult(
                beneficiary_name="Bob Smith",
                percentage_entitlement=Decimal("50.00"),
            ),
        ],
    )

    shares = calculate_proportionate_share(assessment)
    assert len(shares) == 2
    # 50% of $120,000 = $60,000
    assert shares[0].section95_net_income_share == Decimal("60000.00")
    # 50% of $15,000 = $7,500
    assert shares[0].franking_credit_grossup == Decimal("7500.00")
    # s 207-35 already includes the gross-up in the trust's s 95 net income, so
    # the share is the net income share alone and the credit is reported
    # separately for the s 207-45 offset.
    assert shares[0].total_taxable_component == Decimal("60000.00")


@pytest.mark.parametrize("entitlements, credits, expected", [
    ([1, 1, 1], "1.00", ["0.34", "0.33", "0.33"]),
    ([1] * 6, "1.00", ["0.15"] + ["0.17"] * 5),
    ([1] * 7, "1.00", ["0.16"] + ["0.14"] * 6),
    ([1] * 6, "0.03", ["0.00"] * 3 + ["0.01"] * 3),
    ([2, 3, 5], "0.05", ["0.01", "0.02", "0.02"]),
    ([1, 1], "0.00", ["0.00", "0.00"]),
])
def test_franking_credits_reconcile_without_changing_taxable_income(entitlements, credits, expected):
    assessment = TrustIncomeAssessment(
        financial_year=2026, trust_name="Synthetic Trust",
        trust_accounting_income=Decimal(sum(entitlements)),
        section95_net_taxable_income=Decimal("120.00"),
        franking_credits=Decimal(credits),
        beneficiaries=[resident_adult(str(i), fixed_entitlement_amount=Decimal(amount))
                       for i, amount in enumerate(entitlements)],
    )
    shares = calculate_proportionate_share(assessment)
    assert [s.franking_credit_grossup for s in shares] == [Decimal(value) for value in expected]
    assert sum(s.franking_credit_grossup for s in shares) == Decimal(credits)
    assert sum(s.total_taxable_component for s in shares) == Decimal("120.00")
    assert all(s.total_taxable_component == s.section95_net_income_share for s in shares)


def test_negative_franking_credit_pool_is_refused():
    assessment = TrustIncomeAssessment(
        financial_year=2026, trust_name="Synthetic Trust",
        trust_accounting_income=Decimal("3.00"), section95_net_taxable_income=Decimal("3.00"),
        franking_credits=Decimal("-1.00"),
        beneficiaries=[resident_adult(str(i), fixed_entitlement_amount=Decimal("1.00")) for i in range(3)],
    )
    with pytest.raises(ValueError, match="franking credits must be non-negative"):
        calculate_proportionate_share(assessment)

def test_section100a_risk_zones():
    # Red zone scenario 1 (PCG 2022/2 paragraph 34): an adult child's
    # entitlement paid to a parent for expenses incurred before age 18.
    red_res = evaluate_section100a_risk(
        beneficiary_name="Charlie (Adult Child)",
        distribution_amount=Decimal("45000.00"),
        beneficiary_is_adult_child=True,
        entitlement_applied_to_pre_18_expenses=True,
    )
    assert red_res.risk_zone == Section100ARiskZone.RED
    assert red_res.is_ordinary_family_dealing is None
    # The s 99A rate is named, not hard-coded: the old "47%" was uncited and
    # conflated the trustee rate with the individual top effective rate.
    assert "47%" not in red_res.tax_consequence_summary
    assert "top rate applying under s 99A" in red_res.tax_consequence_summary

    # Green Zone: Beneficiary receives and retains funds
    green_res = evaluate_section100a_risk(
        beneficiary_name="David (Adult Child)",
        distribution_amount=Decimal("30000.00"),
        **{
            **ALL_FACTS_NEGATIVE,
            "beneficiary_is_adult_child": True,
            "beneficiary_actually_received_funds": True,
            "received_within_two_years": True,
        },
    )
    assert green_res.risk_zone == Section100ARiskZone.GREEN
    # PCG 2022/2's green zone is a compliance-resourcing stance; it does not
    # decide the s 100A(13) ordinary family dealing exception.
    assert green_res.is_ordinary_family_dealing is None

def test_section99b_corpus_exemption():
    # $100k foreign trust distribution, $40k is original settled corpus
    receipt = ForeignTrustReceipt(
        beneficiary_name="Emma Resident",
        gross_amount_received_aud=Decimal("100000.00"),
        beneficiary_was_resident_during_year=True,
        corpus_amount_aud=Decimal("40000.00"),
    )
    res = evaluate_section99b_liability(receipt)
    assert res.corpus_exemption == Decimal("40000.00")
    assert res.assessable_income_under_s99b == Decimal("60000.00")

def test_trust_resolution_validation():
    # Valid timely resolution
    valid_sched = TrustResolutionSchedule(
        trust_name="Smith Family Trust",
        financial_year=2025,
        resolution_date=date(2025, 6, 25),
        is_signed_by_trustee=True,
        streaming_powers_in_deed=True,
        default_beneficiary_clause_exists=True,
        allocated_percentages_total=Decimal("100.00"),
    )
    is_valid, issues = validate_trust_resolution(valid_sched)
    assert is_valid is True
    assert len(issues) == 0

    # Late resolution after 30 June
    late_sched = TrustResolutionSchedule(
        trust_name="Smith Family Trust",
        financial_year=2025,
        resolution_date=date(2025, 7, 5),
        is_signed_by_trustee=True,
        streaming_powers_in_deed=True,
        default_beneficiary_clause_exists=True,
        allocated_percentages_total=Decimal("100.00"),
    )
    is_valid_late, issues_late = validate_trust_resolution(late_sched)
    assert is_valid_late is False
    assert any("after 30 June" in issue for issue in issues_late)

def test_section100a_does_not_default_to_green():
    # No facts at all: the result names the gap rather than assigning any zone.
    result = evaluate_section100a_risk(
        beneficiary_name="Unspecified",
        distribution_amount=Decimal("10000.00"),
    )
    assert result.risk_zone == Section100ARiskZone.FACTS_NOT_ESTABLISHED
    assert result.unestablished_facts == tuple(ALL_FACTS_NEGATIVE)
    # No zone assigned decides nothing, including the s 100A(13) exception.
    assert result.is_ordinary_family_dealing is None

    # Every fact stated as False is a different answer: the arrangement meets no
    # green criterion and matches no red example.
    stated = evaluate_section100a_risk(
        beneficiary_name="Unspecified",
        distribution_amount=Decimal("10000.00"),
        **ALL_FACTS_NEGATIVE,
    )
    assert stated.risk_zone == Section100ARiskZone.OUTSIDE_GREEN
    assert stated.unestablished_facts == ()


def test_an_unstated_fact_cannot_reach_the_green_zone():
    # One asserted mitigator with the rest unstated used to return GREEN, which
    # reads as the ATO declining to review an arrangement nobody described.
    for mitigator in (
        "beneficiary_actually_received_funds",
        "funds_used_for_beneficiary_direct_benefit",
        "commercial_loan_agreement_in_place",
    ):
        result = evaluate_section100a_risk(
            beneficiary_name="A",
            distribution_amount=Decimal("50000.00"),
            **{mitigator: True},
        )
        assert result.risk_zone == Section100ARiskZone.FACTS_NOT_ESTABLISHED
        assert mitigator not in result.unestablished_facts
        assert result.unestablished_facts
        assert all(fact in result.tax_consequence_summary for fact in result.unestablished_facts)
        assert result.is_ordinary_family_dealing is None

    # State the remaining facts and the same mitigator reaches the green zone.
    established = evaluate_section100a_risk(
        beneficiary_name="A",
        distribution_amount=Decimal("50000.00"),
        **{**ALL_FACTS_NEGATIVE, "funds_used_for_beneficiary_direct_benefit": True},
    )
    assert established.risk_zone == Section100ARiskZone.GREEN
    assert established.unestablished_facts == ()


def test_a_corporate_upe_without_a_loan_is_outside_green_not_red():
    # PCG 2022/2 has no red zone scenario for a corporate UPE left unpaid, and
    # after Bendel [2026] HCA 18 the UPE is not itself a Division 7A loan. It
    # fails green zone scenario 3B, which leaves the arrangement unzoned.
    unstated_loan = evaluate_section100a_risk(
        beneficiary_name="Family Co",
        distribution_amount=Decimal("80000.00"),
        **{**ALL_FACTS_NEGATIVE, "corporate_beneficiary_unpaid_present_entitlement": True,
           "commercial_loan_agreement_in_place": None},
    )
    assert unstated_loan.risk_zone == Section100ARiskZone.FACTS_NOT_ESTABLISHED
    assert not any("Corporate beneficiary UPE" in f for f in unstated_loan.risk_factors_identified)

    stated = evaluate_section100a_risk(
        beneficiary_name="Family Co",
        distribution_amount=Decimal("80000.00"),
        **{**ALL_FACTS_NEGATIVE, "corporate_beneficiary_unpaid_present_entitlement": True},
    )
    assert stated.risk_zone == Section100ARiskZone.OUTSIDE_GREEN
    assert any("Corporate beneficiary UPE" in f and "Bendel" in f
               for f in stated.risk_factors_identified)

    # A green route elsewhere does not survive the unpaid, unlent entitlement.
    with_direct_benefit = evaluate_section100a_risk(
        beneficiary_name="Family Co",
        distribution_amount=Decimal("80000.00"),
        **{**ALL_FACTS_NEGATIVE, "corporate_beneficiary_unpaid_present_entitlement": True,
           "funds_used_for_beneficiary_direct_benefit": True},
    )
    assert with_direct_benefit.risk_zone == Section100ARiskZone.OUTSIDE_GREEN


def test_parental_retention_alone_is_outside_green_not_red():
    # Red zone scenario 1 is narrower than retention in general: it needs the
    # entitlement applied to expenses incurred before the beneficiary turned 18.
    result = evaluate_section100a_risk(
        beneficiary_name="Charlie",
        distribution_amount=Decimal("45000.00"),
        **{**ALL_FACTS_NEGATIVE, "beneficiary_is_adult_child": True,
           "funds_retained_by_parents_without_loan": True},
    )
    assert result.risk_zone == Section100ARiskZone.OUTSIDE_GREEN
    assert any("retained by parents" in f for f in result.risk_factors_identified)


@pytest.mark.parametrize(
    "facts",
    [
        # Receipt after the 2-year window fails green zone scenario 2.
        {"beneficiary_actually_received_funds": True},
        # A loan on commercial terms without the rest of scenario 3A or 3B.
        {"commercial_loan_agreement_in_place": True},
        # Any paragraph 32 exclusion defeats every green scenario.
        {"funds_used_for_beneficiary_direct_benefit": True, "paragraph_32_exclusion_present": True},
        {"beneficiary_actually_received_funds": True, "received_within_two_years": True,
         "paragraph_32_exclusion_present": True},
    ],
)
def test_a_green_scenario_needs_every_condition_its_paragraph_names(facts):
    result = evaluate_section100a_risk(
        beneficiary_name="A",
        distribution_amount=Decimal("50000.00"),
        **{**ALL_FACTS_NEGATIVE, **facts},
    )
    assert result.risk_zone == Section100ARiskZone.OUTSIDE_GREEN


def test_an_established_red_zone_trigger_stands_while_other_facts_are_unstated():
    # RED is the guideline's answer on the facts it was given, and it is not the
    # favourable direction, so an unstated fact elsewhere does not withdraw it.
    result = evaluate_section100a_risk(
        beneficiary_name="Charlie",
        distribution_amount=Decimal("45000.00"),
        circular_flow_of_funds=True,
    )
    assert result.risk_zone == Section100ARiskZone.RED
    assert result.unestablished_facts


@pytest.mark.parametrize(
    "amount",
    [
        Decimal("0"),
        Decimal("-0.01"),
        Decimal("NaN"),
        Decimal("Infinity"),
        Decimal("-Infinity"),
    ],
)
def test_section100a_rejects_non_positive_or_non_finite_distributions(amount):
    with pytest.raises(ValueError, match="positive and finite"):
        evaluate_section100a_risk(
            beneficiary_name="Example Beneficiary",
            distribution_amount=amount,
        )


def test_division6_rejects_percentages_that_do_not_total_100():
    assessment = TrustIncomeAssessment(
        financial_year=2025,
        trust_name="Overallocated Trust",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("100000.00"),
        beneficiaries=[
            resident_adult(beneficiary_name="A", percentage_entitlement=Decimal("60.00")),
            resident_adult(beneficiary_name="B", percentage_entitlement=Decimal("60.00")),
        ],
    )
    try:
        calculate_proportionate_share(assessment)
    except ValueError as exc:
        assert "not 100" in str(exc)
    else:
        raise AssertionError("over-allocation must be refused")


def test_trust_resolution_rejects_zero_percent_and_missing_deed_facts():
    schedule = TrustResolutionSchedule(
        trust_name="Smith Family Trust",
        financial_year=2025,
        resolution_date=date(2025, 6, 25),
        is_signed_by_trustee=True,
        streaming_powers_in_deed=False,
        default_beneficiary_clause_exists=False,
        allocated_percentages_total=Decimal("0.00"),
    )
    is_valid, issues = validate_trust_resolution(schedule)
    assert is_valid is False
    assert any("100%" in issue for issue in issues)
    assert any("streaming" in issue for issue in issues)
    assert any("default beneficiary" in issue for issue in issues)


def test_streamed_amounts_are_refused_not_misallocated():
    # Streaming previously allocated 150% of the franking credit pool.
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("130000.00"),
        franked_dividends=Decimal("70000.00"), franking_credits=Decimal("30000.00"),
        beneficiaries=[
            resident_adult("A", percentage_entitlement=Decimal("50.00"),
                                   specifically_streamed_franked_dividends=Decimal("70000.00")),
            resident_adult("B", percentage_entitlement=Decimal("50.00")),
        ],
    )
    with pytest.raises(ValueError, match="Division 6E"):
        calculate_proportionate_share(t)


def test_shares_reconcile_to_the_net_income():
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("100000.00"),
        beneficiaries=[
            resident_adult("A", fixed_entitlement_amount=Decimal("33333.33")),
            resident_adult("B", fixed_entitlement_amount=Decimal("33333.33")),
            resident_adult("C", fixed_entitlement_amount=Decimal("33333.34")),
        ],
    )
    shares = calculate_proportionate_share(t)
    assert sum(s.section95_net_income_share for s in shares) == Decimal("100000.00")


def test_franking_credits_are_not_added_on_top_of_the_net_income_share():
    # s 207-35 already includes the gross-up in the trust's s 95 net income.
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("130000.00"),
        franked_dividends=Decimal("70000.00"), franking_credits=Decimal("30000.00"),
        beneficiaries=[resident_adult("A", percentage_entitlement=Decimal("100.00"))],
    )
    share = calculate_proportionate_share(t)[0]
    assert share.section95_net_income_share == Decimal("130000.00")
    assert share.franking_credit_grossup == Decimal("30000.00")
    assert share.total_taxable_component == Decimal("130000.00")


def test_unmodelled_cases_fail_closed():
    base = dict(financial_year=2025, trust_name="T",
                trust_accounting_income=Decimal("100000.00"),
                section95_net_taxable_income=Decimal("100000.00"))
    with pytest.raises(ValueError, match="s 99 or s 99A"):
        calculate_proportionate_share(TrustIncomeAssessment(beneficiaries=[], **base))
    with pytest.raises(ValueError, match="s 99 or s 99A"):
        calculate_proportionate_share(TrustIncomeAssessment(
            beneficiaries=[resident_adult("A", percentage_entitlement=Decimal("100.00"))],
            financial_year=2025, trust_name="T",
            trust_accounting_income=Decimal("0.00"),
            section95_net_taxable_income=Decimal("50000.00")))
    with pytest.raises(ValueError, match="non-resident"):
        calculate_proportionate_share(TrustIncomeAssessment(
            beneficiaries=[BeneficiaryEntitlement("NR", is_resident=False,
                                                  is_under_legal_disability=False,
                                                  percentage_entitlement=Decimal("100.00"))],
            **base))
    with pytest.raises(ValueError, match="at most 100 per cent"):
        calculate_proportionate_share(TrustIncomeAssessment(
            beneficiaries=[resident_adult("A", percentage_entitlement=Decimal("150.00")),
                           resident_adult("B", percentage_entitlement=Decimal("-50.00"))],
            **base))


def test_minor_beneficiary_is_assessed_to_the_trustee():
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("100000.00"),
        beneficiaries=[BeneficiaryEntitlement("Minor", is_resident=True,
                                              is_under_legal_disability=True,
                                              percentage_entitlement=Decimal("100.00"))],
    )
    assert "s 98" in calculate_proportionate_share(t)[0].assessed_under_section


def test_s99b_corpus_proviso_and_residency():
    from louisgoldberg.section99b import ForeignTrustReceipt, evaluate_section99b_liability
    r = evaluate_section99b_liability(ForeignTrustReceipt(
        "A", Decimal("100000.00"), True, corpus_amount_aud=Decimal("20000.00"),
        corpus_attributable_to_notional_assessable_income_aud=Decimal("5000.00")))
    assert r.corpus_exemption == Decimal("15000.00")
    assert r.assessable_income_under_s99b == Decimal("85000.00")
    with pytest.raises(ValueError, match="resident"):
        evaluate_section99b_liability(ForeignTrustReceipt(
            "B", Decimal("100000.00"), beneficiary_was_resident_during_year=False))
    with pytest.raises(ValueError, match="residency during the year of income is not established"):
        evaluate_section99b_liability(ForeignTrustReceipt(
            "B", Decimal("100000.00"), None))  # type: ignore[arg-type]
    with pytest.raises(ValueError, match="non-negative"):
        evaluate_section99b_liability(ForeignTrustReceipt("C", Decimal("100000.00"), True,
                                                          corpus_amount_aud=Decimal("-1.00")))


def test_green_zone_does_not_claim_the_ordinary_family_dealing_exception():
    from louisgoldberg.section100a import Section100ARiskZone, evaluate_section100a_risk
    res = evaluate_section100a_risk(
        beneficiary_name="A", distribution_amount=Decimal("50000.00"),
        **{**ALL_FACTS_NEGATIVE, "beneficiary_actually_received_funds": True,
           "received_within_two_years": True})
    assert res.risk_zone == Section100ARiskZone.GREEN
    assert res.is_ordinary_family_dealing is None
    assert "not a determination" in res.tax_consequence_summary


def test_entitlements_that_do_not_reconcile_exactly_are_refused():
    # Three 33.33% shares of $10m leave $1,000 unallocated. Tolerating that as a
    # rounding difference handed the whole $1,000 to one beneficiary.
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("10000000.00"),
        section95_net_taxable_income=Decimal("10000000.00"),
        beneficiaries=[
            resident_adult("A", percentage_entitlement=Decimal("33.33")),
            resident_adult("B", percentage_entitlement=Decimal("33.33")),
            resident_adult("C", percentage_entitlement=Decimal("33.33")),
        ],
    )
    with pytest.raises(ValueError) as excinfo:
        calculate_proportionate_share(t)
    assert "99.99%, not 100%" in str(excinfo.value)
    assert "1000.00 of the income of the trust estate is unallocated" in str(excinfo.value)

    # Same gap on the fixed basis: a cent short of the income of the trust
    # estate is a cent no beneficiary is entitled to.
    short = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("100000.00"),
        beneficiaries=[resident_adult(n, fixed_entitlement_amount=Decimal("33333.33"))
                       for n in "ABC"],
    )
    with pytest.raises(ValueError) as fixed_exc:
        calculate_proportionate_share(short)
    assert "fixed entitlements sum to 99999.99, not the 100000.00" in str(fixed_exc.value)
    assert "0.01 of the income of the trust estate is unallocated" in str(fixed_exc.value)

    # Mixed bases reconcile against the same income of the trust estate.
    mixed = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("100000.00"),
        beneficiaries=[
            resident_adult("A", fixed_entitlement_amount=Decimal("50000.00")),
            resident_adult("B", percentage_entitlement=Decimal("40.00")),
        ],
    )
    with pytest.raises(ValueError, match="10000.00 of the income of the trust estate is unallocated"):
        calculate_proportionate_share(mixed)

    # Over-allocation is named as such, not as unallocated income.
    over = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("100000.00"),
        beneficiaries=[
            resident_adult("A", percentage_entitlement=Decimal("60.00")),
            resident_adult("B", percentage_entitlement=Decimal("40.01")),
        ],
    )
    with pytest.raises(ValueError, match="over-allocate the income of the trust estate by 10.00"):
        calculate_proportionate_share(over)


def test_equal_fixed_entitlements_that_exhaust_the_income_are_allocated():
    # Seven equal fixed entitlements imply 14.29% each, 100.03% once rounded,
    # but they sum exactly to the income of the trust estate.
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("70000.00"),
        section95_net_taxable_income=Decimal("70000.00"),
        beneficiaries=[resident_adult(n, fixed_entitlement_amount=Decimal("10000.00"))
                       for n in "ABCDEFG"],
    )
    shares = calculate_proportionate_share(t)
    assert len(shares) == 7
    assert sum(s.section95_net_income_share for s in shares) == Decimal("70000.00")
    assert shares[0].proportion_percentage == Decimal("14.29")


def test_rounding_residual_moves_only_sub_cent_dust():
    # Thirds of $10m as fixed amounts reconcile exactly; the larger s 95 pool
    # makes the quantised shares overshoot by one cent, which the residual step
    # takes back off the largest share.
    fixed = [Decimal("3333333.33"), Decimal("3333333.33"), Decimal("3333333.34")]
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("10000000.00"),
        section95_net_taxable_income=Decimal("12000000.00"),
        beneficiaries=[resident_adult(n, fixed_entitlement_amount=f)
                       for n, f in zip("ABC", fixed)],
    )
    shares = calculate_proportionate_share(t)
    assert sum(s.section95_net_income_share for s in shares) == Decimal("12000000.00")
    for share, amount in zip(shares, fixed):
        exact = t.section95_net_taxable_income * (amount / t.trust_accounting_income)
        assert abs(share.section95_net_income_share - exact) <= Decimal("0.01")


def test_section95_loss_is_not_allocated_to_beneficiaries():
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("-40000.00"),
        beneficiaries=[resident_adult("A", percentage_entitlement=Decimal("100.00"))],
    )
    with pytest.raises(ValueError, match="loss is not allocated to beneficiaries"):
        calculate_proportionate_share(t)


@pytest.mark.parametrize("count,pool", [(4, "0.02"), (10, "0.05"), (4, "0.01"), (4, "0.00"), (4, "100.00")])
def test_section95_rounding_never_creates_negative_taxable_shares(count, pool):
    assessment = TrustIncomeAssessment(
        financial_year=2026,
        trust_name="Example Trust",
        trust_accounting_income=Decimal(100 * count),
        section95_net_taxable_income=Decimal(pool),
        beneficiaries=[resident_adult(str(i), fixed_entitlement_amount=Decimal("100"))
                       for i in range(count)],
    )
    shares = calculate_proportionate_share(assessment)
    assert all(share.section95_net_income_share >= 0 for share in shares)
    assert sum(share.section95_net_income_share for share in shares) == Decimal(pool)
    assert all(share.total_taxable_component == share.section95_net_income_share for share in shares)


def test_resolution_before_the_income_year_started_is_refused():
    schedule = TrustResolutionSchedule(
        trust_name="Smith Family Trust",
        financial_year=2025,
        resolution_date=date(2015, 6, 25),
        is_signed_by_trustee=True,
        streaming_powers_in_deed=True,
        default_beneficiary_clause_exists=True,
        allocated_percentages_total=Decimal("100.00"),
    )
    is_valid, issues = validate_trust_resolution(schedule)
    assert is_valid is False
    assert any("predates the 2025 income year" in issue for issue in issues)


def test_funds_not_received_is_recorded_as_a_risk_factor():
    # Non-receipt is a fact s 100A turns on, so it is recorded, but alone it
    # establishes no red-zone pattern, and green-zone dealings like a Div 7A
    # commercial loan involve non-receipt by definition, so the recorded fact
    # must not drive the zone.
    not_received = evaluate_section100a_risk(
        beneficiary_name="A", distribution_amount=Decimal("50000.00"),
        **ALL_FACTS_NEGATIVE)
    unstated = evaluate_section100a_risk(
        beneficiary_name="A", distribution_amount=Decimal("50000.00"),
        **{**ALL_FACTS_NEGATIVE, "beneficiary_actually_received_funds": None})
    assert not_received != unstated
    assert any("did not receive the funds" in f for f in not_received.risk_factors_identified)
    assert unstated.risk_factors_identified == []
    assert not_received.risk_zone == Section100ARiskZone.OUTSIDE_GREEN
    assert not_received.is_ordinary_family_dealing is None
    # Unstated receipt is not the same answer as stated non-receipt: no zone.
    assert unstated.risk_zone == Section100ARiskZone.FACTS_NOT_ESTABLISHED

    lent_commercially = evaluate_section100a_risk(
        beneficiary_name="A", distribution_amount=Decimal("50000.00"),
        **{**ALL_FACTS_NEGATIVE, "commercial_loan_agreement_in_place": True,
           "retention_scenario_conditions_met": True})
    assert lent_commercially.risk_zone == Section100ARiskZone.GREEN
    assert any("did not receive the funds" in f for f in lent_commercially.risk_factors_identified)


def test_cent_exact_mixed_entitlements_are_allocated():
    # A fixed leg plus a percentage leg that lands on a whole cent reconciles
    # even though the percentage's exact dollar value has no finite sub-cent
    # representation: reconciliation happens at the module's cent granularity.
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("1234567.01"),
        section95_net_taxable_income=Decimal("1234567.01"),
        beneficiaries=[
            resident_adult("A", fixed_entitlement_amount=Decimal("823085.83")),
            resident_adult("B", percentage_entitlement=Decimal("33.33")),
        ],
    )
    shares = calculate_proportionate_share(t)
    assert sum(s.section95_net_income_share for s in shares) == Decimal("1234567.01")


def test_residency_is_a_required_input_not_a_default():
    # A default of True routed an unstated residency straight past the
    # s 98(2A)/(3) refusal, which is the one case the model cannot compute.
    with pytest.raises(TypeError, match="is_resident"):
        BeneficiaryEntitlement("A", percentage_entitlement=Decimal("100.00"))


def test_an_unestablished_legal_disability_is_refused_not_read_as_an_adult():
    base = dict(financial_year=2025, trust_name="T",
                trust_accounting_income=Decimal("100000.00"),
                section95_net_taxable_income=Decimal("100000.00"))
    unstated = TrustIncomeAssessment(
        beneficiaries=[BeneficiaryEntitlement("A", is_resident=True,
                                              percentage_entitlement=Decimal("100.00"))],
        **base,
    )
    with pytest.raises(ValueError, match="legal disability is not established"):
        calculate_proportionate_share(unstated)

    # Stated either way, the allocation runs and names the assessed taxpayer.
    adult = TrustIncomeAssessment(
        beneficiaries=[resident_adult("A", percentage_entitlement=Decimal("100.00"))], **base
    )
    assert "s 97" in calculate_proportionate_share(adult)[0].assessed_under_section


def test_an_unestablished_resolution_fact_is_neither_valid_nor_a_breach():
    schedule = TrustResolutionSchedule(
        trust_name="Smith Family Trust",
        financial_year=2025,
        resolution_date=date(2025, 6, 25),
        is_signed_by_trustee=True,
        streaming_powers_in_deed=None,
        default_beneficiary_clause_exists=None,
        allocated_percentages_total=Decimal("100.00"),
    )
    is_valid, issues = validate_trust_resolution(schedule)
    assert is_valid is None
    assert not is_valid  # a caller reading it as a boolean gets "not validated"
    assert any("Not established: streaming_powers_in_deed, "
               "default_beneficiary_clause_exists" in issue for issue in issues)
    # The unestablished facts are not reported as deed defects.
    assert not any("Deed does not record streaming powers" in issue for issue in issues)

    # Stated as False, the same facts are breaches and the result is False.
    stated = TrustResolutionSchedule(
        trust_name="Smith Family Trust",
        financial_year=2025,
        resolution_date=date(2025, 6, 25),
        is_signed_by_trustee=True,
        streaming_powers_in_deed=False,
        default_beneficiary_clause_exists=False,
        allocated_percentages_total=Decimal("100.00"),
    )
    stated_valid, stated_issues = validate_trust_resolution(stated)
    assert stated_valid is False
    assert any("Deed does not record streaming powers" in issue for issue in stated_issues)
    assert not any("Not established" in issue for issue in stated_issues)


def test_s99b_names_every_nil_exemption_in_one_caveat():
    # The nil defaults stay: they give the largest assessable amount. What they
    # cannot do is read as figures somebody established.
    res = evaluate_section99b_liability(ForeignTrustReceipt(
        beneficiary_name="A", gross_amount_received_aud=Decimal("100000.00"),
        beneficiary_was_resident_during_year=True,
        corpus_amount_aud=Decimal("40000.00")))
    assert res.assessable_income_under_s99b == Decimal("60000.00")
    assert len(res.caveats) == 1
    assert "not_assessable_to_resident_aud" in res.caveats[0]
    assert "already_assessed_under_div6_aud" in res.caveats[0]
    assert "corpus_amount_aud" not in res.caveats[0]

    # Every exemption and the s 99B(2)(a) add-back supplied and positive:
    # nothing to caveat.
    full = evaluate_section99b_liability(ForeignTrustReceipt(
        beneficiary_name="A", gross_amount_received_aud=Decimal("100000.00"),
        beneficiary_was_resident_during_year=True,
        corpus_amount_aud=Decimal("40000.00"),
        corpus_attributable_to_notional_assessable_income_aud=Decimal("5000.00"),
        not_assessable_to_resident_aud=Decimal("10000.00"),
        already_assessed_under_div6_aud=Decimal("5000.00")))
    assert full.caveats == ()


def test_s99b_residency_is_a_required_input_not_a_default():
    # A default of True assessed a receipt on an assumed residency, and the
    # command line had no way to state the fact at all.
    with pytest.raises(TypeError, match="beneficiary_was_resident_during_year"):
        ForeignTrustReceipt("A", Decimal("100000.00"))

    # Stated either way, the answers are the ones the engine always gave.
    resident = evaluate_section99b_liability(
        ForeignTrustReceipt("A", Decimal("100000.00"), True,
                            corpus_amount_aud=Decimal("40000.00")))
    assert resident.assessable_income_under_s99b == Decimal("60000.00")
    with pytest.raises(ValueError, match="only where the beneficiary was a resident"):
        evaluate_section99b_liability(ForeignTrustReceipt("A", Decimal("100000.00"), False))


def test_s99b_names_an_omitted_corpus_add_back_in_the_caveat():
    # The s 99B(2)(a) add-back is the one nil that favours the taxpayer: omitting
    # it exempts the whole corpus and lowers the assessable amount.
    omitted = evaluate_section99b_liability(ForeignTrustReceipt(
        beneficiary_name="A", gross_amount_received_aud=Decimal("100000.00"),
        beneficiary_was_resident_during_year=True,
        corpus_amount_aud=Decimal("40000.00")))
    assert omitted.corpus_exemption == Decimal("40000.00")
    assert len(omitted.caveats) == 1
    assert "corpus_attributable_to_notional_assessable_income_aud" in omitted.caveats[0]
    assert "smallest assessable amount" in omitted.caveats[0]

    # Stated, it reduces the exemption and is not caveated.
    stated = evaluate_section99b_liability(ForeignTrustReceipt(
        beneficiary_name="A", gross_amount_received_aud=Decimal("100000.00"),
        beneficiary_was_resident_during_year=True,
        corpus_amount_aud=Decimal("40000.00"),
        corpus_attributable_to_notional_assessable_income_aud=Decimal("5000.00")))
    assert stated.corpus_exemption == Decimal("35000.00")
    assert "corpus_attributable" not in stated.caveats[0]

    # With no corpus claimed there is nothing for the add-back to reduce.
    no_corpus = evaluate_section99b_liability(ForeignTrustReceipt(
        beneficiary_name="A", gross_amount_received_aud=Decimal("100000.00"),
        beneficiary_was_resident_during_year=True))
    assert "corpus_attributable" not in no_corpus.caveats[0]
