from __future__ import annotations

import csv
from pathlib import Path

import pytest

from wiptally.cli import main

ROOT = Path(__file__).resolve().parents[1]
SAMPLE = ROOT / "examples" / "sample_contracts.csv"


def _rows(path: Path) -> dict[str, dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        return {row["contract_id"]: row for row in csv.DictReader(handle)}


def test_sample_schedule_pins_the_worked_examples(tmp_path: Path) -> None:
    out = tmp_path / "wip-schedule.csv"
    code = main(["schedule", str(SAMPLE), "-o", str(out), "--as-at", "2026-08-31"])
    assert code == 2
    rows = _rows(out)
    assert len(rows) == 5

    hunter = rows["HUNTER-CIVIL-01"]
    assert hunter["revenue_to_date"] == "500000.00"
    assert hunter["contract_asset"] == "50000.00"
    assert hunter["contract_liability"] == "0.00"
    assert hunter["percent_complete"] == "0.5"

    faded = rows["HUNTER-CIVIL-02"]
    assert faded["revenue_to_date"] == "466666.67"
    assert faded["contract_liability"] == "33333.33"
    assert "profit_fade" in faded["flags"]
    assert "stale_cost_to_complete" in faded["flags"]

    mine = rows["MINE-ROM-01"]
    assert mine["transaction_price"] == "2200000.00"
    assert mine["variable_consideration_excluded"] == "300000.00"
    assert mine["revenue_to_date"] == "1650000.00"

    power = rows["POWER-CIVIL-01"]
    assert power["revenue_to_date"] == "371000.00"
    assert power["contract_asset"] == "71000.00"
    assert "onerous_contract_review_aasb_137" in power["flags"]

    # Pinned whole, not field by field. A truncated source row shifts trailing
    # columns silently, and only the whole row shows it.
    assert rows["EXPLORATORY-01"] == {
        # The period the row was computed for travels with the row, so the
        # review pack cannot bind a June schedule to an August header.
        "as_at": "2026-08-31",
        "contract_id": "EXPLORATORY-01",
        "customer": "Example Principal Pty Ltd",
        "description": "Early works outcome not yet measurable",
        "progress_method": "cost_to_cost",
        "original_contract_sum": "600000.00",
        "approved_variations": "0.00",
        "unapproved_variations_estimate": "0.00",
        "variable_consideration_included": "0.00",
        "variable_consideration_excluded": "0.00",
        "transaction_price": "600000.00",
        "costs_incurred": "100000.00",
        "inefficiency_rework_wastage": "0.00",
        "uninstalled_materials": "0.00",
        "progress_cost": "100000.00",
        "estimated_cost_to_complete": "400000.00",
        "estimated_cost_at_completion": "500000.00",
        "progress_eac": "500000.00",
        "percent_complete": "0.00",
        "revenue_to_date": "100000.00",
        "certified_billings": "0.00",
        "uncertified_claims": "0.00",
        "contract_asset": "100000.00",
        "contract_liability": "0.00",
        "gross_profit_at_completion": "100000.00",
        "margin_at_completion": "0.1666666666666666666666666667",
        "prior_margin_at_completion": "",
        "profit_fade_points": "",
        # No prior period, so no period revenue. Not 99999.90.
        "period_revenue": "",
        "retention_withheld": "0.00",
        "retention_classification": "review",
        "gst_on_certified_billings": "0.00",
        "gst_on_retention": "0.00",
        "committed_outstanding": "0.00",
        "flags": "outcome_not_reasonably_measurable|etc_has_no_commitments",
    }


def test_portfolio_totals_are_not_netted(tmp_path: Path) -> None:
    out = tmp_path / "wip-schedule.csv"
    main(["schedule", str(SAMPLE), "-o", str(out), "--as-at", "2026-08-31"])
    rows = _rows(out)
    assets = sum(float(row["contract_asset"]) for row in rows.values())
    liabilities = sum(float(row["contract_liability"]) for row in rows.values())
    # The test uses float only to assert the two sides both exist. The engine
    # never does. A netted  one-line "WIP" figure would cancel these.
    assert assets > 0
    assert liabilities > 0


def test_refuses_to_overwrite_the_source(tmp_path: Path) -> None:
    source = tmp_path / "contracts.csv"
    source.write_text(SAMPLE.read_text(encoding="utf-8"), encoding="utf-8")
    assert main(["schedule", str(source), "-o", str(source)]) == 1


def test_review_pack_binds_source_hash(tmp_path: Path) -> None:
    schedule = tmp_path / "wip-schedule.csv"
    pack = tmp_path / "practitioner-review.md"
    assert main(["schedule", str(SAMPLE), "-o", str(schedule), "--as-at", "2026-08-31"]) == 2
    assert main(
        [
            "review-pack",
            str(schedule),
            "--source",
            str(SAMPLE),
            "-o",
            str(pack),
            "--as-at",
            "2026-08-31",
        ]
    ) == 2
    text = pack.read_text(encoding="utf-8")
    assert "AASB 15" in text
    assert "Do not offset the two sides." in text
    assert "HUNTER-CIVIL-02" in text
    assert "Source SHA-256" in text


def test_review_pack_rejects_tampered_schedule(tmp_path: Path) -> None:
    schedule = tmp_path / "wip-schedule.csv"
    pack = tmp_path / "practitioner-review.md"
    assert main(["schedule", str(SAMPLE), "-o", str(schedule), "--as-at", "2026-08-31"]) == 2

    original = schedule.read_bytes()
    tampered = original.replace(b"500000.00", b"500001.00", 1)
    assert tampered != original
    schedule.write_bytes(tampered)

    assert main(
        [
            "review-pack",
            str(schedule),
            "--source",
            str(SAMPLE),
            "-o",
            str(pack),
            "--as-at",
            "2026-08-31",
        ]
    ) == 1
    assert not pack.exists()


def test_review_pack_refuses_a_schedule_from_another_period(tmp_path: Path) -> None:
    """A June schedule must not be bound under an August sign-off header."""
    schedule = tmp_path / "wip-schedule.csv"
    pack = tmp_path / "practitioner-review.md"
    assert main(["schedule", str(SAMPLE), "-o", str(schedule), "--as-at", "2026-06-30"]) == 2
    assert main(
        [
            "review-pack",
            str(schedule),
            "--source",
            str(SAMPLE),
            "-o",
            str(pack),
            "--as-at",
            "2026-08-31",
        ]
    ) == 1
    assert not pack.exists()


def test_as_at_must_be_a_real_iso_date(tmp_path: Path) -> None:
    """A transposed reporting date must not date the evidence."""
    out = tmp_path / "wip-schedule.csv"
    assert main(["schedule", str(SAMPLE), "-o", str(out), "--as-at", "2026-31-08"]) == 1
    assert not out.exists()


def test_non_utf8_source_reports_an_error_not_a_traceback(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """An ANSI-codepage ledger export is a data error, not a stack trace."""
    source = tmp_path / "cp1252.csv"
    source.write_bytes(
        SAMPLE.read_text(encoding="utf-8")
        .replace("Synthetic Colliery JV", "Bäcker Colliery JV")
        .encode("cp1252")
    )
    out = tmp_path / "wip-schedule.csv"
    assert main(["schedule", str(source), "-o", str(out)]) == 1
    assert capsys.readouterr().err.startswith("error: ")


def test_oversized_field_reports_an_error_not_a_traceback(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """A field past the csv module's limit is a data error, not a stack trace."""
    header, first_row = SAMPLE.read_text(encoding="utf-8").splitlines()[:2]
    fields = first_row.split(",")
    fields[2] = '"' + "x" * (csv.field_size_limit() + 1000) + '"'
    source = tmp_path / "oversized.csv"
    source.write_text(f"{header}\n{','.join(fields)}\n", encoding="utf-8")
    out = tmp_path / "wip-schedule.csv"
    assert main(["schedule", str(source), "-o", str(out)]) == 1
    assert capsys.readouterr().err.startswith("error: ")


def test_mapping_file_renames_columns(tmp_path: Path) -> None:
    contracts = tmp_path / "jobs.csv"
    contracts.write_text(
        "Job,Contract sum,Cost to date,ETC,Certified to date,committed_outstanding\n"
        "MAP-1,1000.00,400.00,400.00,450.00,380.00\n",
        encoding="utf-8",
    )
    mapping = tmp_path / "map.json"
    mapping.write_text(
        ROOT.joinpath("examples", "mapping.example.json").read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    out = tmp_path / "wip-schedule.csv"
    assert main(
        ["schedule", str(contracts), "-o", str(out), "--mapping-file", str(mapping)]
    ) == 0
    rows = _rows(out)
    assert rows["MAP-1"]["revenue_to_date"] == "500.00"
    assert rows["MAP-1"]["contract_asset"] == "50.00"


def test_review_pack_refuses_to_overwrite_its_mapping_file(tmp_path: Path) -> None:
    schedule = tmp_path / "wip-schedule.csv"
    assert main(
        ["schedule", str(SAMPLE), "-o", str(schedule), "--as-at", "2026-08-31"]
    ) == 2
    mapping = tmp_path / "mapping.md"
    original = b"{}\n"
    mapping.write_bytes(original)

    assert main(
        [
            "review-pack",
            str(schedule),
            "--source",
            str(SAMPLE),
            "--mapping-file",
            str(mapping),
            "-o",
            str(mapping),
            "--as-at",
            "2026-08-31",
        ]
    ) == 1
    assert mapping.read_bytes() == original
