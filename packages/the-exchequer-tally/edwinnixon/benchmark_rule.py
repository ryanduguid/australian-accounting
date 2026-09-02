"""
Division 203 Benchmark Rule compliance under Sections 203-25 to 203-55 of the ITAA 1997.
Ensures all frankable distributions within a franking period bear the same franking percentage.
"""

from dataclasses import dataclass, replace
from datetime import date
from decimal import Decimal, ROUND_HALF_UP
from typing import List, Optional, Tuple


@dataclass(frozen=True)
class DistributionEvent:
    event_date: date
    recipient_name: str
    distribution_amount: Decimal  # Cash/asset distribution value (unfranked + franked net)
    franking_credit: Decimal
    # None means the event states no rate of its own: BenchmarkRuleValidator.add_distribution
    # substitutes the validator's rate, so a 30% company is not scored at the base rate. A
    # sentinel is needed because an explicit 0.25 is indistinguishable from the old default.
    corporate_tax_rate: Optional[Decimal] = None

    def __post_init__(self) -> None:
        for name in ("distribution_amount", "franking_credit"):
            value = getattr(self, name)
            if not value.is_finite() or value < Decimal("0.00"):
                raise ValueError(f"{name} must be a non-negative finite amount, got {value}")
        rate = self.corporate_tax_rate
        if rate is not None and not (rate.is_finite() and Decimal("0.00") < rate < Decimal("1.00")):
            raise ValueError(
                f"corporate_tax_rate must be a finite fraction between 0 and 1, got {rate}"
            )

    @property
    def maximum_franking_credit(self) -> Decimal:
        """Maximum credit for this distribution at this event's rate (s 202-60)."""
        if self.distribution_amount <= Decimal("0.00"):
            return Decimal("0.00")
        rate = Decimal("0.25") if self.corporate_tax_rate is None else self.corporate_tax_rate
        return self.distribution_amount * (rate / (Decimal("1.00") - rate))

    @property
    def franking_ratio(self) -> Decimal:
        """
        Actual franking ratio (s 203-35), unrounded and capped at 1: a credit
        above the s 202-60 maximum does not raise the ratio past fully franked,
        so an over-credited first distribution cannot set a benchmark above 100%.
        """
        max_credit = self.maximum_franking_credit
        if max_credit <= Decimal("0.00"):
            return Decimal("0.00")
        return min(self.franking_credit / max_credit, Decimal("1.00"))

    @property
    def franking_percentage(self) -> Decimal:
        """The franking ratio as a percentage (s 203-35), rounded to 2dp for display."""
        return (self.franking_ratio * Decimal("100.00")).quantize(
            Decimal("0.01"), rounding=ROUND_HALF_UP
        )


@dataclass(frozen=True)
class BenchmarkRuleViolation:
    event_date: date
    recipient_name: str
    benchmark_percentage: Decimal
    actual_percentage: Decimal
    variance_percentage: Decimal
    consequence_type: str  # "OVER_FRANKING_TAX" or "FRANKING_DEBIT"
    penalty_or_debit_amount: Decimal
    statutory_reference: str


class BenchmarkRuleValidator:
    """
    Validates distributions across a franking period against the benchmark franking percentage (s 203-25).
    """
    def __init__(self, corporate_tax_rate: Decimal = Decimal("0.25")):
        self.corporate_tax_rate = corporate_tax_rate
        self.distributions: List[DistributionEvent] = []

    def add_distribution(self, event: DistributionEvent) -> None:
        # An event that states no rate of its own takes the validator's: the s 202-60
        # maximum credit is worked out at the entity's own corporate tax rate.
        if event.corporate_tax_rate is None:
            event = replace(event, corporate_tax_rate=self.corporate_tax_rate)
        self.distributions.append(event)

    def _in_date_order(self) -> List[DistributionEvent]:
        """
        Distributions ordered as s 203-30 reads them: by date within the franking
        period, not by the order they happened to be added.
        """
        return sorted(self.distributions, key=lambda event: event.event_date)

    @property
    def benchmark_percentage(self) -> Optional[Decimal]:
        """
        The benchmark percentage is set by the first frankable distribution in
        the period (s 203-30). A zero-amount event distributes nothing, so it
        is not a frankable distribution and must not set a 0% benchmark that
        turns every later credited distribution into a breach.
        """
        for event in self._in_date_order():
            if event.distribution_amount > Decimal("0.00"):
                return event.franking_percentage
        return None

    def validate_distributions(self) -> Tuple[bool, List[BenchmarkRuleViolation]]:
        """
        Check all subsequent distributions against the established benchmark percentage.
        """
        ordered = self._in_date_order()
        start = next(
            (i for i, e in enumerate(ordered) if e.distribution_amount > Decimal("0.00")),
            None,
        )
        if start is None:
            return True, []

        benchmark = ordered[start]
        benchmark_pct = benchmark.franking_percentage
        benchmark_ratio = benchmark.franking_ratio
        violations: List[BenchmarkRuleViolation] = []

        for dist in ordered[start + 1:]:
            actual_pct = dist.franking_percentage
            diff = actual_pct - benchmark_pct

            # Compare in dollars at the event's own rate: a percentage-only
            # comparison lets credit variances that scale with distribution
            # size pass unnoticed. One cent of tolerance absorbs rounding.
            # The benchmark credit takes the unrounded ratio: the 2dp display
            # percentage moves the benchmark by more than a cent once the
            # distribution is large, so identically franked distributions breached.
            max_credit = dist.maximum_franking_credit
            benchmark_credit = benchmark_ratio * max_credit
            credit_diff = dist.franking_credit - benchmark_credit

            if abs(credit_diff) > Decimal("0.01"):

                if credit_diff > Decimal("0.00"):
                    # Over-franking tax applies (s 203-50(1))
                    over_credit = credit_diff
                    violations.append(
                        BenchmarkRuleViolation(
                            event_date=dist.event_date,
                            recipient_name=dist.recipient_name,
                            benchmark_percentage=benchmark_pct,
                            actual_percentage=actual_pct,
                            variance_percentage=diff,
                            consequence_type="OVER_FRANKING_TAX",
                            penalty_or_debit_amount=over_credit.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP),
                            statutory_reference="s 203-50(1) ITAA 1997: Over-franking tax payable on excess franking credits.",
                        )
                    )
                else:
                    # Franking debit arises (s 203-50(2))
                    under_debit = -credit_diff
                    violations.append(
                        BenchmarkRuleViolation(
                            event_date=dist.event_date,
                            recipient_name=dist.recipient_name,
                            benchmark_percentage=benchmark_pct,
                            actual_percentage=actual_pct,
                            variance_percentage=diff,
                            consequence_type="FRANKING_DEBIT",
                            penalty_or_debit_amount=under_debit.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP),
                            statutory_reference="s 203-50(2) ITAA 1997: Franking account debit arises equal to shortfall.",
                        )
                    )

        return len(violations) == 0, violations
