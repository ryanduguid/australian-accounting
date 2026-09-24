"""Require the reviewed component checks before a release can publish."""

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY = "87767ec809dc7f77bcd45808219adaf67841ae7b"
# These are component jobs from successful main-branch runs, never skip-tolerant
# aggregate gates. Review the list when a component's CI contract changes.
REQUIRED = {
    "release-ato-benchmark-compare.yml": [
        ".github/workflows/no-ai-attribution.yml: Attribution policy / Attribution policy runner",
        ".github/workflows/ci.yml: ato-benchmark-compare / build",
        ".github/workflows/ci.yml: ato-benchmark-compare / dependency-audit",
        ".github/workflows/ci.yml: ato-benchmark-compare / lint",
        ".github/workflows/ci.yml: ato-benchmark-compare / test (3.10)",
        ".github/workflows/ci.yml: ato-benchmark-compare / test (3.12)",
        ".github/workflows/ci.yml: ato-benchmark-compare / test (3.13)",
        ".github/workflows/ci.yml: ato-benchmark-compare / test (3.14)",
        ".github/workflows/ci.yml: ato-benchmark-compare / test-windows",
        ".github/workflows/boundaries.yml: boundaries",
        ".github/workflows/codeql.yml: Analyze Python"
    ],
    "release-aus-accounting-mcp.yml": [
        ".github/workflows/no-ai-attribution.yml: Attribution policy / Attribution policy runner",
        ".github/workflows/ci.yml: dependency-audit",
        ".github/workflows/ci.yml: lint",
        ".github/workflows/ci.yml: test (ubuntu-latest, 3.10)",
        ".github/workflows/ci.yml: test (ubuntu-latest, 3.12)",
        ".github/workflows/ci.yml: test (ubuntu-latest, 3.13)",
        ".github/workflows/ci.yml: test (ubuntu-latest, 3.14)",
        ".github/workflows/ci.yml: test (windows-latest, 3.12)",
        ".github/workflows/boundaries.yml: boundaries",
        ".github/workflows/codeql.yml: Analyze Python"
    ],
    "release-australian-tax-calculators.yml": [
        ".github/workflows/no-ai-attribution.yml: Attribution policy / Attribution policy runner",
        ".github/workflows/ci.yml: australian-tax-calculators / build",
        ".github/workflows/ci.yml: australian-tax-calculators / dependency-audit",
        ".github/workflows/ci.yml: australian-tax-calculators / lint",
        ".github/workflows/ci.yml: australian-tax-calculators / test (3.10)",
        ".github/workflows/ci.yml: australian-tax-calculators / test (3.12)",
        ".github/workflows/ci.yml: australian-tax-calculators / test (3.13)",
        ".github/workflows/ci.yml: australian-tax-calculators / test (3.14)",
        ".github/workflows/ci.yml: australian-tax-calculators / test-windows",
        ".github/workflows/boundaries.yml: boundaries",
        ".github/workflows/codeql.yml: Analyze Python"
    ],
    "release-div7a-loan-review.yml": [
        ".github/workflows/no-ai-attribution.yml: Attribution policy / Attribution policy runner",
        ".github/workflows/ci.yml: div7a-loan-review / build",
        ".github/workflows/ci.yml: div7a-loan-review / dependency-audit",
        ".github/workflows/ci.yml: div7a-loan-review / lint",
        ".github/workflows/ci.yml: div7a-loan-review / test (3.10)",
        ".github/workflows/ci.yml: div7a-loan-review / test (3.12)",
        ".github/workflows/ci.yml: div7a-loan-review / test (3.13)",
        ".github/workflows/ci.yml: div7a-loan-review / test (3.14)",
        ".github/workflows/boundaries.yml: boundaries",
        ".github/workflows/codeql.yml: Analyze Python"
    ],
    "release-payday-super-checker.yml": [
        ".github/workflows/no-ai-attribution.yml: Attribution policy / Attribution policy runner",
        ".github/workflows/ci.yml: payday-super-checker / build",
        ".github/workflows/ci.yml: payday-super-checker / dependency-audit",
        ".github/workflows/ci.yml: payday-super-checker / lint",
        ".github/workflows/ci.yml: payday-super-checker / test (3.10)",
        ".github/workflows/ci.yml: payday-super-checker / test (3.12)",
        ".github/workflows/ci.yml: payday-super-checker / test (3.13)",
        ".github/workflows/ci.yml: payday-super-checker / test (3.14)",
        ".github/workflows/ci.yml: payday-super-checker / test-windows",
        ".github/workflows/boundaries.yml: boundaries",
        ".github/workflows/codeql.yml: Analyze Python"
    ],
    "release-solomons-sword.yml": [
        ".github/workflows/no-ai-attribution.yml: Attribution policy / Attribution policy runner",
        ".github/workflows/ci.yml: solomons-sword / build",
        ".github/workflows/ci.yml: solomons-sword / dependency-audit",
        ".github/workflows/ci.yml: solomons-sword / lint",
        ".github/workflows/ci.yml: solomons-sword / test (3.10)",
        ".github/workflows/ci.yml: solomons-sword / test (3.12)",
        ".github/workflows/ci.yml: solomons-sword / test (3.13)",
        ".github/workflows/ci.yml: solomons-sword / test (3.14)",
        ".github/workflows/boundaries.yml: boundaries",
        ".github/workflows/codeql.yml: Analyze Python"
    ],
    "release-the-exchequer-tally.yml": [
        ".github/workflows/no-ai-attribution.yml: Attribution policy / Attribution policy runner",
        ".github/workflows/ci.yml: the-exchequer-tally / build",
        ".github/workflows/ci.yml: the-exchequer-tally / dependency-audit",
        ".github/workflows/ci.yml: the-exchequer-tally / lint",
        ".github/workflows/ci.yml: the-exchequer-tally / test (3.10)",
        ".github/workflows/ci.yml: the-exchequer-tally / test (3.12)",
        ".github/workflows/ci.yml: the-exchequer-tally / test (3.13)",
        ".github/workflows/ci.yml: the-exchequer-tally / test (3.14)",
        ".github/workflows/boundaries.yml: boundaries",
        ".github/workflows/codeql.yml: Analyze Python"
    ],
    "release-the-wip-tally.yml": [
        ".github/workflows/no-ai-attribution.yml: Attribution policy / Attribution policy runner",
        ".github/workflows/ci.yml: the-wip-tally / build",
        ".github/workflows/ci.yml: the-wip-tally / dependency-audit",
        ".github/workflows/ci.yml: the-wip-tally / lint",
        ".github/workflows/ci.yml: the-wip-tally / test (3.10)",
        ".github/workflows/ci.yml: the-wip-tally / test (3.12)",
        ".github/workflows/ci.yml: the-wip-tally / test (3.13)",
        ".github/workflows/ci.yml: the-wip-tally / test (3.14)",
        ".github/workflows/ci.yml: the-wip-tally / test-windows",
        ".github/workflows/boundaries.yml: boundaries",
        ".github/workflows/codeql.yml: Analyze Python"
    ]
}


