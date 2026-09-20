import csv
import json
from datetime import date
from decimal import Decimal

import pytest
from paydaysuper import rates as rates_module
from paydaysuper.rates import RatesError, load_gic


def write_table(tmp_path, quarters):
    (tmp_path / "gic_rates.json").write_text(
        json.dumps({"quarters": quarters}), encoding="utf-8"
    )
    return tmp_path


GOOD = {"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "11.43", "seen": "2026-08-02"}


def test_a_good_table_still_loads(tmp_path, monkeypatch):
    monkeypatch.setattr(rates_module, "DATA_DIR", write_table(tmp_path, [GOOD]))
    table = load_gic()
    assert table.last_known == date(2026, 9, 30)
    assert table.daily_rate(date(2026, 8, 1)) == Decimal("11.43") / 100 / 365


def test_unsorted_contiguous_quarters_are_sorted_and_accepted(tmp_path, monkeypatch):
    following = dict(GOOD, **{"from": "2026-10-01", "to": "2026-12-31"})
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [following, GOOD])
    )

    table = load_gic()

    assert table.last_known == date(2026, 12, 31)
    assert table.daily_rate(date(2026, 9, 30)) == Decimal("11.43") / 100 / 365


def test_a_reversed_interval_is_refused(tmp_path, monkeypatch):
    reversed_quarter = dict(GOOD, **{"from": "2026-09-30", "to": "2026-07-01"})
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [reversed_quarter])
    )

    with pytest.raises(RatesError, match="ends before it starts") as exc:
        load_gic()

    assert "2026-09-30 to 2026-07-01" in str(exc.value)


@pytest.mark.parametrize(
    ("following_start", "relation"),
    [("2026-09-30", "overlap"), ("2026-10-02", "gap")],
)
def test_non_contiguous_intervals_name_both_ranges(
    tmp_path, monkeypatch, following_start, relation
):
    following = dict(GOOD, **{"from": following_start, "to": "2026-12-31"})
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [GOOD, following])
    )

    with pytest.raises(RatesError, match=relation) as exc:
        load_gic()

    message = str(exc.value)
    assert "2026-07-01 to 2026-09-30" in message
    assert f"{following_start} to 2026-12-31" in message


def test_an_unreadable_rate_raises_rates_error_naming_the_entry(tmp_path, monkeypatch):
    """decimal.InvalidOperation is an ArithmeticError, so an unguarded
    Decimal() here escaped the CLI's handler and printed a traceback."""
    bad = dict(GOOD, annual_pct="eleven point four")
    monkeypatch.setattr(rates_module, "DATA_DIR", write_table(tmp_path, [bad]))
    with pytest.raises(RatesError) as exc:
        load_gic()
    message = str(exc.value)
    assert "eleven point four" in message
    assert "2026-07-01 to 2026-09-30" in message


@pytest.mark.parametrize("value", ["nan", "NaN", "Infinity", "-Infinity", "sNaN"])
def test_a_non_finite_rate_is_refused(tmp_path, monkeypatch, value):
    """Decimal builds these happily, and every money figure downstream would
    come out nan without a word said."""
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [dict(GOOD, annual_pct=value)])
    )
    with pytest.raises(RatesError) as exc:
        load_gic()
    assert "finite" in str(exc.value)


def test_a_negative_rate_is_refused(tmp_path, monkeypatch):
    """A stray minus sign printed notional earnings of $-112.12 and an SG
    charge estimate running from $-112.12 to $-179.39, bounds inverted."""
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [dict(GOOD, annual_pct="-11.43")])
    )
    with pytest.raises(RatesError) as exc:
        load_gic()
    message = str(exc.value)
    assert "-11.43" in message
    assert "2026-07-01 to 2026-09-30" in message
    assert "negative" in message


def test_a_rate_above_the_ceiling_is_refused(tmp_path, monkeypatch):
    """A dropped decimal point turned 11.43 into 1143, which billed 612934.92
    of notional earnings on a 10000.00 shortfall."""
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [dict(GOOD, annual_pct="1143")])
    )
    with pytest.raises(RatesError) as exc:
        load_gic()
    message = str(exc.value)
    assert "1143" in message
    assert "2026-07-01 to 2026-09-30" in message
    assert "100%" in message


