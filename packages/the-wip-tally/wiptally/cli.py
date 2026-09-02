"""Command line interface.

Exit codes:
    0  schedule produced and no review flags
    1  the run could not produce a schedule
    2  schedule produced and at least one contract needs review
"""

from __future__ import annotations

import argparse
import sys
from datetime import date
from pathlib import Path

from . import __version__
from .csv_io import CsvError, load_mapping, read_contracts
from .model import Schedule
from .money import AmountError
from .report import build_review_pack, render_console, write_review_pack, write_schedule_csv
from .schedule import ScheduleError, measure

EXIT_OK = 0
EXIT_ERROR = 1
EXIT_REVIEW = 2


def _refuse_to_write_over_an_input(out: Path, inputs: list[Path | None]) -> None:
    for candidate in inputs:
        if candidate is None:
            continue
        try:
            if out.resolve() == candidate.resolve():
                raise CsvError(
                    f"refusing to write {out} because this run reads that same file"
                )
        except OSError as exc:
            raise CsvError(f"refusing to write {out}: cannot resolve {candidate} ({exc})") from exc


def _require_suffix(path: Path, suffix: str) -> None:
    if path.suffix.casefold() != suffix:
        raise CsvError(f"{path} must have a {suffix} filename")


def _as_at(raw: str | None) -> str:
    """Return the reporting date in YYYY-MM-DD form.

    This one field dates the evidence, so a transposed `2026-31-08` is refused
    rather than printed on a sign-off pack.
    """
    if raw is None:
        return date.today().isoformat()
    try:
        return date.fromisoformat(raw).isoformat()
    except ValueError as exc:
        raise CsvError(f"--as-at {raw!r} is not a date in YYYY-MM-DD form") from exc


def cmd_schedule(args: argparse.Namespace) -> int:
    source = Path(args.contracts)
    out = Path(args.output)
    _require_suffix(out, ".csv")
    mapping_path = Path(args.mapping_file) if args.mapping_file else None
    _refuse_to_write_over_an_input(out, [source, mapping_path])

    mapping = load_mapping(mapping_path)
    contracts = read_contracts(source, mapping)
    positions = [measure(contract) for contract in contracts]
    as_at = _as_at(args.as_at)
    schedule = Schedule(as_at=as_at, positions=positions, source_name=source.name)
    write_schedule_csv(out, schedule)
    print(render_console(schedule), end="")
    print(f"Wrote {out}")
    return EXIT_REVIEW if schedule.review_rows else EXIT_OK


def cmd_review_pack(args: argparse.Namespace) -> int:
    schedule_path = Path(args.schedule)
    out = Path(args.output)
    _require_suffix(out, ".md")
    source = Path(args.source) if args.source else None
    mapping_path = Path(args.mapping_file) if args.mapping_file else None
    _refuse_to_write_over_an_input(out, [schedule_path, source, mapping_path])

    # Rebuild from the source CSV so the pack cannot drift from a hand-edited schedule.
    if source is None:
        raise CsvError(
            "review-pack needs --source (the contract CSV) so the pack can bind "
            "itself to the facts, not only to a downstream schedule file"
        )
    mapping = load_mapping(mapping_path)
    contracts = read_contracts(source, mapping)
    positions = [measure(contract) for contract in contracts]
    as_at = _as_at(args.as_at)
    schedule = Schedule(as_at=as_at, positions=positions, source_name=source.name)
    if not schedule_path.exists():
        raise CsvError(
            f"{schedule_path} does not exist. Run `wip-tally schedule` first so the "
            f"pack can hash the schedule bytes that were actually reviewed."
        )
    text = build_review_pack(schedule_path, source, schedule)
    write_review_pack(out, text)
    print(f"Wrote {out}")
    return EXIT_REVIEW if schedule.review_rows else EXIT_OK


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="wip-tally",
        description=(
            "Build an AASB 15 construction WIP schedule from a contract CSV. "
            "Nothing leaves this machine. Review aid only, not a determination."
        ),
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    subparsers = parser.add_subparsers(dest="command", required=True)

    schedule = subparsers.add_parser("schedule", help="compute the WIP schedule")
    schedule.add_argument("contracts", help="contract CSV")
    schedule.add_argument("-o", "--output", default="wip-schedule.csv")
    schedule.add_argument("--as-at", help="reporting date YYYY-MM-DD (default: today)")
    schedule.add_argument("--mapping-file", help="JSON map of canonical fields to column headings")
    schedule.set_defaults(func=cmd_schedule)

    pack = subparsers.add_parser("review-pack", help="write a practitioner sign-off pack")
    pack.add_argument("schedule", help="schedule CSV produced by `wip-tally schedule`")
    pack.add_argument("--source", required=True, help="the contract CSV that produced it")
    pack.add_argument("-o", "--output", default="practitioner-review.md")
    pack.add_argument("--as-at", help="reporting date YYYY-MM-DD (default: today)")
    pack.add_argument("--mapping-file", help="JSON map of canonical fields to column headings")
    pack.set_defaults(func=cmd_review_pack)
    return parser


def main(argv: list[str] | None = None) -> int:
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is not None:
            try:
                reconfigure(errors="backslashreplace")
            except (ValueError, OSError):
                pass

    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return int(args.func(args))
    except (CsvError, ScheduleError, AmountError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return EXIT_ERROR
    except OSError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return EXIT_ERROR


if __name__ == "__main__":
    raise SystemExit(main())
