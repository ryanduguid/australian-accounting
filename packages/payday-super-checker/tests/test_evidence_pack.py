"""Fabricated contracts for the combined review workflow."""
import csv
import hashlib
import io
import json
import os
from contextlib import contextmanager
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
    assert "[decision-log.md](decision-log.md)" in review
    for field in ("Reviewer:", "Review date (Australia):", "Conclusion and workpaper reference:"):
        assert field in decision
        assert field not in review
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


@pytest.mark.parametrize("replace_completed_file", [False, True])
def test_writer_refuses_traversal_and_preserves_failed_pack(tmp_path, monkeypatch, replace_completed_file):
    from paydaysuper.evidence_pack import write_evidence_pack

    output = tmp_path / "pack"
    with pytest.raises(ValueError, match="fixed filenames"):
        write_evidence_pack({"../outside.txt": "refuse"}, output)
    assert not output.exists()
    original = Path.open

    def fail_second(path, *args, **kwargs):
        if path.name == "exceptions.json":
            if replace_completed_file:
                with original(output / "decision-log.md", "wb") as stream:
                    stream.write(b"A human has started this decision log")
            raise OSError("fabricated write failure")
        return original(path, *args, **kwargs)

    monkeypatch.setattr(Path, "open", fail_second)
    with pytest.raises(OSError, match="fabricated write failure"):
        write_evidence_pack({name: "fabricated" for name in sorted(FILES)}, output)
    assert {path.name for path in output.iterdir()} == {"decision-log.md"}
    expected = "A human has started this decision log" if replace_completed_file else "fabricated"
    assert (output / "decision-log.md").read_text() == expected
    with pytest.raises(ValueError, match="already exists"):
        write_evidence_pack({name: "fabricated" for name in sorted(FILES)}, output)


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


def test_concurrent_destination_creation_is_not_replaced(tmp_path, monkeypatch):
    from paydaysuper import evidence_pack

    output = tmp_path / "pack"
    mkdir = Path.mkdir
    rename = os.rename

    def racing_mkdir(path, *args, **kwargs):
        if path == output:
            mkdir(output)  # The competing writer claims the name first.
        return mkdir(path, *args, **kwargs)

    def unix_replacing_rename(source, destination):
        # Model Unix's replacement of an empty directory on every test platform.
        mkdir(output)
        output.rmdir()
        return rename(source, destination)

    monkeypatch.setattr(Path, "mkdir", racing_mkdir)
    monkeypatch.setattr(os, "rename", unix_replacing_rename)
    with pytest.raises(FileExistsError):
        evidence_pack.write_evidence_pack({name: "fabricated" for name in FILES}, output)
    assert output.is_dir()
    assert list(output.iterdir()) == []


@pytest.mark.parametrize("name", ["import", "review-pack", "evidence-pack"])
def test_qualified_reserved_filename_still_runs_the_existing_checker(name, tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    Path(name).write_bytes((EVALUATION / "fixtures/receipt_on_due_date.csv").read_bytes())
    assert cli.main(["./" + name, "--as-at", EXPECTED["as_at"], "-o", "report.csv"]) == 0
    assert "employee_id" in Path("report.csv").read_text(encoding="utf-8-sig")


def test_ordinary_checker_writes_csv_in_bounded_chunks(tmp_path, monkeypatch):
    from paydaysuper import report

    header, row = (EVALUATION / "fixtures/receipt_on_due_date.csv").read_text().splitlines()
    source = tmp_path / "contributions.csv"
    source.write_text(header + "\n" + (row + "\n") * 50)
    original_output = report.atomic_text_output

    @contextmanager
    def bounded_output(*args, **kwargs):
        with original_output(*args, **kwargs) as stream:
            original_write = stream.write

            def bounded_write(chunk):
                assert len(chunk) < 4096, "ordinary report output must remain streamed"
                return original_write(chunk)

            monkeypatch.setattr(stream, "write", bounded_write)
            yield stream

    monkeypatch.setattr(report, "atomic_text_output", bounded_output)
    output = tmp_path / "report.csv"
    assert cli.main([str(source), "--as-at", EXPECTED["as_at"], "-o", str(output)]) == 0
    with output.open(encoding="utf-8-sig", newline="") as stream:
        assert len(list(csv.reader(stream))) == 52
