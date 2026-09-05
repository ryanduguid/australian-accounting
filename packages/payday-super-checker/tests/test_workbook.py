"""The Excel workbook is a second implementation of the checker's rules.

It ships with cached values written by desktop Excel (tools/build_workbook.py), so this
test can hold it to the engine's answer for the shipped sample without Excel on the
test machine. If the engine, the calendar, the GIC table or the workbook changes on its
own, this test is what fails.
"""

from __future__ import annotations

import json
import re
import zipfile
from datetime import date, datetime
from decimal import Decimal
from pathlib import Path

import pytest

from paydaysuper import __version__
from paydaysuper.assess import EXPOSED, assess
from paydaysuper.calendar import load_calendar
from paydaysuper.csv_io import DEFAULT_MAPPING, cents, parse_rows
from paydaysuper.rates import load_gic

ROOT = Path(__file__).resolve().parents[1]
WORKBOOK = ROOT / "workbooks" / "payday-super-checker.xlsx"
SAMPLE = ROOT / "examples" / "sample_payrun.csv"
AS_AT = date(2026, 8, 10)
SHEETS = ("Start Here", "Register", "Codes", "Holidays", "GIC", "Summary", "Review Checks",
          "Sources & Version")


@pytest.fixture(scope="module")
def cached():
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    openpyxl = pytest.importorskip("openpyxl")
    return openpyxl.load_workbook(WORKBOOK, data_only=True)


def as_date(value):
    if value in (None, ""):
        return None
    if isinstance(value, datetime):
        return value.date()
    return value


def as_decimal(value):
    return None if value in (None, "") else Decimal(str(value))


def register_rows(book):
    ws = book["Register"]
    titles = [c.value for c in ws[1]]
    return [dict(zip(titles, row)) for row in ws.iter_rows(min_row=2, values_only=True) if row[0]]


def test_workbook_is_macro_free_and_carries_no_build_path(cached):
    with zipfile.ZipFile(WORKBOOK) as archive:
        names = archive.namelist()
        workbook_xml = archive.read("xl/workbook.xml")
        tables = "".join(archive.read(n).decode("utf-8") for n in names if n.startswith("xl/tables/"))
    assert not [n for n in names if "vba" in n.lower() or n.endswith(".bin")]
    assert b"absPath" not in workbook_xml
    assert tuple(cached.sheetnames) == SHEETS
    for column in ("Own_due", "Final_due", "Verdict", "NEC", "SGC_high", "Guard", "Row_problem"):
        assert re.search(rf'name="{column}"[^>]*>\s*<calculatedColumnFormula>', tables), column


def test_cached_values_were_calculated_by_desktop_excel(cached):
    sources = cached["Sources & Version"]
    assert sources["B2"].value == __version__
    assert str(sources["B3"].value).startswith("Excel ")
    assert as_date(cached["Summary"]["B2"].value) == AS_AT
    # The transition confirmation ships as N, as the CLI defaults it, so the shipped
    # sample is BLOCKED at that check exactly as the CLI stops on it without the flag.
    assert cached["Summary"]["B4"].value == "N"
    assert cached["Review Checks"]["B16"].value == "BLOCKED"
    assert cached["Review Checks"]["B4"].value == "BLOCKED"
    assert cached["Start Here"]["A11"].value == "BLOCKED"


