"""The ato-benchmark-compare industry-page builder, tools/build_other_benchmarks.py.

The builder is a Python development tool; the Rust engine only reads the page file it
writes. The loader's side of these pages is held to the engine by the crate's parity
corpus.
"""

from __future__ import annotations

import functools
import importlib.util
import json
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1] / "packages" / "ato-benchmark-compare"
DATA = ROOT / "data"


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
    builder, built = _build(tmp_path, PAGE)
    assert built["excluded_pages"] == []
    (entry,) = built["business_types"]
    assert entry["page_reference"] == "QC12345"
    assert entry["turnover_bands"][1]["rent_to_turnover"] == {"min": "0.01", "max": "0.01"}
    assert built["source"]["dataset_json_sha256"] == builder.canonical_digest(json.dumps(FAB_DATASET))


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


def test_an_unrecognised_heading_above_ranges_stops_the_build() -> None:
    builder = _builder()
    with pytest.raises(builder.BuildError, match="unrecognised benchmark heading"):
        builder.parse_page(PAGE.replace("Rent/turnover", "Rent as a share of turnover"), "fabricated")


def test_the_builder_digest_is_the_one_the_engine_checks() -> None:
    """The engine refuses a page file whose dataset_json_sha256 is not its own digest of
    the dataset, so the shipped file's value is the engine's digest."""
    for pages_path in sorted(DATA.glob("other-benchmarks-*.json")):
        pages = json.loads(pages_path.read_text(encoding="utf-8"))
        dataset = (DATA / pages_path.name.removeprefix("other-")).read_text(encoding="utf-8")
        assert _builder().canonical_digest(dataset) == pages["source"]["dataset_json_sha256"]


@pytest.mark.parametrize(
    ("text", "folded"),
    [
        ("Fish and chips shops", "fish and chips shops"),
        ("Bakeries \u2013 and \u2018hot\u2019 bread", "bakeries and hot bread"),
        ("Caf\u00e9s", "cafe\u0301s"),
        ("STRASSE \u00df", "strasse ss"),
        ("  spaced\tout  ", "spaced out"),
    ],
)
def test_the_builder_folds_names_as_the_engine_does(text: str, folded: str) -> None:
    """The engine's normalise gives these results (its parity corpus covers the rest)."""
    assert _builder().normalise(text) == folded
