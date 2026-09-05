"""The Excel workbook is a second implementation of the engine's rules.

It ships with cached values written by desktop Excel (tools/build_workbook.py), so this
test can hold it to the engine's answer for the mixed sample without Excel on the test
machine. If the engine, the rate table or the workbook changes on its own, this test is
what fails.
"""

from __future__ import annotations

import csv
import re
import zipfile
from decimal import Decimal
from pathlib import Path

import pytest

from div7aloan import __version__, parse_year, review_register
from div7aloan.rates import load_table

ROOT = Path(__file__).resolve().parents[1]
WORKBOOK = ROOT / "workbooks" / "div7a-loan-review.xlsx"
SAMPLE = ROOT / "examples" / "sample_loans_mixed.csv"
SHEETS = ("Start Here", "Register", "Rates", "Summary", "Review Checks", "Sources & Version")
YEAR = "2026-27"


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
    assert sources["B2"].value == __version__
    assert str(sources["B3"].value).startswith("Excel ")
    assert cached["Summary"]["B2"].value == YEAR
    assert cached["Review Checks"]["B10"].value == "REVIEW"
    assert cached["Start Here"]["A11"].value == "REVIEW"


def test_mixed_sample_matches_the_engine_line_by_line(cached):
    with SAMPLE.open(newline="", encoding="utf-8") as fh:
        rows = list(csv.DictReader(fh))
    report = review_register(rows, parse_year(YEAR))
    got = {row["loan_id"]: row for row in register_rows(cached)}
    assert list(got) == [row["loan_id"] for row in rows]
    for line in report.lines:
        row = got[line.loan_id]
        status = "SKIPPED" if line.skipped_reason else "REVIEWED"
        assert row["Status"] == status, line.loan_id
        gate, myr = line.gate, line.myr
        assert (row["Gate_verdict"] or None) == (None if gate is None else gate.verdict.value)
        assert as_decimal(row["Max_term_allowed"]) == (
            None if gate is None else gate.maximum_term_years_allowed), line.loan_id
        assert (row["MYR_verdict"] or None) == (None if myr is None else myr.verdict.value)
        if myr is None:
            continue
        assert as_decimal(row["Term_used"]) == myr.remaining_term_years_used, line.loan_id
        assert as_decimal(row["MYR_required"]) == myr.myr_required, line.loan_id
        assert as_decimal(row["Shortfall"]) == myr.shortfall, line.loan_id
        assert as_decimal(row["Exposure"]) == myr.experimental_deemed_dividend_exposure, line.loan_id
    summary = cached["Summary"]
    got_summary = {summary.cell(row=r, column=1).value: summary.cell(row=r, column=2).value
                   for r in range(7, 16)}
    assert got_summary.pop("rows_reviewed") == report.rows_reviewed
    assert as_decimal(got_summary.pop("experimental_total_exposure")) == report.total_exposure
    assert got_summary == report.summary


def test_rates_sheet_matches_the_frozen_table(cached):
    table = load_table()
    ws = cached["Rates"]
    last_row = int(re.search(r"(\d+)$", ws.tables["tblRates"].ref).group(1))
    rows = ws.iter_rows(min_row=2, max_row=last_row, max_col=2, values_only=True)
    got = {year: as_decimal(rate) for year, rate in rows}
    want = {entry.year.label: entry.rate for entry in table.entries.values()}
    assert got == want


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