def test_a_zero_rate_is_accepted(tmp_path, monkeypatch):
    """The boundary. Zero is not a sign error, so the guard must test < 0,
    not <= 0, or a quarter with no interest stops the whole run."""
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [dict(GOOD, annual_pct="0")])
    )
    table = load_gic()
    assert table.daily_rate(date(2026, 8, 1)) == Decimal("0")


def test_the_ceiling_itself_is_accepted(tmp_path, monkeypatch):
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [dict(GOOD, annual_pct="100")])
    )
    assert load_gic().daily_rate(date(2026, 8, 1)) == Decimal("100") / 100 / 365


def test_a_missing_quarters_key_is_named(tmp_path, monkeypatch):
    """The hardening covered every field of every quarter and left the key
    they hang off unguarded: a renamed 'quarters' raised KeyError, which the
    CLI's handler tuple does not catch."""
    (tmp_path / "gic_rates.json").write_text(
        json.dumps({"gic_quarters": [GOOD]}), encoding="utf-8"
    )
    monkeypatch.setattr(rates_module, "DATA_DIR", tmp_path)
    with pytest.raises(RatesError) as exc:
        load_gic()
    message = str(exc.value)
    assert "quarters" in message
    assert "gic_rates.json" in message


def test_a_list_at_the_top_level_is_refused(tmp_path, monkeypatch):
    """A JSON list gave TypeError, which the CLI does not catch either."""
    (tmp_path / "gic_rates.json").write_text(json.dumps([GOOD]), encoding="utf-8")
    monkeypatch.setattr(rates_module, "DATA_DIR", tmp_path)
    with pytest.raises(RatesError) as exc:
        load_gic()
    assert "must be a JSON object" in str(exc.value)


def test_quarters_must_be_a_list(tmp_path, monkeypatch):
    (tmp_path / "gic_rates.json").write_text(
        json.dumps({"quarters": GOOD}), encoding="utf-8"
    )
    monkeypatch.setattr(rates_module, "DATA_DIR", tmp_path)
    with pytest.raises(RatesError) as exc:
        load_gic()
    assert "must be a list" in str(exc.value)


def test_a_table_that_is_not_json_is_a_rates_error_naming_the_file(
    tmp_path, monkeypatch
):
    """json.JSONDecodeError is a ValueError, so the CLI already printed
    "error: ..." for a hand-edit that broke the JSON itself, but the message
    was a bare parse error with no path in it. Re-raised as RatesError naming
    the file, the way profiles.load_profiles already does."""
    (tmp_path / "gic_rates.json").write_text('{"quarters": [,]}', encoding="utf-8")
    monkeypatch.setattr(rates_module, "DATA_DIR", tmp_path)
    with pytest.raises(RatesError) as exc:
        load_gic()
    message = str(exc.value)
    assert "is not valid JSON" in message
    assert "gic_rates.json" in message


def test_the_cli_prints_a_top_level_rates_error_without_a_traceback(
    tmp_path, monkeypatch, capsys
):
    from paydaysuper.cli import EXIT_ERROR, main

    from conftest import SAMPLE

    (tmp_path / "gic_rates.json").write_text(
        json.dumps({"gic_quarters": [GOOD]}), encoding="utf-8"
    )
    monkeypatch.setattr(rates_module, "DATA_DIR", tmp_path)
    assert main([str(SAMPLE), "-o", str(tmp_path / "r.csv"), "--as-at", "2026-08-10"]) == (
        EXIT_ERROR
    )
    err = capsys.readouterr().err
    assert err.startswith("error: ")
    assert "Traceback" not in err
    assert "quarters" in err


def test_a_missing_rate_key_is_named(tmp_path, monkeypatch):
    entry = {k: v for k, v in GOOD.items() if k != "annual_pct"}
    monkeypatch.setattr(rates_module, "DATA_DIR", write_table(tmp_path, [entry]))
    with pytest.raises(RatesError) as exc:
        load_gic()
    assert "annual_pct" in str(exc.value)


