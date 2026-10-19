"""Keep the type gate scoped and capable of rejecting representative defects."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCOPES = {
    "packages/payday-super-checker": "paydaysuper",
    "packages/gst-review": "gstreview",
    "apps/aus-accounting-mcp": "aus_accounting_mcp",
    "apps/lodgeit-calculator-adapter": "lodgeitadapter",
    "packages/au-tax-rates-data": "rates.py",
}


class PyrightGateTests(unittest.TestCase):
    def test_every_component_retains_its_typecheck_scope(self) -> None:
        for directory, target in SCOPES.items():
            with self.subTest(component=directory):
                project = tomllib.loads((ROOT / directory / "pyproject.toml").read_text())
                config = project["tool"]["pyright"]
                self.assertEqual(config["include"], [target])
                self.assertEqual(config["pythonVersion"], "3.14")
                self.assertEqual(config["typeCheckingMode"], "standard")
                self.assertNotIn("ignore", config)
                self.assertNotIn("exclude", config)
                self.assertTrue((ROOT / directory / target).exists())

    def test_root_retains_every_runtime_package(self) -> None:
        project = tomllib.loads((ROOT / "pyproject.toml").read_text())
        expected = {f"{directory}/{target}" for directory, target in SCOPES.items()
                    if target != "rates.py"}
        self.assertEqual(set(project["tool"]["pyright"]["include"]), expected)

    def test_type_gate_rejects_representative_invalid_code(self) -> None:
        fixture = ROOT / "tests" / "typing" / "invalid_types.py.txt"
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            (work / "invalid.py").write_text(fixture.read_text(), encoding="utf-8")
            (work / "pyrightconfig.json").write_text(json.dumps({
                "include": ["invalid.py"], "pythonVersion": "3.14",
                "typeCheckingMode": "standard",
            }), encoding="utf-8")
            result = subprocess.run(
                [sys.executable, "-m", "pyright", "--outputjson"],
                cwd=work, capture_output=True, text=True, timeout=60, check=False,
            )
        self.assertEqual(result.returncode, 1, result.stderr or result.stdout)
        report = json.loads(result.stdout)
        self.assertEqual(report["summary"]["filesAnalyzed"], 1)
        rules = {item["rule"] for item in report["generalDiagnostics"]}
        self.assertTrue({
            "reportAssignmentType", "reportReturnType", "reportArgumentType",
            "reportGeneralTypeIssues",
            "reportAttributeAccessIssue",
        }.issubset(rules), rules)
        self.assertGreaterEqual(report["summary"]["errorCount"], 10)
