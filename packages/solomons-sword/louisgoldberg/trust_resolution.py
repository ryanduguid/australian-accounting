"""
Trust Distribution Resolution verification and 30 June deadline compliance.
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

    @property
    def is_effective_by_year_end(self) -> bool:
        # Effective resolution must be made on or before 30 June (or earlier if deed specifies)
        deadline = date(self.financial_year, 6, 30)
        return self.resolution_date <= deadline

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

    if not schedule.is_effective_by_year_end:
        issues.append(f"Resolution dated {schedule.resolution_date} is after 30 June {schedule.financial_year} deadline.")
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
    if schedule.streaming_powers_in_deed is False:
        issues.append("Deed does not record streaming powers; specific streaming cannot be assumed.")
    if schedule.default_beneficiary_clause_exists is False:
        issues.append("Deed has no default beneficiary clause; unresolved income may be taxed to the trustee.")

    unestablished = [
        name
        for name, value in (
            ("is_signed_by_trustee", schedule.is_signed_by_trustee),
            ("streaming_powers_in_deed", schedule.streaming_powers_in_deed),
            ("default_beneficiary_clause_exists", schedule.default_beneficiary_clause_exists),
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