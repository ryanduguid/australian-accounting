"""Importer and join refusals the vendor fixtures do not reach."""

import json
from datetime import date
from decimal import Decimal

import pytest
from paydaysuper import profiles
from paydaysuper.csv_io import CsvError
from paydaysuper.importers import PayrollRow, SuperRow, join, read_payroll, read_super

SUPER_HEADER = "Employee,Employee Number,Contribution Type,Period Start,Period End,Payment Date,Amount\n"
SUPER_ROW = "Test Employee One,E1,Superannuation Guarantee,01/07/2026,09/07/2026,{paid},612.00\n"
PAYROLL_HEADER = "Employee,Employee Number,Payment Date,Pay Period End,Superannuation Guarantee\n"


def _write(tmp_path, name, text):
    path = tmp_path / name
    path.write_text(text, encoding="utf-8")
    return path


def test_an_export_with_no_header_row_is_refused(tmp_path):
    with pytest.raises(CsvError, match="super.csv has no header row$"):
        read_super(_write(tmp_path, "super.csv", ""))


def test_an_export_that_is_not_utf8_is_refused(tmp_path):
    path = tmp_path / "super.csv"
    path.write_bytes((SUPER_HEADER + SUPER_ROW.format(paid="14/07/2026")).replace("One", "Één").encode("cp1252"))
    with pytest.raises(CsvError, match=r"super.csv is not UTF-8 text \(byte 0xc9 at position"):
        read_super(path)


def test_an_export_with_a_header_and_no_rows_is_refused(tmp_path):
    with pytest.raises(CsvError, match="super.csv has a header but no data rows$"):
        read_super(_write(tmp_path, "super.csv", SUPER_HEADER))


def test_a_payment_date_with_a_utc_offset_is_refused_with_its_cell_named(tmp_path):
    path = _write(tmp_path, "super.csv", SUPER_HEADER + SUPER_ROW.format(paid="2026-07-14T10:00:00Z"))
    with pytest.raises(CsvError, match="^row 2: paid date value '2026-07-14T10:00:00Z' carries a UTC"):
        read_super(path)


def test_a_payment_date_that_is_not_a_date_is_refused(tmp_path):
    path = _write(tmp_path, "super.csv", SUPER_HEADER + SUPER_ROW.format(paid="someday"))
    with pytest.raises(CsvError, match="^row 2: cannot read paid date value 'someday' as a date$"):
        read_super(path)


def test_an_export_with_no_super_guarantee_rows_is_refused(tmp_path):
    text = SUPER_HEADER + SUPER_ROW.format(paid="14/07/2026").replace("Superannuation Guarantee", "Salary Sacrifice")
    with pytest.raises(CsvError, match="super.csv has rows but none of them is super guarantee"):
        read_super(_write(tmp_path, "super.csv", text))


def test_a_payroll_row_without_a_payday_is_refused(tmp_path):
    path = _write(tmp_path, "payroll.csv", PAYROLL_HEADER + "Test Employee One,E1,,09/07/2026,612.00\n")
    with pytest.raises(CsvError, match="^row 2: payday is empty$"):
        read_payroll(path)


def _custom_super_profile(tmp_path, monkeypatch, columns):
    folder = tmp_path / "profiles"
    folder.mkdir()
    (folder / "test-super.json").write_text(
        json.dumps({
            "key": "test-super",
            "name": "Test super export",
            "role": "super",
            "signature": ["Employee"],
            "columns": columns,
        }),
        encoding="utf-8",
    )
    monkeypatch.setattr(profiles, "PROFILE_DIR", folder)


def test_a_super_profile_without_an_amount_column_is_refused(tmp_path, monkeypatch):
    _custom_super_profile(tmp_path, monkeypatch, {"employee_name": ["Employee"]})
    path = _write(tmp_path, "super.csv", "Employee,Paid\nTest Employee One,14/07/2026\n")
    with pytest.raises(CsvError, match="super.csv: no amount column found for profile test-super$"):
        read_super(path, vendor="test-super")


def test_a_super_profile_without_a_contribution_filter_reads_every_row(tmp_path, monkeypatch):
    _custom_super_profile(
        tmp_path, monkeypatch,
        {"employee_name": ["Employee"], "paid_date": ["Paid"], "amount": ["Amount"]},
    )
    path = _write(tmp_path, "super.csv", "Employee,Paid,Amount\nTest Employee One,14/07/2026,612.00\n")
    rows, profile, _ = read_super(path, vendor="test-super")
    assert profile.sg_filter is None  # nosec B101: pytest outcome assertion
    assert [(row.employee_name, row.paid_date, row.amount) for row in rows] == [  # nosec B101: pytest outcome assertion
        ("Test Employee One", date(2026, 7, 14), Decimal("612.00")),
    ]


def test_a_payroll_row_with_no_employee_is_refused():
    payroll = [PayrollRow(None, "", date(2026, 7, 9), None, Decimal("612.00"), 2)]
    with pytest.raises(CsvError, match="^row 2: the employee column is empty$"):
        join(payroll, [])


def test_a_super_row_with_only_a_period_start_covers_that_day_alone():
    from paydaysuper.join import _covers

    row = SuperRow(None, "A", date(2026, 7, 1), None, date(2026, 7, 14), Decimal("612.00"), 2)
    assert _covers(row, date(2026, 7, 1)) is True  # nosec B101: pytest outcome assertion
    assert _covers(row, date(2026, 7, 9)) is False  # nosec B101: pytest outcome assertion
