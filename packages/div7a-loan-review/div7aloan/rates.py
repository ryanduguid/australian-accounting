"""The benchmark interest rate for a year of income: ITAA 1936 s 109N(2).

    "The benchmark interest rate for the year of income is the Indicator
    Lending Rates-Bank variable housing loans interest rate last published by
    the Reserve Bank of Australia before the start of the year of income."
    -- s 109N(2)

A year of income starts on 1 July. The RBA publishes statistical table F5 in
arrears, in the first week of the following month, so the last figure
published before 1 July is the MAY figure. The June figure appears in early
July, after the year of income has already started, and is the usual wrong
rate: June 2025 was 8.27 per cent while May 2025 was 8.37 per cent, and 8.37
is the benchmark rate for 2025-26.

The table is frozen in div7aloan/data/benchmark_rates.csv and reviewed by
hand. Nothing here touches the network. A year outside the reviewed coverage
is UNKNOWN, not an extrapolation from the nearest year: a benchmark rate
guessed one year forward moves every minimum yearly repayment in the file.
"""
from __future__ import annotations

import csv
import json
from dataclasses import dataclass, field
from decimal import Decimal
from pathlib import Path
from typing import Iterable

from .money import MoneyError, parse_rate, rate_str
from .verdicts import RateVerdict
from .years import YearError, YearOfIncome, parse_year

DATA_DIR = Path(__file__).resolve().parent / "data"
RATES_PATH = DATA_DIR / "benchmark_rates.csv"

FROZEN_ORIGIN = "frozen table"
OVERRIDE_ORIGIN = "reviewed override"


class RatesError(ValueError):
    """The rate table, or an operator's override of it, cannot be trusted."""


@dataclass(frozen=True)
class BenchmarkEntry:
    """One reviewed year in the table."""

    year: YearOfIncome
    rate: Decimal
    rba_table: str
    rba_series: str
    rba_month: str
    source: str
    verify_at: str
    seen: str
    origin: str = FROZEN_ORIGIN


@dataclass(frozen=True)
class RateResult:
    """s 109N(2) benchmark rate, with enough provenance to audit it later."""

    verdict: RateVerdict
    year_of_income: str
    rate: Decimal | None = None
    rba_table: str = ""
    rba_series: str = ""
    rba_month: str = ""
    source: str = ""
    verify_at: str = ""
    seen: str = ""
    origin: str = ""
    table_reviewed_until: str = ""
    table_reviewed_on: str = ""
    reason: str | None = None
    statutory_trace: tuple[str, ...] = field(default_factory=tuple)

    @property
    def rate_text(self) -> str | None:
        return None if self.rate is None else rate_str(self.rate)

    def to_json_dict(self) -> dict:
        return {
            "verdict": self.verdict.value,
            "year_of_income": self.year_of_income,
            "benchmark_rate": self.rate_text,
            "provenance": {
                "origin": self.origin,
                "rba_table": self.rba_table,
                "rba_series": self.rba_series,
                "rba_month": self.rba_month,
                "source": self.source,
                "verify_at": self.verify_at,
                "entry_seen": self.seen,
                "table_reviewed_until": self.table_reviewed_until,
                "table_reviewed_on": self.table_reviewed_on,
            },
            "reason": self.reason,
            "statutory_trace": list(self.statutory_trace),
        }


