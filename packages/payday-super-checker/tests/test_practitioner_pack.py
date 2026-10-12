import csv
import hashlib
import subprocess
from pathlib import Path

import pytest
from paydaysuper import cli
from paydaysuper.practitioner_pack import (
    EXPECTED_REPORT_HEADER,
    PractitionerPackError,
    load_report_snapshot,
    render_practitioner_pack,
    write_practitioner_pack,
)
from paydaysuper.report import CSV_HEADER

NOTE_TEXT = (
    "payday-super-checker 0.1.3, source C:\\Private\\Client A\\pay.csv, "
    "as at 2026-09-10. Legal content current at 2026-08-15. "
    "EXPERIMENTAL ESTIMATES: the ATO assesses the charge."
)
ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture(autouse=True)
def _run_report_tests_inside_tmp_path(tmp_path, monkeypatch):
    """Practitioner-pack paths are confined to cwd; keep pytest's tmp dir as cwd."""
    monkeypatch.chdir(tmp_path)


def _row(**overrides):
    values = {
        "row": "2",
        "employee_id": "000123",
        "qe_day": "2026-08-06",
        "pathway": "usual period",
        "due_date": "2026-08-17",
        "verdict": "LATE",
        "days_late": "3",
        "lateness_measured_to": "fund receipt",
        "sg_amount": "100.00",
        "final_shortfall": "0.00",
        "notional_earnings": "1.00",
        "uplift_best_case": "0.20",
        "uplift_worst_case": "0.60",
        "sgc_estimate_low": "1.20",
        "sgc_estimate_high": "1.60",
        "caveats": "verify receipt evidence",
        "notes": "fabricated test row",
        "unassessable_between": "",
    }
    values.update(overrides)
    return [values[name] for name in EXPECTED_REPORT_HEADER]


def _note(**overrides):
    values = {name: "" for name in EXPECTED_REPORT_HEADER}
    values["employee_id"] = "NOTE"
    values["notes"] = NOTE_TEXT
    values.update(overrides)
    return [values[name] for name in EXPECTED_REPORT_HEADER]


