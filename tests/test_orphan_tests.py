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
    "tools/tool/package.json": '{"scripts": {"test": "node --test --test-concurrency=1 tests/runtime.test.mjs"}}',
    "tools/tool/tests/runtime.test.mjs": "",
    "tools/kept/Cargo.toml": "",
}
KEPT = {"tools/kept": "a fixture crate, built by hand"}
# What the fixture leaves unrun, each for the reason the first test gives.
UNRUN = {"app/scripts/test_outside.py", "app/tests/helper_test.py", "crates/second/wasm", "crates/unlisted"}
TOOL_TEST = "          cargo +1.99 test --locked\n"
CONDITION = "runner.os == 'Linux' && (inputs.package == 'covered' || inputs.package == 'elsewhere')"
PYTEST = "pytest -q --cov=app"
LINUX = "    runs-on: ubuntu-latest\n"
# The called job on 2 runners, each with its own tier.
TIERS = (
    "    runs-on: ${{ matrix.os }}\n"
    "    strategy:\n"
    "      matrix:\n"
    "        include:\n"
    "          - os: ubuntu-latest\n"
    "            tier: full\n"
    "          - os: windows-latest\n"
    "            tier: quick\n"
)


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

    def test_credits_only_what_a_step_runs(self) -> None:
        cases = {
            # A job runs only when the jobs it needs succeed.
            "a caller that never runs": (
                "ci.yml",
                CALLER.replace("  rust:\n", "  rust:\n    needs: setup\n    if: needs.setup.result == 'skipped'\n"),
                UNRUN | {"crates/covered", "crates/covered/wasm", "crates/second"},
            ),
            "input tests no call can pass": (
                "rust.yml",
                CALLED.replace(CONDITION, "inputs.package == 'covered' && inputs.package == 'second'"),
                UNRUN | {"crates/covered/wasm"},
            ),
            "2 runner systems at once": (
                "rust.yml",
                CALLED.replace(
                    CONDITION,
                    "runner.os == 'Linux' && runner.os == 'Windows' && (inputs.package == 'covered')",
                ),
                UNRUN | {"crates/covered/wasm"},
            ),
            "a runner system the job does not use": (
                "rust.yml",
                CALLED.replace(CONDITION, "runner.os == 'Windows' && (inputs.package == 'covered')"),
                UNRUN | {"crates/covered/wasm"},
            ),
            "a runner system one matrix row uses": (
                "rust.yml",
                CALLED.replace(LINUX, TIERS).replace(
                    CONDITION, "runner.os == 'Windows' && (inputs.package == 'covered')"
                ),
                UNRUN,
            ),
            "a needs output the job and step set apart": (
                "rust.yml",
                CALLED.replace(LINUX, "    needs: setup\n    if: needs.setup.outputs.run == 'true'\n" + LINUX)
                .replace(CONDITION, "needs.setup.outputs.run == 'false' && " + CONDITION),
                UNRUN | {"crates/covered/wasm"},
            ),
            # GitHub compares strings without regard to case.
            "an input in another case": (
                "rust.yml",
                CALLED.replace(CONDITION, "runner.os == 'Linux' && inputs.package != 'COVERED'"),
                UNRUN - {"crates/second/wasm"} | {"crates/covered/wasm"},
            ),
            # Bash deletes a backslash-newline.
            "a command split over 2 lines": (
                "ci.yml", CALLER.replace(TOOL_TEST, "          cargo +1.99 test --lock\\\n          ed\n"), UNRUN,
            ),
        }
        for case, (workflow, text, unrun) in cases.items():
            with self.subTest(case=case):
                self.assertEqual(self.unrun({f".github/workflows/{workflow}": text}), unrun)

    def test_stops_outside_the_grammar(self) -> None:
        caller, called = ".github/workflows/ci.yml", ".github/workflows/rust.yml"
        cases = {
            "an anchor": (
                {caller: CALLER.replace("fail-fast: false", "fail-fast: &off false")}, "anchor",
            ),
            "a literal false": (
                {caller: CALLER.replace("  rust:\n", "  rust:\n    if: ${{ false }}\n")},
                "'false' is outside the condition grammar",
            ),
            "a disjunction beside an input test": (
                {called: CALLED.replace(CONDITION, "runner.os == 'Linux' || inputs.package == 'covered'")},
                "outside the condition grammar",
            ),
            "a false term beside an input test": (
                {called: CALLED.replace(CONDITION, "false && inputs.package == 'covered'")},
                "'false' is outside the condition grammar",
            ),
            "a matrix term": (
                {called: CALLED.replace(LINUX, TIERS).replace(
                    CONDITION, "runner.os == 'Windows' && matrix.tier == 'full' && (inputs.package == 'covered')"
                )},
                "outside the condition grammar",
            ),
            "a github term the job and step set apart": (
                {called: CALLED.replace(LINUX, "    if: github.ref == 'refs/heads/main'\n" + LINUX)
                 .replace(CONDITION, "github.ref != 'refs/heads/main' && " + CONDITION)},
                "outside the condition grammar",
            ),
            "a needs output of a job it does not need": (
                {called: CALLED.replace(CONDITION, "needs.setup.outputs.run == 'true' && " + CONDITION)},
                "a job it does not need",
            ),
            # GitHub compares a boolean with a string as numbers.
            "a boolean input": (
                {
                    caller: CALLER.replace(
                        "      package: ${{ matrix.crate }}\n", "      package: ${{ matrix.crate }}\n      quick: true\n"
                    ),
                    called: CALLED.replace("      root:\n", "      quick:\n        type: boolean\n      root:\n")
                    .replace(CONDITION, "inputs.quick == 'true' && " + CONDITION),
                },
                "without a known inputs.quick",
            ),
            "a self-hosted runner": (
                {called: CALLED.replace(LINUX, "    runs-on: ubuntu-custom\n")}, "without a known runner.os",
            ),
            "an excluded row": (
                {called: CALLED.replace(LINUX, (
                    "    runs-on: ${{ matrix.os }}\n"
                    "    strategy:\n"
                    "      matrix:\n"
                    "        os: [ubuntu-latest, windows-latest]\n"
                    "        exclude: [{os: ubuntu-latest}]\n"
                ))},
                "outside the matrix grammar",
            ),
            "another shell": (
                {caller: CALLER.replace(
                    "        working-directory: tools/tool\n",
                    "        working-directory: tools/tool\n        shell: python\n",
                )},
                "shell 'python' is outside the step grammar",
            ),
            "a directory change": (
                {caller: CALLER.replace("          npm test\n", "          cd .. && npm test\n")},
                "outside the step grammar",
            ),
            "an unresolved directory": (
                {caller: CALLER.replace(
                    "working-directory: tools/tool", "working-directory: ${{ matrix.tool }}"
                )},
                "cannot resolve the working directory",
            ),
            "a heredoc": (
                {caller: CALLER.replace(
                    TOOL_TEST, "          cat > notes.txt <<'EOF'\n          cargo test\n          EOF\n"
                )},
                "outside the step grammar",
            ),
            "a condition in the script": (
                {caller: CALLER.replace(
                    TOOL_TEST, "          if false; then cargo +1.99 test --locked; fi\n"
                )},
                "outside the step grammar",
            ),
            "a short-circuit": (
                {caller: CALLER.replace(TOOL_TEST, "          false && cargo +1.99 test --locked\n")},
                "outside the step grammar",
            ),
            "a short-circuit into a subshell": (
                {caller: CALLER.replace(TOOL_TEST, "          false&&(cargo +1.99 test --locked)\n")},
                "outside the step grammar",
            ),
            "a function that is never called": (
                {caller: CALLER.replace(TOOL_TEST, "          f ( ) { cargo +1.99 test --locked; }\n")},
                "outside the step grammar",
            ),
            "quoted shell punctuation": (
                {caller: CALLER.replace(TOOL_TEST, "          printf '%s\\n' '()'\n" + TOOL_TEST)},
                "outside the step grammar",
            ),
            "a substitution that only prints": (
                {caller: CALLER.replace(TOOL_TEST, '          echo "$(echo "; cargo test; ")"\n')},
                "outside the step grammar",
            ),
            "a build without the tests": (
                {caller: CALLER.replace(TOOL_TEST, "          cargo +1.99 test --locked --no-run\n")},
                "outside the step grammar",
            ),
            "a build split over 2 lines": (
                {caller: CALLER.replace(TOOL_TEST, "          cargo +1.99 test --locked --no-\\\n          run\n")},
                re.escape("'cargo +1.99 test --locked --no-run' is outside the step grammar"),
            ),
            "another package": (
                {caller: CALLER.replace(TOOL_TEST, "          cargo +1.99 test --locked -p other\n")},
                "outside the step grammar",
            ),
            "another fuzz directory": (
                {caller: CALLER.replace(TOOL_TEST, "          cargo +1.99 fuzz run --fuzz-dir other\n")},
                "outside the step grammar",
            ),
            "a unittest filter": (
                {caller: CALLER.replace(TOOL_TEST, "          python -m unittest -kx.py tests/test_a.py\n")},
                "outside the step grammar",
            ),
            "a keyword filter": (
                {caller: CALLER.replace(PYTEST, "pytest -q -k covered --cov=app")}, "'-k' is outside",
            ),
            "a cargo runner in env": (
                {caller: CALLER.replace(
                    "  tool:\n    runs-on: ubuntu-latest\n",
                    "  tool:\n    runs-on: ubuntu-latest\n    env:\n"
                    "      CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER: echo\n",
                )},
                re.escape("env ['CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER'] is outside"),
            ),
            "a keyword filter in PYTEST_ADDOPTS": (
                {caller: CALLER.replace(
                    "        working-directory: app\n",
                    "        working-directory: app\n    env:\n      PYTEST_ADDOPTS: -k covered\n",
                )},
                "'-k' is outside",
            ),
            "an ignored path in addopts": (
                {"app/pyproject.toml": FILES["app/pyproject.toml"]
                 + 'addopts = "--ignore=tests/test_covered.py"\n'},
                "'--ignore=tests/test_covered.py' is outside",
            ),
            "pytest without its pyproject table": (
                {"app/pyproject.toml": '[project]\nname = "app"\n'}, "pytest table",
            ),
            "a recursive glob": (
                {"tools/tool/package.json": FILES["tools/tool/package.json"].replace(
                    "tests/runtime.test.mjs", "tests/**/*.test.mjs"
                )},
                "outside the step grammar",
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
