"""Command-line behaviour for the 2 subcommands the README quick start runs."""

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
        "--pre-18-expenses",
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


def test_a_narrow_console_codepage_does_not_end_the_run():
    # cp437 holds neither the report's dash nor this beneficiary's name, which ended
    # the run with a UnicodeEncodeError after the work was done.
    import os
    import subprocess
    import sys

    result = subprocess.run(
        [sys.executable, "-c", "import sys; from louisgoldberg.cli import main; sys.exit(main())",
         "s100a-check", "--beneficiary", "Zoë Ŧrust", "--amount", "100"],
        capture_output=True, env={**os.environ, "PYTHONIOENCODING": "cp437"}, check=False,
    )
    assert result.returncode == 0, result.stderr.decode("cp437", "replace")
    assert b"Section 100A Risk Evaluation" in result.stdout


def test_s100a_check_takes_the_stated_receipt_from_the_other_flag(monkeypatch, capsys):
    # Same distribution, the opposite member of the receipt group, and the zone
    # moves. That is the flag reaching the evaluation rather than being dropped.
    # The green zone turns on every fact, so every fact is stated.
    code = run(
        monkeypatch,
        "s100a-check",
        "--beneficiary", "Adult Child",
        "--amount", "40000",
        "--received-funds",
        "--no-adult-child",
        "--no-retained-by-parents",
        "--no-circular",
        "--no-corporate-upe",
        "--no-direct-benefit",
        "--no-commercial-loan",
        "--within-two-years",
        "--no-pre-18-expenses",
        "--no-retention-conditions",
        "--no-para-32-exclusion",
    )
    out = capsys.readouterr().out

    assert code == 0
    assert "Risk Zone:               GREEN" in out
    assert "Facts Not Established:" not in out


def test_s100a_check_reports_the_facts_it_was_not_given(monkeypatch, capsys):
    # Unstated facts used to arrive as False, so a single mitigating flag
    # returned GREEN on an arrangement nobody had described.
    code = run(
        monkeypatch,
        "s100a-check",
        "--beneficiary", "Adult Child",
        "--amount", "40000",
        "--received-funds",
    )
    out = capsys.readouterr().out

    assert code == 0
    assert "Risk Zone:               FACTS_NOT_ESTABLISHED" in out
    assert "Facts Not Established:   beneficiary_is_adult_child" in out
    assert "commercial_loan_agreement_in_place" in out
    assert "GREEN" not in out


