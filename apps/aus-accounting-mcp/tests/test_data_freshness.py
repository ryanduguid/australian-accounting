"""get_data_freshness reports what the installed engines record, and no more."""

from __future__ import annotations

import asyncio
import importlib.metadata
import json
from datetime import date
from decimal import Decimal

import pytest
from atobenchmark.dataset import available_years, load
from austaxcalc import calculations
from div7aloan.rates import load_table
from lodgmentcalendar.due_dates import SOURCE_CHECKED as LODGMENT_SOURCE_CHECKED
from paydaysuper.rates import GicQuarter, GicTable

from aus_accounting_mcp import freshness, resources
from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.server import mcp

# Tools that read no bundled dated data: operator-configured folders, a fabricated
# fixture, a fixed refusal, and this report itself.
NO_BUNDLED_DATA = {
    "search_accounting_library",
    "read_accounting_library",
    "search_tax_legislation",
    "read_tax_legislation_section",
    "define_tax_term",
    "search_tax_rates",
    "search_ato_rulings",
    "read_ato_ruling",
    "generate_synthetic_sbr_fixture",
    "refuse_div7a",
    "get_data_freshness",
}


def _by_name(report: dict) -> dict[str, dict]:
    return {dataset["dataset"]: dataset for dataset in report["datasets"]}


def test_every_tool_that_reads_bundled_data_is_covered() -> None:
    registered = {tool.name for tool in asyncio.run(mcp.list_tools())}
    covered = {tool for dataset in freshness.data_freshness()["datasets"]
               for tool in dataset["tools"]}

    assert covered == set(freshness.DATA_TOOLS)
    # A new tool must either name its datasets here or be listed as reading none.
    assert registered - NO_BUNDLED_DATA == covered


def test_the_report_never_claims_the_data_is_current() -> None:
    report = freshness.data_freshness()

    assert report["currency"] == "NOT_ESTABLISHED"
    assert report["bundled_data"] is True
    assert report["live_lookup"] is False
    assert "uvx" in report["statement"]
    assert "do not establish that any figure is current" in report["statement"]
    # No predicted change date of any kind: only recorded facts are reported.
    text = json.dumps(report).lower()
    for word in ("next_change", "expires", "valid_until", "is_current", "latest_release"):
        assert word not in text


def test_versions_come_from_installed_metadata() -> None:
    report = freshness.data_freshness()

    assert report["server"] == {
        "distribution": "aus-accounting-mcp",
        "version": importlib.metadata.version("aus-accounting-mcp"),
    }
    for dataset in report["datasets"]:
        assert dataset["engine_version"] == importlib.metadata.version(dataset["engine"])


def test_a_cached_older_release_reports_its_own_version_and_no_currency(monkeypatch) -> None:
    # uvx can keep serving a cached release. The report names the version that is
    # answering and still makes no claim that a newer one does not exist.
    real = importlib.metadata.version
    monkeypatch.setattr(
        resources, "version",
        lambda name: "0.5.0" if name == "aus-accounting-mcp" else real(name),
    )
    report = freshness.data_freshness()

    assert report["server"]["version"] == "0.5.0"
    assert report["currency"] == "NOT_ESTABLISHED"
    assert "older release" in report["statement"]


def test_an_uninstalled_distribution_is_unknown_not_guessed(monkeypatch) -> None:
    def absent(distribution: str) -> str:
        raise importlib.metadata.PackageNotFoundError(distribution)

    monkeypatch.setattr(resources, "version", absent)
    report = freshness.data_freshness()

    assert report["server"]["version"] == "UNKNOWN"
    assert {dataset["engine_version"] for dataset in report["datasets"]} == {"UNKNOWN"}


def test_local_metadata_is_read_from_the_installed_engines() -> None:
    datasets = _by_name(freshness.data_freshness())

    for year in available_years():
        source = load(year).source
        benchmark = datasets[f"ato_small_business_benchmarks:{year}"]
        assert benchmark["retrieved"] == [source["retrieved"]]
        assert benchmark["provenance"] == dict(source)
        assert benchmark["effective_periods"][0]["id"] == year

    table = load_table()
    div7a = datasets["div7a_benchmark_rates"]
    assert [period["id"] for period in div7a["effective_periods"]] == [
        table.entries[year].year.label for year in sorted(table.entries)
    ]
    assert div7a["provenance"]["reviewed_until"] == table.reviewed_until.label

    for kind, entry in calculations.worksheet_catalogue().items():
        worksheet = datasets[f"tax_worksheet:{kind}"]
        assert [(p["start"], p["end"]) for p in worksheet["effective_periods"]] == [
            (p["start"], p["end"]) for p in entry["supported_periods"]
        ]
        assert worksheet["source_checked"] == entry["source_checked"]

    assert datasets["lodgment_program"]["source_checked"] == LODGMENT_SOURCE_CHECKED


