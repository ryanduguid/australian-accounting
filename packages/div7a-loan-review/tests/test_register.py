"""CSV review mode."""
from __future__ import annotations

import csv
from decimal import Decimal

import pytest

from div7aloan.register import (
    GATE_COLUMNS,
    MYR_COLUMNS,
    RegisterError,
    load_rows,
    require_columns,
    review_register,
    review_register_file,
)
from div7aloan.verdicts import GateVerdict, MyrVerdict
from div7aloan.years import parse_year

D = Decimal
YEAR = parse_year("2026-27")
ALL_COLUMNS = list(GATE_COLUMNS) + list(MYR_COLUMNS)

MET = "examples/sample_loans_myr_met.csv"
MIXED = "examples/sample_loans_mixed.csv"


def row(**overrides) -> dict:
    base = {
        "loan_id": "L-1",
        "borrower_reference": "SYN-1",
        "year_loan_made": "2023-24",
        "written_agreement": "true",
        "terms_in_place_before_lodgment_day": "true",
        "maximum_term_years": "7",
        "secured_by_registered_mortgage_over_real_property": "false",
        "security_coverage_at_first_made": "unknown",
        "interest_rate_for_years_after_year_loan_made": "0.0827",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
        "remaining_term_years": "5",
        "payments_applied_during_the_year": "25556.00",
        "out_of_scope_reason": "",
    }
    base.update(overrides)
    return base


# --- required columns --------------------------------------------------


def test_a_missing_required_column_is_refused():
    with pytest.raises(RegisterError, match="written_agreement"):
        require_columns(["loan_id"], GATE_COLUMNS, "test.csv")


def test_the_error_names_every_missing_column():
    with pytest.raises(RegisterError) as caught:
        require_columns(["loan_id"], ("written_agreement", "maximum_term_years"), "test.csv")
    assert "written_agreement" in str(caught.value)
    assert "maximum_term_years" in str(caught.value)


def test_a_register_without_the_repayment_columns_is_refused_for_myr(tmp_path):
    path = tmp_path / "gate_only.csv"
    path.write_text(",".join(GATE_COLUMNS) + "\nL-1,true,true,7,false,unknown,0.0827,2023-24\n",
                    encoding="utf-8")
    with pytest.raises(RegisterError, match="amalgamated_loan_unpaid_at_end_of_previous_year"):
        review_register_file(path, YEAR)


def test_the_same_register_is_accepted_for_the_gate(tmp_path):
    path = tmp_path / "gate_only.csv"
    path.write_text(",".join(GATE_COLUMNS) + "\nL-1,true,true,7,false,unknown,0.0827,2023-24\n",
                    encoding="utf-8")
    report = review_register_file(path, YEAR, gate_only=True)
    assert report.lines[0].gate.verdict is GateVerdict.COMPLYING
    assert report.lines[0].myr is None


def test_a_header_with_no_rows_is_refused(tmp_path):
    path = tmp_path / "empty.csv"
    path.write_text(",".join(ALL_COLUMNS) + "\n", encoding="utf-8")
    with pytest.raises(RegisterError, match="no loan rows"):
        load_rows(path, ALL_COLUMNS)


def test_a_missing_file_is_refused(tmp_path):
    with pytest.raises(RegisterError, match="cannot read"):
        load_rows(tmp_path / "nope.csv", ALL_COLUMNS)


def test_a_byte_order_mark_does_not_break_the_first_column(tmp_path):
    """Excel writes UTF-8 with a BOM, which would otherwise make the first
    column name unrecognisable and refuse a perfectly good register."""
    path = tmp_path / "bom.csv"
    path.write_text(",".join(GATE_COLUMNS) + "\nL-1,true,true,7,false,unknown,0.0827,2023-24\n",
                    encoding="utf-8-sig")
    assert load_rows(path, GATE_COLUMNS)[0]["loan_id"] == "L-1"


