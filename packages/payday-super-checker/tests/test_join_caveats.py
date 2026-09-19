"""PSC-2 durable fix: structural join warnings travel with the files.

The four degraded-join triggers and the duplicate-name case must surface in
the canonical CSV the import writes, survive the checker's own reader, and
reach the exported report's caveats column and the evidence pack, so a
reviewer holding only an output file sees the qualification that governs
its verdicts. Genuine ambiguous joins stay refusals, so they can never read
as an unqualified ON_TIME.
"""

import csv
from datetime import date

import pytest
from paydaysuper import LAW_CONTENT_DATE
from paydaysuper.assess import assess
from paydaysuper.calendar import load_calendar
from paydaysuper.csv_io import DEFAULT_MAPPING, CsvError, parse_rows
from paydaysuper.evidence_pack import build_evidence_pack
from paydaysuper.importers import import_files
from paydaysuper.rates import load_gic
from paydaysuper.report import render_csv

AS_AT = date(2026, 8, 20)

PAYROLL_FULL = (
    "Employee Name,Employee ID,Date,Pay Period End,Superannuation Guarantee\n"
    "Anchor,E001,06/08/2026,06/08/2026,120.00\n"
)
PAYROLL_NO_ID = (
    "Employee Name,Date,Pay Period End,Superannuation Guarantee\n"
    "Anchor,06/08/2026,06/08/2026,120.00\n"
)
PAYROLL_NO_PERIOD_END = (
    "Employee Name,Employee ID,Date,Superannuation Guarantee\n"
    "Anchor,E001,06/08/2026,120.00\n"
)
SUPER_FULL = (
    "Employee Name,Employee ID,Superannuation Category,Period From,Period To,"
    "Paid Date,Amount\n"
    "Anchor,E001,Superannuation Guarantee,01/08/2026,06/08/2026,14/08/2026,120.00\n"
)
SUPER_ONE_PERIOD = (
    "Employee Name,Employee ID,Superannuation Category,Period To,Paid Date,Amount\n"
    "Anchor,E001,Superannuation Guarantee,06/08/2026,14/08/2026,120.00\n"
)
SUPER_NO_PERIODS = (
    "Employee Name,Employee ID,Superannuation Category,Paid Date,Amount\n"
    "Anchor,E001,Superannuation Guarantee,14/08/2026,120.00\n"
)

# One degraded pair and the warning fragment each must reach the file.
FOUR_TRIGGERS = [
    (PAYROLL_NO_ID, SUPER_FULL, "matched on employee name"),
    (PAYROLL_NO_PERIOD_END, SUPER_FULL, "falls back"),
    (PAYROLL_FULL, SUPER_ONE_PERIOD, "single day"),
    (PAYROLL_FULL, SUPER_NO_PERIODS, "no pay period columns"),
]


def _canonical_rows(path):
    with open(path, newline="", encoding="utf-8-sig") as f:
        return list(csv.DictReader(f))


def _run_import(tmp_path, payroll_text, super_text):
    payroll = tmp_path / "payroll.csv"
    payroll.write_text(payroll_text, encoding="utf-8")
    super_csv = tmp_path / "super.csv"
    super_csv.write_text(super_text, encoding="utf-8")
    out = tmp_path / "contributions.csv"
    import_files(payroll, super_csv, out)
    return out


def test_each_trigger_writes_its_warning_into_the_canonical_file(tmp_path):
    for payroll_text, super_text, fragment in FOUR_TRIGGERS:
        out = _run_import(tmp_path, payroll_text, super_text)
        rows = _canonical_rows(out)
        assert rows, (payroll_text, super_text)
        for row in rows:
            assert fragment in row["join_caveats"], (
                payroll_text,
                super_text,
                row["join_caveats"],
            )


def test_clean_join_writes_no_join_caveats(tmp_path):
    # A clean import writes an empty caveat column: the caveat channel keeps
    # meaning "this row needs attention", and an unimpaired join does not.
    out = _run_import(tmp_path, PAYROLL_FULL, SUPER_FULL)
    for row in _canonical_rows(out):
        assert row["join_caveats"] == ""


def test_duplicate_names_merge_and_the_file_says_how(tmp_path):
    # Two payroll rows for different people who share a name, matched under
    # name mode: both rows carry the structural warning in the file.
    payroll = tmp_path / "payroll.csv"
    payroll.write_text(
        "Employee Name,Date,Pay Period End,Superannuation Guarantee\n"
        "Robin Smith,06/08/2026,06/08/2026,120.00\n"
        "Robin Smith,20/08/2026,20/08/2026,120.00\n",
        encoding="utf-8",
    )
    super_csv = tmp_path / "super.csv"
    super_csv.write_text(SUPER_FULL.replace("Anchor", "Robin Smith"), encoding="utf-8")
    out = tmp_path / "contributions.csv"
    # One merged employee with two positive paydays, so the import's
    # statutory-allocation gate needs the same reconciliation confirmation
    # production would supply.
    report = import_files(
        payroll, super_csv, out, statutory_allocation_confirmed=True
    )
    assert report.key_mode == "name"
    rows = _canonical_rows(out)
    assert len(rows) == 2
    for row in rows:
        assert "matched on employee name" in row["join_caveats"]


def test_join_caveats_survive_the_reader_and_reach_the_report(tmp_path):
    # End to end: import (degraded) -> canonical file -> checker reader ->
    # assessment -> report. The operator then adds the fund-receipt evidence
    # the import cannot know, which flips the verdict to ON_TIME; the join
    # warning must still sit in that row's caveats, so the ON_TIME cannot
    # read as an unqualified verdict.
    out = _run_import(tmp_path, PAYROLL_NO_ID, SUPER_FULL)
    lines = parse_rows(out, DEFAULT_MAPPING)
    assert lines and lines[0].join_caveats
    assert any("matched on employee name" in c for c in lines[0].join_caveats)

    with open(out, newline="", encoding="utf-8-sig") as f:
        rows = list(csv.DictReader(f))
    for row in rows:
        row["fund_received_date"] = "2026-08-14"
    received_path = tmp_path / "with-receipt.csv"
    with open(received_path, "w", newline="", encoding="utf-8-sig") as f:
        writer = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)

    lines = parse_rows(received_path, DEFAULT_MAPPING)
    results = assess(lines, load_calendar(), load_gic(), AS_AT)
    assert results[0].verdict == "ON_TIME", results[0].verdict
    assert any("matched on employee name" in c for c in results[0].caveats)

    report_csv = render_csv(results, AS_AT, LAW_CONTENT_DATE)
    assert "matched on employee name" in report_csv

    pack = build_evidence_pack(results, as_at=AS_AT)
    assert "matched on employee name" in pack["report.csv"]


def test_competing_identical_paydays_stay_refused(tmp_path):
    # A super payment with no period covering two payroll rows identical in
    # payday, period and amount has no defensible allocation, so the import
    # refuses and writes no file: the ambiguity can never reach a verdict.
    payroll = tmp_path / "payroll.csv"
    payroll.write_text(
        "Employee Name,Employee ID,Date,Pay Period End,Superannuation Guarantee\n"
        "Anchor,E001,06/08/2026,06/08/2026,120.00\n"
        "Anchor,E001,06/08/2026,06/08/2026,120.00\n",
        encoding="utf-8",
    )
    super_csv = tmp_path / "super.csv"
    super_csv.write_text(SUPER_NO_PERIODS, encoding="utf-8")
    out = tmp_path / "contributions.csv"
    with pytest.raises(CsvError, match="no defensible way"):
        import_files(payroll, super_csv, out)
    assert not out.exists()