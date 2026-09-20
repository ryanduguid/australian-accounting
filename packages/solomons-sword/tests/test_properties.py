"""Seeded allocation properties: every allocated column foots to its pool."""

from decimal import ROUND_DOWN, Decimal

import pytest
from hypothesis import assume, example, given, seed, settings
from hypothesis import strategies as st
from louisgoldberg.division6 import (
    BeneficiaryEntitlement,
    TrustIncomeAssessment,
    calculate_proportionate_share,
)

PROPERTY_SETTINGS = settings(max_examples=100, database=None, deadline=None)
CENTS = st.decimals(min_value=Decimal("0.01"), max_value=Decimal("100000000.00"), places=2)
CREDITS = st.decimals(min_value=Decimal("0.00"), max_value=Decimal("100000000.00"), places=2)
WEIGHTS = st.lists(st.integers(min_value=1, max_value=10000), min_size=2, max_size=8)


def percentages_from(weights: list[int]) -> list[Decimal]:
    """Split 100.00 across the weights at two places, largest remainder first."""
    total = sum(weights)
    exact = [Decimal(w) * 100 / total for w in weights]
    floored = [p.quantize(Decimal("0.01"), rounding=ROUND_DOWN) for p in exact]
    shortfall = int((Decimal("100.00") - sum(floored, Decimal("0.00"))) * 100)
    order = sorted(range(len(weights)), key=lambda i: exact[i] - floored[i], reverse=True)
    for i in order[:shortfall]:
        floored[i] += Decimal("0.01")
    return floored


def assessment_for(percentages, trust_income, s95_net, credits) -> TrustIncomeAssessment:
    return TrustIncomeAssessment(
        financial_year=2026,
        trust_name="Synthetic Trust",
        trust_accounting_income=trust_income,
        section95_net_taxable_income=s95_net,
        franking_credits=credits,
        beneficiaries=[
            BeneficiaryEntitlement(f"B{i}", percentage_entitlement=pct)
            for i, pct in enumerate(percentages)
        ],
    )


def test_fixed_entitlement_is_not_changed_by_percentage_rounding():
    assessment = TrustIncomeAssessment(
        financial_year=2026,
        trust_name="Synthetic Trust",
        trust_accounting_income=Decimal("10.00"),
        section95_net_taxable_income=Decimal("10.00"),
        beneficiaries=[
            BeneficiaryEntitlement("Fixed", fixed_entitlement_amount=Decimal("5.00")),
            BeneficiaryEntitlement("P1", percentage_entitlement=Decimal("16.67")),
            BeneficiaryEntitlement("P2", percentage_entitlement=Decimal("16.67")),
            BeneficiaryEntitlement("P3", percentage_entitlement=Decimal("16.67")),
        ],
    )
    shares = calculate_proportionate_share(assessment)

    assert shares[0].trust_income_entitlement == Decimal("5.00")
    assert sum(s.trust_income_entitlement for s in shares) == Decimal("10.00")


def test_three_near_equal_percentage_shares_foot_to_the_trust_income():
    shares = calculate_proportionate_share(
        assessment_for(
            [Decimal("33.33"), Decimal("33.33"), Decimal("33.34")],
            Decimal("10.00"),
            Decimal("10.00"),
            Decimal("0.00"),
        )
    )
    assert [s.trust_income_entitlement for s in shares] == [
        Decimal("3.33"), Decimal("3.33"), Decimal("3.34"),
    ]
    assert sum(s.trust_income_entitlement for s in shares) == Decimal("10.00")
    assert sum(s.section95_net_income_share for s in shares) == Decimal("10.00")


@seed(0x5010)
@PROPERTY_SETTINGS
@given(weights=WEIGHTS, trust_income=CENTS, s95_net=CENTS, credits=CREDITS)
@example(weights=[3333, 3333, 3334], trust_income=Decimal("10.00"), s95_net=Decimal("10.00"), credits=Decimal("0.00"))
@example(weights=[1] * 7, trust_income=Decimal("1.00"), s95_net=Decimal("1.00"), credits=Decimal("1.00"))
def test_every_allocated_column_foots_to_its_pool(weights, trust_income, s95_net, credits):
    percentages = percentages_from(weights)
    assume(all(pct > Decimal("0.00") for pct in percentages))
    shares = calculate_proportionate_share(assessment_for(percentages, trust_income, s95_net, credits))

    assert sum(s.trust_income_entitlement for s in shares) == trust_income
    assert sum(s.section95_net_income_share for s in shares) == s95_net
    assert sum(s.franking_credit_grossup for s in shares) == credits
    assert all(s.trust_income_entitlement >= Decimal("0.00") for s in shares)
    assert all(s.section95_net_income_share >= Decimal("0.00") for s in shares)
    assert all(s.franking_credit_grossup >= Decimal("0.00") for s in shares)
    assert all(s.total_taxable_component == s.section95_net_income_share for s in shares)


@pytest.mark.parametrize("weights", [[1, 1, 1], [2, 3, 5], [1] * 6])
def test_percentages_helper_splits_exactly_one_hundred(weights):
    assert sum(percentages_from(weights), Decimal("0.00")) == Decimal("100.00")
