"""
Trust Distribution Resolution verification and the 30 June or earlier deed deadline.
"""

from dataclasses import dataclass
from datetime import date
from decimal import Decimal
from typing import List, Optional, Tuple


@dataclass(frozen=True)
class TrustResolutionSchedule:
    trust_name: str
    financial_year: int  # Year the income year ends: 2025 is 1 July 2024 to 30 June 2025
    resolution_date: date
    # The 3 facts below come from the executed resolution and the deed, and each
    # is tristate: True, False, or None where the operator has not established
    # it. None is not False, and it is not compliance either.
    is_signed_by_trustee: Optional[bool]
    streaming_powers_in_deed: Optional[bool]
    default_beneficiary_clause_exists: Optional[bool]
    allocated_percentages_total: Decimal
    # Whether the resolution streams capital gains or franked distributions to
    # specific beneficiaries. Only then does the deed need streaming powers.
    uses_specific_streaming: Optional[bool]
    # The deed's own deadline for the resolution, or 30 June of the financial
    # year where the deed sets none. None means the deed has not been read.
    deed_resolution_deadline: Optional[date]

    @property
    def resolution_deadline(self) -> Optional[date]:
        # 30 June, or the deed's earlier date. A deed cannot extend 30 June.
        if self.deed_resolution_deadline is None:
            return None
        return min(date(self.financial_year, 6, 30), self.deed_resolution_deadline)

    @property
    def is_effective_by_year_end(self) -> Optional[bool]:
        # After 30 June is late whatever the deed says; on or before it, the
        # answer turns on the deed's own deadline.
        if self.resolution_date > date(self.financial_year, 6, 30):
            return False
        deadline = self.resolution_deadline
        return None if deadline is None else self.resolution_date <= deadline

    @property
    def is_within_income_year(self) -> bool:
        # A resolution distributes the income of the year it is made in, so a
        # date before that year opened on 1 July belongs to a different year and
        # is a mismatch, not an early resolution.
        return self.resolution_date >= date(self.financial_year - 1, 7, 1)


def validate_trust_resolution(
    schedule: TrustResolutionSchedule,
) -> Tuple[Optional[bool], List[str]]:
    """
    Validate that a trust distribution resolution complies with statutory and deed requirements.

    The first element is None wherever a fact the answer turns on was not
    established, whether or not an established fact is also breached, because an
    unestablished fact leaves the result unknown and a breach found alongside it
    does not settle the rest. With every fact established, it is True where
    nothing is breached and False where something is. None is not compliance: the
    issues list names each unestablished fact and each breach found, and a caller
    that treats the result as a boolean reads it as "not validated", which is
    what it is.
    """
    issues: List[str] = []

    if schedule.resolution_date > date(schedule.financial_year, 6, 30):
        issues.append(f"Resolution dated {schedule.resolution_date} is after 30 June {schedule.financial_year} deadline.")
    elif schedule.is_effective_by_year_end is False:
        issues.append(
            f"Resolution dated {schedule.resolution_date} is after the deed's "
            f"deadline of {schedule.resolution_deadline}."
        )
    if not schedule.is_within_income_year:
        issues.append(
            f"Resolution dated {schedule.resolution_date} predates the "
            f"{schedule.financial_year} income year, which began on 1 July "
            f"{schedule.financial_year - 1}; it does not distribute that year's income."
        )
    if schedule.is_signed_by_trustee is False:
        issues.append("Trustee resolution is not executed/signed.")
    if schedule.allocated_percentages_total != Decimal("100.00"):
        issues.append(f"Allocated income percentages sum to {schedule.allocated_percentages_total}%, not 100%.")
    streams = schedule.uses_specific_streaming
    if streams is True and schedule.streaming_powers_in_deed is False:
        issues.append("Resolution streams specific income but the deed does not record streaming powers.")
    if schedule.default_beneficiary_clause_exists is False:
        issues.append("Deed has no default beneficiary clause; unresolved income may be taxed to the trustee.")

    # Streaming powers matter only to a resolution that streams, so neither
    # streaming fact is needed once either rules the defect out.
    streaming_settled = streams is False or schedule.streaming_powers_in_deed is True
    unestablished = [
        name
        for name, value in (
            ("is_signed_by_trustee", schedule.is_signed_by_trustee),
            ("uses_specific_streaming", True if streaming_settled else streams),
            ("streaming_powers_in_deed", True if streaming_settled else schedule.streaming_powers_in_deed),
            ("default_beneficiary_clause_exists", schedule.default_beneficiary_clause_exists),
            ("deed_resolution_deadline", schedule.is_effective_by_year_end),
        )
        if value is None
    ]
    if unestablished:
        issues.append(
            "Not established: "
            + ", ".join(unestablished)
            + ". Each is a fact of the executed resolution or the deed. Read it and "
            "state it as True or False; until then the resolution is neither validated "
            "nor shown to be in breach."
        )
        return None, issues

    return len(issues) == 0, issues