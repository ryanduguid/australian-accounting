"""Verdict vocabularies.

Verdicts are enums, not free text, because the MCP adapter this engine is
built for will branch on them. UNKNOWN and REFUSED are first-class results
rather than errors: a review that cannot establish a statutory fact must say
so, and must not degrade into a best-effort number.
"""
from __future__ import annotations

from enum import Enum


class StrEnum(str, Enum):
    """str-valued Enum. Python 3.11 has enum.StrEnum, 3.10 does not, and this
    package supports 3.10."""

    def __str__(self) -> str:
        return str(self.value)


class RateVerdict(StrEnum):
    KNOWN = "KNOWN"
    UNKNOWN = "UNKNOWN"


class GateVerdict(StrEnum):
    """s 109N(1). COMPLYING means every limb is established on the operator's
    own facts, not that the loan is safe."""

    COMPLYING = "COMPLYING"
    NOT_COMPLYING = "NOT_COMPLYING"
    UNKNOWN = "UNKNOWN"


class MyrVerdict(StrEnum):
    """s 109E(5). REFUSED means the question is outside this engine; UNKNOWN
    means a fact needed to answer it was not established."""

    MYR_MET = "MYR_MET"
    MYR_SHORT = "MYR_SHORT"
    UNKNOWN = "UNKNOWN"
    REFUSED = "REFUSED"


class RowStatus(StrEnum):
    """A register row that was not reviewed at all."""

    SKIPPED = "SKIPPED"


#: The summary counts a register review reports, in display order.
SUMMARY_KEYS = (
    GateVerdict.COMPLYING.value,
    GateVerdict.NOT_COMPLYING.value,
    MyrVerdict.MYR_MET.value,
    MyrVerdict.MYR_SHORT.value,
    MyrVerdict.UNKNOWN.value,
    MyrVerdict.REFUSED.value,
    RowStatus.SKIPPED.value,
)
