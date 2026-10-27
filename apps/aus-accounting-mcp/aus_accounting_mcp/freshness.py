"""What each bundled dataset records about its period, version and sources.

Every period, date and URL here is read from the installed engines: their
loaders, their catalogues and the data files they ship. Nothing is looked up,
and nothing is inferred about when a figure will next change. A fact an engine
does not record is reported as `UNKNOWN`.

The installed release can be older than the latest one: `uvx` can keep serving a
cached release, so the server version reported here is the one answering, not
the newest. No field here says that a figure is current.
"""

from __future__ import annotations

import json
from collections.abc import Iterable
from datetime import date
from pathlib import Path
from typing import Any, Literal, get_args

from atobenchmark.dataset import available_years, load
from austaxcalc import calculations
from div7aloan import LAW_CONTENT_DATE as DIV7A_LAW_CONTENT_DATE
from div7aloan.rates import load_table
from gstreview.bas import ATO_BAS_INSTRUCTIONS, ATO_BAS_METHOD
from gstreview.coding import GST_ACT, SOURCE_CHECKED as GST_SOURCE_CHECKED
from lodgmentcalendar.due_dates import (
    ACTIVITY_YEAR,
    PROGRAM_YEAR,
    RETURN_YEAR,
    SOURCE_CHECKED as LODGMENT_SOURCE_CHECKED,
)
from lodgmentcalendar.holidays import COVERS_FROM, COVERS_TO, HOLIDAY_PROVISION
from lodgmentcalendar.rules import SOURCES as LODGMENT_SOURCES
from paydaysuper import LAW_CONTENT_DATE as PAYDAY_LAW_CONTENT_DATE
from paydaysuper.calendar import DATA_DIR as CALENDAR_DATA_DIR, load_calendar
from paydaysuper.rates import DATA_DIR as RATES_DATA_DIR, load_gic, load_rates

from .errors import InputError
from .resources import SERVER_DISTRIBUTION, _installed

UNKNOWN = "UNKNOWN"
INSIDE = "INSIDE_RECORDED_PERIOD"
OUTSIDE = "OUTSIDE_RECORDED_PERIOD"

#: The tools that read bundled dated data, which a caller can filter by.
DataTool = Literal[
    "list_ato_benchmark_industries",
    "get_ato_benchmarks",
    "get_div7a_benchmark_rate",
    "review_div7a_loan",
    "calc_payday_super_deadline",
    "review_payday_super_contributions",
    "build_payday_super_evidence_pack",
    "calculate_tax_worksheet",
    "get_lodgment_due_dates",
    "review_gst_coding",
    "reconcile_bas",
]
DATA_TOOLS: tuple[str, ...] = get_args(DataTool)

PAYDAY_TOOLS = [
    "calc_payday_super_deadline",
    "review_payday_super_contributions",
    "build_payday_super_evidence_pack",
]

STATEMENT = (
    "These are the periods, versions and source dates recorded in the installed release. "
    "They do not establish that any figure is current. uvx and other package caches can "
    "keep serving an older release, and official figures can change after a source was "
    "checked. This tool performs no lookup and cannot tell whether a newer release or "
    "newer official figures exist. UNKNOWN means the installed engine records no such "
    "fact. An as_at_status of INSIDE_RECORDED_PERIOD means the release holds data for "
    "that date, not that the data is current or unchanged. Verify every mutable fact "
    "against current official sources before relying on a result."
)
NOT_LISTED = (
    "The library, legislation corpus and rulings tools read folders the operator "
    "configures; their rows carry their own capture dates and are not listed here. "
    "generate_synthetic_sbr_fixture uses no dated data."
)


def _financial_year(label: str, basis: str) -> dict[str, str]:
    """A YYYY-YY label as the 1 July to 30 June year it names."""
    if not (len(label) == 7 and label[4] == "-" and label[:4].isdigit()
            and label[5:].isdigit() and int(label[5:]) == (int(label[:4]) + 1) % 100):
        raise ValueError(f"not a financial year label: {label!r}")
    start = int(label[:4])
    return {
        "id": label,
        "basis": basis,
        "start": date(start, 7, 1).isoformat(),
        "end": date(start + 1, 6, 30).isoformat(),
    }


def _iso_or_unknown(value: object) -> str:
    """An engine date string, or UNKNOWN when the engine recorded none."""
    if isinstance(value, str):
        try:
            return date.fromisoformat(value).isoformat()
        except ValueError:
            pass
    return UNKNOWN


