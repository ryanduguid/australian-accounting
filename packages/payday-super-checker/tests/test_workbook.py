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
from datetime import date, datetime, timedelta
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


def test_charge_estimate_is_withheld_past_the_gic_table(cached):
    """Engine 0.1.6 refuses a day past the GIC table unless --allow-stale-gic is
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


# ---------------------------------------------------------------------------
# Generator-level checks. Excel is not run here, so these hold the formulas the
# builder writes; the shipped cached values still come from a desktop Excel pass
# (tools/build_workbook.py).
# ---------------------------------------------------------------------------


@pytest.fixture(scope="module")
def builder():
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "payday_build_workbook", ROOT / "tools" / "build_workbook.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_every_register_date_is_read_as_a_whole_calendar_day(builder):
    """The checker drops a zone-less time component; noon on the deadline is not late."""
    calc = builder.formulas()
    for column, source in (("Pay_day", "payment_date"), ("Remit_day", "remitted_date"),
                           ("Recv_day", "fund_received_date"),
                           ("Next_day", "next_standard_payday")):
        assert calc[column] == builder.whole_day(source)
        assert "INT(" in calc[column]
    # Nothing downstream may reach past the whole-day columns to the raw date cells.
    downstream = {name: formula for name, formula in calc.items()
                  if name not in {"Pay_day", "Remit_day", "Recv_day", "Next_day",
                                  "Row_problem", "Guard"}}
    for name, formula in downstream.items():
        for source in ("payment_date", "remitted_date", "fund_received_date",
                       "next_standard_payday"):
            assert f"[{source}]]" not in formula, (name, source)
    assert builder.AS_AT != builder.AS_AT_INPUT
    assert builder.ASSESS != builder.ASSESS_INPUT


def test_a_blank_register_date_stays_blank_through_the_whole_day_columns(builder):
    """A blank date cell read as a number is 30 December 1899, not a missing date.

    Excel gives a bare reference to an empty cell the value 0, so a whole-day
    column that returned the raw reference made every blank date look supplied:
    ISNUMBER saw a number, Settled and Remit accepted day 0 as on or before the
    as-at date, and the shipped sample's unremitted EMP004 turned from UNPAID
    into LATE.
    """
    for source in ("payment_date", "remitted_date", "fund_received_date",
                   "next_standard_payday"):
        formula = builder.whole_day(source)
        assert formula.startswith(f'=IF({builder.T(source)}="","",')
    # The control: a supplied date is still truncated to its calendar day.
    assert "INT(" in builder.whole_day("payment_date")


def test_the_register_refuses_the_dates_and_amounts_the_checker_refuses(builder):
    from paydaysuper.csv_io import LATEST_SANE_YEAR

    problem = builder.formulas()["Row_problem"]
    assert f"DATE({LATEST_SANE_YEAR},12,31)" in problem
    # The Summary coverage date is bound the same way, so a fabricated far-future
    # coverage cannot mark deadlines as covered.
    source = (ROOT / "tools" / "build_workbook.py").read_text(encoding="utf-8")
    assert "AND(ISNUMBER({COVERAGE}),{COVERAGE}<={FAR_DATE})" in source
    assert "date-not-real" in problem
    assert "amount-under-half-a-cent" in problem
    for column in builder.AMOUNT_COLUMNS:
        assert f"ROUND(tblLines[[#This Row],[{column}]],2)=0" in problem


def test_amount_invariants_compare_cents_not_raw_input(builder):
    """1000.001 against 1000.004 is one figure to the checker, so it is here too."""
    calc = builder.formulas()
    problem = calc["Row_problem"]
    for pair in ("remitted_amount>sg", "matched_amount>sg", "remitted_amount>matched"):
        assert pair in problem
    assert "ROUND(tblLines[[#This Row],[remitted_amount]],2)>ROUND(" in problem
    assert "ROUND(tblLines[[#This Row],[matched_amount]],2)>ROUND(" in problem
    # The receipt-covers-liability test is the one that read a fully received
    # 1000.004 payday as UNPAID.
    assert ("tblLines[[#This Row],[Receipt_credit]]>=ROUND(tblLines[[#This Row],[sg_amount]],2)"
            in calc["Branch"])


def test_the_gic_table_holds_only_published_contiguous_quarters(builder):
    """No estimated rows: a day past the table withholds the estimate instead of
    reading a carried-forward rate, and the accrual end stays uncapped."""
    rows, last_known = builder.read_gic()
    assert rows and all(row[4] == "known" for row in rows)
    assert last_known == rows[-1][1]
    for earlier, later in zip(rows, rows[1:]):
        assert later[0] == earlier[1] + timedelta(days=1)
    assert not hasattr(builder, "ESTIMATE_UNTIL")
    assert "Past_gic_table" in builder.formulas()["NEC"]

def test_a_bare_receipt_date_is_left_unknown_in_the_workbook(builder):
    """A fund-receipt date with neither amount column filled evidences timing
    only. The branch formula sends such a row to the U codes before any
    receipt branch can read the whole-liability cap as a full receipt, the
    covers test itself needs a stated amount, and the s 18D offset waits for
    one. Verified through desktop Excel on 20 Sep 2026: bare timely receipt
    U2 (UNPAID or ON_TIME), bare late receipt E with the not-evidenced
    lateness basis and no offset, stated full receipt D1."""
    calc = builder.formulas()
    evidenced = (
        "OR(ISNUMBER(tblLines[[#This Row],[matched_amount]]),"
        "ISNUMBER(tblLines[[#This Row],[remitted_amount]]))"
    )
    assert calc["Branch"].index(f"IF(AND(NOT({evidenced})") < calc["Branch"].index('"A1"')
    # A pre-payment is timely only inside the 12-month window, so a stale one
    # must fall through to the S codes rather than be offered ON_TIME: the
    # window test has to be nested under the pre-payment test, not OR-ed
    # beside the on-or-before-deadline test (which every pre-payment passes).
    settled = "tblLines[[#This Row],[Settled]]"
    pay = "tblLines[[#This Row],[Pay_day]]"
    assert (
        f"IF({settled}<{pay},{settled}>=tblLines[[#This Row],[Earliest_prepay]],OR("
        in calc["Branch"]
    )
    assert '"U1"' in calc["Branch"] and '"U2"' in calc["Branch"] and '"U3"' in calc["Branch"]
    assert f"AND(tblLines[[#This Row],[Receipt_credit]]>=ROUND(tblLines[[#This Row],[sg_amount]],2),{evidenced})" in calc["Branch"]
    assert evidenced in calc["Offset_s18D"]
    assert "as-at date (fund receipt amount not evidenced)" in calc["Lateness_basis"]
    for code, outer in (("U1", "LATE or ON_TIME"), ("U2", "UNPAID or ON_TIME"), ("U3", "NOT_YET_DUE or ON_TIME")):
        assert builder.CODES[code] == ("UNKNOWN", outer, 0, 0, 0)
