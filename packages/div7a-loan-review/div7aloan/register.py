"""Reviewing a loan register: one amalgamated loan per row.

The register is the operator's own working paper, not an accounting export.
Every column is an assertion they have already made; this engine reads them,
applies s 109N and s 109E, and reports what it could and could not establish.

A row is SKIPPED, not reviewed, where the operator marks it out of scope or
where the loan was made in a year this engine will not touch. Division 7A
applies to a loan made before 4 December 1997 only if its terms were varied
on or after that day (s 109D(5)), and a year of income that straddles that
date cannot be placed on one side of it from a year label alone.
"""
from __future__ import annotations

import csv
from dataclasses import dataclass, field, replace
from decimal import Decimal
from pathlib import Path
from typing import Iterable, Sequence

from .facts import FactError, optional_year_of_income
from .gate import GateFacts, GateResult, complying_loan_gate
from .money import cents_str
from .myr import MyrFacts, MyrResult, minimum_yearly_repayment
from .rates import BenchmarkTable, RateOverride
from .verdicts import SUMMARY_KEYS, GateVerdict, MyrVerdict, RowStatus
from .years import YearOfIncome

#: Columns the s 109N gate needs.
GATE_COLUMNS = (
    "loan_id",
    "written_agreement",
    "terms_in_place_before_lodgment_day",
    "maximum_term_years",
    "secured_by_registered_mortgage_over_real_property",
    "security_coverage_at_first_made",
    "interest_rate_for_years_after_year_loan_made",
    "year_loan_made",
)

#: Columns the s 109E minimum yearly repayment needs, on top of the gate's.
MYR_COLUMNS = (
    "amalgamated_loan_unpaid_at_end_of_previous_year",
    "remaining_term_years",
    "payments_applied_during_the_year",
)

#: Columns that are read where present and ignored where absent.
#: year_of_income_being_tested nominates the benchmark floor year for the
#: s 109N(1)(b) limb on that row; the --year flag on the gate command
#: overrides it for every row.
OPTIONAL_COLUMNS = (
    "borrower_reference",
    "out_of_scope_reason",
    "year_of_income_being_tested",
)

#: The last year of income that can contain 4 December 1997. A loan made in
#: this year or earlier is outside v1: see s 109D(5).
FIRST_REVIEWABLE_YEAR = YearOfIncome(1998)


class RegisterError(ValueError):
    """The register cannot be read, or does not carry the columns required."""


@dataclass(frozen=True)
class ReviewLine:
    row_number: int
    loan_id: str
    borrower_reference: str = ""
    gate: GateResult | None = None
    myr: MyrResult | None = None
    skipped_reason: str = ""

    @property
    def is_skipped(self) -> bool:
        return bool(self.skipped_reason)

    @property
    def shortfall(self) -> Decimal:
        """Nil where there is no established shortfall, so the sort is total."""
        if self.myr is None or self.myr.shortfall is None:
            return Decimal("0.00")
        return self.myr.shortfall

    @property
    def has_exposure(self) -> bool:
        return self.myr is not None and self.myr.verdict is MyrVerdict.MYR_SHORT

    @property
    def is_undecided(self) -> bool:
        gate_undecided = self.gate is not None and self.gate.verdict is GateVerdict.UNKNOWN
        myr_undecided = self.myr is not None and self.myr.verdict in (
            MyrVerdict.UNKNOWN,
            MyrVerdict.REFUSED,
        )
        return gate_undecided or myr_undecided

    @property
    def needs_attention(self) -> bool:
        not_complying = self.gate is not None and self.gate.verdict is GateVerdict.NOT_COMPLYING
        return self.has_exposure or self.is_undecided or not_complying

    def to_json_dict(self) -> dict:
        return {
            "row_number": self.row_number,
            "loan_id": self.loan_id,
            "borrower_reference": self.borrower_reference,
            "status": RowStatus.SKIPPED.value if self.is_skipped else "REVIEWED",
            "skipped_reason": self.skipped_reason or None,
            "gate": None if self.gate is None else self.gate.to_json_dict(),
            "myr": None if self.myr is None else self.myr.to_json_dict(),
        }


@dataclass(frozen=True)
class ReviewReport:
    year_of_income: str
    lines: tuple[ReviewLine, ...] = field(default_factory=tuple)
    summary: dict[str, int] = field(default_factory=dict)
    rows_reviewed: int = 0

    @property
    def total_exposure(self) -> Decimal:
        return sum((line.shortfall for line in self.lines), Decimal("0.00"))

    @property
    def needs_attention(self) -> bool:
        return any(line.needs_attention for line in self.lines)

    def to_json_dict(self) -> dict:
        return {
            "year_of_income": self.year_of_income,
            "rows_reviewed": self.rows_reviewed,
            "summary": dict(self.summary),
            "experimental_total_exposure": cents_str(self.total_exposure),
            "lines": [line.to_json_dict() for line in self.lines],
            "note": (
                "Counts are per question, not per row: a reviewed row answers both "
                "the s 109N terms question and the s 109E minimum yearly repayment "
                "question, so the counts can sum to more than rows_reviewed."
            ),
        }


def require_columns(fieldnames: Sequence[str] | None, needed: Iterable[str], where: str) -> None:
    """Refuse a register that does not carry the columns a command needs.

    Reading a missing column as blank would turn every absent fact into
    "unknown" and produce a file of UNKNOWN verdicts that looks like a
    considered review rather than a mis-shaped file.
    """
    present = {name.strip() for name in (fieldnames or [])}
    missing = [name for name in needed if name not in present]
    if missing:
        raise RegisterError(
            f"{where} is missing required column(s): {', '.join(missing)}. "
            f"The register columns are documented in the README."
        )


