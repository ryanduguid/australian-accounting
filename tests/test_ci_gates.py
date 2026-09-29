"""Exercise the merge gates: result rules, job lists and the removed-job tripwire."""

import json
import os
import re
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github/workflows"
CHECK = ROOT / ".github/ci/check_gates.py"
# Each aggregate that branch protection requires, or that a required one needs.
GATES = {
    "boundaries.yml": "boundaries-gates",
    "ci.yml": "tests-gates",
    "ci-lodgeit-adapter.yml": "lodgeit-gates",
    "ci-package.yml": "gates",
    "codeql.yml": "codeql-gates",
    "joined-fixtures.yml": "public-fixtures-gates",
}
SAMPLE = """name: ci
on: [push]
jobs:
  lint:
    runs-on: ubuntu-latest
  test:
    runs-on: ubuntu-latest
    steps:
      - run: |
          echo "  looks: like a key"
  ci-gates:
    needs: [lint, test]
"""
SAMPLE_PATH = ".github/workflows/ci.yml"
OK = {"lint": {"result": "success"}, "test": {"result": "success"}}


def gate_job(workflow: str, gate: str) -> str:
    return (WORKFLOWS / workflow).read_text(encoding="utf-8").split(f"\n  {gate}:\n", 1)[1]


def git(cwd: Path, *args: str) -> None:
    identity = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid",
                "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.invalid"}
    subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, env={**os.environ, **identity})


class AdapterGateTests(unittest.TestCase):
    def test_adapter_gate_rejects_failed_cancelled_skipped_and_missing_results(self) -> None:
        job = gate_job("ci-lodgeit-adapter.yml", "lodgeit-gates")
        self.assertIn("needs: [test, lint, dependency-audit]", job)
        self.assertIn("if: always()", job)
        script = textwrap.dedent(job.split("        run: |\n", 1)[1].split("\n      - ", 1)[0])
        success = {name: {"result": "success"} for name in ("test", "lint", "dependency-audit")}
        cases = [(success, 0), ({}, 1)]
        for name in success:
            for result in ("failure", "cancelled", "skipped"):
                cases.append(({**success, name: {"result": result}}, 1))
            cases.append(({key: value for key, value in success.items() if key != name}, 1))
        for results, expected in cases:
            with self.subTest(results=results):
                actual = subprocess.run([sys.executable, "-c", script], env={"RESULTS": json.dumps(results)},
                                        capture_output=True, text=True, check=False)
                self.assertEqual(actual.returncode, expected, actual.stderr)


class GateWiringTests(unittest.TestCase):
    def test_every_gate_runs_the_check_on_its_own_workflow_and_needs_every_job(self) -> None:
        sys.path.insert(0, str(CHECK.parent))
        try:
            import check_gates
        finally:
            sys.path.pop(0)
        for workflow, gate in GATES.items():
            with self.subTest(workflow=workflow):
                job = gate_job(workflow, gate)
                self.assertIn("if: always()", job)
                self.assertIn("fetch-depth: 2", job)
                self.assertIn("PR_BODY: ${{ github.event.pull_request.body }}", job)
                call = re.search(r"(?m)^        run: python3 \.github/ci/check_gates\.py (\S+) (\S+)(.*)$", job)
                assert call is not None, workflow
                self.assertEqual(call.group(1, 2), (f".github/workflows/{workflow}", gate))
                needs = re.search(r"(?m)^    needs: \[(.*)\]$", job)
                assert needs is not None, workflow
                listed = {name.strip() for name in needs.group(1).split(",")}
                text = (WORKFLOWS / workflow).read_text(encoding="utf-8")
                self.assertEqual(check_gates.job_ids(text) - {gate}, listed)


class CheckGatesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / ".github/workflows").mkdir(parents=True)
        git(self.root, "init", "-q", "-b", "main")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def write(self, text: str) -> None:
        (self.root / SAMPLE_PATH).write_text(text, encoding="utf-8")

    def check(self, results: dict, *extra: str, event: str = "push", body: str = "") -> tuple[int, str]:
        env = {**os.environ, "RESULTS": json.dumps(results), "GITHUB_EVENT_NAME": event, "PR_BODY": body}
        done = subprocess.run([sys.executable, str(CHECK), SAMPLE_PATH, "ci-gates", *extra], cwd=self.root,
                              env=env, capture_output=True, text=True, check=False)
        return done.returncode, (done.stdout + done.stderr).strip()

    def test_results_and_completeness(self) -> None:
        self.write(SAMPLE)
        self.assertEqual(self.check(OK)[0], 0)
        for bad in ("failure", "cancelled", "skipped"):
            self.assertEqual(self.check({**OK, "test": {"result": bad}}), (1, "Jobs did not succeed: test"))
        self.assertEqual(self.check({**OK, "test": {"result": "skipped"}}, "--no-results")[0], 0)
        self.assertEqual(self.check({"lint": OK["lint"]}), (1, "Add these jobs to needs: test"))
        self.assertEqual(self.check({"lint": OK["lint"]}, "--exempt", "test")[0], 0)

    def test_every_job_key_spelling_is_seen_or_fails_closed(self) -> None:
        for spelling in ("extra:", "'extra':", '"extra":', "extra :", "'extra' :", "extra: # note"):
            with self.subTest(spelling=spelling):
                self.write(SAMPLE + f"  {spelling}\n    runs-on: ubuntu-latest\n")
                self.assertEqual(self.check(OK), (1, "Add these jobs to needs: extra"))
        for unreadable in ("? extra", '"ex\\x74ra":'):
            with self.subTest(unreadable=unreadable):
                self.write(SAMPLE + f"  {unreadable}\n    runs-on: ubuntu-latest\n")
                code, out = self.check(OK)
                self.assertEqual(code, 1)
                self.assertIn("Cannot read the job key", out)
        self.write(SAMPLE.replace("\n  ", "\n    "))
        self.assertIn("Cannot find ci-gates", self.check(OK)[1])
        self.write(SAMPLE + "# note\nenv:\n  FOO: bar\n")
        self.assertEqual(self.check(OK)[0], 0)

    def test_a_removed_job_needs_a_declaration_on_a_pull_request(self) -> None:
        self.write(SAMPLE)
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", "base")
        git(self.root, "checkout", "-q", "-b", "feature")
        self.write(SAMPLE.replace("  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: |\n"
                                  '          echo "  looks: like a key"\n', "").replace("[lint, test]", "[lint]"))
        git(self.root, "commit", "-q", "-am", "drop test")
        git(self.root, "checkout", "-q", "main")
        git(self.root, "merge", "-q", "--no-ff", "--no-edit", "feature")
        lint_only = {"lint": OK["lint"]}
        code, out = self.check(lint_only, event="pull_request")
        self.assertEqual(code, 1)
        self.assertIn(f"These jobs were removed from {SAMPLE_PATH}: test", out)
        self.assertEqual(self.check(lint_only, event="pull_request", body=f"removed-jobs: {SAMPLE_PATH}#test")[0], 0)
        for body in ("removed-jobs: test", f"removed-jobs: {SAMPLE_PATH}#tests"):
            self.assertEqual(self.check(lint_only, event="pull_request", body=body)[0], 1, body)
        self.assertEqual(self.check(lint_only, event="push")[0], 0)

    def test_a_missing_merge_parent_fails_closed(self) -> None:
        self.write(SAMPLE)
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", "only")
        self.assertNotEqual(self.check(OK, event="pull_request")[0], 0)


if __name__ == "__main__":
    unittest.main()
