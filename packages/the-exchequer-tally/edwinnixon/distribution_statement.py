"""
Distribution Statement generation under Sections 202-75 and 202-80 of the ITAA 1997.
"""

from dataclasses import dataclass
from datetime import date
from decimal import Decimal, ROUND_DOWN, ROUND_HALF_UP


@dataclass(frozen=True)
class DistributionStatement:
    entity_name: str
    abn_or_acn: str
    recipient_name: str
    payment_date: date
    franked_amount: Decimal
    unfranked_amount: Decimal
    franking_credit: Decimal
    franking_percentage: Decimal
    corporate_tax_rate: Decimal
    withholding_tax_deducted: Decimal = Decimal("0.00")

    @property
    def total_distribution(self) -> Decimal:
        return self.franked_amount + self.unfranked_amount

    @property
    def gross_assessable_income(self) -> Decimal:
        """Total assessable gross-up for resident recipient (s 44 & s 207-20 ITAA 1997)."""
        return self.total_distribution + self.franking_credit


def generate_distribution_statement(
    entity_name: str,
    abn_or_acn: str,
    recipient_name: str,
    payment_date: date,
    total_distribution: Decimal,
    franking_percentage: Decimal,
    corporate_tax_rate: Decimal = Decimal("0.25"),
) -> DistributionStatement:
    """
    Generate the details a distribution statement must carry (s 202-75, s 202-80).
    """
    if not total_distribution.is_finite() or total_distribution <= Decimal("0.00"):
        raise ValueError(f"total_distribution must be a positive finite amount, got {total_distribution}")
    # Whole cents only: the franked and unfranked halves are each stated to the cent,
    # so a sub-cent total is split into halves that do not add back to it.
    if total_distribution != total_distribution.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP):
        raise ValueError(f"total_distribution must be a whole number of cents, got {total_distribution}")
    if not franking_percentage.is_finite() or not (
        Decimal("0.00") <= franking_percentage <= Decimal("100.00")
    ):
        raise ValueError(
            f"franking_percentage must be finite and between 0 and 100, got "
            f"{franking_percentage}"
        )
    if not corporate_tax_rate.is_finite() or not (
        Decimal("0.00") < corporate_tax_rate < Decimal("1.00")
    ):
        raise ValueError(
            f"corporate_tax_rate must be a finite fraction between 0 and 1, got "
            f"{corporate_tax_rate}"
        )
    pct = franking_percentage / Decimal("100.00")
    franked_portion = (total_distribution * pct).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
    unfranked_portion = (total_distribution - franked_portion).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)

    # Maximum franking credit on franked portion: franked_amount * [tax_rate / (1 - tax_rate)].
    # Rounded DOWN: s 202-60 caps the credit at the maximum, so rounding a
    # third-of-a-cent up would state a credit above the statutory cap.
    multiplier = corporate_tax_rate / (Decimal("1.00") - corporate_tax_rate)
    franking_credit = (franked_portion * multiplier).quantize(Decimal("0.01"), rounding=ROUND_DOWN)

    return DistributionStatement(
        entity_name=entity_name,
        abn_or_acn=abn_or_acn,
        recipient_name=recipient_name,
        payment_date=payment_date,
        franked_amount=franked_portion,
        unfranked_amount=unfranked_portion,
        franking_credit=franking_credit,
        franking_percentage=franking_percentage,
        corporate_tax_rate=corporate_tax_rate,
    )
