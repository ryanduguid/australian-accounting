"""Command-line entry point.

Exit codes follow the sibling engines: 0 where nothing is exposed and nothing
is undecided, 2 where a shortfall or an undecided row needs a human, and 1
where the input or the rate table could not be read at all.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from . import LAW_COMPILATION, LAW_CONTENT_DATE, __version__
from .facts import FactError
from .gate import GateResult
from .money import cents_str
from .myr import MyrResult
from .rates import RatesError, RateResult, benchmark_rate, load_override, load_table
from .register import RegisterError, ReviewReport, review_register_file
from .verdicts import GateVerdict, MyrVerdict, RateVerdict
from .years import YearError, parse_year

EXIT_OK = 0
EXIT_ERROR = 1
EXIT_ATTENTION = 2

BANNER = (
    "Experimental review aid. Not a Division 7A determination, not an ATO "
    "assessment, and not tax, legal or financial advice."
)


def _reconfigure_stdout_for_unicode() -> None:
    """Redirected stdout on Windows falls back to the locale encoding, which
    cannot represent every character an operator's loan_id or borrower
    reference might carry."""
    reconfigure = getattr(sys.stdout, "reconfigure", None)
    if reconfigure is not None:
        reconfigure(encoding="utf-8", errors="backslashreplace")


def _assert_no_floats(value: object, path: str = "$") -> None:
    """Refuse to emit a JSON float.

    Every amount in this engine is a Decimal, and every amount in its JSON is
    a quoted decimal string. A float that reached the output would be parsed
    by the reader as an IEEE double and would quietly lose cents, so this is
    checked at the point of emission rather than trusted to review.
    """
    if isinstance(value, float):
        raise AssertionError(
            f"refusing to emit a JSON float at {path}: amounts must be decimal strings"
        )
    if isinstance(value, dict):
        for key, item in value.items():
            _assert_no_floats(item, f"{path}.{key}")
    elif isinstance(value, (list, tuple)):
        for index, item in enumerate(value):
            _assert_no_floats(item, f"{path}[{index}]")


def _emit_json(document: dict) -> None:
    _assert_no_floats(document)
    print(json.dumps(document, indent=2, ensure_ascii=False, allow_nan=False))


def _envelope(command: str, payload: dict) -> dict:
    return {
        "tool": "div7a-loan-review",
        "version": __version__,
        "command": command,
        "law_content_date": LAW_CONTENT_DATE,
        "law_compilation": LAW_COMPILATION,
        "disclaimer": BANNER,
        **payload,
    }


def _print_lines(prefix: str, items: tuple[str, ...] | list[str]) -> None:
    for item in items:
        print(f"  {prefix} {item}")


def _print_rate(result: RateResult) -> None:
    if result.verdict is RateVerdict.KNOWN:
        print(f"{result.year_of_income} benchmark interest rate: {result.rate_text}")
        print(
            f"  source: RBA table {result.rba_table} series {result.rba_series}, "
            f"{result.rba_month} figure ({result.origin})"
        )
        print(f"  entry reviewed: {result.seen}")
    else:
        print(f"{result.year_of_income} benchmark interest rate: UNKNOWN")
        print(f"  {result.reason}")
    print(
        f"  table reviewed to {result.table_reviewed_until} on {result.table_reviewed_on}"
    )


def _print_gate(gate: GateResult) -> None:
    print(f"  s 109N gate: {gate.verdict}  (benchmark year {gate.benchmark_year_used})")
    for limb in gate.limbs:
        print(f"    [{limb.state.value:<7}] {limb.cite}: {limb.finding}")


def _print_myr(myr: MyrResult) -> None:
    print(f"  s 109E MYR: {myr.verdict}")
    # These three are set together or not at all: a REFUSED or UNKNOWN result
    # carries reasons instead of figures.
    required, applied, shortfall = myr.myr_required, myr.payments_applied, myr.shortfall
    if required is not None and applied is not None and shortfall is not None:
        print(f"    required : {cents_str(required)}")
        print(f"    applied  : {cents_str(applied)}")
        print(f"    shortfall: {cents_str(shortfall)}")
        exposure = myr.experimental_deemed_dividend_exposure
        if exposure is not None:
            print(
                f"    experimental deemed-dividend exposure: {cents_str(exposure)} "
                "(review aid, not an assessment)"
            )
    _print_lines("!", myr.reasons)


def _print_report(report: ReviewReport, show_trace: bool) -> None:
    print(f"Loan register review for {report.year_of_income}")
    print(f"rows reviewed: {report.rows_reviewed}")
    counts = ", ".join(f"{key} {value}" for key, value in report.summary.items())
    print(f"summary: {counts}")
    print(
        "  counts are per question, not per row: a reviewed row answers both the "
        "s 109N terms question and the s 109E repayment question"
    )
    print(
        f"experimental total exposure: {cents_str(report.total_exposure)} "
        "(review aid, not an ATO assessment)"
    )
    print()
    for line in report.lines:
        label = f"[{line.row_number}] {line.loan_id}"
        if line.borrower_reference:
            label += f" ({line.borrower_reference})"
        print(label)
        if line.is_skipped:
            print(f"  SKIPPED: {line.skipped_reason}")
            print()
            continue
        if line.gate is not None:
            _print_gate(line.gate)
        if line.myr is not None:
            _print_myr(line.myr)
        if show_trace:
            if line.gate is not None:
                _print_lines("|", line.gate.statutory_trace)
            if line.myr is not None:
                _print_lines("|", line.myr.statutory_trace)
        print()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="div7a-loan-review",
        description=(
            "Experimental review of private-company loans against the ITAA 1936 "
            "s 109N complying-loan criteria and the s 109E minimum yearly "
            "repayment. Not a Division 7A determination."
        ),
        epilog=BANNER,
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    sub = parser.add_subparsers(dest="command", required=True)

    def common(p: argparse.ArgumentParser) -> None:
        p.add_argument(
            "--format", choices=("text", "json"), default="text", help="output format"
        )
        p.add_argument(
            "--rates-override",
            metavar="FILE",
            help=(
                "reviewed benchmark-rate override (JSON carrying verified_until and a "
                "citation); required for any year outside the frozen table"
            ),
        )

    rate = sub.add_parser("rate", help="the s 109N(2) benchmark rate for a year of income")
    rate.add_argument("--year", required=True, help="year of income, e.g. 2026-27")
    common(rate)

    gate = sub.add_parser("gate", help="test a register against the s 109N(1) criteria")
    gate.add_argument("--input", required=True, help="loan register CSV")
    gate.add_argument(
        "--year",
        help=(
            "year of income to read the benchmark floor for. Omit to use each row's "
            "year_loan_made, which is the year s 109N(1)(b) points at"
        ),
    )
    common(gate)

    myr = sub.add_parser("myr", help="the s 109E minimum yearly repayment and any shortfall")
    myr.add_argument("--input", required=True, help="loan register CSV")
    myr.add_argument("--year", required=True, help="year of income being reviewed, e.g. 2026-27")
    common(myr)

    review = sub.add_parser("review", help="the s 109N gate and the s 109E repayment together")
    review.add_argument("--input", required=True, help="loan register CSV")
    review.add_argument("--year", required=True, help="year of income being reviewed")
    review.add_argument(
        "--trace", action="store_true", help="print the statutory trace for every row"
    )
    common(review)

    return parser


def _load_tables(args: argparse.Namespace):
    table = load_table()
    override = load_override(args.rates_override) if args.rates_override else None
    return table, override


def _run_rate(args: argparse.Namespace) -> int:
    table, override = _load_tables(args)
    result = benchmark_rate(parse_year(args.year), table=table, override=override)
    if args.format == "json":
        _emit_json(_envelope("rate", result.to_json_dict()))
    else:
        print(BANNER)
        print()
        _print_rate(result)
    return EXIT_OK if result.verdict is RateVerdict.KNOWN else EXIT_ATTENTION


def _run_register(args: argparse.Namespace, command: str) -> int:
    table, override = _load_tables(args)
    gate_only = command == "gate"
    myr_only = command == "myr"

    if gate_only:
        # Without --year the gate anchors to each row's own year_loan_made,
        # which is the year s 109N(1)(b) points at. With --year it tests every
        # row against that year's benchmark instead, and says so in the result.
        nominated = parse_year(args.year) if args.year else None
        report = review_register_file(
            args.input,
            nominated or _label_year(args.input),
            gate_only=True,
            gate_benchmark_year=nominated,
            table=table,
            override=override,
        )
    else:
        report = review_register_file(
            args.input,
            parse_year(args.year),
            gate_only=False,
            myr_only=myr_only,
            table=table,
            override=override,
        )

    if args.format == "json":
        _emit_json(_envelope(command, report.to_json_dict()))
    else:
        print(BANNER)
        print()
        _print_report(report, show_trace=getattr(args, "trace", False))
    return EXIT_ATTENTION if report.needs_attention else EXIT_OK


def _label_year(path: str):
    """A gate run without --year still needs a year to label the report with.

    The gate itself anchors to each row's own year_loan_made, so this only
    names the report. The first row's year is used where there is one, and
    the table's latest reviewed year otherwise.
    """
    from .facts import optional_year_of_income
    from .register import GATE_COLUMNS, load_rows

    for row in load_rows(path, GATE_COLUMNS):
        year = optional_year_of_income(row.get("year_loan_made"), "year_loan_made")
        if year is not None:
            return year
    return load_table().reviewed_until


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    _reconfigure_stdout_for_unicode()
    try:
        if args.command == "rate":
            return _run_rate(args)
        return _run_register(args, args.command)
    except (RatesError, RegisterError, FactError, YearError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return EXIT_ERROR
    except OSError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return EXIT_ERROR


if __name__ == "__main__":
    raise SystemExit(main())
