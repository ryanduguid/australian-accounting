"""Static boundary checks for this repository. Standard library only.

Two properties are proved with the ast module over every production module:

1. no engine production module imports ``aus_accounting_mcp`` or a sibling engine
   package (engines depend on nothing in this repository);
2. no production module, engine or application, uses a relative import whose level
   climbs out of its own top-level package directory.

Each property has an in-memory positive control so a scan that silently finds
nothing cannot pass. Run from the repository root:

    python -m unittest -v tests/test_boundaries.py
"""

from __future__ import annotations

import ast
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RELEASE_POLICY_SHA = "787db4590e725cfd37104c8a9dd9e75f7fd4c018"

RELEASE_CALLERS = {
    "aus-accounting-mcp": "apps/aus-accounting-mcp",
    "ato-benchmark-compare": "packages/ato-benchmark-compare",
    "payday-super-checker": "packages/payday-super-checker",
    "div7a-loan-review": "packages/div7a-loan-review",
    "the-exchequer-tally": "packages/the-exchequer-tally",
    "solomons-sword": "packages/solomons-sword",
    "the-wip-tally": "packages/the-wip-tally",
}

ENGINES = {
    "packages/ato-benchmark-compare": "atobenchmark",
    "packages/payday-super-checker": "paydaysuper",
    "packages/div7a-loan-review": "div7aloan",
    "packages/the-exchequer-tally": "edwinnixon",
    "packages/solomons-sword": "louisgoldberg",
    "packages/the-wip-tally": "wiptally",
}
APPLICATION = {"apps/aus-accounting-mcp": "aus_accounting_mcp"}
FORBIDDEN_FOR_ENGINES = frozenset({"aus_accounting_mcp", *ENGINES.values()})
PATH_FILTER_KEY = re.compile(
    r"(?<![\w-])['\"]?(paths(?:-ignore)?)['\"]?\s*:"
)


def trigger_path_filters(workflow: str) -> list[str]:
    """Find path-filter keys in either block or flow-style ``on`` mappings."""
    lines = workflow.splitlines()
    trigger_lines: list[str] = []
    for index, line in enumerate(lines):
        match = re.match(r"^on\s*:(.*)$", line)
        if match is None:
            continue
        trigger_lines.append(match.group(1))
        for following in lines[index + 1 :]:
            if following and not following[0].isspace() and not following.startswith("#"):
                break
            trigger_lines.append(following)
        break
    return PATH_FILTER_KEY.findall("\n".join(trigger_lines))


def imported_top_levels(tree: ast.AST) -> set[str]:
    """Top-level names of every absolute import in the tree."""
    names: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            names.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.level == 0 and node.module:
            names.add(node.module.split(".")[0])
    return names


def escaping_relative_imports(tree: ast.AST, depth: int) -> list[str]:
    """Relative imports whose level exceeds ``depth``.

    ``depth`` is the number of path components from the top-level package directory
    to the module file, so a module directly inside the package has depth 1 and may
    use ``from . import x`` (level 1) but not ``from .. import x`` (level 2).
    """
    found: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and node.level > depth:
            names = ", ".join(alias.name for alias in node.names)
            found.append(f"level {node.level} import of {node.module or '.'} ({names})")
    return found


def production_modules(component: str, package: str) -> list[tuple[Path, int]]:
    package_dir = ROOT / component / package
    modules = sorted(package_dir.rglob("*.py"))
    return [(path, len(path.relative_to(package_dir).parts)) for path in modules]


class BoundaryTests(unittest.TestCase):
    def test_imported_diff_coverage_waits_for_a_mainline_baseline(self) -> None:
        sentinels = {
            "ci-ato-benchmark-compare.yml": (
                "packages/ato-benchmark-compare/atobenchmark/mapping.py"
            ),
            "ci-payday-super-checker.yml": (
                "packages/payday-super-checker/paydaysuper/assess.py"
            ),
        }
        for workflow_name, sentinel in sentinels.items():
            workflow = (
                ROOT / ".github" / "workflows" / workflow_name
            ).read_text(encoding="utf-8")
            with self.subTest(workflow=workflow_name):
                self.assertIn(f'git cat-file -e "origin/main:{sentinel}"', workflow)

    def test_anchor_required_checks_are_not_suppressed_by_path_filters(self) -> None:
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )
        self.assertEqual(trigger_path_filters(workflow), [])

        positive_controls = (
            "on:\n  push:\n    paths: [packages/**]\njobs: {}\n",
            "on:\n  pull_request:\n    paths-ignore:\n      - docs/**\njobs: {}\n",
            "on: {push: {paths: [packages/**]}}\njobs: {}\n",
            "on: {pull_request: {paths-ignore: [docs/**]}}\njobs: {}\n",
        )
        for control in positive_controls:
            with self.subTest(control=control):
                self.assertTrue(trigger_path_filters(control))

    def test_release_callers_pin_the_landed_policy_and_matching_identity(self) -> None:
        for component, source_directory in RELEASE_CALLERS.items():
            workflow = (
                ROOT / ".github" / "workflows" / f"release-{component}.yml"
            ).read_text(encoding="utf-8")
            with self.subTest(component=component):
                self.assertIn(
                    "uses: ryanduguid/release-policy/.github/workflows/"
                    f"release-python.yml@{RELEASE_POLICY_SHA}",
                    workflow,
                )
                self.assertIn(f"source-directory: {source_directory}", workflow)
                self.assertIn(f"tag-prefix: {component}", workflow)

    def test_every_component_has_production_modules(self) -> None:
        for component, package in {**ENGINES, **APPLICATION}.items():
            with self.subTest(component=component):
                self.assertTrue((ROOT / component / "pyproject.toml").is_file())
                self.assertGreater(len(production_modules(component, package)), 0)

    def test_engines_do_not_import_the_application_or_each_other(self) -> None:
        for component, package in ENGINES.items():
            forbidden = FORBIDDEN_FOR_ENGINES - {package}
            for path, _depth in production_modules(component, package):
                with self.subTest(module=str(path.relative_to(ROOT))):
                    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
                    self.assertEqual(imported_top_levels(tree) & forbidden, set())

    def test_no_production_module_escapes_its_package_by_relative_import(self) -> None:
        for component, package in {**ENGINES, **APPLICATION}.items():
            for path, depth in production_modules(component, package):
                with self.subTest(module=str(path.relative_to(ROOT))):
                    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
                    self.assertEqual(escaping_relative_imports(tree, depth), [])

    def test_positive_control_for_forbidden_imports(self) -> None:
        tree = ast.parse(
            "import aus_accounting_mcp\n"
            "from paydaysuper.assess import assess\n"
            "import json\n"
        )
        self.assertEqual(
            imported_top_levels(tree) & FORBIDDEN_FOR_ENGINES,
            {"aus_accounting_mcp", "paydaysuper"},
        )

    def test_positive_control_for_escaping_relative_imports(self) -> None:
        inside = ast.parse("from . import money\nfrom .money import parse\n")
        escaping = ast.parse("from .. import other_engine\nfrom ...apps import server\n")
        self.assertEqual(escaping_relative_imports(inside, depth=1), [])
        self.assertEqual(
            escaping_relative_imports(escaping, depth=1),
            ["level 2 import of . (other_engine)", "level 3 import of apps (server)"],
        )


if __name__ == "__main__":
    unittest.main()
