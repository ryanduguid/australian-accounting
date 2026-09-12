"""Command-line behaviour for the two subcommands the README quick start runs."""

import argparse
from decimal import Decimal

import pytest
from louisgoldberg.cli import decimal_type, main


def run(monkeypatch, *argv):
    monkeypatch.setattr("sys.argv", ["solomons-sword", *argv])
    return main()


def test_s100a_check_reports_the_risk_zone_and_the_factors(monkeypatch, capsys):
    # The README quick start invocation, argument for argument.
    code = run(
        monkeypatch,
        "s100a-check",
        "--beneficiary", "Adult Child",
        "--amount", "40000",
        "--adult-child",
        "--retained-by-parents",
    )
    out = capsys.readouterr().out

    assert code == 0
    assert "Section 100A Risk Evaluation" in out
    assert "Adult Child" in out
    assert "Distribution Amount:     $40,000.00" in out
    assert "Risk Zone:               RED" in out
    # None is the model's undecided, and has to read as that on a workpaper.
    assert "Ordinary Family Dealing: Not determined" in out
    assert "Output contains client data." in out


def test_s100a_check_takes_the_stated_receipt_from_the_other_flag(monkeypatch, capsys):
    # Same distribution, the opposite member of the receipt group, and the zone
    # moves. That is the flag reaching the evaluation rather than being dropped.
    code = run(
        monkeypatch,
        "s100a-check",
        "--beneficiary", "Adult Child",
        "--amount", "40000",
        "--received-funds",
    )
    out = capsys.readouterr().out

    assert code == 0
    assert "Risk Zone:               GREEN" in out


def test_s99b_check_deducts_the_corpus_exemption(monkeypatch, capsys):
    code = run(
        monkeypatch,
        "s99b-check",
        "--beneficiary", "Jane Doe",
        "--gross", "150000",
        "--corpus", "50000",
    )
    out = capsys.readouterr().out

    assert code == 0
    assert "Section 99B Assessment — Jane Doe" in out
    assert "Gross Receipt:           $150,000.00" in out
    assert "Corpus Exemption:        $50,000.00" in out
    assert "Assessable under s99B:   $100,000.00" in out


def test_the_two_receipt_flags_cannot_both_be_given(monkeypatch, capsys):
    # They state opposite facts about the same event. argparse refuses the pair
    # before any evaluation runs, so neither one silently wins.
    with pytest.raises(SystemExit) as exit_info:
        run(
            monkeypatch,
            "s100a-check",
            "--beneficiary", "Adult Child",
            "--amount", "40000",
            "--received-funds",
            "--funds-not-received",
        )

    assert exit_info.value.code == 2
    assert "not allowed with argument" in capsys.readouterr().err


@pytest.mark.parametrize("raw", ["not-a-number", "40,000", ""])
def test_decimal_type_refuses_what_is_not_a_decimal(raw):
    with pytest.raises(argparse.ArgumentTypeError, match="not a decimal amount"):
        decimal_type(raw)


@pytest.mark.parametrize("raw", ["NaN", "Infinity", "-Infinity"])
def test_decimal_type_refuses_the_non_finite_amounts(raw):
    # Decimal parses these cleanly and they then compare false against every
    # threshold, so a NaN distribution would evaluate to a clean risk zone.
    with pytest.raises(argparse.ArgumentTypeError, match="not a finite decimal amount"):
        decimal_type(raw)


def test_decimal_type_keeps_the_exact_decimal():
    value = decimal_type("40000.50")
    assert value == Decimal("40000.50")
    assert type(value) is Decimal


def test_a_money_argument_that_is_not_a_decimal_stops_the_run(monkeypatch, capsys):
    with pytest.raises(SystemExit) as exit_info:
        run(
            monkeypatch,
            "s100a-check",
            "--beneficiary", "Adult Child",
            "--amount", "forty thousand",
        )

    assert exit_info.value.code == 2
    assert "not a decimal amount" in capsys.readouterr().err


def test_a_refused_amount_is_reported_as_an_error_line_not_a_traceback(monkeypatch, capsys):
    # The engine refuses a non-positive distribution. The CLI has to turn that
    # into an error line, because a traceback on a workpaper run tells the
    # operator nothing about which figure was wrong.
    code = run(
        monkeypatch,
        "s100a-check",
        "--beneficiary", "Adult Child",
        "--amount", "0",
    )

    assert code == 2
    assert "error: distribution amount must be positive and finite" in capsys.readouterr().err


def test_no_subcommand_prints_the_help_and_fails(monkeypatch, capsys):
    code = run(monkeypatch)
    out = capsys.readouterr().out

    assert code == 1
    assert "s100a-check" in out
    assert "s99b-check" in out
