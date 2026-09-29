"""Public calculations refuse malformed facts before interpreting them."""

from datetime import date
from decimal import Decimal

import pytest
from louisgoldberg.division6 import (
    BeneficiaryEntitlement,
    TrustIncomeAssessment,
    calculate_proportionate_share,
)
from louisgoldberg.section99b import ForeignTrustReceipt, evaluate_section99b_liability
from louisgoldberg.section100a import evaluate_section100a_risk
from louisgoldberg.trust_resolution import TrustResolutionSchedule, validate_trust_resolution

INVALID_FACTS = [0, 1, 0.0, 1.0, Decimal("0"), "False", "true", [], {}]
RESOLUTION_FACTS = (
    "is_signed_by_trustee",
    "streaming_powers_in_deed",
    "default_beneficiary_clause_exists",
    "uses_specific_streaming",
)
RISK_FACTS = (
    "beneficiary_is_adult_child",
    "funds_retained_by_parents_without_loan",
    "circular_flow_of_funds",
    "corporate_beneficiary_unpaid_present_entitlement",
    "beneficiary_actually_received_funds",
    "funds_used_for_beneficiary_direct_benefit",
    "commercial_loan_agreement_in_place",
    "entitlement_applied_to_pre_18_expenses",
    "received_within_two_years",
    "retention_scenario_conditions_met",
    "paragraph_32_exclusion_present",
)


@pytest.mark.parametrize("field", RESOLUTION_FACTS)
@pytest.mark.parametrize("invalid", INVALID_FACTS)
def test_resolution_refuses_non_boolean_facts(field, invalid):
    facts = dict(
        is_signed_by_trustee=True,
        streaming_powers_in_deed=True,
        default_beneficiary_clause_exists=True,
        uses_specific_streaming=False,
    )
    facts[field] = invalid
    schedule = TrustResolutionSchedule(
        trust_name="Fabricated Trust",
        financial_year=2026,
        resolution_date=date(2026, 6, 30),
        deed_resolution_deadline=date(2026, 6, 30),
        allocated_percentages_total=Decimal("100"),
        **facts,
    )
    with pytest.raises(ValueError, match=field + " must be True, False or None"):
        validate_trust_resolution(schedule)


@pytest.mark.parametrize("field", RISK_FACTS)
@pytest.mark.parametrize("invalid", INVALID_FACTS)
def test_risk_assessment_refuses_non_boolean_facts(field, invalid):
    facts = dict.fromkeys(RISK_FACTS, False)
    facts["funds_used_for_beneficiary_direct_benefit"] = True
    facts[field] = invalid
    with pytest.raises(ValueError, match=field + " must be True, False or None"):
        evaluate_section100a_risk("Fabricated Beneficiary", Decimal("100"), **facts)


@pytest.mark.parametrize("field", ("is_resident", "is_under_legal_disability"))
@pytest.mark.parametrize("invalid", INVALID_FACTS)
def test_allocation_refuses_non_boolean_facts(field, invalid):
    facts = dict(is_resident=True, is_under_legal_disability=False)
    facts[field] = invalid
    assessment = TrustIncomeAssessment(
        financial_year=2026,
        trust_name="Fabricated Trust",
        trust_accounting_income=Decimal("100"),
        section95_net_taxable_income=Decimal("100"),
        beneficiaries=[BeneficiaryEntitlement(
            beneficiary_name="Fabricated Beneficiary",
            percentage_entitlement=Decimal("100"),
            **facts,
        )],
    )
    with pytest.raises(ValueError, match=field + " must be True, False or None"):
        calculate_proportionate_share(assessment)


@pytest.mark.parametrize("invalid", INVALID_FACTS)
def test_foreign_receipt_refuses_non_boolean_residency(invalid):
    receipt = ForeignTrustReceipt(
        beneficiary_name="Fabricated Beneficiary",
        gross_amount_received_aud=Decimal("100"),
        beneficiary_was_resident_during_year=invalid,
    )
    with pytest.raises(
        ValueError, match="beneficiary_was_resident_during_year must be True, False or None"
    ):
        evaluate_section99b_liability(receipt)
