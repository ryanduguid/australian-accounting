"""The ato-benchmark-compare Excel workbook is a second implementation of the engine's rules.

It ships with cached values written by desktop Excel (the Rust `build_workbook` example,
`cargo run --example build_workbook`), so this test can hold it to the engine's answer
for the bakery example without Excel on the test machine. The engine is a Rust crate;
its comparison of that example is frozen in
tests/data/workbook-sample.json, which `cargo test` holds to the engine. If the engine,
the benchmark data or the workbook changes on its own, one of the two tests fails.
"""

from __future__ import annotations

import json
import re
import zipfile
from decimal import Decimal
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1] / "packages" / "ato-benchmark-compare"
ENGINE_COMPARISON = ROOT / "tests" / "data" / "workbook-sample.json"
# The crate version the shipped workbook was built and recalculated against; a
# desktop Excel rebuild records the current version, and this constant moves with it.
WORKBOOK_ENGINE_VERSION = "1.0.0"
WORKBOOK = ROOT / "workbooks" / "ato-benchmark-compare.xlsx"
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
    return json.loads(ENGINE_COMPARISON.read_text(encoding="utf-8"))


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
    assert sources["B2"].value == WORKBOOK_ENGINE_VERSION
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
    assert Decimal(str(results["B8"].value)) == Decimal(comparison["turnover"])
    assert results["B9"].value == comparison["turnover_band"]["label"]
    for row, entry in enumerate(comparison["ratios"], 2):
        key = entry["ratio"]
        assert results.cell(row=row, column=1).value == key
        actual = Decimal(str(results.cell(row=row, column=3).value)).quantize(Decimal("0.0001"))
        assert actual == Decimal(entry["value"]), key
        assert results.cell(row=row, column=6).value == entry["status"], key
        assert (results.cell(row=row, column=7).value == "key") == entry["is_key_ratio"], key
        if entry["benchmark_min"] is not None:
            assert Decimal(str(results.cell(row=row, column=4).value)) == Decimal(entry["benchmark_min"])
            assert Decimal(str(results.cell(row=row, column=5).value)) == Decimal(entry["benchmark_max"])
        # The Range source column is the engine's benchmark_source, row for row.
        assert (results.cell(row=row, column=8).value or None) == entry["benchmark_source"], key


def test_benchmark_table_matches_the_shipped_data(cached):
    """Read the JSON directly, not through the engine, so a merge fault in either
    implementation shows up here as a disagreement."""
    page_keys = (
        "cost_of_sales_to_turnover", "labour_to_turnover", "rent_to_turnover",
        "motor_vehicle_to_turnover",
    )

    def pair(value):
        if value is None:
            return None, None
        return Decimal(value["min"]), Decimal(value["max"])

    expected = []
    for path in sorted((ROOT / "data").glob("benchmarks-*.json")):
        data = json.loads(path.read_text(encoding="utf-8"))
        pages_path = path.with_name(f"other-{path.name}")
        pages = {}
        if pages_path.is_file():
            for entry in json.loads(pages_path.read_text(encoding="utf-8"))["business_types"]:
                pages[entry["name"]] = entry
        for business_type in data["business_types"]:
            page = pages.get(business_type["name"])
            for position, band in enumerate(business_type["turnover_bands"]):
                page_band = page["turnover_bands"][position] if page else {}
                from_page = [key for key in page_keys if page_band.get(key) is not None]
                cos = band["cost_of_sales_to_turnover"] or page_band.get("cost_of_sales_to_turnover")
                expected.append(
                    (
                        data["benchmark_year"], business_type["name"], business_type["key_ratio"],
                        band["band"], Decimal(band["turnover_from"]),
                        None if band["turnover_to"] is None else Decimal(band["turnover_to"]),
                        *pair(cos),
                        *pair(band["total_expenses_to_turnover"]),
                        data["source"]["sha256"],
                        *pair(page_band.get("labour_to_turnover")),
                        *pair(page_band.get("rent_to_turnover")),
                        *pair(page_band.get("motor_vehicle_to_turnover")),
                        " ".join(from_page) or None,
                        page["text_sha256"] if from_page else None,
                    )
                )
    rows = list(cached["Benchmarks"].iter_rows(min_row=2, max_col=22, values_only=True))
    rows = [r for r in rows if r[0] is not None]
    assert len(rows) == len(expected)

    def decimal(value):
        return None if value is None else Decimal(str(value))

    for row, want in zip(rows, expected):
        got = (row[0], row[1], row[2], row[3], decimal(row[5]), decimal(row[7]), decimal(row[8]),
               decimal(row[9]), decimal(row[10]), decimal(row[11]), row[13],
               *(decimal(value) for value in row[14:20]), row[20] or None, row[21])
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
