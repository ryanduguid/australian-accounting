"""A fund-receipt date evidences timing, never the amount received.

Before v0.1.7 a receipt date on a row with neither ``matched_amount`` nor
``remitted_amount`` was read as a receipt of the whole SG amount, which let
a date alone reach ON_TIME. These tests pin the safer reading: the amount is
unknown, the row is left UNKNOWN between the outcome a full receipt earns and
the one a partial receipt earns, and no shortfall figure is built from an
amount the file does not state. Every interface that carries a verdict
(assessment, report CSV, console, exit code, evidence pack) is covered.
"""

from __future__ import annotations

import csv
from datetime import date
from decimal import Decimal

import pytest
from paydaysuper.assess import LATE, ON_TIME, UNKNOWN, UNPAID
from paydaysuper.calendar import load_calendar
from paydaysuper.cli import EXIT_LATE_FOUND, EXIT_OK
from paydaysuper.cli import main as cli_main
from paydaysuper.deadlines import ContribLine
from paydaysuper.rates import load_gic
from paydaysuper.report import assess

# After 28 Jul 2026, so the LCR 2026/1 transition gate is not what answers.
QE_DAY = date(2026, 8, 6)
DUE = date(2026, 8, 17)  # 7 business days after the QE day
AS_AT = date(2026, 9, 10)
NO_AMOUNT = "carries no amount"


def line(**kwargs) -> ContribLine:
    base = dict(employee_id="E1", qe_day=QE_DAY, sg_amount=Decimal("540.00"), row=2)
    base.update(kwargs)
    return ContribLine(**base)


def one(contrib: ContribLine, as_at: date = AS_AT, **kwargs):
    return assess([contrib], load_calendar(), load_gic(), as_at, **kwargs)[0]


def test_a_timely_receipt_with_no_amount_is_unknown_not_on_time():
    result = one(line(received=date(2026, 8, 12)))
    assert result.deadline.due == DUE
    assert result.verdict == UNKNOWN
    assert result.horizon_verdicts == (UNPAID, ON_TIME)
    assert result.base_shortfall is None and result.final_shortfall is None
    assert result.nec is None
    caveat = next(c for c in result.caveats if NO_AMOUNT in c)
    assert "2026-08-12" in caveat
    assert "$540.00" in caveat
    assert "matched_amount" in caveat
    assert "not read as ON_TIME" in caveat


def test_a_timely_receipt_with_no_amount_before_the_due_date_passes_is_still_unknown():
    result = one(line(received=date(2026, 8, 12)), as_at=date(2026, 8, 14))
    assert result.verdict == UNKNOWN
    assert result.horizon_verdicts == ("NOT_YET_DUE", ON_TIME)


def test_an_explicit_full_amount_is_on_time():
    result = one(line(received=date(2026, 8, 12), matched_amount=Decimal("540.00")))
    assert result.verdict == ON_TIME
    assert result.horizon_verdicts is None
    assert not any(NO_AMOUNT in c for c in result.caveats)


def test_a_ten_column_part_payment_evidences_its_own_amount():
    """remitted_amount on a dated remittance is the other explicit amount."""
    result = one(
        line(
            remitted=date(2026, 8, 10),
            remitted_amount=Decimal("200.00"),
            received=date(2026, 8, 12),
        )
    )
    assert result.verdict == UNPAID
    assert result.base_shortfall == Decimal("340.00")
    assert not any(NO_AMOUNT in c for c in result.caveats)


def test_an_explicit_partial_amount_is_not_on_time():
    result = one(line(received=date(2026, 8, 12), matched_amount=Decimal("300.00")))
    assert result.verdict == UNPAID
    assert result.base_shortfall == Decimal("240.00")


def test_a_late_receipt_with_no_amount_is_late_and_keeps_the_whole_shortfall():
    """Late is late whatever arrived, but the s 18D reduction needs the amount.
    Before this rule the blank column made the final shortfall nil."""
    result = one(line(received=date(2026, 8, 25)))
    assert result.verdict == LATE
    assert result.base_shortfall == Decimal("540.00")
    assert result.final_shortfall == Decimal("540.00")
    assert result.offset_s18d is False
    assert result.lateness_basis == "as-at date (fund receipt amount not evidenced)"
    assert any("s 18D reduction" in c and "not applied" in c for c in result.caveats)
    assert any("are a maximum" in n for n in result.notes)

    stated = one(line(received=date(2026, 8, 25), matched_amount=Decimal("540.00")))
    assert stated.verdict == LATE
    assert stated.final_shortfall == Decimal("0.00")
    assert stated.lateness_basis == "fund receipt"


def test_a_prepayment_with_no_amount_inside_the_window_is_unknown():
    result = one(line(received=date(2026, 7, 30)))
    assert result.verdict == UNKNOWN
    assert result.horizon_verdicts == (UNPAID, ON_TIME)


