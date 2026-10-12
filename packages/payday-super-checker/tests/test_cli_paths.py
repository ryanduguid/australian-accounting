"""Command-line refusals and failure paths the worked examples do not reach."""

import io
from pathlib import Path

from paydaysuper import cli
from paydaysuper.cli import EXIT_ERROR, EXIT_LATE_FOUND, EXIT_OK, main

from conftest import SAMPLE

IMPORTERS = Path(__file__).parent / "fixtures" / "importers"
CONFIRM = "--confirm-transition-allocation"


def _check(tmp_path, *extra):
    return main([str(SAMPLE), "-o", str(tmp_path / "report.csv"), "--as-at", "2026-08-10", CONFIRM, *extra])


def test_streams_without_reconfigure_still_receive_the_error(tmp_path, monkeypatch):
    stdout, stderr = io.StringIO(), io.StringIO()
    monkeypatch.setattr("sys.stdout", stdout)
    monkeypatch.setattr("sys.stderr", stderr)
    missing = tmp_path / "missing.csv"
    code = main([str(missing), "-o", str(tmp_path / "report.csv"), "--as-at", "2026-08-10"])
    assert code == EXIT_ERROR  # nosec B101: pytest outcome assertion
    assert stderr.getvalue().startswith("error: file not found: ")  # nosec B101: pytest outcome assertion


def test_an_as_at_with_a_utc_offset_is_refused_with_the_flag_named(tmp_path, capsys):
    code = main([str(SAMPLE), "-o", str(tmp_path / "report.csv"), "--as-at", "2026-08-10T00:00:00Z"])
    assert code == EXIT_ERROR  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err.startswith(  # nosec B101: pytest outcome assertion
        "error: --as-at value '2026-08-10T00:00:00Z' carries a UTC or timezone offset marker"
    )


def test_an_as_at_that_is_not_a_date_is_refused(tmp_path, capsys):
    code = main([str(SAMPLE), "-o", str(tmp_path / "report.csv"), "--as-at", "soon"])
    assert code == EXIT_ERROR  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err == (  # nosec B101: pytest outcome assertion
        "error: --as-at expects a date such as 2026-08-10 or 10/08/2026, got 'soon'\n"
    )


def test_an_input_that_cannot_be_read_names_it(tmp_path, capsys):
    folder = tmp_path / "pay.csv"
    folder.mkdir()
    code = main([str(folder), "-o", str(tmp_path / "report.csv"), "--as-at", "2026-08-10"])
    assert code == EXIT_ERROR  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err.startswith(f"error: cannot read {folder}: ")  # nosec B101: pytest outcome assertion


def test_a_report_that_cannot_be_written_names_it(tmp_path, capsys):
    output = tmp_path / "missing" / "report.csv"
    code = main([str(SAMPLE), "-o", str(output), "--as-at", "2026-08-10", CONFIRM])
    assert code == EXIT_ERROR  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err.startswith(f"error: cannot write {output}: ")  # nosec B101: pytest outcome assertion


def test_a_date_overflow_is_one_error_line(tmp_path, monkeypatch, capsys):
    # The year ceiling on every date read keeps a sentinel such as 9999-12-31
    # out; the handler is the backstop if an overflow arrives another way.
    def overflow(*args, **kwargs):
        raise OverflowError("date value out of range")

    monkeypatch.setattr(cli, "assess", overflow)
    assert _check(tmp_path) == EXIT_ERROR  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err == (  # nosec B101: pytest outcome assertion
        "error: a date in this file is too far in the future to work with. "
        "Check for placeholder dates such as 9999-12-31.\n"
    )


def test_more_than_ten_unassessable_lines_are_counted_after_the_first_ten(tmp_path, capsys):
    path = tmp_path / "pay.csv"
    rows = "".join(f"LATE{n:02},2029-03-01,900.00,2029-04-20,2029-04-22,no,no,,no\n" for n in range(12))
    path.write_text(
        "employee_id,payment_date,sg_amount,remitted_date,fund_received_date,"
        "first_contribution_to_fund,out_of_cycle,next_standard_payday,defined_benefit\n" + rows,
        encoding="utf-8",
    )
    report = tmp_path / "r.csv"
    code = main([str(path), "-o", str(report), "--as-at", "2029-06-01", CONFIRM])
    out = capsys.readouterr().out
    assert code == EXIT_LATE_FOUND  # nosec B101: pytest outcome assertion
    assert "12 line(s) cannot be assessed" in out  # nosec B101: pytest outcome assertion
    assert f"  ... and 2 more unassessable line(s) (see {report})" in out  # nosec B101: pytest outcome assertion


def test_an_import_without_warnings_prints_no_warning_block(tmp_path, capsys):
    code = main([
        "import",
        "--payroll", str(IMPORTERS / "xero_payroll.csv"),
        "--super", str(IMPORTERS / "xero_super.csv"),
        "-o", str(tmp_path / "contributions.csv"),
    ])
    out = capsys.readouterr().out
    assert code == EXIT_OK  # nosec B101: pytest outcome assertion
    assert "paydays: matched 2," in out  # nosec B101: pytest outcome assertion
    assert "warnings (" not in out  # nosec B101: pytest outcome assertion


def _report_in(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    assert _check(tmp_path) == EXIT_LATE_FOUND  # nosec B101: pytest outcome assertion


def test_review_pack_names_a_report_that_is_not_there(tmp_path, monkeypatch, capsys):
    monkeypatch.chdir(tmp_path)
    code = main(["review-pack", "missing.csv", "-o", "pack.md"])
    assert code == EXIT_ERROR  # nosec B101: pytest outcome assertion
    err = capsys.readouterr().err
    assert err.startswith("error: file not found: ") and err.endswith("missing.csv\n")  # nosec B101: pytest outcome assertion


def test_review_pack_names_a_report_it_cannot_read(tmp_path, monkeypatch, capsys):
    monkeypatch.chdir(tmp_path)
    (tmp_path / "folder.csv").mkdir()
    code = main(["review-pack", "folder.csv", "-o", "pack.md"])
    assert code == EXIT_ERROR  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err.startswith("error: cannot read ")  # nosec B101: pytest outcome assertion


def test_review_pack_names_a_pack_it_cannot_write(tmp_path, monkeypatch, capsys):
    _report_in(tmp_path, monkeypatch)
    capsys.readouterr()
    (tmp_path / "pack.md").mkdir()
    code = main(["review-pack", "report.csv", "-o", "pack.md"])
    assert code == EXIT_ERROR  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err.startswith("error: cannot write pack.md: ")  # nosec B101: pytest outcome assertion