def test_a_quoted_multiline_value_preserves_its_newline(tmp_path):
    path = tmp_path / "multiline.csv"
    with path.open("w", encoding="utf-8", newline="") as handle:
        handle.write('loan_id,borrower_reference\nL-1,"first line\nsecond line"\n')
    assert load_rows(path, ("loan_id", "borrower_reference"))[0][
        "borrower_reference"
    ] == "first line\nsecond line"


def test_a_multiline_numeric_cell_is_not_joined_into_a_valid_amount(tmp_path):
    path = tmp_path / "multiline-number.csv"
    malformed = row(payments_applied_during_the_year="25\n556.00")
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=malformed)
        writer.writeheader()
        writer.writerow(malformed)

    with pytest.raises(RegisterError, match="payments_applied_during_the_year"):
        review_register_file(path, YEAR)


# --- verdicts and counts ----------------------------------------------


def test_a_clean_register_needs_no_attention():
    report = review_register([row()], YEAR)
    assert report.summary["COMPLYING"] == 1
    assert report.summary["MYR_MET"] == 1
    assert report.needs_attention is False
    assert report.total_exposure == D("0.00")


def test_a_short_row_reports_exposure():
    report = review_register([row(payments_applied_during_the_year="20000.00")], YEAR)
    assert report.summary["MYR_SHORT"] == 1
    assert report.needs_attention is True
    assert report.total_exposure == D("5556.00")


def test_counts_cover_every_summary_key():
    from div7aloan.verdicts import SUMMARY_KEYS

    report = review_register([row()], YEAR)
    assert set(report.summary) == set(SUMMARY_KEYS)


def test_a_row_marked_out_of_scope_is_skipped_not_reviewed():
    report = review_register([row(out_of_scope_reason="s 109XA UPE")], YEAR)
    assert report.summary["SKIPPED"] == 1
    assert report.rows_reviewed == 0
    line = report.lines[0]
    assert line.is_skipped
    assert "s 109XA UPE" in line.skipped_reason
    assert line.gate is None


def test_a_loan_made_before_4_december_1997_is_skipped():
    report = review_register([row(year_loan_made="1996-97")], YEAR)
    assert report.summary["SKIPPED"] == 1
    assert "4 December 1997" in report.lines[0].skipped_reason
    assert "109D(5)" in report.lines[0].skipped_reason


def test_the_year_straddling_4_december_1997_is_also_skipped():
    """1997-98 contains 4 December 1997, and a year label cannot say which
    side of it the loan falls on."""
    report = review_register([row(year_loan_made="1997-98")], YEAR)
    assert report.summary["SKIPPED"] == 1


def test_the_first_reviewable_year_is_not_skipped():
    report = review_register([row(year_loan_made="1998-99")], YEAR)
    assert report.summary["SKIPPED"] == 0
    assert report.rows_reviewed == 1


def test_a_non_complying_row_refuses_the_repayment():
    report = review_register(
        [row(interest_rate_for_years_after_year_loan_made="0.0100")], YEAR
    )
    assert report.summary["NOT_COMPLYING"] == 1
    assert report.summary["REFUSED"] == 1
    assert report.lines[0].myr.myr_required is None


def test_an_unknown_gate_counts_once_as_unknown_and_once_as_refused():
    """A reviewed row answers two questions, so it can contribute to two
    counts. That is why the counts are documented as per question."""
    report = review_register([row(written_agreement="unknown")], YEAR)
    assert report.summary["UNKNOWN"] == 1
    assert report.summary["REFUSED"] == 1
    assert report.rows_reviewed == 1


