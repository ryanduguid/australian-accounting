"""
Trust Distribution Resolution verification and 30 June deadline compliance.
"""

from dataclasses import dataclass
from datetime import date
from decimal import Decimal
from typing import List, Tuple


@dataclass(frozen=True)
class TrustResolutionSchedule:
    trust_name: str
    financial_year: int  # Year the income year ends: 2025 is 1 July 2024 to 30 June 2025
    resolution_date: date
    is_signed_by_trustee: bool
    streaming_powers_in_deed: bool
    default_beneficiary_clause_exists: bool
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


def validate_trust_resolution(schedule: TrustResolutionSchedule) -> Tuple[bool, List[str]]:
    """
    Validate that a trust distribution resolution complies with statutory and deed requirements.
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
    if not schedule.is_signed_by_trustee:
        issues.append("Trustee resolution is not executed/signed.")
    if schedule.allocated_percentages_total != Decimal("100.00"):
        issues.append(f"Allocated income percentages sum to {schedule.allocated_percentages_total}%, not 100%.")
    if not schedule.streaming_powers_in_deed:
        issues.append("Deed does not record streaming powers; specific streaming cannot be assumed.")
    if not schedule.default_beneficiary_clause_exists:
        issues.append("Deed has no default beneficiary clause; unresolved income may be taxed to the trustee.")

    is_valid = len(issues) == 0
    return is_valid, issues