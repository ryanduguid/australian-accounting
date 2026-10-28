"""The div7a-loan-review Excel workbook is a second implementation of the engine's rules.

It ships with cached values written by desktop Excel (the Rust `build_workbook` example
in packages/div7a-loan-review), so this test can hold it to the engine's answer for the
mixed sample without Excel on the test machine. The engine is a Rust crate; its review of that sample is frozen in
tests/data/workbook-sample.json, which `cargo test` holds to the engine. If the engine,
the rate table or the workbook changes on its own, one of the two tests fails.
"""

from __future__ import annotations

import csv
import json
import re
import zipfile
from decimal import Decimal
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1] / "packages" / "div7a-loan-review"
ENGINE_REVIEW = ROOT / "tests" / "data" / "workbook-sample.json"
# The crate version the shipped workbook was last built and recalculated against; a
# desktop Excel rebuild records the current one, and this constant moves with it.
WORKBOOK_ENGINE_VERSION = "1.0.0"
WORKBOOK = ROOT / "workbooks" / "div7a-loan-review.xlsx"
SAMPLE = ROOT / "examples" / "sample_loans_mixed.csv"
SHEETS = ("Start Here", "Register", "Rates", "Summary", "Review Checks", "Sources & Version")
YEAR = "2026-27"
YEAR_HELPERS = ("Year_made_text", "Year_tested_text")


@pytest.fixture(scope="module")
def cached():
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    openpyxl = pytest.importorskip("openpyxl")
    return openpyxl.load_workbook(WORKBOOK, data_only=True)


def register_rows(book):
    ws = book["Register"]
    titles = [c.value for c in ws[1]]
    return [dict(zip(titles, row)) for row in ws.iter_rows(min_row=2, values_only=True) if row[0]]


def as_decimal(value):
    if value is None or value == "":
        return None
    return Decimal(str(value))


def test_workbook_is_macro_free_and_carries_no_build_path(cached):
    with zipfile.ZipFile(WORKBOOK) as archive:
        names = archive.namelist()
        workbook_xml = archive.read("xl/workbook.xml")
        tables = "".join(archive.read(n).decode("utf-8") for n in names if n.startswith("xl/tables/"))
    assert not [n for n in names if "vba" in n.lower() or n.endswith(".bin")]
    assert b"absPath" not in workbook_xml
    assert tuple(cached.sheetnames) == SHEETS
    for column in ("Status", "Gate_verdict", "MYR_verdict", "Shortfall", "Guard", "Input_problem"):
        assert re.search(rf'name="{column}"[^>]*>\s*<calculatedColumnFormula>', tables), column


def test_cached_values_were_calculated_by_desktop_excel(cached):
    sources = cached["Sources & Version"]
    assert sources["B2"].value == WORKBOOK_ENGINE_VERSION
    assert str(sources["B3"].value).startswith("Excel ")
    assert cached["Summary"]["B2"].value == YEAR
    assert cached["Review Checks"]["B9"].value == "REVIEW", "shipped sample loans are flagged"
    assert cached["Review Checks"]["C9"].value == 8
    assert cached["Review Checks"]["B10"].value == "REVIEW"
    assert cached["Start Here"]["A11"].value == "REVIEW"
    assert "Interest-floor interpretation unresolved" in cached["Review Checks"]["A13"].value
    assert "COMPLYING does not establish" in cached["Review Checks"]["A13"].value