def _dates(values: Iterable[object]) -> list[str] | str:
    known = sorted({day for day in map(_iso_or_unknown, values) if day != UNKNOWN})
    return known or UNKNOWN


def _urls(values: Iterable[object]) -> list[str] | str:
    known = list(dict.fromkeys(
        value for value in values
        if isinstance(value, str) and value.startswith("https://")
    ))
    return known or UNKNOWN


def _engine_document(directory: Path, name: str) -> dict[str, Any]:
    """An engine's shipped data file, for the provenance its loader does not expose."""
    return json.loads((directory / name).read_text(encoding="utf-8"))


def _dataset(
    dataset: str,
    description: str,
    tools: list[str],
    engine: str,
    periods: list[dict[str, str]] | str,
    sources: list[str] | str,
    retrieved: list[str] | str,
    source_checked: str,
    provenance: dict[str, Any],
) -> dict[str, Any]:
    return {
        "dataset": dataset,
        "description": description,
        "tools": tools,
        "engine": engine,
        "engine_version": _installed(engine) or UNKNOWN,
        "effective_periods": periods,
        "sources": sources,
        "retrieved": retrieved,
        "source_checked": source_checked,
        "provenance": provenance,
    }


def _benchmarks() -> list[dict[str, Any]]:
    datasets = []
    for year in available_years():
        source = dict(load(year).source)
        datasets.append(_dataset(
            f"ato_small_business_benchmarks:{year}",
            f"ATO small business benchmarks for {year}.",
            ["list_ato_benchmark_industries", "get_ato_benchmarks"],
            "ato-benchmark-compare",
            [_financial_year(year, "benchmark financial year")],
            _urls([source.get("resource_url"), source.get("dataset_page")]),
            _dates([source.get("retrieved")]),
            UNKNOWN,
            source,
        ))
    return datasets


def _div7a() -> dict[str, Any]:
    table = load_table()
    entries = [table.entries[year] for year in sorted(table.entries)]
    return _dataset(
        "div7a_benchmark_rates",
        "Division 7A s 109N(2) benchmark interest rates by year of income.",
        ["get_div7a_benchmark_rate", "review_div7a_loan"],
        "div7a-loan-review",
        [_financial_year(entry.year.label, "year of income") for entry in entries],
        _urls([url for entry in entries for url in (entry.primary_url, entry.verify_at)]),
        _dates(entry.retrieved_on for entry in entries),
        _iso_or_unknown(table.reviewed_on),
        {
            "reviewed_until": table.reviewed_until.label,
            "reviewed_on": table.reviewed_on,
            "law_content_date": DIV7A_LAW_CONTENT_DATE,
            "tables": [source.to_json_dict() for source in table.sources],
        },
    )


def _gic() -> dict[str, Any]:
    table = load_gic()  # the engine's own validation of the shipped table
    quarters = _engine_document(RATES_DATA_DIR, "gic_rates.json")["quarters"]
    return _dataset(
        "payday_gic_rates",
        "General interest charge quarters used for Payday Super notional earnings.",
        PAYDAY_TOOLS,
        "payday-super-checker",
        [
            {"id": f"{q['from']}/{q['to']}", "basis": "GIC quarter",
             "start": q["from"], "end": q["to"]}
            for q in quarters
        ],
        _urls(q.get("verify_at") for q in quarters),
        _dates(q.get("seen") for q in quarters),
        UNKNOWN,
        {
            "summary": table.provenance(),
            "table": table.source.to_json_dict() if table.source else UNKNOWN,
            "law_content_date": PAYDAY_LAW_CONTENT_DATE,
        },
    )


def _calendar() -> dict[str, Any]:
    calendar = load_calendar()
    official = _engine_document(CALENDAR_DATA_DIR, "business_days.json").get(
        "official_sources", {})
    return _dataset(
        "payday_business_day_calendar",
        "Whole-of-state public holidays under SGAA 1992 s 6(1) for Payday Super deadlines.",
        PAYDAY_TOOLS,
        "payday-super-checker",
        [{
            "id": "verified",
            "basis": "calendar verification span",
            "start": calendar.verified_from.isoformat(),
            "end": calendar.verified_until.isoformat(),
        }],
        _urls(official.values()),
        UNKNOWN,
        _iso_or_unknown(official.get("checked")),
        {
            "coverage_until": calendar.coverage_until.isoformat(),
            "coverage_note": official.get("coverage_note", UNKNOWN),
            "law_content_date": PAYDAY_LAW_CONTENT_DATE,
        },
    )


