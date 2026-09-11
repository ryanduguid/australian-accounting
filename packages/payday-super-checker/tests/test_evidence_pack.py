"""Fabricated contracts for the combined review workflow."""
import csv
import hashlib
import io
import json
from pathlib import Path

import pytest

from paydaysuper import cli


EVALUATION = Path(__file__).resolve().parents[1] / "evaluation/payday_super_evidence"
EXPECTED = json.loads((EVALUATION / "expected_results.json").read_text())
FILES = {"report.csv", "practitioner-review.md", "exceptions.json", "decision-log.md"}


@pytest.mark.parametrize("scenario", EXPECTED["scenarios"], ids=lambda s: s["id"])
def test_pinned_fabricated_pack(scenario, tmp_path, capsys):
    output = tmp_path / "pack"
    source = EVALUATION / "fixtures" / scenario["fixture"]
    before = source.read_bytes()
    code = cli.main([
        "evidence-pack", str(source), "--as-at", EXPECTED["as_at"], "-o", str(output),
    ])
    assert code == scenario["expected_exit"]
    assert source.read_bytes() == before
    assert {p.name for p in output.iterdir()} == FILES
    report = (output / "report.csv").read_bytes()
    rows = list(csv.DictReader(io.StringIO(report.decode("utf-8-sig"))))
    assert "employee_id" not in rows[0]
    assert rows[0]["verdict"] == scenario["expected_verdict"]
    assert rows[0]["due_date"] == scenario["expected_due_date"]
    digest = hashlib.sha256(report).hexdigest()
    review = (output / "practitioner-review.md").read_text(encoding="utf-8")
    queue = json.loads((output / "exceptions.json").read_text(encoding="utf-8"))
    decision = (output / "decision-log.md").read_text(encoding="utf-8")
    assert digest in review and digest in decision
    assert queue["report_sha256"] == digest
    assert queue["schema_version"] == 1
    assert [row["verdict"] for row in queue["exceptions"]] == (
        [] if scenario["expected_verdict"] == "ON_TIME" else [scenario["expected_verdict"]]
    )
    for heading in ("## Evidence reviewed", "## Decisions", "## Practitioner sign-off"):
        assert heading in decision
    assert "APES 110" in decision and "TPB" in decision
    assert "Obtain and verify the fund receipt date" in review or code == 0 or "LATE" in review
    captured = capsys.readouterr()
    combined = captured.out + captured.err + "".join(
        p.read_text(encoding="utf-8-sig") for p in output.iterdir()
    )
    assert "SYN001" not in combined
    assert str(source) not in combined
    # A second destination must produce exactly the same four files.
    second = tmp_path / "second"
    assert cli.main([
        "evidence-pack", str(source), "--as-at", EXPECTED["as_at"], "-o", str(second),
    ]) == code
    assert all((output / name).read_bytes() == (second / name).read_bytes() for name in FILES)


def test_existing_directory_is_preserved(tmp_path, capsys):
    output = tmp_path / "pack"
    output.mkdir()
    sentinel = output / "decision-log.md"
    sentinel.write_text("previous human decisions", encoding="utf-8")
    assert cli.main([
        "evidence-pack", str(EVALUATION / "fixtures/timely_remittance_no_receipt.csv"),
        "--as-at", EXPECTED["as_at"], "-o", str(output),
    ]) == 1
    assert sentinel.read_text() == "previous human decisions"
    assert list(output.iterdir()) == [sentinel]
    assert "already exists" in capsys.readouterr().err


def test_bad_input_leaves_no_pack(tmp_path, capsys):
    source = tmp_path / "invalid.csv"
    source.write_text("employee_id,payment_date,sg_amount\nFAKE,not-a-date,120\n")
    output = tmp_path / "pack"
    assert cli.main(["evidence-pack", str(source), "--as-at", "2026-08-20", "-o", str(output)]) == 1
    assert not output.exists()
    assert "error:" in capsys.readouterr().err


def test_remittance_confirmation_does_not_clear_review_queue(tmp_path):
    source = EVALUATION / "fixtures/timely_remittance_no_receipt.csv"
    assert cli.main([
        "evidence-pack", str(source), "--as-at", EXPECTED["as_at"],
        "--confirm-remittance-only", "-o", str(tmp_path / "pack"),
    ]) == 2


def test_render_failure_leaves_no_directory(tmp_path, monkeypatch, capsys):
    from paydaysuper import evidence_pack

    def fail(*args, **kwargs):
        raise ValueError("fabricated render failure")

    monkeypatch.setattr(evidence_pack, "render_practitioner_pack", fail)
    output = tmp_path / "pack"
    assert cli.main([
        "evidence-pack", str(EVALUATION / "fixtures/timely_remittance_no_receipt.csv"),
        "--as-at", EXPECTED["as_at"], "-o", str(output),
    ]) == 1
    assert not output.exists()
    assert "fabricated render failure" in capsys.readouterr().err


@pytest.mark.parametrize("change, verdict", [
    ({"sg_amount": "0.00"}, "UNKNOWN"),
    ({"defined_benefit": "yes"}, "SKIPPED"),
    ({"remitted_date": ""}, "UNPAID"),
])
def test_every_non_on_time_verdict_requires_review(change, verdict, tmp_path):
    source = tmp_path / "fabricated.csv"
    with (EVALUATION / "fixtures/timely_remittance_no_receipt.csv").open() as stream:
        row = next(csv.DictReader(stream))
    row.update(change)
    with source.open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(row))
        writer.writeheader()
        writer.writerow(row)
    output = tmp_path / "pack"
    assert cli.main(["evidence-pack", str(source), "--as-at", "2026-08-20", "-o", str(output)]) == 2
    queue = json.loads((output / "exceptions.json").read_text())
    assert [r["verdict"] for r in queue["exceptions"]] == [verdict]


def test_writer_refuses_traversal_and_cleans_failed_staging(tmp_path, monkeypatch):
    from paydaysuper.evidence_pack import write_evidence_pack

    output = tmp_path / "pack"
    with pytest.raises(ValueError, match="fixed filenames"):
        write_evidence_pack({"../outside.txt": "refuse"}, output)
    assert not output.exists()
    original = Path.write_bytes

    def fail_second(path, data):
        if path.name == "exceptions.json":
            raise OSError("fabricated write failure")
        return original(path, data)

    monkeypatch.setattr(Path, "write_bytes", fail_second)
    with pytest.raises(OSError, match="fabricated write failure"):
        write_evidence_pack({name: "fabricated" for name in sorted(FILES)}, output)
    assert list(tmp_path.iterdir()) == []


@pytest.mark.parametrize("malformation", ["header", "width"])
def test_projected_report_validation_refuses_format_drift(tmp_path, malformation):
    from paydaysuper.practitioner_pack import PractitionerPackError, parse_report_snapshot

    output = tmp_path / "pack"
    cli.main([
        "evidence-pack", str(EVALUATION / "fixtures/receipt_on_due_date.csv"),
        "--as-at", EXPECTED["as_at"], "-o", str(output),
    ])
    report = (output / "report.csv").read_text(encoding="utf-8-sig")
    if malformation == "header":
        report = report.replace("row,qe_day", "employee_id,qe_day", 1)
    else:
        report = report.replace("\n2,", "\n2,extra,", 1)
    with pytest.raises(PractitionerPackError, match="17"):
        parse_report_snapshot(report.encode(), Path("report.csv"), identifiers_omitted=True)