def load_rows(path: Path | str, needed: Iterable[str]) -> list[dict]:
    path = Path(path)
    try:
        with path.open(encoding="utf-8-sig", newline="") as handle:
            reader = csv.DictReader(handle)
            require_columns(reader.fieldnames, needed, str(path))
            rows = list(reader)
    except OSError as exc:
        raise RegisterError(f"cannot read the loan register at {path}: {exc}")
    if not rows:
        raise RegisterError(f"{path} carries a header but no loan rows")
    return rows


def _skip_reason(row: dict, where: str) -> str:
    marked = str(row.get("out_of_scope_reason", "") or "").strip()
    if marked:
        return f"Operator marked out of scope: {marked}"
    year_made = optional_year_of_income(row.get("year_loan_made"), f"{where} year_loan_made")
    if year_made is not None and year_made < FIRST_REVIEWABLE_YEAR:
        return (
            f"Loan made in {year_made.label}. Division 7A reaches a loan made before "
            "4 December 1997 only where its terms were varied on or after that day "
            "(s 109D(5)), and a year label cannot place the loan either side of that "
            "date. Out of scope for v1."
        )
    return ""


def review_register(
    rows: Iterable[dict],
    year_of_income: YearOfIncome,
    *,
    gate_only: bool = False,
    myr_only: bool = False,
    gate_benchmark_year: YearOfIncome | None = None,
    table: BenchmarkTable | None = None,
    override: RateOverride | None = None,
) -> ReviewReport:
    """Review a loan register for one year of income.

    The s 109N gate is anchored to each row's year_loan_made, which is the
    year s 109N(1)(b) itself points at. Pass gate_benchmark_year to test the
    supplied rate against a later year's benchmark instead; that is a
    practice check on a risen benchmark rather than the s 109N(1)(b) test,
    and the result says so.

    The s 109E minimum yearly repayment is worked out for year_of_income,
    using that year's benchmark rate: a benchmark rate that has risen since
    the loan was written raises the minimum yearly repayment on an existing
    complying loan, which is the common way a long-standing loan falls short.
    """
    lines: list[ReviewLine] = []
    summary = {key: 0 for key in SUMMARY_KEYS}
    reviewed = 0

    for n, row in enumerate(rows, start=1):
        where = f"row {n}"
        loan_id = str(row.get("loan_id", "") or "").strip() or f"(row {n})"
        reference = str(row.get("borrower_reference", "") or "").strip()

        skip = _skip_reason(row, where)
        if skip:
            summary[RowStatus.SKIPPED.value] += 1
            lines.append(
                ReviewLine(row_number=n, loan_id=loan_id, borrower_reference=reference,
                           skipped_reason=skip)
            )
            continue

        reviewed += 1
        gate_facts = GateFacts.from_mapping(row, where)
        if gate_benchmark_year is not None:
            gate_facts = replace(gate_facts, year_of_income_being_tested=gate_benchmark_year)
        gate = complying_loan_gate(gate_facts, table=table, override=override)

        myr: MyrResult | None = None
        if not gate_only:
            myr_facts = MyrFacts.from_mapping(row, year_of_income, gate, where)
            myr = minimum_yearly_repayment(myr_facts, table=table, override=override)

        if not myr_only:
            if gate.verdict is GateVerdict.COMPLYING:
                summary[GateVerdict.COMPLYING.value] += 1
            elif gate.verdict is GateVerdict.NOT_COMPLYING:
                summary[GateVerdict.NOT_COMPLYING.value] += 1

        undecided = gate.verdict is GateVerdict.UNKNOWN and not myr_only
        if myr is not None:
            if myr.verdict is MyrVerdict.MYR_MET:
                summary[MyrVerdict.MYR_MET.value] += 1
            elif myr.verdict is MyrVerdict.MYR_SHORT:
                summary[MyrVerdict.MYR_SHORT.value] += 1
            elif myr.verdict is MyrVerdict.REFUSED:
                summary[MyrVerdict.REFUSED.value] += 1
            else:
                undecided = True
        if undecided:
            summary[MyrVerdict.UNKNOWN.value] += 1

        lines.append(
            ReviewLine(
                row_number=n,
                loan_id=loan_id,
                borrower_reference=reference,
                gate=None if myr_only else gate,
                myr=myr,
            )
        )

    return ReviewReport(
        year_of_income=year_of_income.label,
        lines=tuple(sorted(lines, key=_line_order)),
        summary=summary,
        rows_reviewed=reviewed,
    )


def _line_order(line: ReviewLine) -> tuple:
    """Exposure first, largest shortfall first; then the rest of the work; then
    clean rows; then skipped rows. Ties keep the register's own order."""
    if line.is_skipped:
        band = 3
    elif line.has_exposure:
        band = 0
    elif line.needs_attention:
        band = 1
    else:
        band = 2
    return (band, -line.shortfall, line.row_number)


def review_register_file(
    path: Path | str,
    year_of_income: YearOfIncome,
    *,
    gate_only: bool = False,
    myr_only: bool = False,
    gate_benchmark_year: YearOfIncome | None = None,
    table: BenchmarkTable | None = None,
    override: RateOverride | None = None,
) -> ReviewReport:
    needed = list(GATE_COLUMNS)
    if not gate_only:
        needed += list(MYR_COLUMNS)
    rows = load_rows(path, needed)
    try:
        return review_register(
            rows,
            year_of_income,
            gate_only=gate_only,
            myr_only=myr_only,
            gate_benchmark_year=gate_benchmark_year,
            table=table,
            override=override,
        )
    except FactError as exc:
        raise RegisterError(f"{path}: {exc}")
