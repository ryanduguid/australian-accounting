from __future__ import annotations

import csv
import json
import re
from pathlib import Path

import pytest
from paydaysuper.cli import main

ROOT = Path(__file__).resolve().parent.parent
PACK = ROOT / "evaluation" / "receipt_amount"
CONTRACT = json.loads((PACK / "expected_results.json").read_text(encoding="utf-8"))
SCENARIOS = CONTRACT["scenarios"]
REFUSALS = CONTRACT["refusals"]


@pytest.mark.parametrize("scenario", SCENARIOS, ids=[item["id"] for item in SCENARIOS])
def test_scenario_reproduces_the_declared_result(scenario: dict, tmp_path: Path) -> None:
    output = tmp_path / "report.csv"
    code = main(
        [str(PACK / "fixtures" / scenario["fixture"]), "--as-at", CONTRACT["as_at"], "-o", str(output)]
    )
    assert code == scenario["expected_exit"]
    with output.open(encoding="utf-8-sig", newline="") as handle:
        rows = list(csv.DictReader(handle))
    assert [row["employee_id"] for row in rows] == ["SYN001", "NOTE"]
    assert rows[0]["due_date"] == scenario["expected_due_date"]
    assert rows[0]["verdict"] == scenario["expected_verdict"]
    assert rows[0]["final_shortfall"] == scenario["expected_final_shortfall"]


@pytest.mark.parametrize("refusal", REFUSALS, ids=[item["id"] for item in REFUSALS])
def test_refusal_exits_before_assessment(
    refusal: dict, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    output = tmp_path / "report.csv"
    code = main(
        [str(PACK / "fixtures" / refusal["fixture"]), "--as-at", CONTRACT["as_at"], "-o", str(output)]
    )
    assert code == refusal["expected_exit"]
    assert refusal["expected_error"] in capsys.readouterr().err
    assert not output.exists()


def test_every_fixture_is_declared_once() -> None:
    declared = [item["fixture"] for item in SCENARIOS + REFUSALS]
    assert sorted(declared) == sorted(path.name for path in (PACK / "fixtures").iterdir())


def test_contract_tracks_the_checker_version_and_pending_review() -> None:
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    version = re.search(r'(?m)^version = "([^"]+)"$', pyproject).group(1)
    readme = (PACK / "README.md").read_text(encoding="utf-8")
    assert CONTRACT["product_release"] == version
    assert f"Product release `{version}`" in readme
    assert CONTRACT["practitioner_review"] == "pending"
    assert CONTRACT["human_decision"] in readme


def test_readme_table_matches_the_contract() -> None:
    readme = (PACK / "README.md").read_text(encoding="utf-8")
    for scenario in SCENARIOS:
        row = next(line for line in readme.splitlines() if line.startswith(f"| `{scenario['id']}` |"))
        cells = [cell.strip() for cell in row.strip("|").split("|")]
        assert cells[3] == f"`{scenario['expected_verdict']}`"
        assert cells[4] == str(scenario["expected_exit"])
        if scenario["expected_final_shortfall"]:
            assert cells[5].startswith(scenario["expected_final_shortfall"])
