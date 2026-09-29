"""The industry-page ranges: how the builder reads a page and how the loader merges it."""

from __future__ import annotations

import functools
import importlib.util
import json
from pathlib import Path

import pytest
from atobenchmark import dataset as ds

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "atobenchmark" / "data"


@functools.cache
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


# ---------------------------------------------------------------------------
# The builder end to end, on a fabricated dataset and fabricated pages.

FAB_DATASET = {
    "schema_version": 1,
    "benchmark_year": "2023-24",
    "business_type_count": 1,
    "source": {"sha256": "ab" * 32},
    "business_types": [
        {
            "name": "Fabricated services",
            "key_ratio": "total_expenses_to_turnover",
            "turnover_bands": [
                {
                    "band": "low", "label": "$50,000 - $150,000", "turnover_from": "50000",
                    "turnover_from_inclusive": True, "turnover_to": "150000",
                    "total_expenses_to_turnover": {"min": "0.54", "max": "0.73"},
                    "cost_of_sales_to_turnover": None,
                },
                {
                    "band": "high", "label": "More than $150,000", "turnover_from": "150000",
                    "turnover_from_inclusive": False, "turnover_to": None,
                    "total_expenses_to_turnover": {"min": "0.79", "max": "0.89"},
                    "cost_of_sales_to_turnover": None,
                },
            ],
        }
    ],
}
ADDRESS = (
    "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/"
    "small-business-benchmarks/in-detail/fabricated-services"
)


def _build(tmp_path: Path, page: str, expect: frozenset[str] = frozenset()):
    builder = _builder()
    pages = tmp_path / "pages"
    pages.mkdir()
    (pages / "001.txt").write_text(page, encoding="utf-8")
    (pages / "index.json").write_text(
        json.dumps(
            [{"url": ADDRESS, "final_url": ADDRESS, "file": "001.txt", "status": 200,
              "truncated": False, "title": "Fabricated services | Australian Taxation Office"}]
        ),
        encoding="utf-8",
    )
    dataset = tmp_path / "benchmarks-2023-24.json"
    dataset.write_text(json.dumps(FAB_DATASET), encoding="utf-8")
    return builder, builder.build(pages, dataset, "2026-09-29", expect)


def test_the_builder_includes_a_page_that_agrees(tmp_path: Path) -> None:
    _, built = _build(tmp_path, PAGE)
    assert built["excluded_pages"] == []
    (entry,) = built["business_types"]
    assert entry["page_reference"] == "QC12345"
    assert entry["turnover_bands"][1]["rent_to_turnover"] == {"min": "0.01", "max": "0.01"}
    assert built["source"]["dataset_json_sha256"] == ds.canonical_digest(json.dumps(FAB_DATASET))


@pytest.mark.parametrize(
    ("edit", "reason"),
    [
        (lambda p: p.replace("54%\u00a0to\u00a073%", "55%\u00a0to\u00a073%"), "in the dataset"),
        (lambda p: p.replace("'Total expenses' divided 'Annual turnover'",
                             "'Cost of sales' divided by 'Annual turnover'"),
         "does not print the dataset's total_expenses_to_turnover"),
        (lambda p: p.replace("Other benchmarks for 2023\u201324", "Other benchmarks for 2022\u201323"),
         "other benchmarks are for 2022-23"),
        (lambda p: p.replace("the 2023\u201324 financial year", "the 2022\u201323 financial year"),
         "the page covers 2022-23"),
    ],
)
def test_the_builder_leaves_out_a_page_that_disagrees(tmp_path: Path, edit, reason) -> None:
    builder, built = _build(
        tmp_path, edit(PAGE), expect=frozenset({"Fabricated services"})
    )
    assert built["business_types"] == []
    (excluded,) = built["excluded_pages"]
    assert reason in excluded["reason"]


def test_an_unexpected_exclusion_stops_the_build(tmp_path: Path) -> None:
    builder = _builder()
    with pytest.raises(builder.BuildError, match="differ from --expect-excluded"):
        _build(tmp_path, PAGE.replace("54%\u00a0to\u00a073%", "55%\u00a0to\u00a073%"))


def test_a_mistyped_key_heading_is_recorded_when_its_ranges_agree(tmp_path: Path) -> None:
    _, built = _build(tmp_path, PAGE.replace("Key benchmarks for 2023\u201324", "Key benchmarks for 2023\u201323"))
    (entry,) = built["business_types"]
    assert entry["page_anomalies"] == [
        "key table heading says 2023-23; the page and its ranges are 2023-24"
    ]


def test_the_builder_reads_a_second_key_range_after_its_averages() -> None:
    page = PAGE.replace(
        "Average total expenses\n63%\n",
        "Average total expenses\n63%\n70%\n'Cost of sales' divided by 'Annual turnover'\n"
        "10% to 20%\n15% to 25%\n",
    )
    key = _builder().parse_page(page, "fabricated")["sections"]["key"]["ranges"]
    assert key["cost_of_sales_to_turnover"] == [("0.1", "0.2"), ("0.15", "0.25")]


def test_a_page_without_its_reference_is_refused() -> None:
    builder = _builder()
    with pytest.raises(builder.BuildError, match="QC reference"):
        builder.parse_page(PAGE.replace("QC12345", ""), "fabricated")


# ---------------------------------------------------------------------------
# The loader and the comparison.


def test_the_shipped_page_file_counts() -> None:
    pages = json.loads((DATA / "other-benchmarks-2023-24.json").read_text(encoding="utf-8"))
    ranges = [
        value
        for entry in pages["business_types"]
        for band in entry["turnover_bands"]
        for key, value in band.items()
        if key != "band" and value is not None
    ]
    assert len(pages["business_types"]) == 99
    assert len(ranges) == 708
    assert sum(1 for value in ranges if value["min"] == value["max"]) == 35


