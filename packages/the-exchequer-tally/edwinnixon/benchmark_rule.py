"""
Division 203 Benchmark Rule compliance under Sections 203-25 to 203-55 of the ITAA 1997.
Tests whether all frankable distributions within a franking period bear the same
franking percentage.
"""

from dataclasses import dataclass, replace
from datetime import date
from decimal import ROUND_HALF_UP, Decimal
from typing import List, Optional, Tuple


def _validated_rate(rate: Optional[Decimal], where: str) -> Decimal:
    """Return a stated corporate tax rate, refusing a missing or out-of-range one."""
    if rate is None:
        raise ValueError(
            f"{where}: corporate_tax_rate is required. The entity's corporate tax "
            "rate is a fact to be supplied, not a base rate to be assumed"
        )
    if not (rate.is_finite() and Decimal("0.00") < rate < Decimal("1.00")):
        raise ValueError(
            f"{where}: corporate_tax_rate must be a finite fraction between 0 and 1, "
            f"got {rate}"
        )
    return rate


@dataclass(frozen=True)
class DistributionEvent:
    event_date: date
    recipient_name: str
    distribution_amount: Decimal  # Cash/asset distribution value (unfranked + franked net)
    franking_credit: Decimal
    # None means the event states no rate of its own: BenchmarkRuleValidator.add_distribution
    # substitutes the validator's rate, so a 30% company is not scored at the base rate. A
    # sentinel is needed because an explicit 0.25 is indistinguishable from a default. An
    # event still carrying the sentinel has no rate, so it refuses to measure itself.
    corporate_tax_rate: Optional[Decimal] = None

    def __post_init__(self) -> None:
        for name in ("distribution_amount", "franking_credit"):
            value = getattr(self, name)
            if not value.is_finite() or value < Decimal("0.00"):
                raise ValueError(f"{name} must be a non-negative finite amount, got {value}")
        if self.corporate_tax_rate is not None:
            _validated_rate(self.corporate_tax_rate, "DistributionEvent")

    @property
    def maximum_franking_credit(self) -> Decimal:
        """
        Maximum credit for this distribution at this event's rate (s 202-60).

        An event with no stated rate is refused rather than measured at the base
        rate: a 30% company's maximum credit is not the base rate entity's.
        """
        if self.distribution_amount <= Decimal("0.00"):
            return Decimal("0.00")
        rate = _validated_rate(
            self.corporate_tax_rate,
            f"distribution to {self.recipient_name} on {self.event_date.isoformat()} "
            "(state a rate on the event, or add it through "
            "BenchmarkRuleValidator.add_distribution to take the validator's)",
        )
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
    Tests distributions across a franking period against the benchmark franking
    percentage (s 203-25).

    The entity's corporate tax rate is a required input. It sets the s 202-60
    maximum credit every comparison is measured against, so a 30% company scored
    at the base rate would read as under-franked on fully franked distributions.
    """
    def __init__(self, corporate_tax_rate: Decimal):
        self.corporate_tax_rate = _validated_rate(
            corporate_tax_rate, "BenchmarkRuleValidator"
        )
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

    def validate_distributions(self) -> Tuple[Optional[bool], List[BenchmarkRuleViolation]]:
        """
        Check all subsequent distributions against the established benchmark percentage.

        Returns (True, []) where the rule is met, (False, violations) where it is
        breached, and (None, []) where it was not tested because the period holds
        no frankable distribution to set a benchmark, the same absence
        `benchmark_percentage` reports. None is not a finding of compliance: a
        period nothing was tested in cannot comply, and treating it as compliant
        reports a passed test that never ran.
        """
        ordered = self._in_date_order()
        start = next(
            (i for i, e in enumerate(ordered) if e.distribution_amount > Decimal("0.00")),
            None,
        )
        if start is None:
            return None, []

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
            # A stated credit above the s 202-60 maximum is capped at that
            # maximum (s 202-65) before the Division 203 comparison: a 100%
            # benchmark and an over-credited later distribution have no
            # differential, and the overstated statement is a separate matter.
            max_credit = dist.maximum_franking_credit
            benchmark_credit = benchmark_ratio * max_credit
            credit_diff = min(dist.franking_credit, max_credit) - benchmark_credit

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
