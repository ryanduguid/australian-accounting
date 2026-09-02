"""
Section 100A Reimbursement Agreement risk assessment aligned with
ATO Practical Compliance Guideline PCG 2022/2 and TR 2022/4.
"""

from dataclasses import dataclass
from decimal import Decimal
from enum import Enum
from typing import List, Optional


class Section100ARiskZone(str, Enum):
    """PCG 2022/2 zones, plus the residual case the guideline leaves unzoned.

    The finalised guideline carries three zones: white (entitlements conferred
    in income years before 1 July 2014), green (low risk) and red (high risk).
    Draft PCG 2022/D1 also had a blue zone, which did not survive into the
    final guideline. The white zone is not decided here because it turns on the
    income year, which this function does not take.
    """

    GREEN = "GREEN"                  # Low risk, ordinary family dealing
    OUTSIDE_GREEN = "OUTSIDE_GREEN"  # Meets no green criterion, matches no red example
    RED = "RED"                      # High risk, ATO dedicates compliance resources


@dataclass(frozen=True)
class Section100AAssessment:
    beneficiary_name: str
    distribution_amount: Decimal
    risk_zone: Section100ARiskZone
    # None where PCG 2022/2 does not decide the s 100A(13) ordinary family
    # dealing exception: the green zone, which is a compliance-resourcing
    # stance, and the unzoned residual, which decides nothing at all.
    is_ordinary_family_dealing: Optional[bool]
    risk_factors_identified: List[str]
    mitigating_factors: List[str]
    tax_consequence_summary: str
    statutory_reference: str


def evaluate_section100a_risk(
    beneficiary_name: str,
    distribution_amount: Decimal,
    beneficiary_is_adult_child: bool = False,
    funds_retained_by_parents_without_loan: bool = False,
    circular_flow_of_funds: bool = False,
    corporate_beneficiary_unpaid_present_entitlement: bool = False,
    beneficiary_actually_received_funds: bool | None = None,
    funds_used_for_beneficiary_direct_benefit: bool = False,  # e.g., education, medical, independent asset
    commercial_loan_agreement_in_place: bool = False,
) -> Section100AAssessment:
    """
    Evaluate Section 100A risk zone under PCG 2022/2.
    """
    if not distribution_amount.is_finite() or distribution_amount <= 0:
        raise ValueError("distribution amount must be positive and finite")

    risk_factors: List[str] = []
    mitigating: List[str] = []

    # Check Red Zone triggers (PCG 2022/2 Appendix 1)
    if circular_flow_of_funds:
        risk_factors.append("Circular flow of funds detected (e.g. trust -> beneficiary -> company -> trust)")
    if beneficiary_is_adult_child and funds_retained_by_parents_without_loan:
        risk_factors.append("Adult child present entitlement retained by parents for general living costs without commercial terms")
    if corporate_beneficiary_unpaid_present_entitlement and not commercial_loan_agreement_in_place:
        risk_factors.append("Corporate beneficiary UPE without Div 7A compliant loan agreement or sub-trust")

    # Check Green Zone qualifications (PCG 2022/2 Appendix 2)
    if beneficiary_actually_received_funds is True and not funds_retained_by_parents_without_loan:
        mitigating.append("Beneficiary received and retained full economic benefit of entitlement")
    if funds_used_for_beneficiary_direct_benefit:
        mitigating.append("Funds applied directly for beneficiary's education, medical, or capital asset acquisition")
    if commercial_loan_agreement_in_place:
        mitigating.append("Funds lent under documented arm's-length commercial terms with interest paid")

    # Determine Risk Zone
    if risk_factors:
        zone = Section100ARiskZone.RED
        is_ofd = False
        consequence = (
            "HIGH RISK: High likelihood of s 100A application. If s 100A applies, the entitlement is disregarded "
            "and the trustee is assessed at the top rate applying under s 99A of the ITAA 1936."
        )
    elif mitigating:
        zone = Section100ARiskZone.GREEN
        # PCG 2022/2's green zone is a compliance-resourcing stance, not a
        # ruling that the s 100A(13) ordinary family dealing exception applies.
        is_ofd = None
        consequence = (
            "GREEN ZONE: the arrangement matches a green zone example, so the ATO states it "
            "will not dedicate compliance resources to it, subject to the guideline's own "
            "exclusions. That is not a determination that s 100A does not apply, and it does "
            "not decide the s 100A(13) ordinary family dealing exception, which turns on the "
            "facts of the dealing."
        )
    else:
        zone = Section100ARiskZone.OUTSIDE_GREEN
        # No zone means no finding either way, so the ordinary family dealing
        # exception is undecided rather than answered "no" on no facts.
        is_ofd = None
        consequence = (
            "OUTSIDE THE GREEN ZONE: the arrangement meets no green zone criterion and matches no red zone "
            "example, so PCG 2022/2 assigns it no zone. Further factual inquiry and contemporaneous "
            "documentation required."
        )

    # An entitlement the beneficiary never receives is a fact s 100A turns on
    # (TR 2022/4), so record it whenever the operator states it. Recorded
    # after zoning: alone it establishes neither a reimbursement agreement
    # nor a red-zone pattern, and the green-zone dealings this module models
    # (a Div 7A commercial loan, funds applied directly for the beneficiary)
    # involve non-receipt by definition, so the fact must not drive the zone.
    if beneficiary_actually_received_funds is False:
        risk_factors.append("Beneficiary did not receive the funds representing the present entitlement")

    return Section100AAssessment(
        beneficiary_name=beneficiary_name,
        distribution_amount=distribution_amount,
        risk_zone=zone,
        is_ordinary_family_dealing=is_ofd,
        risk_factors_identified=risk_factors,
        mitigating_factors=mitigating,
        tax_consequence_summary=consequence,
        statutory_reference="s 100A ITAA 1936; ATO PCG 2022/2; TR 2022/4",
    )
