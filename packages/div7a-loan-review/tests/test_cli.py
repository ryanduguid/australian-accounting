"""The command line, its exit codes, and the shape of its JSON.

The no-float tests here are the load-bearing ones. Every amount that leaves
this engine must be a quoted decimal string: a JSON number would be parsed by
almost every reader as an IEEE double, and the MCP adapter this engine is
built for would then be handing a float to a tax calculation.
"""
from __future__ import annotations

import contextlib
import io
import json

import pytest

from div7aloan.cli import main

MET = "examples/sample_loans_myr_met.csv"
MIXED = "examples/sample_loans_mixed.csv"

JSON_COMMANDS = [
    ["rate", "--year", "2026-27"],
    ["rate", "--year", "2027-28"],
    ["gate", "--input", MIXED],
    ["gate", "--input", MIXED, "--year", "2026-27"],
    ["myr", "--input", MIXED, "--year", "2026-27"],
    ["review", "--input", MIXED, "--year", "2026-27"],
    ["review", "--input", MET, "--year", "2026-27"],
]


def run(argv):
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = main(argv)
    return code, out.getvalue(), err.getvalue()


def run_json(argv):
    code, out, _ = run(argv + ["--format", "json"])
    return code, json.loads(out)


# --- no floats ---------------------------------------------------------


def _float_paths(value, path="$"):
    if isinstance(value, float):
        return [path]
    if isinstance(value, dict):
        return [p for k, v in value.items() for p in _float_paths(v, f"{path}.{k}")]
    if isinstance(value, list):
        return [p for n, v in enumerate(value) for p in _float_paths(v, f"{path}[{n}]")]
    return []


@pytest.mark.parametrize("argv", JSON_COMMANDS, ids=lambda a: " ".join(a))
def test_json_output_carries_no_floats(argv):
    _, document = run_json(argv)
    assert _float_paths(document) == []


@pytest.mark.parametrize("argv", JSON_COMMANDS, ids=lambda a: " ".join(a))
def test_no_amount_is_written_as_a_json_number(argv):
    """Stronger than checking the parsed document: this fails if the raw text
    ever holds a bare decimal literal, whatever the parser would make of it."""
    seen = []

    def refuse(text):
        seen.append(text)
        return text

    _, out, _ = run(argv + ["--format", "json"])
    json.loads(out, parse_float=refuse)
    assert seen == [], f"decimal literals emitted as JSON numbers: {seen}"


def test_amounts_are_strings_with_two_decimal_places():
    _, document = run_json(["review", "--input", MIXED, "--year", "2026-27"])
    line = document["lines"][0]
    for key in ("myr_required", "payments_applied", "shortfall"):
        value = line["myr"][key]
        assert isinstance(value, str)
        assert value.count(".") == 1 and len(value.split(".")[1]) == 2
    assert isinstance(document["experimental_total_exposure"], str)


def test_the_emitter_refuses_a_float_it_is_handed():
    """The guard is checked at the point of emission rather than trusted to
    review, so it has to actually fire."""
    from div7aloan.cli import _assert_no_floats

    with pytest.raises(AssertionError, match=r"\$\.lines\[0\]\.shortfall"):
        _assert_no_floats({"lines": [{"shortfall": 1.5}]})


# --- exit codes --------------------------------------------------------


def test_a_clean_register_exits_zero():
    code, _, _ = run(["review", "--input", MET, "--year", "2026-27"])
    assert code == 0


def test_a_register_needing_attention_exits_two():
    code, _, _ = run(["review", "--input", MIXED, "--year", "2026-27"])
    assert code == 2


def test_a_known_rate_exits_zero():
    code, _, _ = run(["rate", "--year", "2026-27"])
    assert code == 0


def test_an_unreviewed_rate_year_exits_two():
    code, _, _ = run(["rate", "--year", "2027-28"])
    assert code == 2


def test_a_missing_file_exits_one_without_a_traceback():
    code, out, err = run(["review", "--input", "no-such-file.csv", "--year", "2026-27"])
    assert code == 1
    assert err.startswith("error: ")
    assert "Traceback" not in err


