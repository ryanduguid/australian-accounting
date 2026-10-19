"""Reconcile lodged BAS figures with the ledger lines behind them, period by period.

The reconciliation reads facts the caller supplies: ledger lines with GST codes and
dates, the tax periods, the lodged label figures and the GST accounting basis. It
contacts no service and writes nothing.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import date
from decimal import ROUND_DOWN, Decimal
from typing import Any

from ._checks import is_date, is_money
from ._version import __version__
from .coding import GST_ACT, SOURCE_CHECKED

ATO_BAS_METHOD = (
    "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/"
    "completing-your-bas-for-gst/choose-a-method-to-complete-your-bas"
)
ATO_BAS_INSTRUCTIONS = "https://www.ato.gov.au/print/section/9fc804ad-a043-4a35-b540-16b71d9ca9bf"

SALE = "sale"
PURCHASE = "purchase"
LABELS = ("G1", "G2", "G3", "G10", "G11", "1A", "1B")
CATEGORIES = frozenset({"taxable", "capital", "gst_free", "capital_gst_free", "export",
                        "input_taxed", "excluded"})

# The G labels each side and category reaches. Taxable categories also carry their
# GST to 1A (sales) or 1B (purchases). A pair missing here is a coding the
# reconciliation cannot place, such as a capital code on a sale.
_G_LABELS: dict[tuple[str, str], tuple[str, ...]] = {
    (SALE, "taxable"): ("G1",),
    (SALE, "gst_free"): ("G1", "G3"),
    (SALE, "export"): ("G1", "G2"),
    (SALE, "input_taxed"): ("G1",),
    (SALE, "excluded"): (),
    (PURCHASE, "taxable"): ("G11",),
    (PURCHASE, "capital"): ("G10",),
    (PURCHASE, "gst_free"): ("G11",),
    (PURCHASE, "capital_gst_free"): ("G10",),
    (PURCHASE, "input_taxed"): ("G11",),
    (PURCHASE, "excluded"): (),
}
_CARRIES_GST = frozenset({"taxable", "capital"})

# A convenience for common Xero and MYOB AU codes; pass bas_codes for others.
DEFAULT_BAS_CODES: dict[str, str] = {
    "GST on Income": "taxable",
    "GST on Expenses": "taxable",
    "GST on Capital": "capital",
    "GST Free Income": "gst_free",
    "GST Free Expenses": "gst_free",
    "GST Free Capital": "capital_gst_free",
    "GST Free Exports": "export",
    "Input Taxed": "input_taxed",
    "BAS Excluded": "excluded",
    "GST": "taxable",
    "CAP": "capital",
    "FRE": "gst_free",
    "EXP": "export",
    "INP": "input_taxed",
    "N-T": "excluded",
}

NOT_DECIDED = [
    "Whether a difference needs a revised BAS or can be corrected on a later one.",
    "Whether any line's GST code is right. The reconciliation takes the codes as given.",
    "Which accounting basis applies (GST Act s 29-40). The caller states it.",
    "Part payments on the cash basis: supply one line per payment, with the gross and GST "
    "paid.",
    "Labels other than G1, G2, G3, G10, G11, 1A and 1B, adjustments (GST Act Division 19), "
    "and GST on imports.",
]


@dataclass(frozen=True)
class LedgerLine:
    """One ledger line on the sale or purchase side.

    ``gross`` includes GST and ``gst`` is the GST in it; both are negative for a credit
    note. ``payment_date`` is None while unpaid.
    """

    id: str
    side: str
    gross: Decimal
    gst: Decimal
    gst_code: str
    invoice_date: date
    payment_date: date | None = None


@dataclass(frozen=True)
class Period:
    """A tax period, first and last day inclusive."""

    label: str
    start: date
    end: date

    def holds(self, day: date) -> bool:
        return self.start <= day <= self.end


def reconcile_bas(
    lines: list[LedgerLine],
    periods: list[Period],
    lodged: dict[str, dict[str, Decimal]],
    *,
    basis: str,
    g1_includes_gst: bool = True,
    bas_codes: dict[str, str] | None = None,
) -> dict[str, Any]:
    """Compare ledger-derived label figures with the lodged figures for each period."""
    codes = DEFAULT_BAS_CODES if bas_codes is None else bas_codes
    _validate_options(basis, g1_includes_gst, codes)
    _validate_periods(periods)
    _validate_lodged(lodged, {period.label for period in periods})
    _validate_lines(lines)

    tally = _Tally(periods)
    for line in lines:
        tally.add(line, basis, g1_includes_gst, codes)

    return {
        "ok": True,
        "engine": "gst-review",
        "engine_version": __version__,
        "review": "bas_reconciliation",
        "basis": basis,
        "periods": [tally.period_row(period, lodged.get(period.label)) for period in periods],
        "not_lodged": [period.label for period in periods
                       if tally.attributed[period.label] and period.label not in lodged],
        "no_ledger_lines": [period.label for period in periods
                            if period.label in lodged and not tally.attributed[period.label]],
        "outside_periods": tally.outside_periods,
        "unpaid": tally.unpaid,
        "line_issues": tally.line_issues,
        "facts_used": {
            "basis": basis,
            "g1_includes_gst": g1_includes_gst,
            "bas_codes": dict(codes),
            "rounding": "Ledger totals are rounded down to whole dollars before comparison.",
        },
        "not_decided": list(NOT_DECIDED),
        "sources": [GST_ACT, ATO_BAS_METHOD, ATO_BAS_INSTRUCTIONS],
        "source_checked": SOURCE_CHECKED,
        "warnings": [
            "Ledger lines, codes, periods and lodged figures are the caller's facts and are "
            "not independently verified.",
            "Review aid only, not advice, an assessment or a lodgment. Obtain human review.",
        ],
    }


class _Tally:
    """Label totals and line lists per period, built one line at a time."""

    def __init__(self, periods: list[Period]) -> None:
        self.periods = periods
        self.totals: dict[str, dict[str, Decimal]] = {
            period.label: dict.fromkeys(LABELS, Decimal(0)) for period in periods}
        self.contributors: dict[str, dict[str, list[str]]] = {
            period.label: {label: [] for label in LABELS} for period in periods}
        self.attributed: dict[str, list[str]] = {period.label: [] for period in periods}
        self.kept_out: dict[str, list[str]] = {period.label: [] for period in periods}
        self.timing: dict[str, list[str]] = {period.label: [] for period in periods}
        self.line_issues: list[dict[str, str]] = []
        self.outside_periods: list[str] = []
        self.unpaid: list[str] = []

    def period_of(self, day: date | None) -> Period | None:
        return next((period for period in self.periods if day and period.holds(day)), None)

    def add(self, line: LedgerLine, basis: str, g1_includes_gst: bool,
            codes: dict[str, str]) -> None:
        day = _attribution_date(line, basis)
        period = self.period_of(day)
        invoiced_in = self.period_of(line.invoice_date)
        if invoiced_in is not None and invoiced_in is not period:
            self.timing[invoiced_in.label].append(line.id)
        if day is None:
            self.unpaid.append(line.id)
            return
        if period is None:
            self.outside_periods.append(line.id)
            return
        self.attributed[period.label].append(line.id)

        issue = _issue(line, codes)
        if issue:
            self.line_issues.append({"id": line.id, "issue": issue})
        reached = [] if issue else _labels_reached(line, codes[line.gst_code])
        if not reached:
            self.kept_out[period.label].append(line.id)
        amount = line.gross if g1_includes_gst else line.gross - line.gst
        for label in reached:
            self.totals[period.label][label] += line.gst if label in ("1A", "1B") else amount
            self.contributors[period.label][label].append(line.id)

    def period_row(self, period: Period, lodged: dict[str, Decimal] | None) -> dict[str, Any]:
        return {
            "period": period.label,
            "start": period.start.isoformat(),
            "end": period.end.isoformat(),
            "lodged": lodged is not None,
            "labels": {
                label: _compare(self.totals[period.label][label],
                                None if lodged is None else lodged.get(label),
                                self.contributors[period.label][label])
                for label in LABELS
            },
            "kept_out": self.kept_out[period.label],
            "timing": self.timing[period.label],
        }


def _labels_reached(line: LedgerLine, category: str) -> list[str]:
    reached = list(_G_LABELS[(line.side, category)])
    if category in _CARRIES_GST:
        reached.append("1A" if line.side == SALE else "1B")
    return reached


def _attribution_date(line: LedgerLine, basis: str) -> date | None:
    """GST Act ss 29-5 and 29-10: the cash basis follows payment, accruals the earlier
    of payment and invoice."""
    if basis == "cash":
        return line.payment_date
    if line.payment_date is None:
        return line.invoice_date
    return min(line.invoice_date, line.payment_date)


def _issue(line: LedgerLine, codes: dict[str, str]) -> str | None:
    category = codes.get(line.gst_code)
    if category is None:
        return f"GST code {line.gst_code!r} is not in the code table."
    if (line.side, category) not in _G_LABELS:
        return f"GST code {line.gst_code!r} ({category}) cannot be placed on a {line.side}."
    if line.gst != 0 and category not in _CARRIES_GST:
        return f"GST of {line.gst} on code {line.gst_code!r}, which carries no GST."
    return None


def _compare(ledger: Decimal, lodged: Decimal | None, lines: list[str]) -> dict[str, Any]:
    whole = ledger.to_integral_value(rounding=ROUND_DOWN)
    row: dict[str, Any] = {"ledger": str(ledger), "ledger_whole_dollars": str(whole),
                           "lodged": None if lodged is None else str(lodged),
                           "difference": None, "lines": []}
    if ledger < 0:
        row["status"] = "negative_ledger"
        row["lines"] = lines
    elif lodged is None:
        row["status"] = "not_reported"
    elif whole == lodged:
        row["status"] = "matches"
    else:
        row["status"] = "differs"
        row["difference"] = str(whole - lodged)
        row["lines"] = lines
    return row


def _validate_options(basis: str, g1_includes_gst: bool, codes: dict[str, str]) -> None:
    if basis not in ("cash", "accruals"):
        raise ValueError("basis must be 'cash' or 'accruals'.")
    if not isinstance(g1_includes_gst, bool):
        raise ValueError("g1_includes_gst must be a boolean.")
    unknown = sorted(set(codes.values()) - CATEGORIES)
    if unknown:
        raise ValueError(f"Unknown BAS categories in the code table: {unknown}.")


def _validate_periods(periods: list[Period]) -> None:
    labels = [period.label for period in periods]
    if len(set(labels)) != len(labels) or not all(isinstance(x, str) and x for x in labels):
        raise ValueError("Period labels must be unique non-empty strings.")
    for period in periods:
        if (not is_date(period.start, optional=False) or not is_date(period.end, optional=False)
                or period.end < period.start):
            raise ValueError(f"Period {period.label!r} needs a start on or before its end.")
    ordered = sorted(periods, key=lambda period: period.start)
    for earlier, later in zip(ordered, ordered[1:]):
        if earlier.end >= later.start:
            raise ValueError(f"Period {later.label!r} overlaps {earlier.label!r}.")


def _validate_lodged(lodged: dict[str, dict[str, Decimal]], labels: set[str]) -> None:
    for label, figures in lodged.items():
        if label not in labels:
            raise ValueError(f"Lodged figures for unknown period {label!r}.")
        for key, value in figures.items():
            if key not in LABELS:
                raise ValueError(f"Unsupported BAS label {key!r} for {label!r}.")
            if not is_money(value) or value < 0 or value != value.to_integral_value():
                raise ValueError(f"Lodged {key} for {label!r} must be whole non-negative "
                                 "dollars as a Decimal.")


def _validate_lines(lines: list[LedgerLine]) -> None:
    seen: set[str] = set()
    for line in lines:
        if not isinstance(line.id, str) or not line.id or line.id in seen:
            raise ValueError(f"Line ids must be unique non-empty strings: {line.id!r}.")
        seen.add(line.id)
        if line.side not in (SALE, PURCHASE) or not isinstance(line.gst_code, str):
            raise ValueError(f"Line {line.id!r} needs a side of 'sale' or 'purchase' and a "
                             "GST code.")
        if not (is_date(line.invoice_date, optional=False)
                and is_date(line.payment_date, optional=True)):
            raise ValueError(f"Line {line.id!r} needs an invoice date and an optional "
                             "payment date.")
        if not (is_money(line.gross) and is_money(line.gst)):
            raise ValueError(f"Line {line.id!r} amounts must be finite Decimals, at most "
                             "2 decimal places and 1e12.")
        if abs(line.gst) > abs(line.gross) or line.gst * line.gross < 0:
            raise ValueError(f"Line {line.id!r} GST must share the sign of the gross and "
                             "not exceed it.")