def test_a_bad_cell_is_reported_against_its_row(tmp_path):
    path = tmp_path / "bad.csv"
    header = ",".join(ALL_COLUMNS)
    good = "L-1,2023-24,true,true,7,false,unknown,0.0827,100000.00,5,25556.00"
    bad = "L-2,2023-24,ture,true,7,false,unknown,0.0827,100000.00,5,25556.00"
    order = ["loan_id", "year_loan_made", "written_agreement",
             "terms_in_place_before_lodgment_day", "maximum_term_years",
             "secured_by_registered_mortgage_over_real_property",
             "security_coverage_at_first_made",
             "interest_rate_for_years_after_year_loan_made",
             "amalgamated_loan_unpaid_at_end_of_previous_year",
             "remaining_term_years", "payments_applied_during_the_year"]
    path.write_text(",".join(order) + "\n" + good + "\n" + bad + "\n", encoding="utf-8")
    with pytest.raises(RegisterError, match="row 2"):
        review_register_file(path, YEAR)
    assert header  # the canonical order is documented in the README


# --- ordering ----------------------------------------------------------


def test_exposure_comes_first_largest_shortfall_first():
    rows = [
        row(loan_id="small", payments_applied_during_the_year="25000.00"),
        row(loan_id="clean"),
        row(loan_id="large", payments_applied_during_the_year="1000.00"),
    ]
    report = review_register(rows, YEAR)
    assert [line.loan_id for line in report.lines][:2] == ["large", "small"]
    assert report.lines[0].shortfall > report.lines[1].shortfall


def test_clean_rows_sit_above_skipped_ones():
    rows = [
        row(loan_id="skipped", out_of_scope_reason="out of scope"),
        row(loan_id="clean"),
        row(loan_id="undecided", written_agreement="unknown"),
    ]
    report = review_register(rows, YEAR)
    assert [line.loan_id for line in report.lines] == ["undecided", "clean", "skipped"]


def test_ties_keep_the_registers_own_order():
    rows = [row(loan_id=f"L-{n}") for n in range(1, 5)]
    report = review_register(rows, YEAR)
    assert [line.loan_id for line in report.lines] == ["L-1", "L-2", "L-3", "L-4"]


# --- the shipped samples ----------------------------------------------


def test_the_clean_sample_runs_to_a_verdict_with_no_flags():
    report = review_register_file(MET, YEAR)
    assert report.rows_reviewed == 3
    assert report.summary["COMPLYING"] == 3
    assert report.summary["MYR_MET"] == 3
    assert report.summary["UNKNOWN"] == 0
    assert report.summary["REFUSED"] == 0
    assert report.summary["SKIPPED"] == 0
    assert report.needs_attention is False
    assert report.total_exposure == D("0.00")


def test_the_mixed_sample_carries_unknowns_and_a_non_complying_loan():
    report = review_register_file(MIXED, YEAR)
    assert report.summary["NOT_COMPLYING"] >= 1
    assert report.summary["UNKNOWN"] >= 1
    assert report.summary["SKIPPED"] >= 1
    assert report.summary["MYR_SHORT"] >= 1
    assert report.needs_attention is True


def test_the_mixed_sample_leads_with_its_exposure():
    report = review_register_file(MIXED, YEAR)
    assert report.lines[0].myr.verdict is MyrVerdict.MYR_SHORT
    assert report.lines[0].shortfall == D("2212.37")


def test_the_gate_can_be_anchored_to_a_later_year():
    """A loan written at the 2021-22 benchmark of 0.0452 is complying on the
    Act's own anchor, and falls short of the 2026-27 benchmark on the
    later-year practice check."""
    facts = row(year_loan_made="2021-22", interest_rate_for_years_after_year_loan_made="0.0452")
    on_the_act = review_register([facts], YEAR)
    assert on_the_act.lines[0].gate.verdict is GateVerdict.COMPLYING
    assert on_the_act.lines[0].gate.benchmark_year_used == "2021-22"

    later = review_register([facts], YEAR, gate_benchmark_year=YEAR)
    assert later.lines[0].gate.verdict is GateVerdict.NOT_COMPLYING
    assert later.lines[0].gate.benchmark_year_used == "2026-27"
    assert any("109N(1)(b)" in caveat for caveat in later.lines[0].gate.caveats)


def test_total_exposure_is_a_decimal():
    report = review_register_file(MIXED, YEAR)
    assert isinstance(report.total_exposure, Decimal)
