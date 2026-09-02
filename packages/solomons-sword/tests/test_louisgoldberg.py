import pytest
from datetime import date
from decimal import Decimal
from louisgoldberg.division6 import TrustIncomeAssessment, BeneficiaryEntitlement, calculate_proportionate_share
from louisgoldberg.section100a import evaluate_section100a_risk, Section100ARiskZone
from louisgoldberg.section99b import ForeignTrustReceipt, evaluate_section99b_liability
from louisgoldberg.trust_resolution import TrustResolutionSchedule, validate_trust_resolution

def test_division6_proportionate_approach():
    # Trust with $100k accounting income, $120k s95 taxable net income (due to non-deductible adjustments)
    assessment = TrustIncomeAssessment(
        financial_year=2025,
        trust_name="Smith Family Trust",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("120000.00"),
        franking_credits=Decimal("15000.00"),
        beneficiaries=[
            BeneficiaryEntitlement(
                beneficiary_name="Alice Smith",
                percentage_entitlement=Decimal("50.00"),
            ),
            BeneficiaryEntitlement(
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

def test_section100a_risk_zones():
    # Red Zone: Adult child distribution retained by parents without loan
    red_res = evaluate_section100a_risk(
        beneficiary_name="Charlie (Adult Child)",
        distribution_amount=Decimal("45000.00"),
        beneficiary_is_adult_child=True,
        funds_retained_by_parents_without_loan=True,
    )
    assert red_res.risk_zone == Section100ARiskZone.RED
    assert red_res.is_ordinary_family_dealing is False
    # The s 99A rate is named, not hard-coded: the old "47%" was uncited and
    # conflated the trustee rate with the individual top effective rate.
    assert "47%" not in red_res.tax_consequence_summary
    assert "top rate applying under s 99A" in red_res.tax_consequence_summary

    # Green Zone: Beneficiary receives and retains funds
    green_res = evaluate_section100a_risk(
        beneficiary_name="David (Adult Child)",
        distribution_amount=Decimal("30000.00"),
        beneficiary_is_adult_child=True,
        funds_retained_by_parents_without_loan=False,
        beneficiary_actually_received_funds=True,
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
    result = evaluate_section100a_risk(
        beneficiary_name="Unspecified",
        distribution_amount=Decimal("10000.00"),
    )
    assert result.risk_zone == Section100ARiskZone.OUTSIDE_GREEN
    # No zone assigned decides nothing, including the s 100A(13) exception.
    assert result.is_ordinary_family_dealing is None


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
            BeneficiaryEntitlement(beneficiary_name="A", percentage_entitlement=Decimal("60.00")),
            BeneficiaryEntitlement(beneficiary_name="B", percentage_entitlement=Decimal("60.00")),
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
            BeneficiaryEntitlement("A", percentage_entitlement=Decimal("50.00"),
                                   specifically_streamed_franked_dividends=Decimal("70000.00")),
            BeneficiaryEntitlement("B", percentage_entitlement=Decimal("50.00")),
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
            BeneficiaryEntitlement("A", fixed_entitlement_amount=Decimal("33333.33")),
            BeneficiaryEntitlement("B", fixed_entitlement_amount=Decimal("33333.33")),
            BeneficiaryEntitlement("C", fixed_entitlement_amount=Decimal("33333.34")),
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
        beneficiaries=[BeneficiaryEntitlement("A", percentage_entitlement=Decimal("100.00"))],
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
            beneficiaries=[BeneficiaryEntitlement("A", percentage_entitlement=Decimal("100.00"))],
            financial_year=2025, trust_name="T",
            trust_accounting_income=Decimal("0.00"),
            section95_net_taxable_income=Decimal("50000.00")))
    with pytest.raises(ValueError, match="non-resident"):
        calculate_proportionate_share(TrustIncomeAssessment(
            beneficiaries=[BeneficiaryEntitlement("NR", is_resident=False, percentage_entitlement=Decimal("100.00"))],
            **base))
    with pytest.raises(ValueError, match="at most 100 per cent"):
        calculate_proportionate_share(TrustIncomeAssessment(
            beneficiaries=[BeneficiaryEntitlement("A", percentage_entitlement=Decimal("150.00")),
                           BeneficiaryEntitlement("B", percentage_entitlement=Decimal("-50.00"))],
            **base))


def test_minor_beneficiary_is_assessed_to_the_trustee():
    t = TrustIncomeAssessment(
        financial_year=2025, trust_name="T",
        trust_accounting_income=Decimal("100000.00"),
        section95_net_taxable_income=Decimal("100000.00"),
        beneficiaries=[BeneficiaryEntitlement("Minor", is_under_legal_disability=True,
                                              percentage_entitlement=Decimal("100.00"))],
    )
    assert "s 98" in calculate_proportionate_share(t)[0].assessed_under_section


def test_s99b_corpus_proviso_and_residency():
    from louisgoldberg.section99b import ForeignTrustReceipt, evaluate_section99b_liability
    r = evaluate_section99b_liability(ForeignTrustReceipt(
        "A", Decimal("100000.00"), corpus_amount_aud=Decimal("20000.00"),
        corpus_attributable_to_notional_assessable_income_aud=Decimal("5000.00")))
    assert r.corpus_exemption == Decimal("15000.00")
    assert r.assessable_income_under_s99b == Decimal("85000.00")
    with pytest.raises(ValueError, match="resident"):
        evaluate_section99b_liability(ForeignTrustReceipt(
            "B", Decimal("100000.00"), beneficiary_was_resident_during_year=False))
    with pytest.raises(ValueError, match="non-negative"):
        evaluate_section99b_liability(ForeignTrustReceipt("C", Decimal("100000.00"),
                                                          corpus_amount_aud=Decimal("-1.00")))


def test_green_zone_does_not_claim_the_ordinary_family_dealing_exception():
    from louisgoldberg.section100a import evaluate_section100a_risk, Section100ARiskZone
    res = evaluate_section100a_risk(
        beneficiary_name="A", distribution_amount=Decimal("50000.00"),
        beneficiary_actually_received_funds=True)
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
            BeneficiaryEntitlement("A", percentage_entitlement=Decimal("33.33")),
            BeneficiaryEntitlement("B", percentage_entitlement=Decimal("33.33")),
            BeneficiaryEntitlement("C", percentage_entitlement=Decimal("33.33")),
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
        beneficiaries=[BeneficiaryEntitlement(n, fixed_entitlement_amount=Decimal("33333.33"))
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
            BeneficiaryEntitlement("A", fixed_entitlement_amount=Decimal("50000.00")),
            BeneficiaryEntitlement("B", percentage_entitlement=Decimal("40.00")),
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
            BeneficiaryEntitlement("A", percentage_entitlement=Decimal("60.00")),
            BeneficiaryEntitlement("B", percentage_entitlement=Decimal("40.01")),
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
        beneficiaries=[BeneficiaryEntitlement(n, fixed_entitlement_amount=Decimal("10000.00"))
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
        beneficiaries=[BeneficiaryEntitlement(n, fixed_entitlement_amount=f)
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
        beneficiaries=[BeneficiaryEntitlement("A", percentage_entitlement=Decimal("100.00"))],
    )
    with pytest.raises(ValueError, match="loss is not allocated to beneficiaries"):
        calculate_proportionate_share(t)


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
        beneficiary_actually_received_funds=False)
    unstated = evaluate_section100a_risk(
        beneficiary_name="A", distribution_amount=Decimal("50000.00"))
    assert not_received != unstated
    assert any("did not receive the funds" in f for f in not_received.risk_factors_identified)
    assert unstated.risk_factors_identified == []
    assert not_received.risk_zone == Section100ARiskZone.OUTSIDE_GREEN
    assert not_received.is_ordinary_family_dealing is None

    lent_commercially = evaluate_section100a_risk(
        beneficiary_name="A", distribution_amount=Decimal("50000.00"),
        beneficiary_actually_received_funds=False,
        commercial_loan_agreement_in_place=True)
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
            BeneficiaryEntitlement("A", fixed_entitlement_amount=Decimal("823085.83")),
            BeneficiaryEntitlement("B", percentage_entitlement=Decimal("33.33")),
        ],
    )
    shares = calculate_proportionate_share(t)
    assert sum(s.section95_net_income_share for s in shares) == Decimal("1234567.01")
