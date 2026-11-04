"""The orphan check: every test file and crate runs in some workflow step."""

import re
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / ".github/ci/check_orphan_tests.py"

sys.path.insert(0, str(CHECK.parent))
try:
    import check_orphan_tests as orphans
finally:
    sys.path.pop(0)

CALLER = """name: ci
on:
  pull_request:
jobs:
  python:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: app
    steps:
      - uses: actions/checkout@0000000000000000000000000000000000000000 # v1
      - run: uv run --locked --extra dev pytest -q --cov=app  # NOSONAR: a trailing comment
  rust:
    strategy:
      fail-fast: false
      matrix:
        include:
          - crate: covered
          - crate: second
    uses: ./.github/workflows/rust.yml
    with:
      package: ${{ matrix.crate }}
  tool:
    runs-on: ubuntu-latest
    steps:
      - name: Test the tool
        working-directory: tools/tool
        run: |
          # npm test runs package.json's test script.
          cargo +1.99 test --locked
          npm test
"""
CALLED = """name: rust
on:
  workflow_call:
    inputs:
      package:
        description: >-
          The crate directory under root.
        type: string
        required: true
      root:
        type: string
        default: crates
jobs:
  check:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ${{ inputs.root }}/${{ inputs.package }}
    steps:
      - run: cargo test --locked
      - if: runner.os == 'Linux' && (inputs.package == 'covered' || inputs.package == 'elsewhere')
        run: >-
          cargo test
          --manifest-path wasm/Cargo.toml --locked
"""
FILES = {
    ".github/workflows/ci.yml": CALLER,
    ".github/workflows/rust.yml": CALLED,
    "app/pyproject.toml": '[tool.pytest.ini_options]\ntestpaths = ["tests"]\npython_files = ["test_*.py"]\n',
    "app/tests/test_covered.py": "",
    "app/tests/helper_test.py": "",
    "app/scripts/test_outside.py": "",
    "crates/covered/Cargo.toml": "",
    "crates/covered/wasm/Cargo.toml": "",
    "crates/second/Cargo.toml": "",
    "crates/second/wasm/Cargo.toml": "",
    "crates/unlisted/Cargo.toml": "",
    "tools/tool/Cargo.toml": "",
    "tools/tool/package.json": '{"scripts": {"test": "node --test --test-concurrency=1 tests/*.test.mjs"}}',
    "tools/tool/tests/runtime.test.mjs": "",
    "tools/kept/Cargo.toml": "",
}
KEPT = {"tools/kept": "a fixture crate, built by hand"}
# What the fixture leaves unrun, each for the reason the first test gives.
UNRUN = {"app/scripts/test_outside.py", "app/tests/helper_test.py", "crates/second/wasm", "crates/unlisted"}
TOOL_TEST = "          cargo +1.99 test --locked\n"
CONDITION = "runner.os == 'Linux' && (inputs.package == 'covered' || inputs.package == 'elsewhere')"
PYTEST = "pytest -q --cov=app"


