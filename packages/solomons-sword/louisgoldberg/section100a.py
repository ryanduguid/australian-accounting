"""
Section 100A Reimbursement Agreement risk assessment aligned with
ATO Practical Compliance Guideline PCG 2022/2 and TR 2022/4.
"""

from dataclasses import dataclass
from decimal import Decimal
from enum import Enum
from typing import List, Optional


class Section100ARiskZone(str, Enum):
    """PCG 2022/2 zones, the residual case the guideline leaves unzoned, and
    the case where the facts a zone turns on were not established.

    The finalised guideline carries 3 zones: white (entitlements conferred
    in income years before 1 July 2014), green (low risk) and red (high risk).
    Draft PCG 2022/D1 also had a blue zone, which did not survive into the
    final guideline. The white zone is not decided here because it turns on the
    income year, which this function does not take.

    FACTS_NOT_ESTABLISHED is not a PCG zone. It is this engine reporting that
    it was not given the facts a zone turns on, which is a different answer from
    OUTSIDE_GREEN: that one means the stated facts meet no green criterion and
    match no red example.
    """

    GREEN = "GREEN"                  # Low risk, ordinary family dealing
    OUTSIDE_GREEN = "OUTSIDE_GREEN"  # Meets no green criterion, matches no red example
    RED = "RED"                      # High risk, ATO dedicates compliance resources
    FACTS_NOT_ESTABLISHED = "FACTS_NOT_ESTABLISHED"  # A fact a zone turns on was not stated


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
    # The facts that were not stated, in the order the function takes them.
    # Empty where every fact was established as True or False.
    unestablished_facts: tuple[str, ...] = ()


def evaluate_section100a_risk(
    beneficiary_name: str,
    distribution_amount: Decimal,
    beneficiary_is_adult_child: bool | None = None,
    funds_retained_by_parents_without_loan: bool | None = None,
    circular_flow_of_funds: bool | None = None,
    corporate_beneficiary_unpaid_present_entitlement: bool | None = None,
    beneficiary_actually_received_funds: bool | None = None,
    funds_used_for_beneficiary_direct_benefit: bool | None = None,  # for example, education, medical, independent asset
    commercial_loan_agreement_in_place: bool | None = None,
) -> Section100AAssessment:
    """
    Evaluate Section 100A risk zone under PCG 2022/2.

    Every fact is tristate: True, False, or None where the operator has not
    established it. None is not False. A green zone says the ATO will not
    dedicate compliance resources to the arrangement, so it is refused while any
    fact it turns on is unstated, and the result is FACTS_NOT_ESTABLISHED naming
    those facts. An established red-zone trigger still returns RED, because that
    is the guideline's own answer on the facts it was given.
    """
    if not distribution_amount.is_finite() or distribution_amount <= 0:
        raise ValueError("distribution amount must be positive and finite")

    # The green zone turns on all of these: the 3 mitigating facts that can put
    # an arrangement in it, and the red-zone facts that would keep it out.
    facts: dict[str, bool | None] = {
        "beneficiary_is_adult_child": beneficiary_is_adult_child,
        "funds_retained_by_parents_without_loan": funds_retained_by_parents_without_loan,
        "circular_flow_of_funds": circular_flow_of_funds,
        "corporate_beneficiary_unpaid_present_entitlement": (
            corporate_beneficiary_unpaid_present_entitlement
        ),
        "beneficiary_actually_received_funds": beneficiary_actually_received_funds,
        "funds_used_for_beneficiary_direct_benefit": funds_used_for_beneficiary_direct_benefit,
        "commercial_loan_agreement_in_place": commercial_loan_agreement_in_place,
    }
    unestablished = tuple(name for name, value in facts.items() if value is None)

    risk_factors: List[str] = []
    mitigating: List[str] = []

    # Check Red Zone triggers (PCG 2022/2 Appendix 1). Each limb needs every
    # fact it reads: a trigger asserted on an unstated fact would be a definite
    # answer drawn from a gap, and the unzoned result below reports the gap.
    if circular_flow_of_funds is True:
        risk_factors.append("Circular flow of funds detected (e.g. trust -> beneficiary -> company -> trust)")
    if beneficiary_is_adult_child is True and funds_retained_by_parents_without_loan is True:
        risk_factors.append("Adult child present entitlement retained by parents for general living costs without commercial terms")
    if corporate_beneficiary_unpaid_present_entitlement is True and commercial_loan_agreement_in_place is False:
        risk_factors.append("Corporate beneficiary UPE without Div 7A compliant loan agreement or sub-trust")

    # Check Green Zone qualifications (PCG 2022/2 Appendix 2)
    if beneficiary_actually_received_funds is True and funds_retained_by_parents_without_loan is False:
        mitigating.append("Beneficiary received and retained full economic benefit of entitlement")
    if funds_used_for_beneficiary_direct_benefit is True:
        mitigating.append("Funds applied directly for beneficiary's education, medical, or capital asset acquisition")
    if commercial_loan_agreement_in_place is True:
        mitigating.append("Funds lent under documented arm's-length commercial terms with interest paid")

    # Determine Risk Zone
    if risk_factors:
        zone = Section100ARiskZone.RED
        # PCG 2022/2 paragraph 33: red-zone status does not establish s 100A.
        is_ofd = None
        consequence = (
            "HIGH RISK: the ATO prioritises review of red-zone arrangements. This does not "
            "determine whether the ordinary family dealing exception or s 100A applies. "
            "If s 100A applies, the entitlement is disregarded "
            "and the trustee is assessed at the top rate applying under s 99A of the ITAA 1936."
        )
    elif unestablished:
        zone = Section100ARiskZone.FACTS_NOT_ESTABLISHED
        is_ofd = None
        consequence = (
            "FACTS NOT ESTABLISHED: no zone is assigned because these facts were not "
            f"stated: {', '.join(unestablished)}. The green zone turns on every one of "
            "them, so it is not available here, and this is not a finding that the "
            "arrangement meets no green zone criterion. State each fact as True or "
            "False from the trust's records, or treat the arrangement as unzoned and "
            "make the factual inquiry."
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
        unestablished_facts=unestablished,
    )