def test_s99b_check_deducts_the_corpus_exemption(monkeypatch, capsys):
    code = run(
        monkeypatch,
        "s99b-check",
        "--beneficiary", "Jane Doe",
        "--gross", "150000",
        "--corpus", "50000",
        "--resident-during-year",
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


def test_an_s100a_amount_too_large_to_print_is_one_error_line(monkeypatch, capsys):
    code = run(
        monkeypatch,
        "s100a-check",
        "--beneficiary", "Adult Child",
        "--amount", "1e1000000",
        "--adult-child",
    )

    captured = capsys.readouterr()
    assert code == 2
    assert captured.out == ""
    assert captured.err == (
        "error: distribution amount 1E+1000000 is outside the supported range; "
        "amounts must be below 1E+1000000\n"
    )


@pytest.mark.parametrize("amount", ["1e1000000", "1e999999999"])
def test_an_s99b_add_back_that_cancels_the_corpus_is_still_range_checked(monkeypatch, capsys, amount):
    # Equal amounts cancel without a decimal signal, and the basis would write the
    # add-back out in fixed point.
    code = run(
        monkeypatch,
        "s99b-check",
        "--beneficiary", "X",
        "--gross", "0",
        "--corpus", amount,
        "--corpus-attributable", amount,
        "--resident-during-year",
    )

    captured = capsys.readouterr()
    assert code == 2
    assert captured.out == ""
    assert captured.err == (
        f"error: corpus_attributable_to_notional_assessable_income_aud {Decimal(amount)} "
        "is outside the supported range; amounts must be below 1E+1000000\n"
    )


def test_a_cancelling_add_back_at_the_supported_limit_still_evaluates(monkeypatch, capsys):
    code = run(
        monkeypatch,
        "s99b-check",
        "--beneficiary", "X",
        "--gross", "0",
        "--corpus", "9e999999",
        "--corpus-attributable", "9e999999",
        "--resident-during-year",
    )

    assert code == 0
    assert "Assessable under s99B:   $0.00" in capsys.readouterr().out


def test_amounts_whose_arithmetic_overflows_are_one_error_line(monkeypatch, capsys):
    # Each amount is finite and accepted; their sum is not representable.
    code = run(
        monkeypatch,
        "s99b-check",
        "--beneficiary", "Jane Doe",
        "--gross", "9e999999",
        "--corpus", "9e999999",
        "--not-assessable-to-resident", "9e999999",
        "--resident-during-year",
    )

    captured = capsys.readouterr()
    assert code == 2
    assert captured.out == ""
    assert captured.err == (
        "error: the supplied amounts produce a value outside the decimal arithmetic "
        "range this command supports\n"
    )


@pytest.mark.parametrize("argv,message", [
    # Before Python 3.14, argparse reads a separate "-1e999999999" as an option flag.
    (("s100a-check", "--beneficiary", "X", "--amount=-1e999999999", "--adult-child"),
     "error: distribution amount must be positive and finite"),
    (("s99b-check", "--beneficiary", "X", "--gross", "1e999999999"),
     "error: residency during the year of income is not established"),
    (("s99b-check", "--beneficiary", "X", "--gross", "1e999999999", "--not-resident-during-year"),
     "error: s 99B(1) ITAA 1936 applies only where the beneficiary was a resident"),
    (("s99b-check", "--beneficiary", "X", "--gross", "0", "--corpus", "0",
      "--corpus-attributable", "1e999999999", "--resident-during-year"),
     "error: corpus attributable to notionally assessable income cannot exceed the corpus amount"),
    (("s99b-check", "--beneficiary", "X", "--gross", "0", "--corpus", "1e999999999",
      "--corpus-attributable", "1e999999999", "--not-assessable-to-resident", "1",
      "--resident-during-year"),
     "error: exemptions ($1.00) exceed the gross receipt ($0.00)"),
], ids=["negative-s100a", "residency-unstated", "not-resident", "attributable-over-corpus",
        "exemptions-over-gross"])
def test_existing_validation_still_answers_first_for_astronomical_amounts(monkeypatch, capsys, argv, message):
    assert run(monkeypatch, *argv) == 2
    captured = capsys.readouterr()
    assert captured.out == ""
    assert captured.err.startswith(message)
    assert captured.err.count("\n") == 1


@pytest.mark.parametrize("argv", [
    ("s100a-check", "--beneficiary", "Adult Child", "--amount", "1000000000000.01", "--adult-child"),
    ("s99b-check", "--beneficiary", "Jane Doe", "--gross", "1000000000000.01", "--resident-during-year"),
], ids=["s100a-amount", "s99b-gross"])
def test_amounts_above_one_trillion_still_evaluate(monkeypatch, capsys, argv):
    assert run(monkeypatch, *argv) == 0
    assert "1,000,000,000,000.01" in capsys.readouterr().out


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


def test_s99b_check_refuses_a_run_that_does_not_state_residency(monkeypatch, capsys):
    # s 99B(1) turns on residency. Without the flag the engine has no fact, and
    # an error line beats an assessable amount computed on an assumption.
    code = run(
        monkeypatch,
        "s99b-check",
        "--beneficiary", "Jane Doe",
        "--gross", "150000",
    )

    assert code == 2
    assert "residency during the year of income is not established" in capsys.readouterr().err


def test_s99b_check_names_the_nil_exemptions_it_was_given(monkeypatch, capsys):
    code = run(
        monkeypatch,
        "s99b-check",
        "--beneficiary", "Jane Doe",
        "--gross", "150000",
        "--corpus", "50000",
        "--resident-during-year",
    )
    out = capsys.readouterr().out

    assert code == 0
    assert "Caveat:" in out
    assert "not_assessable_to_resident_aud" in out