def test_mixed_sample_matches_the_engine_line_by_line(cached):
    with SAMPLE.open(newline="", encoding="utf-8") as fh:
        rows = list(csv.DictReader(fh))
    report = json.loads(ENGINE_REVIEW.read_text(encoding="utf-8"))
    assert report["year_of_income"] == YEAR
    got = {row["loan_id"]: row for row in register_rows(cached)}
    assert list(got) == [row["loan_id"] for row in rows]
    for line in report["lines"]:
        row = got[line["loan_id"]]
        assert row["Status"] == line["status"], line["loan_id"]
        gate, myr = line["gate"], line["myr"]
        assert (row["Gate_verdict"] or None) == (None if gate is None else gate["verdict"])
        assert as_decimal(row["Max_term_allowed"]) == (
            None if gate is None else as_decimal(gate["maximum_term_years_allowed"])), line["loan_id"]
        assert (row["MYR_verdict"] or None) == (None if myr is None else myr["verdict"])
        if myr is None:
            continue
        for column, key in (("Term_used", "remaining_term_years_used"), ("MYR_required", "myr_required"),
                            ("Shortfall", "shortfall"), ("Exposure", "experimental_deemed_dividend_exposure")):
            assert as_decimal(row[column]) == as_decimal(myr[key]), (line["loan_id"], column)
    summary = cached["Summary"]
    got_summary = {summary.cell(row=r, column=1).value: summary.cell(row=r, column=2).value
                   for r in range(7, 16)}
    assert got_summary.pop("rows_reviewed") == report["rows_reviewed"]
    assert as_decimal(got_summary.pop("experimental_total_exposure")) == as_decimal(
        report["experimental_total_exposure"])
    assert got_summary == report["summary"]


def test_rates_sheet_matches_the_frozen_table(cached):
    lines = (ROOT / "data" / "benchmark_rates.csv").read_text(encoding="utf-8").splitlines()
    table = csv.DictReader(line for line in lines if not line.startswith("#"))
    ws = cached["Rates"]
    last_row = int(re.search(r"(\d+)$", ws.tables["tblRates"].ref).group(1))
    rows = ws.iter_rows(min_row=2, max_row=last_row, max_col=2, values_only=True)
    got = {year: as_decimal(rate) for year, rate in rows}
    want = {row["year_of_income"]: Decimal(row["rate"]) for row in table}
    assert got == want


def test_the_summary_year_selector_grows_with_the_rates_table():
    # The Rates sheet invites the reviewer to add a year outside the frozen table. A
    # validation range fixed at build time left that year unselectable, so the source
    # has to be the table column itself, through a defined name.
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    openpyxl = pytest.importorskip("openpyxl")
    book = openpyxl.load_workbook(WORKBOOK)

    assert book.defined_names["RateYears"].value == "tblRates[year_of_income]"
    validations = [
        dv for dv in book["Summary"].data_validations.dataValidation
        if "B2" in str(dv.sqref)
    ]
    # openpyxl reports the stored formula, which has no leading "=". Excel shows it as
    # "=RateYears" in the validation dialog.
    assert [dv.formula1 for dv in validations] == ["RateYears"]


def test_input_columns_are_text_or_validated_so_pasted_values_stay_inert():
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    openpyxl = pytest.importorskip("openpyxl")
    book = openpyxl.load_workbook(WORKBOOK)
    ws = book["Register"]
    titles = [c.value for c in ws[1]]
    for column in ("loan_id", "year_loan_made", "out_of_scope_reason"):
        assert ws.cell(row=2, column=titles.index(column) + 1).number_format == "@", column
    guard = ws.cell(row=2, column=titles.index("Guard") + 1).value
    assert "ISFORMULA(" in guard
    assert ws.protection.sheet is False, "pasting extends the table"
    assert book["Summary"].protection.sheet is True
    assert book["Review Checks"].protection.sheet is True


def test_year_helpers_extend_the_table_without_moving_public_columns():
    """The builder's formula tests run under `cargo test`; this holds the shipped file."""
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    openpyxl = pytest.importorskip("openpyxl")
    book = openpyxl.load_workbook(WORKBOOK)
    ws = book["Register"]
    titles = [cell.value for cell in ws[1]]
    assert titles[-2:] == list(YEAR_HELPERS)
    # These established positions are used by existing native probes and consumers.
    assert titles.index("Status") == 14
    assert titles.index("Gate_verdict") == 23
    assert titles.index("Max_term_allowed") == 24
    assert titles.index("Input_problem") == 34
    for helper in YEAR_HELPERS:
        column = titles.index(helper) + 1
        # Excel can combine adjacent hidden columns into one dimension range.
        assert any(dimension.hidden and dimension.min <= column <= dimension.max
                   for dimension in ws.column_dimensions.values())
        assert ws.tables["tblLoans"].tableColumns[column - 1].calculatedColumnFormula is not None
        assert all(ws.cell(row, column).data_type == "f" for row in range(2, ws.max_row + 1))
    book.close()