def test_gic_periods_and_dates_follow_the_shipped_table(monkeypatch) -> None:
    # Fabricated tables make a hardcoded period or date fail this check.
    table = GicTable([GicQuarter(date(2030, 1, 1), date(2030, 3, 31), Decimal("10"))])
    document = {"quarters": [{"from": "2030-01-01", "to": "2030-03-31",
                              "verify_at": "https://example.invalid/gic",
                              "seen": "2030-01-02"}]}
    monkeypatch.setattr(freshness, "load_gic", lambda: table)
    monkeypatch.setattr(freshness, "_engine_document", lambda directory, name: document)
    gic = _by_name(freshness.data_freshness(as_at="2030-03-31"))["payday_gic_rates"]

    assert gic["effective_periods"] == [{"id": "2030-01-01/2030-03-31", "basis": "GIC quarter",
                                         "start": "2030-01-01", "end": "2030-03-31"}]
    assert gic["retrieved"] == ["2030-01-02"]
    assert gic["sources"] == ["https://example.invalid/gic"]
    assert gic["as_at_status"] == "INSIDE_RECORDED_PERIOD"


def test_a_fact_the_engine_does_not_record_is_unknown(monkeypatch) -> None:
    report = freshness.data_freshness(as_at="2026-10-28")
    datasets = _by_name(report)

    # gst-review records its sources and review date but no effective period.
    gst = datasets["gst_review_rules"]
    assert gst["effective_periods"] == "UNKNOWN"
    assert gst["as_at_status"] == "UNKNOWN"
    assert gst["retrieved"] == "UNKNOWN"
    assert gst["source_checked"] != "UNKNOWN"
    # The worksheet catalogue records a review date, not a retrieval date.
    assert datasets["tax_worksheet:payg_withholding"]["retrieved"] == "UNKNOWN"

    # A benchmark source with no retrieval date is reported as UNKNOWN, not filled in.
    class Undated:
        source = {"resource_url": "https://example.invalid/benchmarks.xlsx"}

    monkeypatch.setattr(freshness, "available_years", lambda: ["2030-31"])
    monkeypatch.setattr(freshness, "load", lambda year: Undated())
    benchmark = _by_name(freshness.data_freshness())["ato_small_business_benchmarks:2030-31"]
    assert benchmark["retrieved"] == "UNKNOWN"
    assert benchmark["source_checked"] == "UNKNOWN"


@pytest.mark.parametrize(
    "as_at,expected",
    [
        ("2026-06-30", "OUTSIDE_RECORDED_PERIOD"),
        ("2026-07-01", "INSIDE_RECORDED_PERIOD"),
        ("2027-06-30", "INSIDE_RECORDED_PERIOD"),
        ("2027-07-01", "OUTSIDE_RECORDED_PERIOD"),
    ],
)
def test_as_at_boundaries_of_the_lodgment_program(as_at, expected) -> None:
    report = freshness.data_freshness(as_at=as_at, tool="get_lodgment_due_dates")

    assert report["as_at"] == as_at
    assert [dataset["dataset"] for dataset in report["datasets"]] == ["lodgment_program"]
    assert report["datasets"][0]["as_at_status"] == expected


def test_a_date_in_a_gap_between_recorded_periods_is_outside() -> None:
    periods = [
        {"id": "2024-25", "basis": "income year", "start": "2024-07-01", "end": "2025-06-30"},
        {"id": "2026-27", "basis": "income year", "start": "2026-07-01", "end": "2027-06-30"},
    ]

    assert freshness._as_at_status(periods, date(2025, 12, 1)) == "OUTSIDE_RECORDED_PERIOD"
    assert freshness._as_at_status(periods, date(2026, 7, 1)) == "INSIDE_RECORDED_PERIOD"


def test_without_as_at_no_dataset_carries_a_status() -> None:
    report = freshness.data_freshness()

    assert report["as_at"] is None
    assert all("as_at_status" not in dataset for dataset in report["datasets"])


@pytest.mark.parametrize("arguments", [{"as_at": "2026-02-30"}, {"tool": "refuse_div7a"}])
def test_invalid_arguments_are_refused(arguments) -> None:
    with pytest.raises(InputError):
        freshness.data_freshness(**arguments)


@pytest.mark.parametrize("label", ["2026-28", "2026/27", "abcd-ef", "2026-2027"])
def test_a_label_that_is_not_one_financial_year_is_refused(label) -> None:
    with pytest.raises(ValueError):
        freshness._financial_year(label, "income year")


@pytest.mark.parametrize(
    "value,expected",
    [
        ("2026-08-28", "2026-08-28"),
        # An override appends its citation to the review date; that is not a date.
        ("2026-08-28; override synthetic citation", "UNKNOWN"),
        ("", "UNKNOWN"),
        (None, "UNKNOWN"),
    ],
)
def test_only_an_iso_date_is_reported_as_a_date(value, expected) -> None:
    assert freshness._iso_or_unknown(value) == expected