class OrphanTestsTests(unittest.TestCase):
    def check(self, files: dict[str, str], allowed: dict[str, str]) -> list[str]:
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            for path, text in files.items():
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                (root / path).write_text(text, encoding="utf-8")
            return orphans.problems(root, list(files), allowed)

    def unrun(self, files: dict[str, str]) -> set[str]:
        return {
            problem.split(": no workflow step runs", 1)[0]
            for problem in self.check(FILES | files, KEPT)
        }

    def test_names_each_orphan_and_nothing_a_step_runs(self) -> None:
        self.assertEqual(self.check(FILES, KEPT), [
            # python_files leaves it out, although it is under testpaths.
            "app/scripts/test_outside.py: no workflow step runs this test file",
            "app/tests/helper_test.py: no workflow step runs this test file",
            # The step's condition names only the covered crate.
            "crates/second/wasm: no workflow step runs this crate",
            "crates/unlisted: no workflow step runs this crate",
        ])

    def test_allow_list_entries_must_still_apply(self) -> None:
        allowed = {
            "crates/unlisted": "",
            "app/tests/test_gone.py": "removed",
            "app/tests/test_covered.py": "slow",
            "tools/kept": "a fixture crate",
        }
        problems = self.check(FILES, allowed)
        self.assertIn("orphan_tests_allowed.txt: give crates/unlisted a reason", problems)
        self.assertIn(
            "orphan_tests_allowed.txt: app/tests/test_gone.py is not a test file or crate", problems
        )
        self.assertIn(
            "orphan_tests_allowed.txt: a workflow step runs app/tests/test_covered.py now; "
            "remove its entry",
            problems,
        )
        self.assertNotIn("crates/unlisted: no workflow step runs this crate", problems)

    def test_credits_nothing_a_step_does_not_run(self) -> None:
        cases = {
            "a caller that never runs": (
                "ci.yml", CALLER.replace("  rust:\n", "  rust:\n    if: ${{ false }}\n"),
                {"crates/covered", "crates/covered/wasm", "crates/second"},
            ),
            "input tests no call can pass": (
                "rust.yml",
                CALLED.replace(CONDITION, "inputs.package == 'covered' && inputs.package == 'second'"),
                {"crates/covered/wasm"},
            ),
            "an input test beside another term in a disjunction": (
                "rust.yml",
                CALLED.replace(CONDITION, "runner.os == 'Linux' || inputs.package == 'covered'"),
                {"crates/covered/wasm"},
            ),
            "a build without the tests": (
                "ci.yml", CALLER.replace(TOOL_TEST, "          cargo +1.99 test --locked --no-run\n"),
                {"tools/tool"},
            ),
        }
        for case, (workflow, text, unrun) in cases.items():
            with self.subTest(case=case):
                self.assertEqual(self.unrun({f".github/workflows/{workflow}": text}), UNRUN | unrun)

    def test_stops_on_what_it_does_not_model(self) -> None:
        caller = ".github/workflows/ci.yml"
        cases = {
            "an anchor": (
                {caller: CALLER.replace("fail-fast: false", "fail-fast: &off false")}, "anchor",
            ),
            "a directory change": (
                {caller: CALLER.replace("          npm test\n", "          cd .. && npm test\n")},
                "changes directory",
            ),
            "an unresolved directory": (
                {caller: CALLER.replace(
                    "working-directory: tools/tool", "working-directory: ${{ matrix.tool }}"
                )},
                "cannot resolve the working directory",
            ),
            "a keyword filter": (
                {caller: CALLER.replace(PYTEST, "pytest -q -k covered --cov=app")}, "narrow",
            ),
            "an ignored path in addopts": (
                {"app/pyproject.toml": FILES["app/pyproject.toml"]
                 + 'addopts = "--ignore=tests/test_covered.py"\n'},
                "narrow",
            ),
            "a heredoc": (
                {caller: CALLER.replace(
                    TOOL_TEST, "          cat > notes.txt <<'EOF'\n          cargo test\n          EOF\n"
                )},
                "control flow or a heredoc",
            ),
            "a condition in the script": (
                {caller: CALLER.replace(
                    TOOL_TEST, "          if false; then cargo +1.99 test --locked; fi\n"
                )},
                "control flow or a heredoc",
            ),
            "a recursive glob": (
                {"tools/tool/package.json": FILES["tools/tool/package.json"].replace(
                    "tests/*.test.mjs", "tests/**/*.test.mjs"
                )},
                "recursive glob",
            ),
        }
        for case, (files, message) in cases.items():
            with self.subTest(case=case), self.assertRaisesRegex(orphans.Unreadable, message):
                self.check(FILES | files, {})

    def test_every_change_runs_the_check(self) -> None:
        workflow = (ROOT / ".github/workflows/boundaries.yml").read_text(encoding="utf-8")
        boundaries = re.search(r"(?ms)^  boundaries:\n(.*?)(?=^  \S)", workflow)
        assert boundaries is not None
        self.assertIn("run: python .github/ci/check_orphan_tests.py\n", boundaries.group(1))
        self.assertNotIn("paths", workflow.split("\njobs:\n", 1)[0])


if __name__ == "__main__":
    unittest.main()