def test_a_null_rate_is_refused(tmp_path, monkeypatch):
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [dict(GOOD, annual_pct=None)])
    )
    with pytest.raises(RatesError):
        load_gic()


def test_an_unreadable_quarter_date_is_named(tmp_path, monkeypatch):
    monkeypatch.setattr(
        rates_module, "DATA_DIR", write_table(tmp_path, [dict(GOOD, to="30/09/2026")])
    )
    with pytest.raises(RatesError) as exc:
        load_gic()
    assert "YYYY-MM-DD" in str(exc.value)


def test_the_cli_prints_a_rate_error_without_a_traceback(tmp_path, monkeypatch, capsys):
    from paydaysuper.cli import EXIT_ERROR, main

    from conftest import SAMPLE

    monkeypatch.setattr(
        rates_module,
        "DATA_DIR",
        write_table(tmp_path, [dict(GOOD, annual_pct="11,43")]),
    )
    assert main([str(SAMPLE), "-o", str(tmp_path / "r.csv"), "--as-at", "2026-08-10"]) == (
        EXIT_ERROR
    )
    err = capsys.readouterr().err
    assert err.startswith("error: ")
    assert "Traceback" not in err


def test_a_day_past_the_last_quarter_is_refused_by_default(tmp_path, monkeypatch):
    """Reusing the last known rate behind a staleness warning put a rate the
    ATO has not published into both SG-charge exposure totals. The estimate is
    now something the operator asks for."""
    monkeypatch.setattr(rates_module, "DATA_DIR", write_table(tmp_path, [GOOD]))
    table = load_gic()
    beyond = date(2026, 12, 1)
    assert beyond > table.last_known

    with pytest.raises(RatesError) as exc:
        table.daily_rate(beyond)
    message = str(exc.value)
    assert "2026-12-01" in message
    assert table.last_known.isoformat() in message
    assert "--allow-stale-gic" in message

    assert table.daily_rate(beyond, allow_stale=True) > Decimal("0")


def test_the_cli_reports_a_row_the_gic_table_cannot_reach(tmp_path, monkeypatch, capsys):
    """The console path. A period past the published quarters withholds the
    charge estimate for that row and keeps everything the GIC table has no part
    in: the run still succeeds, the verdict stands, and the caveat names the
    last quarter on record and the flag that buys the estimate."""
    from paydaysuper.cli import EXIT_ERROR, main

    from conftest import SAMPLE

    monkeypatch.setattr(rates_module, "DATA_DIR", write_table(tmp_path, [GOOD]))
    # rates.json lives beside gic_rates.json, so copy the real one across.
    (tmp_path / "rates.json").write_text(
        (rates_module.PACKAGE_DIR / "data" / "rates.json").read_text(encoding="utf-8"),
        encoding="utf-8",
    )

    report = tmp_path / "r.csv"
    argv = [
        str(SAMPLE),
        "-o",
        str(report),
        "--as-at",
        "2027-06-30",
        "--confirm-transition-allocation",
    ]
    assert main(argv) != EXIT_ERROR
    out = capsys.readouterr().out
    assert "notional earnings and SG charge estimate not assessed" in out
    assert "carry no charge estimate" in out
    assert "2026-09-30" in out
    assert "--allow-stale-gic" in out

    exposed = [
        row
        for row in csv.DictReader(report.read_text(encoding="utf-8-sig").splitlines())
        if row["verdict"] in {"LATE", "UNPAID"}
    ]
    assert exposed, "the sample file has no exposed row to report"
    charge_columns = (
        "notional_earnings",
        "uplift_best_case",
        "uplift_worst_case",
        "sgc_estimate_low",
        "sgc_estimate_high",
    )
    unassessed = []
    for row in exposed:
        # Every exposed row keeps its shortfall, which rests on the deadline
        # and the receipt facts. A row whose notional earnings period ran past
        # 30 Sep 2026 loses all 5 charge figures together, and only those.
        assert row["final_shortfall"]
        blanks = [column for column in charge_columns if row[column] == ""]
        assert not blanks or len(blanks) == len(charge_columns), blanks
        if blanks:
            unassessed.append(row)
            assert "not assessed for this row" in row["caveats"]
        else:
            assert "not assessed for this row" not in row["caveats"]
    assert unassessed, "no row ran past the 1-quarter table this test installed"

    with_flag = tmp_path / "r2.csv"
    argv[2] = str(with_flag)
    assert main(argv + ["--allow-stale-gic"]) != EXIT_ERROR
    estimated = [
        row
        for row in csv.DictReader(with_flag.read_text(encoding="utf-8-sig").splitlines())
        if row["verdict"] in {"LATE", "UNPAID"}
    ]
    assert estimated and all(row["notional_earnings"] for row in estimated)
    assert any("GIC rate table ends" in row["caveats"] for row in estimated)
    assert not any("not assessed for this row" in row["caveats"] for row in estimated)


