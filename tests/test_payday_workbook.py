"""The Payday Super Checker Excel workbook is a second implementation of the checker's rules.

It ships with cached values written by desktop Excel (the package's
`cargo run --example build_workbook`), so this
test can hold it to the engine's answer for the shipped sample without Excel on the
test machine. The engine is a Rust crate; its assessment of that sample is frozen in
tests/data/workbook-sample.json, which `cargo test` holds to the engine. If the engine,
the calendar, the GIC table or the workbook changes on its own, one of the two tests fails.
"""

from __future__ import annotations

import json
import re
import zipfile
from datetime import date, datetime
from decimal import Decimal
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1] / "packages" / "payday-super-checker"
ENGINE_SAMPLE = ROOT / "tests" / "data" / "workbook-sample.json"
# The engine version the shipped workbook was last built against, which the builder
# records from Cargo.toml on Sources & Version. A rebuild after a version bump moves it.
WORKBOOK_ENGINE_VERSION = "1.0.0"
WORKBOOK = ROOT / "workbooks" / "payday-super-checker.xlsx"
AS_AT = date(2026, 8, 10)
SHEETS = ("Start Here", "Register", "Codes", "Holidays", "GIC", "Summary", "Review Checks",
          "Sources & Version")
EXPOSED = ("LATE", "UNPAID")


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


def test_charge_estimate_is_withheld_past_the_gic_table(cached):
    """The engine refuses a day past the GIC table unless --allow-stale-gic is
    passed. The workbook has no opt-in, so every rate-dependent column must go
    blank on such a line and the GIC table must hold published quarters only."""
    with zipfile.ZipFile(WORKBOOK) as archive:
        tables = "".join(archive.read(n).decode("utf-8") for n in archive.namelist()
                         if n.startswith("xl/tables/"))
    for column in ("NEC", "NEC_r", "Uplift_best", "Uplift_worst", "SGC_low", "SGC_high"):
        formula = re.search(rf'name="{column}"[^>]*>\s*<calculatedColumnFormula>([^<]*)<', tables)
        assert formula, column
        assert '[Past_gic_table]]=1,"",' in formula.group(1), column
    ws = cached["GIC"]
    last_row = int(re.search(r"(\d+)$", ws.tables["tblGic"].ref).group(1))
    bases = {r[4] for r in ws.iter_rows(min_row=2, max_row=last_row, max_col=5, values_only=True)}
    assert bases == {"known"}


def test_cached_values_were_calculated_by_desktop_excel(cached):
    sources = cached["Sources & Version"]
    assert sources["B2"].value == WORKBOOK_ENGINE_VERSION
    assert str(sources["B3"].value).startswith("Excel ")
    assert as_date(cached["Summary"]["B2"].value) == AS_AT
    # The transition confirmation ships as N, as the CLI defaults it, so the shipped
    # sample is BLOCKED at that check exactly as the CLI stops on it without the flag.
    assert cached["Summary"]["B4"].value == "N"
    assert cached["Review Checks"]["B16"].value == "BLOCKED"
    assert cached["Review Checks"]["B4"].value == "BLOCKED"
    assert cached["Start Here"]["A11"].value == "BLOCKED"


def test_summary_does_not_total_withheld_estimates_as_zero():
    """A SUM ignores blank row estimates; the summary must retain their uncertainty."""
    openpyxl = pytest.importorskip("openpyxl")
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    summary = openpyxl.load_workbook(WORKBOOK, data_only=False)["Summary"]
    for cell in ("B18", "B19", "B20"):
        formula = summary[cell].value
        assert "tblLines[Past_gic_table]" in formula, cell
        assert '"Not assessed"' in formula, cell


def test_sample_matches_the_engine_line_by_line(cached):
    engine = json.loads(ENGINE_SAMPLE.read_text(encoding="utf-8"))
    assert engine["as_at"] == AS_AT.isoformat()
    assert engine["transition_allocation_confirmed"] is True
    rows = register_rows(cached)
    assert len(rows) == len(engine["rows"])
    for row, result in zip(rows, engine["rows"]):
        employee = result["employee_id"]
        assert row["employee_id"] == employee
        assert row["Verdict"] == result["verdict"], employee
        assert row["Pathway"] == result["pathway"], employee
        due = result["final_due"]
        assert as_date(row["Final_due"]) == (date.fromisoformat(due) if due else None), employee
        assert (row["Unassessable_between"] or "") == result["unassessable_between"], employee
        if result["verdict"] not in EXPOSED:
            assert row["Shortfall_r"] in (None, ""), employee
            continue
        days = None if row["Days_late"] in (None, "") else int(row["Days_late"])
        assert days == result["days_late"], employee
        assert row["Lateness_basis"] == result["lateness_basis"], employee
        for column, key in (("Shortfall_r", "shortfall"), ("NEC_r", "nec"), ("Uplift_best", "up_low"),
                            ("Uplift_worst", "up_high"), ("SGC_low", "low"), ("SGC_high", "high")):
            assert as_decimal(row[column]) == Decimal(result[key]), (employee, column)
    summary = cached["Summary"]
    counts = {summary.cell(row=r, column=1).value: summary.cell(row=r, column=2).value for r in range(10, 16)}
    assert counts == engine["summary"]["counts"]
    assert as_decimal(summary["B16"].value) == engine["summary"]["exposed"]
    assert as_decimal(summary["B19"].value) == Decimal(engine["summary"]["low"])
    assert as_decimal(summary["B20"].value) == Decimal(engine["summary"]["high"])


def test_holiday_and_gic_tables_match_the_shipped_data(cached):
    calendar = json.loads((ROOT / "data" / "business_days.json").read_text(encoding="utf-8"))
    want = sorted(date.fromisoformat(h["date"]) for h in calendar["non_business_days"]
                  if not h["provisional"] and h["date"] >= "2026-01-01")
    ws = cached["Holidays"]
    last_row = int(re.search(r"(\d+)$", ws.tables["tblHolidays"].ref).group(1))
    got = sorted(as_date(r[0]) for r in ws.iter_rows(min_row=2, max_row=last_row, max_col=1, values_only=True))
    assert got == want
    assert as_date(cached["Summary"]["B6"].value) == date.fromisoformat(calendar["verified_until"])
    gic = json.loads((ROOT / "data" / "gic_rates.json").read_text(encoding="utf-8"))
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
