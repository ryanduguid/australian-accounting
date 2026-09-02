"""Re-run the evaluation pack against its pinned expectations.

The point of the pack is that a reviewer does not have to trust the CLI's
pretty-printed output. These tests re-perform every fixture through the
production code path and compare against expected_results.json, which is the
same file the evaluation README's hand-working describes.
"""
from __future__ import annotations

import contextlib
import io
import json
from decimal import Decimal
from pathlib import Path

import pytest

from div7aloan.cli import main

PACK = Path("evaluation/div7a_myr")
EXPECTED = json.loads((PACK / "expected_results.json").read_text(encoding="utf-8"))
SCENARIOS = EXPECTED["scenarios"]
IDS = [scenario["id"] for scenario in SCENARIOS]


def run_fixture(scenario):
    argv = [
        "review",
        "--input",
        str(PACK / "fixtures" / scenario["fixture"]),
        "--year",
        scenario["year_of_income"],
        "--format",
        "json",
    ]
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = main(argv)
    return code, json.loads(out.getvalue())


@pytest.mark.parametrize("scenario", SCENARIOS, ids=IDS)
def test_every_fixture_exists(scenario):
    assert (PACK / "fixtures" / scenario["fixture"]).is_file()


@pytest.mark.parametrize("scenario", SCENARIOS, ids=IDS)
def test_gate_verdict_matches(scenario):
    _, document = run_fixture(scenario)
    assert document["lines"][0]["gate"]["verdict"] == scenario["expected_gate_verdict"]


@pytest.mark.parametrize("scenario", SCENARIOS, ids=IDS)
def test_myr_verdict_matches(scenario):
    _, document = run_fixture(scenario)
    assert document["lines"][0]["myr"]["verdict"] == scenario["expected_myr_verdict"]


@pytest.mark.parametrize("scenario", SCENARIOS, ids=IDS)
def test_amounts_match_to_the_cent(scenario):
    _, document = run_fixture(scenario)
    myr = document["lines"][0]["myr"]
    assert myr["myr_required"] == scenario["expected_myr_required"]
    assert myr["shortfall"] == scenario["expected_shortfall"]
    assert myr["experimental_deemed_dividend_exposure"] == scenario["expected_exposure"]


@pytest.mark.parametrize("scenario", SCENARIOS, ids=IDS)
def test_exit_code_matches(scenario):
    code, _ = run_fixture(scenario)
    assert code == scenario["expected_exit"]


@pytest.mark.parametrize("scenario", SCENARIOS, ids=IDS)
def test_the_inputs_the_readme_hand_works_are_the_inputs_used(scenario):
    """The README's paper working is only useful if it describes the numbers
    the engine actually plugged in."""
    if not scenario["hand_worked"]:
        pytest.skip("not hand-worked in the evaluation README")
    _, document = run_fixture(scenario)
    myr = document["lines"][0]["myr"]
    inputs = scenario["inputs"]
    assert myr["amalgamated_loan_unpaid_at_end_of_previous_year"] == (
        inputs["amalgamated_loan_unpaid_at_end_of_previous_year"]
    )
    assert myr["benchmark_rate"] == inputs["benchmark_rate"]
    assert myr["remaining_term_years_used"] == inputs["remaining_term_years"]


@pytest.mark.parametrize("scenario", SCENARIOS, ids=IDS)
def test_the_hand_working_reproduces_the_engines_answer(scenario):
    """Recompute the fixture straight from the README's stated inputs, using
    the formula rather than the engine's own plumbing."""
    if not scenario["hand_worked"]:
        pytest.skip("not hand-worked in the evaluation README")
    from div7aloan.myr import minimum_yearly_repayment_amount

    inputs = scenario["inputs"]
    recomputed = minimum_yearly_repayment_amount(
        Decimal(inputs["amalgamated_loan_unpaid_at_end_of_previous_year"]),
        Decimal(inputs["benchmark_rate"]),
        Decimal(inputs["remaining_term_years"]),
    )
    assert str(recomputed) == scenario["expected_myr_required"]


def test_the_readme_prints_every_hand_worked_figure():
    """A figure that has drifted out of the prose is worse than no prose: a
    reviewer would re-perform the sum, disagree, and not know which of the two
    was stale."""
    readme = (PACK / "README.md").read_text(encoding="utf-8")
    for scenario in SCENARIOS:
        if not scenario["hand_worked"]:
            continue
        assert scenario["expected_myr_required"] in readme, scenario["id"]
        assert scenario["inputs"]["benchmark_rate"] in readme, scenario["id"]
        if scenario["expected_shortfall"] != "0.00":
            formatted = f"{Decimal(scenario['expected_shortfall']):,}"
            assert formatted in readme or scenario["expected_shortfall"] in readme, scenario["id"]


def test_the_pack_names_its_sources_and_review_date():
    assert EXPECTED["law_compilation"].startswith("C1936A00027")
    urls = {source["url"] for source in EXPECTED["sources"]}
    assert any("legislation.gov.au" in url for url in urls)
    assert any("rba.gov.au" in url for url in urls)
    assert any("duguid.com.au" in url for url in urls)


def test_the_pack_covers_a_met_a_short_a_refusal_and_an_unknown():
    verdicts = {scenario["expected_myr_verdict"] for scenario in SCENARIOS}
    assert {"MYR_MET", "MYR_SHORT", "REFUSED", "UNKNOWN"} <= verdicts


def test_at_least_three_fixtures_are_hand_worked():
    assert sum(1 for scenario in SCENARIOS if scenario["hand_worked"]) >= 3


def test_no_fixture_carries_anything_that_looks_like_a_client_record():
    """Every identifier in the pack is synthetic."""
    for path in (PACK / "fixtures").glob("*.csv"):
        text = path.read_text(encoding="utf-8")
        for row in text.splitlines()[1:]:
            reference = row.split(",")[1]
            assert reference.startswith("SYN-"), f"{path.name}: {reference}"