@dataclass(frozen=True)
class BenchmarkTable:
    entries: dict[int, BenchmarkEntry]
    reviewed_until: YearOfIncome
    reviewed_on: str

    @property
    def earliest(self) -> YearOfIncome:
        return YearOfIncome(min(self.entries))

    @property
    def latest(self) -> YearOfIncome:
        return YearOfIncome(max(self.entries))

    def coverage(self) -> str:
        return f"{self.earliest.label} to {self.latest.label}"

    def with_override(self, override: "RateOverride") -> "BenchmarkTable":
        merged = dict(self.entries)
        for entry in override.entries:
            merged[entry.year.start_year] = entry
        return BenchmarkTable(
            entries=merged,
            reviewed_until=max(self.reviewed_until, override.verified_until),
            reviewed_on=f"{self.reviewed_on}; override {override.citation}",
        )

    def lookup(self, year: YearOfIncome) -> RateResult:
        trace = (
            "ITAA 1936 s 109N(2): benchmark interest rate is the RBA Indicator "
            "Lending Rates - Bank variable housing loans rate last published "
            f"before the start of the year of income ({year.starts_on.isoformat()}).",
        )
        entry = self.entries.get(year.start_year)
        if entry is None:
            return RateResult(
                verdict=RateVerdict.UNKNOWN,
                year_of_income=year.label,
                table_reviewed_until=self.reviewed_until.label,
                table_reviewed_on=self.reviewed_on,
                reason=(
                    f"No reviewed benchmark rate for {year.label}. The frozen table "
                    f"covers {self.coverage()}. Supply a reviewed override file "
                    "(--rates-override) carrying verified_until and a citation, or "
                    "treat this year as UNKNOWN. This engine does not read the RBA "
                    "at runtime and does not extrapolate from an adjacent year."
                ),
                statutory_trace=trace,
            )
        return RateResult(
            verdict=RateVerdict.KNOWN,
            year_of_income=year.label,
            rate=entry.rate,
            rba_table=entry.rba_table,
            rba_series=entry.rba_series,
            rba_month=entry.rba_month,
            source=entry.source,
            verify_at=entry.verify_at,
            seen=entry.seen,
            origin=entry.origin,
            table_reviewed_until=self.reviewed_until.label,
            table_reviewed_on=self.reviewed_on,
            statutory_trace=trace
            + (
                f"Rate taken from RBA table {entry.rba_table} series "
                f"{entry.rba_series}, {entry.rba_month} figure, as reviewed on "
                f"{entry.seen} ({entry.origin}).",
            ),
        )


@dataclass(frozen=True)
class RateOverride:
    entries: tuple[BenchmarkEntry, ...]
    verified_until: YearOfIncome
    citation: str


def _header_scalar(lines: Iterable[str], key: str, path: Path) -> str:
    """Read a '# key: value' line out of the table's comment header.

    The reviewed_until marker lives in the CSV rather than beside it so that
    the coverage claim cannot drift away from the rows it describes.
    """
    prefix = "# " + key + ":"
    for line in lines:
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    raise RatesError(f"{path} is missing its {prefix!r} header line")


def _required(row: dict, key: str, where: str) -> str:
    value = (row.get(key) or "").strip()
    if not value:
        raise RatesError(f"{where} is missing {key!r}")
    return value


def load_table(path: Path | None = None) -> BenchmarkTable:
    path = RATES_PATH if path is None else Path(path)
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise RatesError(f"cannot read the benchmark rate table at {path}: {exc}")

    lines = text.splitlines()
    reviewed_until_label = _header_scalar(lines, "reviewed_until", path)
    reviewed_on = _header_scalar(lines, "reviewed_on", path)
    try:
        reviewed_until = parse_year(reviewed_until_label, f"{path} reviewed_until")
    except YearError as exc:
        raise RatesError(str(exc))

    body = [line for line in lines if not line.startswith("#")]
    entries: dict[int, BenchmarkEntry] = {}
    for n, row in enumerate(csv.DictReader(body), start=1):
        where = f"benchmark rate row {n} in {path}"
        try:
            year = parse_year(_required(row, "year_of_income", where), where)
            rate = parse_rate(_required(row, "rate", where), f"{where} rate")
        except (YearError, MoneyError) as exc:
            raise RatesError(str(exc))
        if year.start_year in entries:
            raise RatesError(f"{where}: {year.label} appears twice in the table")
        entries[year.start_year] = BenchmarkEntry(
            year=year,
            rate=rate,
            rba_table=_required(row, "rba_table", where),
            rba_series=_required(row, "rba_series", where),
            rba_month=_required(row, "rba_month", where),
            source=_required(row, "source", where),
            verify_at=(row.get("verify_at") or "").strip(),
            seen=_required(row, "seen", where),
        )
    if not entries:
        raise RatesError(f"{path} holds no benchmark rates")

    latest = YearOfIncome(max(entries))
    if reviewed_until < latest:
        # The header is the claim a reader trusts. A row past it means the
        # table was extended without re-reviewing the coverage statement.
        raise RatesError(
            f"{path} claims reviewed_until {reviewed_until.label} but carries a rate "
            f"for {latest.label}; re-review the table and update the header"
        )
    return BenchmarkTable(entries=entries, reviewed_until=reviewed_until, reviewed_on=reviewed_on)


