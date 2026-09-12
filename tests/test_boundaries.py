"""Static boundary checks for this repository. Standard library only.

Two properties are proved with the ast module over every production module:

1. no engine production module imports ``aus_accounting_mcp`` or a sibling engine
   package (engines depend on nothing in this repository);
2. no production module, engine or application, uses a relative import whose level
   climbs out of its own top-level package directory.

Each property has an in-memory positive control so a scan that silently finds
nothing cannot pass. The remaining tests hold the CI routing: every engine is
called with the same gates, and the package path filter still routes a change to
the engine that owns it. Run from the repository root:

    python -m unittest -v tests/test_boundaries.py
"""

from __future__ import annotations

import ast
import importlib.util
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RELEASE_POLICY_SHA = "c24612618f177f6dccb7ed9ee2a8648959b87e2e"


def _load_select_package():
    """Import the CI package selector, which lives outside any import path."""
    path = ROOT / ".github" / "ci" / "select_package.py"
    spec = importlib.util.spec_from_file_location("select_package", path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


select_package = _load_select_package()

RELEASE_CALLERS = {
    "australian-tax-calculators": "packages/australian-tax-calculators",
    "aus-accounting-mcp": "apps/aus-accounting-mcp",
    "ato-benchmark-compare": "packages/ato-benchmark-compare",
    "payday-super-checker": "packages/payday-super-checker",
    "div7a-loan-review": "packages/div7a-loan-review",
    "the-exchequer-tally": "packages/the-exchequer-tally",
    "solomons-sword": "packages/solomons-sword",
    "the-wip-tally": "packages/the-wip-tally",
}

ENGINES = {
    "packages/australian-tax-calculators": "austaxcalc",
    "packages/ato-benchmark-compare": "atobenchmark",
    "packages/payday-super-checker": "paydaysuper",
    "packages/div7a-loan-review": "div7aloan",
    "packages/the-exchequer-tally": "edwinnixon",
    "packages/solomons-sword": "louisgoldberg",
    "packages/the-wip-tally": "wiptally",
}
APPLICATION = {"apps/aus-accounting-mcp": "aus_accounting_mcp"}

# The files the changed-line coverage gate holds to complete branch coverage,
# and the engine that owns each set.
CHANGED_LINE_COVERAGE = {
    "ato-benchmark-compare": "atobenchmark/mapping.py",
    "payday-super-checker": "paydaysuper/assess.py,paydaysuper/report.py",
}
FORBIDDEN_FOR_ENGINES = frozenset({"aus_accounting_mcp", *ENGINES.values()})
PATH_FILTER_KEY = re.compile(
    r"(?<![\w-])['\"]?(paths(?:-ignore)?)['\"]?\s*:"
)


def trigger_path_filters(workflow: str) -> list[str]:
    """Find path-filter keys in either block or flow-style ``on`` mappings."""
    lines = workflow.splitlines()
    trigger_lines: list[str] = []
    for index, line in enumerate(lines):
        match = re.match(r"^(?:on|\"on\"|'on')\s*:(.*)$", line)
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
    def test_root_policy_changes_run_every_engine(self) -> None:
        # The root workspace files belong here with the policy files: the root
        # uv.lock is what `uv run --locked` validates from inside every component
        # directory, so a change to it changes what every component resolves.
        shared_paths = {
            "AGENTS.md", "CONTRIBUTING.md", "README.md", "SECURITY.md",
            "IMPORTS.md", ".editorconfig", ".gitignore", ".mailmap",
            ".gitattributes", ".github/workflows/ci.yml",
            "pyproject.toml", "uv.lock", "justfile",
        }
        for component in ENGINES:
            package = Path(component).name
            for path in shared_paths:
                with self.subTest(package=package, path=path):
                    self.assertTrue(select_package.should_run(package, [path]))

    def test_a_change_to_one_engine_does_not_run_another(self) -> None:
        for component in ENGINES:
            package = Path(component).name
            siblings = [
                f"{other}/engine.py" for other in ENGINES if other != component
            ]
            with self.subTest(package=package):
                self.assertTrue(
                    select_package.should_run(package, [f"{component}/engine.py"])
                )
                self.assertFalse(select_package.should_run(package, siblings))
                # The application is not an engine dependency, so an application
                # change alone does not run an engine's gates.
                self.assertFalse(
                    select_package.should_run(
                        package, ["apps/aus-accounting-mcp/server.py"]
                    )
                )

    def test_a_cross_package_move_runs_both_packages(self) -> None:
        # git reports a detected rename as its destination alone, so a file moved
        # out of one package would leave that package's gates unrun even though
        # it lost source. The diff has to name both sides.
        reusable = (ROOT / ".github" / "workflows" / "ci-package.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn("git diff --name-only --no-renames", reusable)

        moved = ["packages/the-wip-tally/moved.py", "packages/solomons-sword/moved.py"]
        for package in ("the-wip-tally", "solomons-sword"):
            with self.subTest(package=package):
                self.assertTrue(select_package.should_run(package, moved))

    def test_an_unknown_comparison_point_runs_every_gate(self) -> None:
        # A dispatch, a new branch and a force push give the workflow no list of
        # changed paths. Selection must fail open rather than skip a gate.
        for component in ENGINES:
            with self.subTest(package=Path(component).name):
                self.assertTrue(select_package.should_run(Path(component).name, []))

    def test_every_engine_is_called_with_the_same_gates(self) -> None:
        caller = (ROOT / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )
        reusable = (ROOT / ".github" / "workflows" / "ci-package.yml").read_text(
            encoding="utf-8"
        )

        self.assertIn("uses: ./.github/workflows/ci-package.yml", caller)
        called = set(re.findall(r"(?m)^          - package: (\S+)$", caller))
        self.assertEqual(called, {Path(component).name for component in ENGINES})
        for component, import_name in ENGINES.items():
            with self.subTest(component=component):
                self.assertIn(
                    f"- package: {Path(component).name}\n"
                    f"            import-name: {import_name}\n",
                    caller,
                )

        # The gate set every engine gets, and the declared Python floor and ceiling.
        # ruff, mypy and coverage read their scope from the engine's own
        # configuration, so the commands are identical for every engine.
        for gate in (
            'uv lock --check',
            "ruff check .\n",
            "uv run --locked --extra dev mypy\n",
            "uv run --locked --extra dev pytest\n",
            "--cov --cov-branch --cov-report=term-missing --cov-report=xml",
            'pip-audit --local --strict',
            "python -m build",
            'python: ["3.10", "3.12", "3.13"]',
        ):
            with self.subTest(gate=gate):
                self.assertIn(gate, reusable)

        # The wheel smoke installs the one built wheel by path, never by name.
        self.assertIn("wheels=(dist/*.whl)", reusable)
        self.assertIn('pip" install --no-index "${wheels[0]}"', reusable)
        self.assertNotIn("--find-links", reusable)

    def test_every_engine_configures_the_shared_gate_scope(self) -> None:
        # ruff check ., mypy and pytest --cov take their scope from the engine's
        # pyproject.toml, so each engine has to declare all three.
        for component, import_name in ENGINES.items():
            pyproject = (ROOT / component / "pyproject.toml").read_text(encoding="utf-8")
            with self.subTest(component=component):
                self.assertIn("[tool.ruff]", pyproject)
                self.assertIn(f'packages = ["{import_name}"]', pyproject)
                self.assertIn(
                    f'[tool.coverage.run]\n'
                    f'# The source pytest --cov measures; CI adds --cov-branch and the reports.\n'
                    f'source = ["{import_name}"]\n',
                    pyproject,
                )

    def test_imported_diff_coverage_waits_for_a_mainline_baseline(self) -> None:
        caller = (ROOT / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )
        reusable = (ROOT / ".github" / "workflows" / "ci-package.yml").read_text(
            encoding="utf-8"
        )

        for package, held in CHANGED_LINE_COVERAGE.items():
            with self.subTest(package=package):
                self.assertIn(f"changed-line-coverage: {held}\n", caller)
                self.assertTrue((ROOT / "packages" / package / held.split(",")[0]).is_file())

        # The gate is fail-closed on changed lines, and the sentinel it waits for
        # is the first held file of whichever engine is running.
        self.assertIn('sentinel="packages/$PACKAGE/${INCLUDE%%,*}"', reusable)
        self.assertIn('git cat-file -e "origin/main:$sentinel"', reusable)
        self.assertIn("--compare-branch=origin/main", reusable)
        self.assertIn("--branch-coverage", reusable)
        self.assertIn("--fail-under=100", reusable)

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
        positive_controls += tuple(
            control.replace("on:", quoted + ":", 1)
            for control in positive_controls for quoted in ('"on"', "'on'")
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
