"""Comparison trials: this repository's engines against the LodgeiT calculators.

A trial is not a test of either engine. Neither is the oracle. Each case
carries its own expected arithmetic, derived by hand from the statute or the
standard, and a disagreement is traced back to that rather than to whichever
engine is more convenient to believe.

The evaluation outcomes are deliberately not the accounting-review states used
elsewhere in this repository. A `MATCH` here says two implementations agreed on
a number. It says nothing about whether a loan complies, whether a benefit is
reportable, or whether a reviewer should sign anything.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum


class Evaluation(str, Enum):
    """The outcome of comparing one case. Not an accounting verdict."""

    MATCH = "MATCH"
    NUMERIC_DIFFERENCE = "NUMERIC_DIFFERENCE"
    SCOPE_MISMATCH = "SCOPE_MISMATCH"
    UNSUPPORTED_PERIOD = "UNSUPPORTED_PERIOD"
    CONTRACT_FAILURE = "CONTRACT_FAILURE"
    UPSTREAM_UNAVAILABLE = "UPSTREAM_UNAVAILABLE"
    LOCAL_REFUSED = "LOCAL_REFUSED"
    NOT_RUN = "NOT_RUN"

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True)
class Comparison:
    """One case, both sides, and what the difference means."""

    case_id: str
    evaluation: Evaluation
    expected: dict[str, Decimal] = field(default_factory=dict)
    local: dict[str, Decimal] = field(default_factory=dict)
    upstream: dict[str, Decimal] = field(default_factory=dict)
    differences: dict[str, Decimal] = field(default_factory=dict)
    reasons: tuple[str, ...] = ()
    local_verdict: str | None = None
    upstream_status: str | None = None

    def to_json_dict(self) -> dict:
        def money(values: dict[str, Decimal]) -> dict[str, str]:
            return {key: format(value, "f") for key, value in sorted(values.items())}

        return {
            "case_id": self.case_id,
            "evaluation": str(self.evaluation),
            "expected": money(self.expected),
            "local": money(self.local),
            "upstream": money(self.upstream),
            "differences": money(self.differences),
            "reasons": list(self.reasons),
            "local_verdict": self.local_verdict,
            "upstream_status": self.upstream_status,
            "boundary": "An evaluation outcome compares arithmetic. It is not a readiness, "
                        "close or ledger-review state and it approves nothing.",
        }


def compare_values(
    expected: dict[str, Decimal],
    local: dict[str, Decimal],
    upstream: dict[str, Decimal],
    *,
    tolerance: Decimal = Decimal("0.01"),
) -> tuple[Evaluation, dict[str, Decimal], tuple[str, ...]]:
    """Compare the fields present on both sides against the expected figures.

    A field missing from either side is a reason, not a silent pass. The
    tolerance is for the last cent only: anything wider hides a method
    difference, which is the thing a trial is for.
    """
    reasons: list[str] = []
    differences: dict[str, Decimal] = {}
    for name in sorted(expected):
        if name not in local:
            reasons.append(f"{name}: the local engine produced no figure")
        elif local[name] != expected[name]:
            reasons.append(
                f"{name}: local {local[name]} differs from the independently derived "
                f"{expected[name]}"
            )
        if name not in upstream:
            reasons.append(f"{name}: the provider returned no figure")
    shared = sorted(set(local) & set(upstream))
    if not shared:
        reasons.append("no field is present on both sides, so nothing was compared")
        return Evaluation.SCOPE_MISMATCH, differences, tuple(reasons)
    for name in shared:
        difference = local[name] - upstream[name]
        if difference.copy_abs() > tolerance:
            differences[name] = difference
            reasons.append(f"{name}: local {local[name]} against provider {upstream[name]}")
    if differences:
        return Evaluation.NUMERIC_DIFFERENCE, differences, tuple(reasons)
    if reasons:
        return Evaluation.NUMERIC_DIFFERENCE, differences, tuple(reasons)
    return Evaluation.MATCH, differences, ()