def test_the_loader_refuses_an_unknown_ratio_key() -> None:
    workbook, pages = _year()
    pages["business_types"][0]["turnover_bands"][0]["wages_to_turnover"] = None
    with pytest.raises(ds.DatasetError, match="unknown ratio key"):
        ds.merge_page_ranges(workbook, json.dumps(pages))


def test_the_page_file_binds_to_the_dataset_file_it_was_checked_against(tmp_path: Path) -> None:
    for name in ("benchmarks-2023-24.json", "other-benchmarks-2023-24.json"):
        (tmp_path / name).write_bytes((DATA / name).read_bytes())
    data = json.loads((tmp_path / "benchmarks-2023-24.json").read_text(encoding="utf-8"))
    data["business_types"][0]["turnover_bands"][0]["label"] += " "
    (tmp_path / "benchmarks-2023-24.json").write_text(json.dumps(data), encoding="utf-8")
    with pytest.raises(ds.DatasetError, match="different 2023-24 dataset file"):
        ds.load("2023-24", data_dir=tmp_path)


def test_a_year_without_a_page_file_loads_as_before(tmp_path: Path) -> None:
    (tmp_path / "benchmarks-2023-24.json").write_bytes((DATA / "benchmarks-2023-24.json").read_bytes())
    data = ds.load("2023-24", data_dir=tmp_path)
    assert data.page_source is None
    assert all(not band.page_ratios for bt in data.business_types for band in bt.bands)
    assert all(bt.page is None for bt in data.business_types)


def _bakery(motor_vehicle: str):
    from decimal import Decimal

    from atobenchmark.ratios import compute
    from atobenchmark.report import compare

    data = ds.load("2023-24")
    bakery = data.get("Bakeries and hot bread shops")
    amounts = {"turnover": "1000000", "other_income": "0", "cost_of_sales": "320000",
               "motor_vehicle": motor_vehicle}
    figures = compute({name: Decimal(value) for name, value in amounts.items()})
    return compare(data, bakery, figures, supplied_fields=set(amounts))


@pytest.mark.parametrize(
    ("motor_vehicle", "status"), [("9900", "below"), ("10000", "within"), ("10100", "above")]
)
def test_a_single_figure_range_is_compared_as_printed(motor_vehicle, status) -> None:
    comparison = _bakery(motor_vehicle)
    verdict = {v.key: v for v in comparison.verdicts}["motor_vehicle_to_turnover"]
    assert verdict.status == status
    assert verdict.benchmark_source == "ato_industry_page"
    assert comparison.outside_key_range is False
    assert any("single figure" in note for note in comparison.notes)


def test_the_payload_marks_page_ranges_and_names_the_page() -> None:
    from atobenchmark.report import to_dict

    payload = to_dict(_bakery("10000"))
    sources = {row["ratio"]: row["benchmark_source"] for row in payload["ratios"]}
    assert sources["cost_of_sales_to_turnover"] == "ato_dataset"
    assert sources["motor_vehicle_to_turnover"] == "ato_industry_page"
    # The raw payload prints the range beside a withheld status (LIMITATIONS
    # ABC-1), so its source travels with it.
    assert sources["labour_to_turnover"] == "ato_industry_page"
    page = payload["industry_page_source"]
    assert page["page_reference"] == "QC43659"
    assert "motor_vehicle_to_turnover" in page["ratios"]


def test_an_excluded_page_leaves_its_industry_as_before() -> None:
    from decimal import Decimal

    from atobenchmark.ratios import compute
    from atobenchmark.report import compare, to_dict

    data = ds.load("2023-24")
    hardware = data.get("Hardware and building supplies retailing")
    assert hardware.page is None
    figures = compute({"turnover": Decimal("500000"), "other_income": Decimal("0"),
                       "rent": Decimal("40000")})
    payload = to_dict(compare(data, hardware, figures, supplied_fields={"turnover", "other_income", "rent"}))
    rent = {row["ratio"]: row for row in payload["ratios"]}["rent_to_turnover"]
    assert rent["status"] == "no benchmark in this dataset"
    assert payload["industry_page_source"] is None


def test_show_lists_the_page_ranges_and_their_page(capsys: pytest.CaptureFixture[str]) -> None:
    from atobenchmark.cli import EXIT_OK, main

    assert main(["show", "bakeries"]) == EXIT_OK
    out = capsys.readouterr().out
    assert "Labour to turnover" in out
    assert "14% to 25% (ATO industry page, a guide only)" in out
    assert "Industry page: https://www.ato.gov.au/" in out


def test_an_unrecognised_heading_above_ranges_stops_the_build() -> None:
    builder = _builder()
    with pytest.raises(builder.BuildError, match="unrecognised benchmark heading"):
        builder.parse_page(PAGE.replace("Rent/turnover", "Rent as a share of turnover"), "fabricated")


@pytest.mark.parametrize(
    ("change", "message"),
    [
        (lambda p: p.update(business_type_count=p["business_type_count"] + 1), "business_type_count"),
        (lambda p: (p["business_types"].pop(), p.update(business_type_count=98)), "do not account for"),
        (lambda p: p["excluded_pages"].append({"name": p["business_types"][0]["name"]}), "do not account for"),
        (lambda p: p.update(excluded_pages="none"), "excluded_pages"),
    ],
)
def test_the_loader_refuses_a_page_file_that_does_not_account_for_every_industry(change, message) -> None:
    workbook, pages = _year()
    change(pages)
    with pytest.raises(ds.DatasetError, match=message):
        ds.merge_page_ranges(workbook, json.dumps(pages))
