from __future__ import annotations

import ast
import csv
from decimal import Decimal
from pathlib import Path

import pytest

from wiptally.csv_io import CANONICAL_FIELDS, CsvError, load_mapping, read_contracts
from wiptally.csvsafe import guard
from wiptally.model import Schedule
from wiptally.report import OUTPUT_COLUMNS, build_review_pack, write_schedule_csv
from wiptally.schedule import measure

ROOT = Path(__file__).resolve().parents[1]
SAMPLE = ROOT / "examples" / "sample_contracts.csv"
ENGINE = ROOT / "wiptally"


def test_money_fields_are_decimal_not_float() -> None:
    contracts = read_contracts(SAMPLE, load_mapping(None))
    for contract in contracts:
        position = measure(contract)
        for name, value in vars(position).items():
            if isinstance(value, float):
                raise AssertionError(f"{name} is a float")
            if isinstance(value, Decimal):
                assert type(value) is Decimal


def test_engine_source_does_not_call_float() -> None:
    banned = {"float"}
    for path in ENGINE.glob("*.py"):
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in ast.walk(tree):
            if isinstance(node, ast.Name) and node.id in banned:
                raise AssertionError(f"{path.name} names float()")
            if isinstance(node, ast.Constant) and isinstance(node.value, float):
                raise AssertionError(f"{path.name} contains a float literal")


def test_formula_injection_is_escaped() -> None:
    assert guard("=cmd") == "'=cmd"
    assert guard("-00123") == "-00123"
    assert guard("-A1") == "'-A1"


def test_every_accepted_input_field_is_read_by_something() -> None:
    """The schema must not accept, validate and store an input nothing reads.

    A `--mapping-file` will happily point a real ledger column at a field the
    engine drops, and the figure then vanishes with no trace in the output.
    """
    engine = (ENGINE / "schedule.py").read_text(encoding="utf-8")
    for name in CANONICAL_FIELDS:
        assert name in OUTPUT_COLUMNS or name in engine, (
            f"{name} is parsed and validated, but no rule and no output column reads it"
        )


def test_review_pack_escapes_pipes_in_ledger_identifiers(tmp_path: Path) -> None:
    """A contract id out of the ledger must not create markdown table columns."""
    source = tmp_path / "contracts.csv"
    with source.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(
            [
                "contract_id",
                "original_contract_sum",
                "costs_incurred",
                "estimated_cost_to_complete",
                "certified_billings",
                "uncertified_claims",
            ]
        )
        writer.writerow(
            ["JOB|A\n| 999 | pwned", "1000.00", "400.00", "400.00", "450.00", "10.00"]
        )

    contracts = read_contracts(source, load_mapping(None))
    schedule = Schedule(
        as_at="2026-08-31",
        positions=[measure(contract) for contract in contracts],
        source_name=source.name,
    )
    out = tmp_path / "wip-schedule.csv"
    write_schedule_csv(out, schedule)
    pack = build_review_pack(out, source, schedule)

    row = next(line for line in pack.splitlines() if line.startswith("| JOB"))
    cells = row.split(" | ")
    assert len(cells) == 4, f"the identifier shifted the row: {row!r}"
    assert cells[0] == r"| JOB\|A \| 999 \| pwned"
    assert cells[1] == "20.00%"
    assert cells[3].startswith("uncertified_claims_present")
    assert cells[3].endswith(" |")


def _markdown_cells(row: str) -> list[str]:
    r"""Split a rendered table row the way a pair-consuming reader does.

    A pipe opens a new column unless an odd-length run of backslashes precedes
    it, so escapes are consumed in pairs. That models CommonMark escaping and
    readers built on it, such as marked. It is deliberately not cmark-gfm,
    whose table scanner recognises only the two-character sequence ``\|`` and
    so keeps the header's column count while corrupting the cell instead. The
    two families wreck a mis-escaped identifier differently, which is why the
    round trip, not the cell count, is the assertion that pins both.
    ``row.split(" | ")`` sees neither, because a pipe smuggled in through an
    identifier carries no surrounding spaces.
    """
    cells: list[str] = []
    current: list[str] = []
    index = 0
    while index < len(row):
        char = row[index]
        if char == "\\" and index + 1 < len(row):
            current.append(row[index : index + 2])
            index += 2
        elif char == "|":
            cells.append("".join(current))
            current = []
            index += 1
        else:
            current.append(char)
            index += 1
    cells.append("".join(current))
    assert cells[0] == "" and cells[-1] == "", f"row is not pipe-delimited: {row!r}"
    return cells[1:-1]