def test_a_malformed_year_exits_one():
    code, _, err = run(["rate", "--year", "2026-2027"])
    assert code == 1
    assert "2026-27" in err


def test_a_year_whose_halves_disagree_exits_one():
    code, _, err = run(["rate", "--year", "2026-28"])
    assert code == 1
    assert "error:" in err


# --- output content ----------------------------------------------------


def test_the_text_output_leads_with_the_disclaimer():
    _, out, _ = run(["review", "--input", MET, "--year", "2026-27"])
    assert out.startswith("Experimental review aid. Not a Division 7A determination")


def test_the_json_envelope_names_the_compilation_it_was_written_against():
    _, document = run_json(["rate", "--year", "2026-27"])
    assert document["law_compilation"].startswith("C1936A00027")
    assert document["tool"] == "div7a-loan-review"
    assert "not a division 7a determination" in document["disclaimer"].lower()


def test_the_rate_command_reports_the_may_figure_and_its_provenance():
    _, document = run_json(["rate", "--year", "2025-26"])
    assert document["benchmark_rate"] == "0.0837"
    assert document["provenance"]["rba_month"] == "2025-05"
    assert document["provenance"]["rba_series"] == "FILRHLBVS"
    assert document["provenance"]["table_reviewed_until"] == "2026-27"


def test_an_unknown_rate_gives_a_reason_not_a_number():
    _, document = run_json(["rate", "--year", "2027-28"])
    assert document["verdict"] == "UNKNOWN"
    assert document["benchmark_rate"] is None
    assert "override" in document["reason"]


def test_the_gate_command_reports_no_repayment_figures():
    _, document = run_json(["gate", "--input", MIXED])
    assert all(line["myr"] is None for line in document["lines"])
    assert document["summary"]["MYR_MET"] == 0
    assert document["summary"]["MYR_SHORT"] == 0


def test_the_myr_command_reports_no_gate_block():
    _, document = run_json(["myr", "--input", MIXED, "--year", "2026-27"])
    reviewed = [line for line in document["lines"] if line["status"] == "REVIEWED"]
    assert reviewed and all(line["gate"] is None for line in reviewed)
    assert document["summary"]["COMPLYING"] == 0


def test_the_review_command_reports_both():
    _, document = run_json(["review", "--input", MIXED, "--year", "2026-27"])
    reviewed = [line for line in document["lines"] if line["status"] == "REVIEWED"]
    assert all(line["gate"] is not None and line["myr"] is not None for line in reviewed)


def test_the_trace_flag_prints_the_statutory_trace():
    _, plain, _ = run(["review", "--input", MET, "--year", "2026-27"])
    _, traced, _ = run(["review", "--input", MET, "--year", "2026-27", "--trace"])
    assert "s 109E(6) formula" not in plain
    assert "s 109E(6) formula" in traced
    assert len(traced) > len(plain)


def test_a_short_row_labels_its_exposure_as_a_review_aid():
    _, out, _ = run(["review", "--input", MIXED, "--year", "2026-27"])
    assert "experimental deemed-dividend exposure" in out
    assert "not an assessment" in out


def test_the_rates_override_flag_reaches_the_rate_command(tmp_path):
    path = tmp_path / "override.json"
    path.write_text(
        json.dumps(
            {
                "verified_until": "2027-28",
                "citation": "RBA F5 FILRHLBVS May 2027, read 2027-07-02",
                "rates": [{"year_of_income": "2027-28", "rate": "0.0850"}],
            }
        ),
        encoding="utf-8",
    )
    code, document = run_json(["rate", "--year", "2027-28", "--rates-override", str(path)])
    assert code == 0
    assert document["benchmark_rate"] == "0.0850"
    assert document["provenance"]["origin"] == "reviewed override"


def test_an_override_without_a_citation_exits_one(tmp_path):
    path = tmp_path / "override.json"
    path.write_text(json.dumps({"verified_until": "2027-28", "rates": []}), encoding="utf-8")
    code, _, err = run(["rate", "--year", "2027-28", "--rates-override", str(path)])
    assert code == 1
    assert "citation" in err