def _write_report(path: Path, rows=None, header=None):
    dest = path
    with dest.open("w", encoding="utf-8-sig", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(header or EXPECTED_REPORT_HEADER)
        writer.writerows(rows or [_row(), _note()])
    return dest


def test_contract_is_deliberately_pinned_to_the_report_writer():
    assert list(EXPECTED_REPORT_HEADER) == CSV_HEADER
    assert len(EXPECTED_REPORT_HEADER) == 18


def test_pack_is_deterministic_private_by_default_and_source_bound(tmp_path):
    source = _write_report(
        Path("report.csv"),
        [
            _row(employee_id="'=SUM(1,1)"),
            _row(
                row="3",
                employee_id="ava.lawson@example.test",
                verdict="AT_RISK",
                days_late="",
                final_shortfall="",
                notional_earnings="",
                uplift_best_case="",
                uplift_worst_case="",
                sgc_estimate_low="",
                sgc_estimate_high="",
                caveats="remittance is not fund receipt | needs follow-up",
            ),
            _note(),
        ],
    )
    snapshot = load_report_snapshot(source)

    first = render_practitioner_pack(snapshot)
    second = render_practitioner_pack(snapshot)

    assert first == second
    assert hashlib.sha256(source.read_bytes()).hexdigest() in first
    assert "ATTENTION REQUIRED" in first
    assert "LATE" in first and "AT_RISK" in first
    assert "source row 2" in first and "source row 3" in first
    assert "000123" not in first
    assert "SUM(1,1)" not in first
    assert "ava.lawson@example.test" not in first
    assert "Client A" in first  # the producer's exact provenance note is retained
    assert "does not lodge, pay, post or make a compliance determination" in first


def test_no_exception_pack_still_requires_human_signoff(tmp_path):
    source = _write_report(
        Path("report.csv"),
        [
            _row(
                verdict="ON_TIME",
                days_late="",
                final_shortfall="",
                notional_earnings="",
                uplift_best_case="",
                uplift_worst_case="",
                sgc_estimate_low="",
                sgc_estimate_high="",
            ),
            _note(),
        ],
    )
    pack = load_report_snapshot(source)
    text = render_practitioner_pack(pack)

    assert pack.needs_attention is False
    assert "NO EXCEPTION INDICATORS" in text
    assert "Practitioner sign-off is still required" in text
    for field in ("Reviewer:", "Review date (Australia):", "Conclusion and workpaper reference:"):
        assert field in text
    assert "decision-log.md" not in text


@pytest.mark.parametrize(
    "rows, message",
    [
        ([_row()], "terminal NOTE"),
        ([_note(), _row()], "terminal NOTE"),
        ([_row(), _note(), _note()], "exactly one NOTE"),
        ([_row(verdict="PASS"), _note()], "unsupported verdict"),
        ([_row(row="two"), _note()], "positive integer"),
        ([_row(qe_day="06/08/2026"), _note()], "ISO date"),
        ([_row(sg_amount="NaN"), _note()], "finite amount"),
        ([_row(sgc_estimate_low="1.21"), _note()], "does not add up"),
    ],
)
def test_malformed_or_internally_inconsistent_reports_fail_closed(tmp_path, rows, message):
    source = _write_report(Path("report.csv"), rows)
    with pytest.raises(PractitionerPackError, match=message):
        load_report_snapshot(source)


def test_employee_identifier_note_is_not_a_terminal_note(tmp_path):
    source = _write_report(Path("report.csv"), [_row(employee_id="NOTE"), _note()])

    snapshot = load_report_snapshot(source)

    assert [row.source_row for row in snapshot.rows] == [2]


def test_note_only_report_is_rejected_instead_of_rendering_no_exceptions(tmp_path):
    source = _write_report(Path("report.csv"), [_note()])

    with pytest.raises(PractitionerPackError, match="at least one contribution row"):
        load_report_snapshot(source)


def test_terminal_note_requires_non_whitespace_provenance(tmp_path):
    source = _write_report(Path("report.csv"), [_row(), _note(notes=" \t ")])

    with pytest.raises(PractitionerPackError, match="malformed terminal NOTE"):
        load_report_snapshot(source)


def test_malformed_terminal_note_with_data_fields_fails_closed(tmp_path):
    source = _write_report(Path("report.csv"), [_row(), _note(row="3")])

    with pytest.raises(PractitionerPackError, match="terminal NOTE"):
        load_report_snapshot(source)


def test_duplicate_or_changed_headers_fail_closed(tmp_path):
    duplicate = list(EXPECTED_REPORT_HEADER)
    duplicate[-1] = "notes"
    source = _write_report(Path("duplicate.csv"), header=duplicate)
    with pytest.raises(PractitionerPackError, match="exact 18-column"):
        load_report_snapshot(source)

    changed = list(EXPECTED_REPORT_HEADER)
    changed[0] = "source_row"
    source = _write_report(Path("changed.csv"), header=changed)
    with pytest.raises(PractitionerPackError, match="exact 18-column"):
        load_report_snapshot(source)


def test_default_practitioner_workpaper_is_ignored():
    rules = (ROOT / ".gitignore").read_text(encoding="utf-8").splitlines()
    manifest = (ROOT / "MANIFEST.in").read_text(encoding="utf-8").splitlines()

    assert "/practitioner-review.md" in rules
    assert "include .gitignore" in manifest

    if (ROOT / ".git").exists():
        result = subprocess.run(
            ["git", "check-ignore", "--no-index", "--quiet", "--", "practitioner-review.md"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0, result.stderr


def test_output_is_atomic_and_replaces_the_link_not_its_target(tmp_path):
    source = _write_report(Path("report.csv"))
    target = tmp_path / "elsewhere.md"
    target.write_text("leave me", encoding="utf-8")
    output = tmp_path / "practitioner-review.md"
    try:
        output.symlink_to(target)
    except OSError:
        pytest.skip("symlink creation is unavailable")

    write_practitioner_pack(load_report_snapshot(source), output)

    assert output.is_file() and not output.is_symlink()
    assert "# Payday Super Practitioner Review Pack" in output.read_text(encoding="utf-8")
    assert target.read_text(encoding="utf-8") == "leave me"


def test_cli_writes_pack_and_uses_attention_exit_code(tmp_path, capsys):
    source = _write_report(Path("report.csv"))
    output = tmp_path / "review.md"

    result = cli.main(["review-pack", str(source), "-o", str(output)])

    assert result == cli.EXIT_LATE_FOUND
    assert output.exists()
    assert f"wrote {output}" in capsys.readouterr().out


def test_cli_refuses_input_output_collision_and_wrong_suffix(tmp_path, capsys):
    source = _write_report(Path("report.csv"))

    assert cli.main(["review-pack", str(source), "-o", str(source)]) == cli.EXIT_ERROR
    assert "overwrite the input report" in capsys.readouterr().err

    assert cli.main(["review-pack", str(source), "-o", str(tmp_path / "review.txt")]) == 1
    assert ".md filename" in capsys.readouterr().err


def test_cli_refuses_input_output_collision_through_a_symlink(tmp_path, capsys):
    source = _write_report(Path("report.csv"))
    output = tmp_path / "review.md"
    try:
        output.symlink_to(source)
    except OSError:
        pytest.skip("symlink creation is unavailable")

    assert cli.main(["review-pack", str(source), "-o", str(output)]) == cli.EXIT_ERROR
    assert "overwrite the input report" in capsys.readouterr().err

def test_relative_report_path_cannot_leave_cwd(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    outside = tmp_path.parent / "escaped-report.csv"
    _write_report(outside)
    try:
        with pytest.raises(PractitionerPackError, match="filename in the working directory"):
            load_report_snapshot(Path("..") / outside.name)
    finally:
        outside.unlink(missing_ok=True)


def test_rejected_report_path_names_the_remedy(tmp_path, monkeypatch, capsys):
    """The confinement is real and stays, but it applies to no other path this
    tool accepts and `-o out/report.csv` is fully supported, so the operator
    who follows the documented 2-command path has to be told how to proceed
    rather than left with a bare refusal."""
    monkeypatch.chdir(tmp_path)
    (tmp_path / "sub").mkdir()
    source = tmp_path / "sub" / "report.csv"
    _write_report(source)

    for selected in ("sub/report.csv", "./report.csv", str(source)):
        assert (
            cli.main(["review-pack", selected, "-o", str(tmp_path / "pack.md")])
            == cli.EXIT_ERROR
        )
        message = capsys.readouterr().err
        assert "filename in the working directory" in message
        assert "cd to the directory holding report.csv" in message


def test_review_pack_path_confinement_is_documented():
    """The confinement was documented nowhere and contradicted both documents'
    stated path boundary, so a reader following either one hit exit 1."""
    readme = (ROOT / "README.md").read_text(encoding="utf-8")

    review_pack_section = readme.partition("### Build a practitioner review pack")[
        2
    ].partition("\n### ")[0]
    assert "bare `.csv` filename in the current directory" in review_pack_section
    assert "`sub/report.csv`" in review_pack_section

    security = (ROOT / "SECURITY.md").read_text(encoding="utf-8")

    for document, heading in ((readme, "## Local file boundary"),
                              (security, "## Local path trust boundary")):
        # Both documents hard-wrap, so compare on a single-spaced rendering.
        boundary = " ".join(document.partition(heading)[2].split())
        assert "review-pack" in boundary
        assert "bare `.csv` filename" in boundary


def test_report_path_must_be_csv(tmp_path):
    source = tmp_path / "report.txt"
    source.write_text("not csv", encoding="utf-8")
    with pytest.raises(PractitionerPackError, match=".csv"):
        load_report_snapshot(source)


NO_EXPOSURE = dict(
    final_shortfall="", notional_earnings="", uplift_best_case="", uplift_worst_case="",
    sgc_estimate_low="", sgc_estimate_high="",
)


@pytest.mark.parametrize(
    "rows, message",
    [
        ([_row(qe_day=""), _note()], "source row 2 has a blank qe_day$"),
        ([_row(qe_day="20260806"), _note()], "source row 2 qe_day must be an ISO date, got '20260806'$"),
        ([_row(sg_amount=""), _note()], "source row 2 has a blank sg_amount$"),
        ([_row(sg_amount="lots"), _note()], "source row 2 sg_amount must be a decimal amount, got 'lots'$"),
        ([_row(sg_amount="-1.00"), _note()], "source row 2 sg_amount must not be negative$"),
        ([_row(sg_amount="100.0"), _note()], "source row 2 sg_amount must use two decimal places$"),
        ([_row(row="0"), _note()], "source row 0 row must be a positive integer$"),
        ([_row(days_late="three"), _note()], "source row 2 days_late must be a non-negative integer$"),
        ([_row(days_late="-1"), _note()], "source row 2 days_late must be a non-negative integer$"),
        ([_row(), _row(), _note()], "^duplicate source row 2$"),
        ([_row(employee_id=""), _note()], "source row 2 has a blank employee_id$"),
        ([_row(pathway=""), _note()], "source row 2 has a blank pathway$"),
        ([_row(final_shortfall=""), _note()], "source row 2 exposed verdict has no final shortfall$"),
        ([_row(notional_earnings=""), _note()], "source row 2 exposed verdict has incomplete exposure amounts$"),
        ([_row(sgc_estimate_high="1.61"), _note()], "source row 2 high SG-charge estimate does not add up$"),
        (
            [_row(uplift_best_case="0.60", uplift_worst_case="0.20", sgc_estimate_low="1.60",
                  sgc_estimate_high="1.20"), _note()],
            "source row 2 exposure range is reversed$",
        ),
        ([_row(verdict="ON_TIME", days_late=""), _note()], "source row 2 non-exposed verdict carries exposure amounts$"),
        ([_row(unassessable_between="LATE or ON_TIME"), _note()], "source row 2 carries candidate verdicts but is not UNKNOWN$"),
        (
            [_row(verdict="UNKNOWN", days_late="", unassessable_between="LATE", **NO_EXPOSURE), _note()],
            "source row 2 has malformed candidate verdicts$",
        ),
        ([_row(days_late=""), _note()], "source row 2 exposed verdict has no days_late$"),
    ],
)
def test_every_report_field_is_checked_before_a_pack_is_written(tmp_path, rows, message):
    source = _write_report(Path("report.csv"), rows)
    with pytest.raises(PractitionerPackError, match=message):
        load_report_snapshot(source)


def test_an_exposed_row_with_the_charge_estimate_withheld_is_accepted(tmp_path):
    withheld = _row(
        verdict="UNPAID", final_shortfall="100.00", notional_earnings="",
        uplift_best_case="", uplift_worst_case="", sgc_estimate_low="", sgc_estimate_high="",
    )
    source = _write_report(Path("report.csv"), [withheld, _note()])
    [row] = load_report_snapshot(source).rows
    assert row.final_shortfall == 100  # nosec B101: pytest outcome assertion
    assert row.sgc_estimate_low is None and row.sgc_estimate_high is None  # nosec B101: pytest outcome assertion


@pytest.mark.parametrize(
    ("name", "message"),
    [("report\x00.csv", "^path contains a NUL byte$"), ("report.txt", "^report.txt must be a .csv checker report$")],
)
def test_the_report_path_must_be_a_csv_file_name(tmp_path, name, message):
    with pytest.raises(PractitionerPackError, match=message):
        load_report_snapshot(name)


def test_a_report_that_is_not_utf8_is_refused(tmp_path):
    from paydaysuper.practitioner_pack import parse_report_snapshot

    with pytest.raises(PractitionerPackError, match="report.csv is not UTF-8 CSV$"):
        parse_report_snapshot("row,employee_id\n2,Employé\n".encode("cp1252"), Path("report.csv"))


def test_a_report_the_strict_csv_reader_refuses_is_malformed(tmp_path):
    from paydaysuper.practitioner_pack import parse_report_snapshot

    with pytest.raises(PractitionerPackError, match="report.csv is malformed CSV: "):
        parse_report_snapshot(b'row,"employee"_id\n', Path("report.csv"))


def test_a_projected_report_row_must_keep_every_field(tmp_path):
    from paydaysuper.practitioner_pack import parse_report_snapshot

    header = ",".join(name for name in EXPECTED_REPORT_HEADER if name != "employee_id")
    data = f"{header}\n2,2026-08-06\n".encode()
    with pytest.raises(PractitionerPackError, match="^projected report row must have 17 fields$"):
        parse_report_snapshot(data, Path("report.csv"), identifiers_omitted=True)


def test_a_report_row_missing_fields_is_refused(tmp_path):
    source = _write_report(Path("report.csv"), [["2", "000123"], _note()])
    with pytest.raises(PractitionerPackError, match="^CSV row 2 has 2 fields; expected 18$"):
        load_report_snapshot(source)