def _markdown_text(cell: str) -> str:
    """Undo one level of backslash escaping, as a markdown reader would."""
    out: list[str] = []
    index = 0
    while index < len(cell):
        if cell[index] == "\\" and index + 1 < len(cell):
            out.append(cell[index + 1])
            index += 2
        else:
            out.append(cell[index])
            index += 1
    return "".join(out)


def test_review_pack_escapes_backslashes_before_pipes(tmp_path: Path) -> None:
    r"""An identifier already holding ``\|`` must survive the table unchanged.

    Escaping the pipe without first escaping the backslash rewrote ``JOB\|A``
    as ``JOB\\|A``, which no renderer reads back as ``JOB\|A``. That lost round
    trip is the defect, and it is renderer-independent. How the damage shows is
    not: cmark-gfm, which GitHub renders with, holds the row at four cells and
    silently drops the backslash to give ``JOB|A``, while a pair-consuming
    reader such as marked takes the pipe as live and gives ``JOB\`` with margin
    and fade one heading to the right. So the round trip is asserted first and
    the cell count second. ``row.split(" | ")`` reported four either way,
    because the smuggled pipe carries no surrounding spaces.
    """
    contract_id = r"JOB\|A"
    source = tmp_path / "contracts.csv"
    with source.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(
            [
                "contract_id",
                "original_contract_sum",
                "costs_incurred",
                "estimated_cost_to_complete",
                "certified_billings",
                "uncertified_claims",
            ]
        )
        writer.writerow([contract_id, "1000.00", "400.00", "400.00", "450.00", "10.00"])

    contracts = read_contracts(source, load_mapping(None))
    schedule = Schedule(
        as_at="2026-08-31",
        positions=[measure(contract) for contract in contracts],
        source_name=source.name,
    )
    out = tmp_path / "wip-schedule.csv"
    write_schedule_csv(out, schedule)
    pack = build_review_pack(out, source, schedule)

    lines = pack.splitlines()
    header = next(line for line in lines if line.startswith("| Contract |"))
    row = next(line for line in lines if line.startswith("| JOB"))
    width = len(_markdown_cells(header))
    assert width == 4, f"the header is no longer four columns: {header!r}"
    cells = _markdown_cells(row)
    # The round trip holds in every conforming renderer, so it leads.
    assert _markdown_text(cells[0].strip()) == contract_id, (
        f"the identifier did not survive the table: {row!r}"
    )
    assert len(cells) == width, f"the identifier shifted the row: {row!r}"
    assert cells[1].strip() == "20.00%"


def test_schedule_totals_do_not_net() -> None:
    contracts = read_contracts(SAMPLE, load_mapping(None))
    schedule = Schedule(
        as_at="2026-08-31",
        positions=[measure(contract) for contract in contracts],
        source_name=SAMPLE.name,
    )
    assert schedule.total_contract_assets > 0
    assert schedule.total_contract_liabilities > 0
    net = schedule.total_contract_assets - schedule.total_contract_liabilities
    assert net != schedule.total_contract_assets
    assert net != schedule.total_contract_liabilities


def test_sample_rows_all_match_the_header_width() -> None:
    """The shipped worked example must not ship a truncated row."""
    with SAMPLE.open(newline="", encoding="utf-8-sig") as handle:
        rows = list(csv.reader(handle))
    width = len(rows[0])
    for line_number, row in enumerate(rows[1:], start=2):
        assert len(row) == width, (
            f"row {line_number} has {len(row)} field(s), not {width}; "
            f"trailing columns would be read as absent"
        )


def test_short_row_is_rejected(tmp_path: Path) -> None:
    header, first_row = SAMPLE.read_text(encoding="utf-8").splitlines()[:2]
    width = len(header.split(","))
    source = tmp_path / "truncated.csv"
    source.write_text(f"{header}\n{first_row.rsplit(',', 1)[0]}\n", encoding="utf-8")
    with pytest.raises(CsvError) as caught:
        read_contracts(source, load_mapping(None))
    message = str(caught.value)
    assert str(source) in message
    assert f"row 2: has {width - 1} field(s) but the header has {width}" in message


def test_surplus_row_is_rejected(tmp_path: Path) -> None:
    header, first_row = SAMPLE.read_text(encoding="utf-8").splitlines()[:2]
    width = len(header.split(","))
    source = tmp_path / "surplus.csv"
    source.write_text(f"{header}\n{first_row},extra\n", encoding="utf-8")
    with pytest.raises(CsvError) as caught:
        read_contracts(source, load_mapping(None))
    assert f"row 2: has {width + 1} field(s) but the header has {width}" in str(caught.value)
