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


class ReasonCode(StrEnum):
    """Why a result carries a reason, as a token a caller can branch on.

    The prose in `reasons` is written for a person and is free to change with
    the wording of the Act or the engine. These codes are the stable half: an
    MCP client deciding whether to ask the operator for a fact
    (`*_UNKNOWN`) or to stop asking (`REFUSED_*`) should branch on these and
    display the prose. A code is never removed without a version bump.

    Three prefixes, one per question:
      `REFUSED_*`   the s 109E question is outside this engine.
      `GATE_*`      a s 109N(1) limb failed or could not be established.
      everything else, ending `_UNKNOWN`, is a fact the operator can supply.
    """

    # REFUSED: the question is outside s 109E, and no further fact helps.
    REFUSED_GATE_RESULT_MISSING = "REFUSED_GATE_RESULT_MISSING"
    REFUSED_GATE_NOT_COMPLYING = "REFUSED_GATE_NOT_COMPLYING"
    REFUSED_GATE_UNKNOWN = "REFUSED_GATE_UNKNOWN"
    REFUSED_GATE_BENCHMARK_YEAR_MISMATCH = "REFUSED_GATE_BENCHMARK_YEAR_MISMATCH"
    REFUSED_YEAR_IS_YEAR_OF_LOAN = "REFUSED_YEAR_IS_YEAR_OF_LOAN"
    REFUSED_YEAR_BEFORE_LOAN = "REFUSED_YEAR_BEFORE_LOAN"
    REFUSED_REMAINING_TERM_NOT_POSITIVE = "REFUSED_REMAINING_TERM_NOT_POSITIVE"
    REFUSED_BENCHMARK_RATE_NOT_POSITIVE = "REFUSED_BENCHMARK_RATE_NOT_POSITIVE"

    # The s 109N(1) gate: one per limb that did not pass, plus the year the
    # benchmark is read for. _FAIL is the limb decided against the loan on the
    # facts supplied; _UNKNOWN is a limb that could not be decided at all.
    GATE_NO_BENCHMARK_YEAR = "GATE_NO_BENCHMARK_YEAR"
    GATE_WRITTEN_AGREEMENT_FAIL = "GATE_WRITTEN_AGREEMENT_FAIL"
    GATE_WRITTEN_AGREEMENT_UNKNOWN = "GATE_WRITTEN_AGREEMENT_UNKNOWN"
    GATE_LODGMENT_DAY_FAIL = "GATE_LODGMENT_DAY_FAIL"
    GATE_LODGMENT_DAY_UNKNOWN = "GATE_LODGMENT_DAY_UNKNOWN"
    GATE_INTEREST_FAIL = "GATE_INTEREST_FAIL"
    GATE_INTEREST_UNKNOWN = "GATE_INTEREST_UNKNOWN"
    GATE_TERM_FAIL = "GATE_TERM_FAIL"
    GATE_TERM_UNKNOWN = "GATE_TERM_UNKNOWN"

    # UNKNOWN: a fact the operator can still establish was not established.
    YEAR_OF_INCOME_UNKNOWN = "YEAR_OF_INCOME_UNKNOWN"
    YEAR_LOAN_MADE_UNKNOWN = "YEAR_LOAN_MADE_UNKNOWN"
    BENCHMARK_RATE_UNKNOWN = "BENCHMARK_RATE_UNKNOWN"
    UNPAID_BALANCE_UNKNOWN = "UNPAID_BALANCE_UNKNOWN"
    PAYMENTS_APPLIED_UNKNOWN = "PAYMENTS_APPLIED_UNKNOWN"
    REMAINING_TERM_UNKNOWN = "REMAINING_TERM_UNKNOWN"


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
