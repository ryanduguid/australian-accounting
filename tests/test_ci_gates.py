"""Exercise the adapter merge gate with failed and incomplete CI results."""

import json
import subprocess
import sys
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class AdapterGateTests(unittest.TestCase):
    def test_adapter_gate_rejects_failed_cancelled_skipped_and_missing_results(self) -> None:
        workflow = (ROOT / ".github/workflows/ci-lodgeit-adapter.yml").read_text(
            encoding="utf-8"
        )
        gate = workflow.split("\n  lodgeit-gates:\n", 1)[1]
        self.assertIn("needs: [test, lint, dependency-audit]", gate)
        self.assertIn("if: always()", gate)
        script = textwrap.dedent(gate.split("        run: |\n", 1)[1])
        success = {name: {"result": "success"} for name in ("test", "lint", "dependency-audit")}
        cases = [(success, 0), ({}, 1)]
        for name in success:
            for result in ("failure", "cancelled", "skipped"):
                cases.append(({**success, name: {"result": result}}, 1))
            cases.append(({key: value for key, value in success.items() if key != name}, 1))
        for results, expected in cases:
            with self.subTest(results=results):
                actual = subprocess.run(
                    [sys.executable, "-c", script],
                    env={"RESULTS": json.dumps(results)},
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(actual.returncode, expected, actual.stderr)


if __name__ == "__main__":
    unittest.main()