def test_sample_matches_the_engine_line_by_line(cached):
    lines = parse_rows(SAMPLE, dict(DEFAULT_MAPPING))
    results = assess(lines, load_calendar(), load_gic(), AS_AT, transition_allocation_confirmed=True)
    rows = register_rows(cached)
    assert len(rows) == len(results)
    for row, result in zip(rows, results):
        loan = result.line.employee_id
        assert row["employee_id"] == loan
        assert row["Verdict"] == result.verdict, loan
        assert row["Pathway"] == result.deadline.pathway, loan
        assert as_date(row["Final_due"]) == result.deadline.due, loan
        unassessable = " or ".join(result.horizon_verdicts) if result.horizon_verdicts else ""
        assert (row["Unassessable_between"] or "") == unassessable, loan
        if result.verdict not in EXPOSED:
            assert row["Shortfall_r"] in (None, ""), loan
            continue
        days = None if row["Days_late"] in (None, "") else int(row["Days_late"])
        assert days == result.days_late, loan
        assert row["Lateness_basis"] == result.lateness_basis, loan
        shortfall = cents(result.final_shortfall)
        nec = cents(result.nec)
        up_low = cents(result.uplift["clean_history"]["vds_within_30d"])
        up_high = cents(result.uplift["prior_history"]["no_vds"])
        assert as_decimal(row["Shortfall_r"]) == shortfall, loan
        assert as_decimal(row["NEC_r"]) == nec, loan
        assert as_decimal(row["Uplift_best"]) == up_low, loan
        assert as_decimal(row["Uplift_worst"]) == up_high, loan
        assert as_decimal(row["SGC_low"]) == shortfall + nec + up_low, loan
        assert as_decimal(row["SGC_high"]) == shortfall + nec + up_high, loan
    summary = cached["Summary"]
    counts = {summary.cell(row=r, column=1).value: summary.cell(row=r, column=2).value for r in range(10, 16)}
    for verdict, count in counts.items():
        assert count == sum(1 for r in results if r.verdict == verdict), verdict
    exposed = [r for r in results if r.verdict in EXPOSED]
    assert as_decimal(summary["B16"].value) == len(exposed)
    assert as_decimal(summary["B19"].value) == sum(
        cents(r.final_shortfall) + cents(r.nec) + cents(r.uplift["clean_history"]["vds_within_30d"])
        for r in exposed)
    assert as_decimal(summary["B20"].value) == sum(
        cents(r.final_shortfall) + cents(r.nec) + cents(r.uplift["prior_history"]["no_vds"])
        for r in exposed)


def test_holiday_and_gic_tables_match_the_shipped_data(cached):
    calendar = json.loads((ROOT / "paydaysuper" / "data" / "business_days.json").read_text(encoding="utf-8"))
    want = sorted(date.fromisoformat(h["date"]) for h in calendar["non_business_days"]
                  if not h["provisional"] and h["date"] >= "2026-01-01")
    ws = cached["Holidays"]
    last_row = int(re.search(r"(\d+)$", ws.tables["tblHolidays"].ref).group(1))
    got = sorted(as_date(r[0]) for r in ws.iter_rows(min_row=2, max_row=last_row, max_col=1, values_only=True))
    assert got == want
    assert as_date(cached["Summary"]["B6"].value) == date.fromisoformat(calendar["verified_until"])
    gic = json.loads((ROOT / "paydaysuper" / "data" / "gic_rates.json").read_text(encoding="utf-8"))
    known = [(date.fromisoformat(q["from"]), date.fromisoformat(q["to"]), Decimal(q["annual_pct"]))
             for q in gic["quarters"]]
    rows = [r for r in cached["GIC"].iter_rows(min_row=2, max_col=5, values_only=True) if r[4] == "known"]
    assert [(as_date(a), as_date(b), as_decimal(c)) for a, b, c, _, _ in rows] == known


def test_input_columns_are_formatted_so_pasted_values_stay_inert():
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    openpyxl = pytest.importorskip("openpyxl")
    book = openpyxl.load_workbook(WORKBOOK)
    ws = book["Register"]
    titles = [c.value for c in ws[1]]
    assert ws.cell(row=2, column=titles.index("employee_id") + 1).number_format == "@"
    # Excel re-saves the format with escaped hyphens (yyyy\-mm\-dd); both mean a date cell.
    date_format = ws.cell(row=2, column=titles.index("payment_date") + 1).number_format
    assert date_format.replace("\\", "") == "yyyy-mm-dd"
    assert "ISFORMULA(" in ws.cell(row=2, column=titles.index("Guard") + 1).value
    assert ws.protection.sheet is False, "pasting extends the table"
    assert book["Summary"].protection.sheet is True
    assert book["Review Checks"].protection.sheet is True
