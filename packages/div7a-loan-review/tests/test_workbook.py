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
    assert cached["Review Checks"]["B9"].value == "REVIEW", "shipped sample loans are flagged"
    assert cached["Review Checks"]["C9"].value == 8
    assert cached["Review Checks"]["B10"].value == "REVIEW"
    assert cached["Start Here"]["A11"].value == "REVIEW"
    assert "Interest-floor interpretation unresolved" in cached["Review Checks"]["A13"].value
    assert "COMPLYING does not establish" in cached["Review Checks"]["A13"].value


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


# ---------------------------------------------------------------------------
# Generator-level checks. Excel is not run here, so these hold the formulas the
# builder writes; the shipped cached values still come from a desktop Excel pass
# (tools/build_workbook.py).
# ---------------------------------------------------------------------------


@pytest.fixture(scope="module")
def builder():
    import importlib.util
    import sys

    root = Path(__file__).resolve().parents[1]
    sys.path.insert(0, str(root))
    spec = importlib.util.spec_from_file_location(
        "div7a_build_workbook", root / "tools" / "build_workbook.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_any_nonblank_exclusion_reason_skips_the_row(builder):
    """register._skip_reason skips a nonblank reason; "unknown" is a reason, not a blank."""
    status = builder.formulas()["Status"]
    assert 'TRIM(tblLoans[[#This Row],[out_of_scope_reason]]&"")<>""' in status
    assert "out_of_scope_reason" not in builder.unknown(
        builder.T("year_loan_made"))  # the helper is not what decides scope any more
    assert builder.unknown(builder.T("out_of_scope_reason")) not in status


def test_only_reviewed_rows_are_validated(builder):
    """The engine skips before parsing, so unused facts on a skipped row block nothing."""
    problem = builder.formulas()["Input_problem"]
    assert problem.startswith(
        '=IF(TRIM(tblLoans[[#This Row],[out_of_scope_reason]]&"")<>"","",')
    # The bool and number tests are the ones the engine reaches only after the skip.
    for guard in (builder.bool_bad("written_agreement"),
                  builder.number_bad("payments_applied_during_the_year")):
        head = problem.index(guard)
        assert 'tblLoans[[#This Row],[Status]]<>"REVIEWED",""' in problem[:head]
    # The formula guard stays on every row: a pasted formula is a tamper signal.
    assert "Status" not in builder.formulas()["Guard"]


def test_the_loan_year_label_is_read_before_the_historical_skip(builder):
    """register._skip_reason parses year_loan_made to decide whether the loan is pre-1998.

    An out-of-bounds label such as `1800-01` is exit 1 in the CLI, so the workbook
    cannot suppress it as a silent historical skip. Only a row the operator has
    marked out of scope escapes the label test, because the engine returns on that
    reason before it parses anything.
    """
    problem = builder.formulas()["Input_problem"]
    made = builder.year_bad("year_loan_made")
    tested = builder.year_bad("year_of_income_being_tested")
    assert made in problem
    skip_gate = 'tblLoans[[#This Row],[Status]]<>"REVIEWED",""'
    assert skip_gate not in problem[:problem.index(made)]
    # The control: the other year label is parsed after the skip, so it is gated.
    assert f'AND(tblLoans[[#This Row],[Status]]="REVIEWED",{tested})' in problem


def test_the_year_guard_matches_the_engine_grammar_and_bounds(builder):
    from div7aloan.years import _EARLIEST, _LATEST

    guard = builder.year_bad("year_loan_made")
    assert 'TRIM(tblLoans[[#This Row],[year_loan_made]]&"")' in guard
    assert f">={_EARLIEST}" in guard and f"<={_LATEST}" in guard
    # Every one of the 6 digit positions is tested, so VALUE cannot read 20e2 as 2000.
    for position in (1, 2, 3, 4, 6, 7):
        assert f',{position},1)))' in guard


def test_the_money_guard_carries_the_engine_limits(builder):
    from div7aloan.money import MAX_MONEY_MAGNITUDE

    for column in ("amalgamated_loan_unpaid_at_end_of_previous_year",
                   "payments_applied_during_the_year"):
        guard = builder.number_bad(column)
        assert f">{MAX_MONEY_MAGNITUDE}" in guard
        assert f"ROUND(N({builder.T(column)}),2)<>N({builder.T(column)})" in guard
    # parse_rate and parse_ratio impose neither limit, so neither does the workbook.
    for column in ("interest_rate_for_years_after_year_loan_made",
                   "security_coverage_at_first_made"):
        guard = builder.number_bad(column)
        assert str(MAX_MONEY_MAGNITUDE) not in guard
        assert "ROUND(" not in guard


def test_the_money_guard_survives_a_text_cell(builder):
    """AND and OR evaluate every argument, so the arithmetic must not error.

    ROUND on a cell holding `unknown` returns #VALUE!, and the error propagates
    out through the ISNUMBER guard that was there to catch it: the shipped
    sample's L-104, whose payments column is `unknown`, turned Input_problem and
    the whole 'Every register value can be read' check into #VALUE!.
    """
    for column in ("amalgamated_loan_unpaid_at_end_of_previous_year",
                   "payments_applied_during_the_year"):
        guard = builder.number_bad(column)
        cell = builder.T(column)
        assert f"ROUND({cell}," not in guard
        assert f"ROUND(N({cell}),2)<>N({cell})" in guard
        # The control: the raw cell is still what ISNUMBER and the sign test read.
        assert f"ISNUMBER({cell})" in guard and f"{cell}<0" in guard


def test_scope_and_lookup_read_the_trimmed_year(builder):
    calc = builder.formulas()
    for name in ("Status", "Floor_year"):
        assert 'LEFT(tblLoans[[#This Row],[year_loan_made]]' not in calc[name]
    assert 'TRIM(tblLoans[[#This Row],[year_loan_made]]&"")' in calc["Floor_year"]
    # The repayment verdict compares the same trimmed label. Comparing the raw cell
    # with the trimmed Floor_year made ` 2023-24 ` REFUSED for a benchmark-year
    # mismatch against itself, where the engine trims and returns MYR_MET, exit 0.
    raw = 'tblLoans[[#This Row],[year_loan_made]]&""'
    for name in ("MYR_verdict", "MYR_reason"):
        formula = calc[name]
        assert f"TRIM({raw})" in formula
        # Every reading of the label goes through TRIM, none of them the raw cell.
        at = formula.find(raw)
        while at != -1:
            assert formula[at - 5:at] == "TRIM(", (name, formula[at - 40:at + 40])
            at = formula.find(raw, at + 1)
