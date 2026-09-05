"""The Excel workbook is a second implementation of the engine's rules.

It ships with cached values written by desktop Excel (tools/build_workbook.py), so this
test can hold it to the engine's answer for the bakery example without Excel on the
test machine. If the engine, the benchmark data or the workbook changes on its own,
this test is what fails.
"""

from __future__ import annotations

import json
import re
import zipfile
from decimal import Decimal
from pathlib import Path

import pytest

from atobenchmark import __version__, dataset, mapping, pnl
from atobenchmark.ratios import compute
from atobenchmark.report import compare

ROOT = Path(__file__).resolve().parents[1]
WORKBOOK = ROOT / "workbooks" / "ato-benchmark-compare.xlsx"
EXAMPLES = ROOT / "examples"
SHEETS = (
    "Start Here", "P&L Import", "Mapping", "Benchmarks", "Calculation", "Results",
    "Review Checks", "Sources & Version",
)


@pytest.fixture(scope="module")
def cached():
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    openpyxl = pytest.importorskip("openpyxl")
    return openpyxl.load_workbook(WORKBOOK, data_only=True)


def bakery_comparison():
    source = pnl.read(EXAMPLES / "bakery-pnl.csv")
    reviewed = mapping.read_mapping(EXAMPLES / "bakery-mapping.csv")
    figures = compute(mapping.route(source.rows, reviewed, False).totals)
    data = dataset.load("2023-24")
    return compare(data, data.get("Bakeries and hot bread shops"), figures)


def test_workbook_is_macro_free_and_carries_no_build_path(cached):
    with zipfile.ZipFile(WORKBOOK) as archive:
        names = archive.namelist()
        workbook_xml = archive.read("xl/workbook.xml")
    assert not [n for n in names if "vba" in n.lower() or n.endswith(".bin")]
    assert b"absPath" not in workbook_xml
    assert tuple(cached.sheetnames) == SHEETS


def test_table_formulas_fill_down_when_rows_are_pasted(cached):
    """Bucket, Guard and Key must be calculated columns, or rows pasted below the example
    get no formula and the result is BLOCKED with no visible reason."""
    with zipfile.ZipFile(WORKBOOK) as archive:
        tables = "".join(
            archive.read(n).decode("utf-8") for n in archive.namelist() if n.startswith("xl/tables/")
        )
    for column in ("Bucket", "Guard", "Key"):
        assert re.search(rf'name="{column}"[^>]*>\s*<calculatedColumnFormula>', tables), column


def test_cached_values_were_calculated_by_desktop_excel(cached):
    sources = cached["Sources & Version"]
    assert sources["B2"].value == __version__
    assert str(sources["B3"].value).startswith("Excel ")
    # The shipped bakery lines are fabricated, so the guard holds the sample at REVIEW
    # until they are overwritten; every other check on the sample is PASS.
    assert cached["Review Checks"]["B12"].value == "REVIEW"
    assert [cached["Review Checks"].cell(row=r, column=2).value for r in range(2, 12)] == ["PASS"] * 10
    assert cached["Review Checks"]["B13"].value == "REVIEW"
    assert cached["Start Here"]["A11"].value == "REVIEW"


def test_bakery_results_match_the_engine(cached):
    comparison = bakery_comparison()
    results = cached["Results"]
    assert Decimal(str(results["B8"].value)) == comparison.figures.turnover
    assert results["B9"].value == comparison.band.label
    for row, verdict in enumerate(comparison.verdicts, 2):
        assert results.cell(row=row, column=1).value == verdict.key
        actual = Decimal(str(results.cell(row=row, column=3).value)).quantize(Decimal("0.0001"))
        assert actual == verdict.ratio, verdict.key
        assert results.cell(row=row, column=6).value == verdict.status, verdict.key
        assert (results.cell(row=row, column=7).value == "key") == verdict.is_key, verdict.key
        if verdict.benchmark is not None:
            assert Decimal(str(results.cell(row=row, column=4).value)) == verdict.benchmark.minimum
            assert Decimal(str(results.cell(row=row, column=5).value)) == verdict.benchmark.maximum


def test_benchmark_table_matches_the_shipped_data(cached):
    expected = []
    for path in sorted((ROOT / "atobenchmark" / "data").glob("benchmarks-*.json")):
        data = json.loads(path.read_text(encoding="utf-8"))
        for business_type in data["business_types"]:
            for band in business_type["turnover_bands"]:
                cos = band["cost_of_sales_to_turnover"]
                total = band["total_expenses_to_turnover"]
                expected.append(
                    (
                        data["benchmark_year"], business_type["name"], business_type["key_ratio"],
                        band["band"], Decimal(band["turnover_from"]),
                        None if band["turnover_to"] is None else Decimal(band["turnover_to"]),
                        None if cos is None else Decimal(cos["min"]),
                        None if cos is None else Decimal(cos["max"]),
                        None if total is None else Decimal(total["min"]),
                        None if total is None else Decimal(total["max"]),
                        data["source"]["sha256"],
                    )
                )
    rows = list(cached["Benchmarks"].iter_rows(min_row=2, max_col=14, values_only=True))
    rows = [r for r in rows if r[0] is not None]
    assert len(rows) == len(expected)

    def decimal(value):
        return None if value is None else Decimal(str(value))

    for row, want in zip(rows, expected):
        got = (row[0], row[1], row[2], row[3], decimal(row[5]), decimal(row[7]), decimal(row[8]),
               decimal(row[9]), decimal(row[10]), decimal(row[11]), row[13])
        assert got == want


def test_account_columns_are_text_so_pasted_formulas_stay_inert():
    if not WORKBOOK.is_file():
        pytest.skip("workbook is not included in the source distribution")
    openpyxl = pytest.importorskip("openpyxl")
    book = openpyxl.load_workbook(WORKBOOK)
    assert book["P&L Import"]["A2"].number_format == "@"
    assert book["Mapping"]["A2"].number_format == "@"
    guard = book["P&L Import"]["D2"].value
    assert re.search(r"ISFORMULA\(", guard)
    assert book["P&L Import"].protection.sheet is False, "pasting extends the table"
    assert book["Calculation"].protection.sheet is True