def load_override(path: Path | str) -> RateOverride:
    """Load an operator's reviewed rate override.

    Refused unless it carries both verified_until and a non-empty citation.
    The whole point of the override is that a human went and read the RBA
    figure; a file that does not say who checked what, and how far, is not a
    review and this engine will not treat it as one.
    """
    path = Path(path)
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise RatesError(f"cannot read the rate override at {path}: {exc}")
    except json.JSONDecodeError as exc:
        raise RatesError(f"{path} is not valid JSON: {exc}")
    if not isinstance(doc, dict):
        raise RatesError(
            f"{path} must be a JSON object with 'verified_until', 'citation' and "
            f"'rates'; it holds a {type(doc).__name__}"
        )

    citation = str(doc.get("citation") or "").strip()
    if not citation:
        raise RatesError(
            f"{path} has no 'citation'. Record where the rate was read from -- RBA "
            "table F5 series FILRHLBVS, the month, and the date you read it -- "
            "before this engine will use it"
        )
    if "verified_until" not in doc:
        raise RatesError(
            f"{path} has no 'verified_until'. State the last year of income the "
            "override has actually been reviewed to"
        )
    try:
        verified_until = parse_year(doc["verified_until"], f"{path} verified_until")
    except YearError as exc:
        raise RatesError(str(exc))

    raw_rates = doc.get("rates")
    if not isinstance(raw_rates, list) or not raw_rates:
        raise RatesError(f"{path}: 'rates' must be a non-empty list of rate objects")

    entries: list[BenchmarkEntry] = []
    for n, row in enumerate(raw_rates, start=1):
        where = f"override rate {n} in {path}"
        if not isinstance(row, dict):
            raise RatesError(f"{where} is not an object")
        try:
            year = parse_year(_required(row, "year_of_income", where), where)
            rate = parse_rate(_required(row, "rate", where), f"{where} rate")
        except (YearError, MoneyError) as exc:
            raise RatesError(str(exc))
        if year > verified_until:
            raise RatesError(
                f"{where} supplies {year.label}, past the file's own verified_until "
                f"{verified_until.label}. Either review that year or remove it"
            )
        entries.append(
            BenchmarkEntry(
                year=year,
                rate=rate,
                rba_table=(row.get("rba_table") or "F5").strip(),
                rba_series=(row.get("rba_series") or "FILRHLBVS").strip(),
                rba_month=(row.get("rba_month") or "").strip(),
                source=(row.get("source") or citation).strip(),
                verify_at=(row.get("verify_at") or "").strip(),
                seen=(row.get("seen") or "").strip(),
                origin=OVERRIDE_ORIGIN,
            )
        )
    return RateOverride(entries=tuple(entries), verified_until=verified_until, citation=citation)


def benchmark_rate(
    year_of_income: object,
    *,
    table: BenchmarkTable | None = None,
    override: RateOverride | None = None,
) -> RateResult:
    """The s 109N(2) benchmark rate for a year of income.

    Returns a RateResult carrying verdict UNKNOWN, with a reason, for any year
    the repository has not reviewed. It does not raise for an unreviewed year:
    an unknown rate is a review outcome, not a program error.
    """
    if isinstance(year_of_income, YearOfIncome):
        year = year_of_income
    else:
        year = parse_year(year_of_income)
    table = load_table() if table is None else table
    if override is not None:
        table = table.with_override(override)
    return table.lookup(year)
