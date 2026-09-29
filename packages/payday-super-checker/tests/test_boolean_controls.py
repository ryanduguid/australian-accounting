"""Control options must not turn textual false into confirmation or disclosure."""
import csv
import io
import json
from datetime import date, timedelta
from decimal import Decimal

import pytest
from paydaysuper import LAW_CONTENT_DATE
from paydaysuper.assess import assess
from paydaysuper.calendar import load_calendar
from paydaysuper.csv_io import CsvError
from paydaysuper.deadlines import ContribLine
from paydaysuper.evidence_pack import build_evidence_pack
from paydaysuper.importers import import_files
from paydaysuper.rates import RatesError, load_gic, load_rates
from paydaysuper.report import console_summary, needs_attention, render_csv
from paydaysuper.sgc import notional_earnings

INVALID = ["false", "true", 0, 1, 0.0, 1.0, None, [], {}]
AS_AT = date(2026, 9, 10)


def contribution():
    return ContribLine(
        employee_id="FABRICATED", qe_day=date(2026, 8, 6), sg_amount=Decimal("540.00"),
        remitted=date(2026, 8, 10), remitted_amount=Decimal("540.00"), row=2,
    )


@pytest.fixture
def results():
    return assess([contribution()], load_calendar(), load_gic(), AS_AT)


@pytest.mark.parametrize("name", ["transition_allocation_confirmed", "allow_stale_gic"])
@pytest.mark.parametrize("invalid", INVALID)
def test_assessment_rejects_malformed_controls_even_when_the_row_does_not_need_them(name, invalid):
    with pytest.raises(ValueError, match=name + ".*boolean"):
        assess([contribution()], load_calendar(), load_gic(), AS_AT, **{name: invalid})


@pytest.mark.parametrize("stale", [False, True])
@pytest.mark.parametrize("invalid", INVALID)
def test_daily_rate_requires_a_boolean_for_known_and_stale_dates(stale, invalid):
    table = load_gic()
    day = table.last_known + timedelta(days=1) if stale else table.last_known
    with pytest.raises(RatesError, match="allow_stale.*boolean"):
        table.daily_rate(day, allow_stale=invalid)


@pytest.mark.parametrize("invalid", INVALID)
def test_earnings_reject_a_malformed_control_when_no_day_accrues(invalid):
    with pytest.raises(ValueError, match="allow_stale.*boolean"):
        notional_earnings(Decimal("540.00"), AS_AT, AS_AT, load_gic(), allow_stale=invalid)


@pytest.mark.parametrize("invalid", INVALID)
def test_attention_requires_boolean_confirmation(results, invalid):
    with pytest.raises(ValueError, match="remittance_only_confirmed.*boolean"):
        needs_attention(results, remittance_only_confirmed=invalid)


@pytest.mark.parametrize("invalid", INVALID)
def test_console_requires_boolean_confirmation(results, invalid):
    with pytest.raises(ValueError, match="remittance_only_confirmed.*boolean"):
        console_summary(results, AS_AT, "fabricated.csv", LAW_CONTENT_DATE, load_rates(),
                        remittance_only_confirmed=invalid)


@pytest.mark.parametrize("invalid", INVALID)
def test_csv_identifier_option_requires_a_boolean(results, invalid):
    with pytest.raises(ValueError, match="include_employee_ids.*boolean"):
        render_csv(results, AS_AT, LAW_CONTENT_DATE, include_employee_ids=invalid)


@pytest.mark.parametrize("invalid", INVALID)
def test_evidence_pack_does_not_record_a_malformed_confirmation(results, invalid):
    with pytest.raises(ValueError, match="remittance_only_confirmed.*boolean"):
        build_evidence_pack(results, as_at=AS_AT, remittance_only_confirmed=invalid)


@pytest.mark.parametrize("invalid", INVALID)
def test_import_requires_boolean_confirmation_before_writing(tmp_path, invalid):
    payroll = tmp_path / "payroll.csv"
    payroll.write_text(
        "Employee Name,Date,Pay Period End,Superannuation Guarantee\n"
        "A,03/07/2026,03/07/2026,100.00\n"
        "A,10/07/2026,10/07/2026,100.00\n", encoding="utf-8",
    )
    super_file = tmp_path / "super.csv"
    super_file.write_text(
        "Employee Name,Superannuation Category,Period From,Period To,Paid Date,Amount\n"
        "A,Superannuation Guarantee,03/07/2026,10/07/2026,15/07/2026,100.00\n", encoding="utf-8",
    )
    original = (payroll.read_bytes(), super_file.read_bytes())
    output = tmp_path / "contributions.csv"
    with pytest.raises(CsvError, match="statutory_allocation_confirmed.*boolean"):
        import_files(payroll, super_file, output, vendor="myob-ar",
                     statutory_allocation_confirmed=invalid)
    assert not output.exists()
    assert (payroll.read_bytes(), super_file.read_bytes()) == original


@pytest.mark.parametrize("flag", [False, True])
def test_literal_report_controls_keep_their_meaning(results, flag):
    report = render_csv(results, AS_AT, LAW_CONTENT_DATE, include_employee_ids=flag)
    header = next(csv.reader(io.StringIO(report)))
    assert ("employee_id" in header) is flag
    assert ("FABRICATED" in report) is flag
    assert needs_attention(results, remittance_only_confirmed=flag) is (not flag)
    pack = build_evidence_pack(results, as_at=AS_AT, remittance_only_confirmed=flag)
    assert json.loads(pack["exceptions.json"])["remittance_only_confirmed"] is flag
    assert "FABRICATED" not in "".join(pack.values())


@pytest.mark.parametrize("flag", [False, True])
def test_literal_controls_work_when_assessment_and_accrual_are_empty(flag):
    table = load_gic()
    assert assess([], load_calendar(), table, AS_AT,
                  transition_allocation_confirmed=flag, allow_stale_gic=flag) == []
    assert notional_earnings(Decimal("540.00"), AS_AT, AS_AT, table, allow_stale=flag) == 0
