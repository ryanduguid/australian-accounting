"""The industry-page ranges: how the builder reads a page and how the loader merges it."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

import pytest
from atobenchmark import dataset as ds

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "atobenchmark" / "data"


def _builder():
    spec = importlib.util.spec_from_file_location(
        "build_other_benchmarks", ROOT / "tools" / "build_other_benchmarks.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# A fabricated page in the layouts the ATO uses: no-break spaces inside ranges,
# curly quotes, a heading without "by", a "Rent/turnover" heading and a single
# figure where both ends of a range round to the same percentage.
PAGE = "\n".join(
    [
        "Performance benchmarks use information reported on tax returns for the "
        "2023\u201324 financial year and are updated each year.",
        "Last updated 16 March 2026",
        "Key benchmarks for 2023\u201324",
        "Annual turnover range",
        "$50,000 \u2013 $150,000",
        "More than $150,000",
        "'Total expenses' divided 'Annual turnover'",
        "54%\u00a0to\u00a073%",
        "79%\u00a0to\u00a089%",
        "Average total expenses",
        "63%",
        "Other benchmarks for 2023\u201324",
        "Annual turnover range",
        "$50,000 \u2013 $150,000",
        "More than $150,000",
        "\u2018Labour\u2019 divided by \u2018Annual turnover\u2019",
        "20% to 30%",
        "25% to 35%",
        "Rent/turnover",
        "0% to 1%",
        "1%",
        "QC12345",
    ]
)


def test_the_builder_reads_every_layout_the_pages_use() -> None:
    page = _builder().parse_page(PAGE, "fabricated")
    assert page["financial_year"] == "2023-24"
    assert page["last_updated"] == "16 March 2026"
    assert page["qc"] == "QC12345"
    key = page["sections"]["key"]
    assert key["ranges"]["total_expenses_to_turnover"] == [("0.54", "0.73"), ("0.79", "0.89")]
    other = page["sections"]["other"]["ranges"]
    assert other["labour_to_turnover"] == [("0.2", "0.3"), ("0.25", "0.35")]
    assert other["rent_to_turnover"] == [("0", "0.01"), ("0.01", "0.01")]


def test_the_builder_refuses_a_range_it_cannot_read() -> None:
    builder = _builder()
    with pytest.raises(builder.BuildError, match="expected a range"):
        builder.parse_page(PAGE.replace("20% to 30%", "20 to 30 per cent"), "fabricated")


def test_the_shipped_page_file_matches_the_shipped_workbook() -> None:
    pages = json.loads((DATA / "other-benchmarks-2023-24.json").read_text(encoding="utf-8"))
    workbook = json.loads((DATA / "benchmarks-2023-24.json").read_text(encoding="utf-8"))
    assert pages["benchmark_year"] == workbook["benchmark_year"]
    assert pages["source"]["dataset_sha256"] == workbook["source"]["sha256"]
    names = {entry["name"] for entry in pages["business_types"]}
    excluded = {entry["name"] for entry in pages["excluded_pages"]}
    assert not names & excluded
    assert names | excluded == {bt["name"] for bt in workbook["business_types"]}
    assert excluded == {"Hardware and building supplies retailing"}


def _year() -> tuple[ds.Dataset, dict]:
    workbook = ds.loads((DATA / "benchmarks-2023-24.json").read_text(encoding="utf-8"))
    pages = json.loads((DATA / "other-benchmarks-2023-24.json").read_text(encoding="utf-8"))
    return workbook, pages


def test_the_loader_merges_page_ranges_without_touching_the_workbook_ranges() -> None:
    loaded = ds.load("2023-24")
    workbook, _ = _year()
    bakery = loaded.get("Bakeries and hot bread shops")
    original = workbook.get("Bakeries and hot bread shops")
    for merged, band in zip(bakery.bands, original.bands):
        for key in ds.RATIO_KEYS:
            assert merged.ratios[key] == band.ratios[key]
        assert "labour_to_turnover" in merged.page_ratios
    assert bakery.page is not None and bakery.page["page_reference"] == "QC43659"
    assert loaded.page_source is not None


@pytest.mark.parametrize(
    ("change", "message"),
    [
        (lambda p: p.update(benchmark_year="2022-23"), "does not match"),
        (lambda p: p["source"].update(dataset_sha256="0" * 64), "different dataset file"),
        (lambda p: p["business_types"][0].update(name="Not an industry"), "not a business type"),
        (lambda p: p["business_types"][0]["turnover_bands"].pop(), "bands do not match"),
    ],
)
def test_the_loader_refuses_a_page_file_that_does_not_belong(change, message) -> None:
    workbook, pages = _year()
    change(pages)
    with pytest.raises(ds.DatasetError, match=message):
        ds.merge_page_ranges(workbook, json.dumps(pages))


def test_the_loader_never_lets_a_page_replace_a_workbook_range() -> None:
    workbook, pages = _year()
    # Cost of sales is the bakery's key ratio, so the workbook publishes it.
    bakery = next(e for e in pages["business_types"] if e["name"].startswith("Bakeries"))
    bakery["turnover_bands"][0]["cost_of_sales_to_turnover"] = {"min": "0.1", "max": "0.2"}
    with pytest.raises(ds.DatasetError, match="replace the workbook"):
        ds.merge_page_ranges(workbook, json.dumps(pages))