def test_a_receipt_after_the_due_date_with_a_possible_item4_extension_stays_unknown():
    """An earlier payday could extend the deadline under item 4, and the
    receipt lands inside that window. The amount gap adds ON_TIME to the
    outer outcomes without pretending the extension is settled."""
    earlier = line(
        employee_id="ITEM4",
        qe_day=date(2026, 7, 23),
        sg_amount=Decimal("100.00"),
        first_to_fund=True,
        row=2,
    )
    later = line(employee_id="ITEM4", received=date(2026, 8, 19), row=3)
    results = assess(
        [earlier, later],
        load_calendar(),
        load_gic(),
        date(2026, 8, 19),
        transition_allocation_confirmed=True,
    )
    second = next(r for r in results if r.line.row == 3)
    assert second.deadline.possible_item4_due is not None
    assert second.deadline.due < date(2026, 8, 19) <= second.deadline.possible_item4_due
    assert second.verdict == UNKNOWN
    assert second.horizon_verdicts == (LATE, ON_TIME)
    assert any(NO_AMOUNT in c for c in second.caveats)
    assert any("item 4" in c for c in second.caveats)


def test_contradictory_amounts_are_still_refused():
    with pytest.raises(ValueError, match="matched_amount must be between zero and"):
        one(line(received=date(2026, 8, 12), matched_amount=Decimal("600.00")))


def test_two_receipts_for_one_payday_are_assessed_on_their_own_amounts():
    """One payday split across two funds arrives as two rows. Each row is
    judged on the amount it states; a stated pair adds up, a blank one does
    not borrow the whole liability."""
    first = line(received=date(2026, 8, 12), matched_amount=Decimal("300.00"), row=2)
    second = line(received=date(2026, 8, 13), matched_amount=Decimal("240.00"), row=3)
    results = assess([first, second], load_calendar(), load_gic(), AS_AT)
    assert [r.verdict for r in results] == [UNPAID, UNPAID]
    assert [r.base_shortfall for r in results] == [Decimal("240.00"), Decimal("300.00")]

    blank = line(received=date(2026, 8, 13), row=3)
    results = assess([first, blank], load_calendar(), load_gic(), AS_AT)
    assert results[1].verdict == UNKNOWN
    assert results[1].horizon_verdicts == (UNPAID, ON_TIME)


def test_the_new_starter_narrative_names_the_missing_amount():
    result = one(line(received=date(2026, 8, 25)))
    narrative = next(c for c in result.caveats if "first_contribution_to_fund" in c)
    assert "only once matched_amount evidences" in narrative
    assert "$0.00" not in narrative


def _write(path, rows: str, header_suffix: str = "") -> None:
    path.write_text(
        "employee_id,payment_date,sg_amount,remitted_date,fund_received_date,"
        "first_contribution_to_fund,out_of_cycle,next_standard_payday,defined_benefit"
        f"{header_suffix}\n{rows}",
        encoding="utf-8",
    )


def test_cli_leaves_a_bare_receipt_row_unassessed_and_exits_two(tmp_path, capsys):
    src = tmp_path / "nine-column.csv"
    _write(src, "E1,2026-08-06,540.00,2026-08-10,2026-08-12,no,no,,no\n")
    out = tmp_path / "report.csv"
    code = cli_main([str(src), "-o", str(out), "--as-at", "2026-09-10"])
    printed = capsys.readouterr().out
    assert code == EXIT_LATE_FOUND
    assert "ON_TIME: 0" in printed and "UNKNOWN: 1" in printed
    assert "UNPAID or ON_TIME" in printed
    assert NO_AMOUNT in printed
    with out.open(encoding="utf-8-sig", newline="") as handle:
        row = next(csv.DictReader(handle))
    assert row["verdict"] == UNKNOWN
    assert row["unassessable_between"] == "UNPAID or ON_TIME"
    assert NO_AMOUNT in row["caveats"]


def test_cli_accepts_the_same_row_once_the_amount_is_stated(tmp_path, capsys):
    src = tmp_path / "eleven-column.csv"
    _write(
        src,
        "E1,2026-08-06,540.00,2026-08-10,2026-08-12,no,no,,no,,540.00\n",
        header_suffix=",remitted_amount,matched_amount",
    )
    code = cli_main([str(src), "-o", str(tmp_path / "report.csv"), "--as-at", "2026-09-10"])
    printed = capsys.readouterr().out
    assert code == EXIT_OK
    assert "ON_TIME: 1" in printed
    assert NO_AMOUNT not in printed


def test_evidence_pack_carries_the_missing_amount_caveat(tmp_path, capsys):
    src = tmp_path / "nine-column.csv"
    _write(src, "E1,2026-08-06,540.00,2026-08-10,2026-08-12,no,no,,no\n")
    pack = tmp_path / "pack"
    code = cli_main(["evidence-pack", str(src), "-o", str(pack), "--as-at", "2026-09-10"])
    capsys.readouterr()
    assert code == EXIT_LATE_FOUND
    report = (pack / "report.csv").read_text(encoding="utf-8-sig")
    assert NO_AMOUNT in report
    assert "ON_TIME" not in report.split("\n")[1].split(",")[:12] or "UNKNOWN" in report
    review = (pack / "practitioner-review.md").read_text(encoding="utf-8")
    assert "UNKNOWN" in review
