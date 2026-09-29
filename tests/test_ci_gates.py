"""Exercise the merge gates with failed, incomplete and unlisted CI results."""

import json
import re
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github/workflows"
# Each aggregate that branch protection requires, or that a required one needs.
GATES = {
    "boundaries.yml": "boundaries-gates",
    "ci.yml": "tests-gates",
    "ci-lodgeit-adapter.yml": "lodgeit-gates",
    "ci-package.yml": "gates",
    "codeql.yml": "codeql-gates",
    "joined-fixtures.yml": "public-fixtures-gates",
}


def gate_scripts(workflow: str, gate: str) -> tuple[list[str], list[str]]:
    """Return the gate's needs and the python scripts of its steps, in order."""
    text = (WORKFLOWS / workflow).read_text(encoding="utf-8")
    job = text.split(f"\n  {gate}:\n", 1)[1]
    needs = re.search(r"(?m)^    needs: \[(.*)\]$", job)
    assert needs is not None, workflow
    blocks = job.split("        run: |\n")[1:]
    return (
        [name.strip() for name in needs.group(1).split(",")],
        [textwrap.dedent(block.split("\n      - ", 1)[0]) for block in blocks],
    )


def run(script: str, results: dict, cwd: Path = ROOT) -> int:
    return subprocess.run(
        [sys.executable, "-c", script],
        cwd=cwd,
        env={"RESULTS": json.dumps(results)},
        capture_output=True,
        text=True,
        check=False,
    ).returncode


class AdapterGateTests(unittest.TestCase):
    def test_adapter_gate_rejects_failed_cancelled_skipped_and_missing_results(self) -> None:
        needs, scripts = gate_scripts("ci-lodgeit-adapter.yml", "lodgeit-gates")
        self.assertEqual(needs, ["test", "lint", "dependency-audit"])
        workflow = (WORKFLOWS / "ci-lodgeit-adapter.yml").read_text(encoding="utf-8")
        self.assertIn("if: always()", workflow.split("\n  lodgeit-gates:\n", 1)[1])
        success = {name: {"result": "success"} for name in needs}
        cases = [(success, 0), ({}, 1)]
        for name in success:
            for result in ("failure", "cancelled", "skipped"):
                cases.append(({**success, name: {"result": result}}, 1))
            cases.append(({key: value for key, value in success.items() if key != name}, 1))
        for results, expected in cases:
            with self.subTest(results=results):
                self.assertEqual(run(scripts[0], results), expected)


class GateCompletenessTests(unittest.TestCase):
    def test_every_gate_needs_every_other_job_in_its_workflow(self) -> None:
        for workflow, gate in GATES.items():
            needs, scripts = gate_scripts(workflow, gate)
            audit = scripts[-1]
            self.assertIn(f'open(".github/workflows/{workflow}"', audit)
            listed = {name: {"result": "success"} for name in needs}
            with self.subTest(workflow=workflow, case="complete"):
                self.assertEqual(run(audit, listed), 0)
            for name in needs:
                with self.subTest(workflow=workflow, case=f"{name} unlisted"):
                    self.assertEqual(run(audit, {k: v for k, v in listed.items() if k != name}), 1)
            # A job added under a quoted key, or a job list the pattern cannot read,
            # must fail rather than pass unseen.
            original = (WORKFLOWS / workflow).read_text(encoding="utf-8")
            variants = {
                "quoted key": original + "\n  'quoted-job':\n    runs-on: ubuntu-latest\n",
                "unreadable": original.replace("\n  ", "\n    "),
            }
            for case, text in variants.items():
                with self.subTest(workflow=workflow, case=case), tempfile.TemporaryDirectory() as tmp:
                    copy = Path(tmp) / ".github" / "workflows" / workflow
                    copy.parent.mkdir(parents=True)
                    copy.write_text(text, encoding="utf-8")
                    self.assertEqual(run(audit, listed, Path(tmp)), 1)

    def test_new_gates_fail_unless_every_needed_job_succeeded(self) -> None:
        for workflow, gate in GATES.items():
            if workflow in ("ci-lodgeit-adapter.yml", "ci-package.yml"):
                continue  # their result rules have their own tests or accept path skips
            needs, scripts = gate_scripts(workflow, gate)
            success = {name: {"result": "success"} for name in needs}
            with self.subTest(workflow=workflow, case="success"):
                self.assertEqual(run(scripts[0], success), 0)
            for result in ("failure", "cancelled", "skipped"):
                with self.subTest(workflow=workflow, case=result):
                    self.assertEqual(run(scripts[0], {**success, needs[0]: {"result": result}}), 1)


if __name__ == "__main__":
    unittest.main()