def test_a_row_past_the_table_keeps_its_verdict_and_loses_only_the_estimate():
    """The same shape as a deadline past the holiday calendar's coverage. The
    verdict and the shortfall rest on the deadline and the receipt facts, which
    the GIC table has no part in, so withholding them would discard a shortfall
    the run established."""
    from paydaysuper.assess import assess
    from paydaysuper.calendar import load_calendar
    from paydaysuper.deadlines import ContribLine

    line = ContribLine("E1", date(2026, 8, 3), Decimal("600.00"), row=2)
    unassessed = assess([line], load_calendar(), load_gic(), date(2027, 6, 30))[0]

    assert unassessed.verdict == "UNPAID"
    assert unassessed.days_late == (date(2027, 6, 30) - unassessed.deadline.due).days
    assert unassessed.base_shortfall == Decimal("600.00")
    assert unassessed.final_shortfall == Decimal("600.00")
    assert unassessed.nec is None
    assert unassessed.uplift is None
    assert unassessed.sgc_low is None and unassessed.sgc_high is None

    caveat = next(
        c for c in unassessed.caveats if "not assessed for this row" in c
    )
    assert load_gic().last_known.isoformat() in caveat
    assert "--allow-stale-gic" in caveat

    estimated = assess(
        [line], load_calendar(), load_gic(), date(2027, 6, 30), allow_stale_gic=True
    )[0]
    assert estimated.verdict == unassessed.verdict
    assert estimated.final_shortfall == unassessed.final_shortfall
    assert estimated.nec is not None and estimated.nec > Decimal("0")
    assert estimated.sgc_high is not None
    assert any("GIC rate table ends" in c for c in estimated.caveats)
    assert not any("not assessed for this row" in c for c in estimated.caveats)


def test_a_period_inside_the_table_is_unaffected():
    """The other side of the boundary, so the pair pins the condition rather
    than only its effect."""
    from paydaysuper.assess import assess
    from paydaysuper.calendar import load_calendar
    from paydaysuper.deadlines import ContribLine

    line = ContribLine("E1", date(2026, 8, 3), Decimal("600.00"), row=2)
    result = assess([line], load_calendar(), load_gic(), load_gic().last_known)[0]

    assert result.verdict == "UNPAID"
    assert result.nec is not None
    assert result.sgc_high is not None
    assert not any("not assessed for this row" in c for c in result.caveats)


def test_a_withheld_estimate_does_not_also_claim_the_rate_was_carried_forward():
    """The staleness caveat says days past the table use the last known rate.
    On a row that withheld the estimate that would contradict the caveat beside
    it, so it is only added where the estimate was actually produced."""
    from paydaysuper.assess import assess
    from paydaysuper.calendar import load_calendar
    from paydaysuper.deadlines import ContribLine

    line = ContribLine("E1", date(2026, 8, 3), Decimal("600.00"), row=2)
    withheld = assess([line], load_calendar(), load_gic(), date(2027, 6, 30))[0]
    assert not any("use the last known rate" in c for c in withheld.caveats)

    estimated = assess(
        [line], load_calendar(), load_gic(), date(2027, 6, 30), allow_stale_gic=True
    )[0]
    assert any("use the last known rate" in c for c in estimated.caveats)