def _sg_parameters() -> dict[str, Any]:
    years = load_rates()["financial_years"]
    return _dataset(
        "payday_super_parameters",
        "SG charge percentage, concessional cap and maximum contributions base by "
        "financial year.",
        PAYDAY_TOOLS,
        "payday-super-checker",
        [_financial_year(label, "financial year") for label in sorted(years)],
        _urls(url for entry in years.values()
              for url in (entry.get("verify_at"), entry.get("cross_check"))),
        _dates(entry.get("seen") for entry in years.values()),
        UNKNOWN,
        {
            "sources": {label: entry.get("source", UNKNOWN) for label, entry in years.items()},
            "law_content_date": PAYDAY_LAW_CONTENT_DATE,
        },
    )


def _worksheets() -> list[dict[str, Any]]:
    return [
        _dataset(
            f"tax_worksheet:{kind}",
            f"Rates and thresholds for the {kind} worksheet.",
            ["calculate_tax_worksheet"],
            "australian-tax-calculators",
            [
                {key: period[key] for key in ("id", "basis", "start", "end")}
                for period in entry["supported_periods"]
            ] or UNKNOWN,
            _urls(entry.get("sources") or [entry.get("source")]),
            UNKNOWN,
            _iso_or_unknown(entry.get("source_checked")),
            {"source_passage": entry.get("source_passage", UNKNOWN)},
        )
        for kind, entry in calculations.worksheet_catalogue().items()
    ]


def _lodgment() -> dict[str, Any]:
    return _dataset(
        "lodgment_program",
        f"ATO {PROGRAM_YEAR} lodgment program due dates and its public holiday table.",
        ["get_lodgment_due_dates"],
        "lodgment-calendar",
        [{
            "id": PROGRAM_YEAR,
            "basis": "lodgment program year and holiday table",
            "start": COVERS_FROM.isoformat(),
            "end": COVERS_TO.isoformat(),
        }],
        _urls(url for url, _provision in LODGMENT_SOURCES.values()),
        UNKNOWN,
        _iso_or_unknown(LODGMENT_SOURCE_CHECKED),
        {
            "activity_statement_year": ACTIVITY_YEAR,
            "income_tax_return_year": RETURN_YEAR,
            "holiday_table": HOLIDAY_PROVISION,
        },
    )


def _gst() -> dict[str, Any]:
    return _dataset(
        "gst_review_rules",
        "GST coding and BAS reconciliation rules.",
        ["review_gst_coding", "reconcile_bas"],
        "gst-review",
        UNKNOWN,
        _urls([GST_ACT, ATO_BAS_METHOD, ATO_BAS_INSTRUCTIONS]),
        UNKNOWN,
        _iso_or_unknown(GST_SOURCE_CHECKED),
        {},
    )


def _as_at_status(periods: list[dict[str, str]] | str, as_at: date) -> str:
    if isinstance(periods, str):
        return UNKNOWN
    inside = any(
        date.fromisoformat(period["start"]) <= as_at <= date.fromisoformat(period["end"])
        for period in periods
    )
    return INSIDE if inside else OUTSIDE


def data_freshness(as_at: str | None = None, tool: str | None = None) -> dict[str, Any]:
    """Each bundled dataset's recorded period, engine version and source dates."""
    if tool is not None and tool not in DATA_TOOLS:
        raise InputError(f"tool must be one of {', '.join(DATA_TOOLS)}; got {tool!r}")
    day = None
    if as_at is not None:
        try:
            day = date.fromisoformat(as_at)
        except ValueError as exc:
            raise InputError(f"as_at must be a YYYY-MM-DD date; got {as_at!r}") from exc
    datasets = [
        *_benchmarks(), _div7a(), _gic(), _calendar(), _sg_parameters(), *_worksheets(),
        _lodgment(), _gst(),
    ]
    if tool is not None:
        datasets = [dataset for dataset in datasets if tool in dataset["tools"]]
    if day is not None:
        for dataset in datasets:
            dataset["as_at_status"] = _as_at_status(dataset["effective_periods"], day)
    return {
        "ok": True,
        "server": {
            "distribution": SERVER_DISTRIBUTION,
            "version": _installed(SERVER_DISTRIBUTION) or UNKNOWN,
        },
        "bundled_data": True,
        "live_lookup": False,
        "currency": "NOT_ESTABLISHED",
        "statement": STATEMENT,
        "as_at": None if day is None else day.isoformat(),
        "tool": tool,
        "datasets": datasets,
        "not_listed": NOT_LISTED,
    }
