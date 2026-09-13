from __future__ import annotations

import csv
import json
import subprocess
from pathlib import Path

from paydaysuper.cli import main

ROOT = Path(__file__).resolve().parent.parent
PACK = ROOT / "evaluation" / "payday_super_evidence"
EXPECTED = PACK / "expected_results.json"
CONTRACT = json.loads(EXPECTED.read_text(encoding="utf-8"))

FIXTURE_HEADER = [
    "employee_id",
    "payment_date",
    "sg_amount",
    "remitted_date",
    "fund_received_date",
    "first_contribution_to_fund",
    "out_of_cycle",
    "next_standard_payday",
    "defined_benefit",
]

def test_evidence_boundary_scenarios(tmp_path: Path) -> None:
    for scenario in CONTRACT["scenarios"]:
        output = tmp_path / f"{Path(scenario['fixture']).stem}-report.csv"
        code = main(
            [
                str(PACK / "fixtures" / scenario["fixture"]),
                "--as-at",
                CONTRACT["as_at"],
                "-o",
                str(output),
            ]
        )
        assert code == scenario["expected_exit"]
        with output.open(encoding="utf-8-sig", newline="") as handle:
            rows = list(csv.DictReader(handle))
        assert len(rows) == 2
        assert rows[-1]["employee_id"] == "NOTE"
        assert rows[0]["due_date"] == scenario["expected_due_date"]
        assert rows[0]["verdict"] == scenario["expected_verdict"]


def test_fabricated_fixtures_have_the_canonical_header_and_synthetic_row() -> None:
    for scenario in CONTRACT["scenarios"]:
        with (PACK / "fixtures" / scenario["fixture"]).open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            reader = csv.DictReader(handle)
            rows = list(reader)

        assert reader.fieldnames == FIXTURE_HEADER
        assert len(rows) == 1
        assert rows[0]["employee_id"] == "SYN001"
        assert rows[0]["payment_date"] == "2026-08-06"
        assert rows[0]["sg_amount"] == "120.00"


def test_evaluation_contract_retains_semantic_and_privacy_boundaries() -> None:
    assert CONTRACT["schema_version"] == 1
    assert CONTRACT["source_reviewed"] == "2026-08-15"
    assert len(CONTRACT["sources"]) == 2
    assert all(source["url"].startswith("https://") for source in CONTRACT["sources"])

    human_decision = CONTRACT["human_decision"].casefold()
    for phrase in ("eligible fund receipt", "allocation", "a human must establish"):
        assert phrase in human_decision

    guide = " ".join((PACK / "README.md").read_text(encoding="utf-8").split()).casefold()
    for phrase in (
        "cannot prove on-time",
        "do not establish fund eligibility",
        "not tax, legal or financial advice",
    ):
        assert phrase in guide


def test_product_discovery_files_link_the_pack() -> None:
    pack_path = PACK.relative_to(ROOT).as_posix()
    root_readme = (ROOT / "README.md").read_text(encoding="utf-8")
    llms = (ROOT / "llms.txt").read_text(encoding="utf-8")

    assert pack_path in root_readme
    assert pack_path in llms


def test_only_declared_evaluation_csvs_are_allowlisted() -> None:
    allowed = {
        (PACK / "fixtures" / scenario["fixture"]).relative_to(ROOT).as_posix()
        for scenario in CONTRACT["scenarios"]
    }
    rules = (ROOT / ".gitignore").read_text(encoding="utf-8").splitlines()
    evaluation_prefix = "!evaluation/payday_super_evidence/fixtures/"
    evaluation_allowlist = {
        rule.removeprefix("!") for rule in rules if rule.startswith(evaluation_prefix)
    }
    assert evaluation_allowlist == allowed
    for relative in sorted(allowed):
        assert f"!{relative}" in rules

    if (ROOT / ".git").exists():
        for relative in sorted(allowed):
            result = subprocess.run(
                ["git", "check-ignore", "--no-index", "--quiet", "--", relative],
                cwd=ROOT,
                check=False,
            )
            assert result.returncode == 1, relative
        refused = subprocess.run(
            [
                "git",
                "check-ignore",
                "--no-index",
                "--quiet",
                "--",
                "evaluation/payday_super_evidence/fixtures/client.csv",
            ],
            cwd=ROOT,
            check=False,
        )
        assert refused.returncode == 0


def test_confirmation_changes_the_at_risk_exit_code_but_not_the_verdict(tmp_path: Path) -> None:
    """The guide's refusal boundary: AT_RISK exits 0 once remittance-only is confirmed."""
    fixture = PACK / "fixtures" / "timely_remittance_no_receipt.csv"
    codes = {}
    for label, extra in (("plain", []), ("confirmed", ["--confirm-remittance-only"])):
        output = tmp_path / f"{label}.csv"
        codes[label] = main([str(fixture), "--as-at", CONTRACT["as_at"],
                             *extra, "-o", str(output)])
        with output.open(encoding="utf-8-sig", newline="") as handle:
            rows = list(csv.DictReader(handle))
        assert rows[0]["verdict"] == "AT_RISK"
    assert codes == {"plain": 2, "confirmed": 0}
    # The control: confirmation does not rescue a LATE result.
    late = tmp_path / "late.csv"
    assert main([str(PACK / "fixtures" / "late_remittance_no_receipt.csv"),
                 "--as-at", CONTRACT["as_at"], "--confirm-remittance-only",
                 "-o", str(late)]) == 2