class ReleaseChecksTests(unittest.TestCase):
    def test_every_release_caller_requires_its_component_checks(self) -> None:
        workflows = ROOT / ".github" / "workflows"
        callers = sorted(
            path.name for path in workflows.glob("release*") if path.suffix in {".yml", ".yaml"}
        )
        self.assertEqual(callers, sorted(REQUIRED))
        for filename, expected in REQUIRED.items():
            with self.subTest(workflow=filename):
                text = (workflows / filename).read_text(encoding="utf-8")
                job = text.split("\n  release:\n", 1)[1].split("\n  pypi:", 1)[0]
                self.assertRegex(
                    job,
                    r"(?m)^    uses: ryanduguid/release-policy/\.github/workflows/"
                    r"release-(?:python|archive|skills)\.yml@" + POLICY + r"$",
                )
                permissions = job.split("    permissions:\n", 1)[1].split("    uses:", 1)[0]
                self.assertRegex(permissions, r"(?m)^      actions: read(?: #.*)?$")
                match = re.search(
                    r"^      required-checks: \|\n((?:        [^\n]*\n)+)", job, re.MULTILINE,
                )
                self.assertIsNotNone(match, "missing explicit component checks")
                assert match is not None
                actual = [line.strip() for line in match.group(1).splitlines()]
                self.assertCountEqual(actual, expected)
                self.assertEqual(len(actual), len(set(actual)), "duplicate check selector")
                for selector in actual:
                    path, name = selector.split(": ", 1)
                    self.assertTrue((ROOT / path).is_file(), path)
                    self.assertFalse(name.endswith(" / gates"), name)


if __name__ == "__main__":
    unittest.main()


class NoticePackagingTests(unittest.TestCase):
    """Every distribution ships its licence, and a component that carries a
    NOTICE for third-party data ships that too. The declaration in
    pyproject.toml is what setuptools packages into .dist-info/licenses, so
    a missing entry means a wheel without the notice."""

    def component_dirs(self) -> list[Path]:
        return sorted(
            path.parent
            for pattern in ("packages/*/pyproject.toml", "apps/*/pyproject.toml")
            for path in ROOT.glob(pattern)
        )

    def test_every_component_declares_its_licence_files(self) -> None:
        for component in self.component_dirs():
            text = (component / "pyproject.toml").read_text(encoding="utf-8")
            match = re.search(r'^license-files\s*=\s*\[(.*?)\]', text, re.M)
            self.assertIsNotNone(match, f"{component.name}: no license-files")
            declared = re.findall(r'"([^"]+)"', match.group(1))
            self.assertIn("LICENSE", declared, component.name)
            for name in declared:
                self.assertTrue((component / name).is_file(), f"{component.name}: {name}")
            if (component / "NOTICE").is_file():
                self.assertIn("NOTICE", declared, f"{component.name}: NOTICE not packaged")
                manifest = component / "MANIFEST.in"
                if manifest.is_file():
                    self.assertIn("include NOTICE", manifest.read_text(encoding="utf-8"),
                                  f"{component.name}: NOTICE missing from sdist")
