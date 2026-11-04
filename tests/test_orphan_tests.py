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
# A job for others to need.
SETUP = "  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ready\n"
SETUP_RUN = SETUP.replace("    runs-on:", "    outputs:\n      run: ${{ steps.select.outputs.run }}\n    runs-on:")
STEP_IF = "      - if: " + CONDITION + "\n"
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
        caller, called = ".github/workflows/ci.yml", ".github/workflows/rust.yml"
        crates = {"crates/covered", "crates/covered/wasm", "crates/second"}
        cases = {
            # A job runs only when the jobs it needs succeed.
            "a caller that never runs": (
                {caller: CALLER.replace(
                    "  rust:\n", SETUP + "  rust:\n    needs: setup\n    if: needs.setup.result == 'skipped'\n"
                )},
                UNRUN | crates,
            ),
            "a caller whose needed job never runs": (
                {caller: CALLER.replace("  rust:\n", SETUP + SETUP.replace(
                    "  setup:\n", "  gate:\n    needs: setup\n    if: needs.setup.result == 'skipped'\n"
                ) + "  rust:\n    needs: gate\n")},
                UNRUN | crates,
            ),
            "a needed job that never runs": (
                {called: CALLED.replace("  check:\n", SETUP.replace(
                    "    runs-on:", "    if: inputs.package == 'neither'\n    runs-on:"
                ) + "  check:\n    needs: setup\n")},
                UNRUN | crates,
            ),
            "a job needed by a needed job that never runs": (
                {called: CALLED.replace("  check:\n", SETUP.replace(
                    "    runs-on:", "    if: inputs.package == 'neither'\n    runs-on:"
                ) + SETUP.replace("  setup:\n", "  middle:\n    needs: setup\n") + "  check:\n    needs: middle\n")},
                UNRUN | crates,
            ),
            "input tests no call can pass": (
                {called: CALLED.replace(CONDITION, "inputs.package == 'covered' && inputs.package == 'second'")},
                UNRUN | {"crates/covered/wasm"},
            ),
            "2 runner systems at once": (
                {called: CALLED.replace(
                    CONDITION,
                    "runner.os == 'Linux' && runner.os == 'Windows' && (inputs.package == 'covered')",
                )},
                UNRUN | {"crates/covered/wasm"},
            ),
            "a runner system the job does not use": (
                {called: CALLED.replace(CONDITION, "runner.os == 'Windows' && (inputs.package == 'covered')")},
                UNRUN | {"crates/covered/wasm"},
            ),
            "a runner system one matrix row uses": (
                {called: CALLED.replace(LINUX, TIERS).replace(
                    CONDITION, "runner.os == 'Windows' && (inputs.package == 'covered')"
                )},
                UNRUN,
            ),
            "a needs output the job and step set apart": (
                {called: CALLED.replace(
                    "  check:\n", SETUP_RUN + "  check:\n    needs: setup\n    if: needs.setup.outputs.run == 'true'\n"
                ).replace(CONDITION, "needs.setup.outputs.run == 'false' && " + CONDITION)},
                UNRUN | {"crates/covered/wasm"},
            ),
            "a needs output the job sets for the step": (
                {called: CALLED.replace(
                    "  check:\n", SETUP_RUN + "  check:\n    needs: setup\n    if: needs.setup.outputs.run == 'true'\n"
                ).replace(CONDITION, "needs.setup.outputs.run == 'true' && " + CONDITION)},
                UNRUN,
            ),
            # A literal block keeps its final line break, so no call passes 'covered'.
            "an input from a literal block": (
                {caller: CALLER.replace(
                    "      package: ${{ matrix.crate }}\n", "      package: |\n        covered\n"
                )},
                UNRUN | crates,
            ),
            # An empty keep block is its line breaks, so the input is not ''.
            "an input from an empty keep block": (
                {
                    caller: CALLER.replace(
                        "      package: ${{ matrix.crate }}\n", "      package: ${{ matrix.crate }}\n      gate: |+\n\n"
                    ),
                    called: CALLED.replace("      root:\n", "      gate:\n        type: string\n      root:\n")
                    .replace(CONDITION, "inputs.gate == '' && " + CONDITION),
                },
                UNRUN | {"crates/covered/wasm"},
            ),
            # A block that ends the file without a line break has none to keep.
            "an input from a block at the end of the file": (
                {caller: CALLER.split("  tool:\n")[0].replace(
                    "      package: ${{ matrix.crate }}\n", "      package: |\n        covered"
                )},
                UNRUN | {"crates/second", "tools/tool", "tools/tool/tests/runtime.test.mjs"},
            ),
            # YAML reads '' between single quotes as one quote.
            "a doubled single quote": (
                {called: CALLED.replace(STEP_IF, "      - if: 'runner.os != ''Linux'''\n")},
                UNRUN | {"crates/covered/wasm"},
            ),
            # GitHub compares strings without regard to case, but keeps their spaces.
            "an input in another case": (
                {called: CALLED.replace(CONDITION, "runner.os == 'Linux' && inputs.package != 'COVERED'")},
                UNRUN - {"crates/second/wasm"} | {"crates/covered/wasm"},
            ),
            "an input with other spacing": (
                {
                    caller: CALLER.replace(
                        "      package: ${{ matrix.crate }}\n", "      package: ${{ matrix.crate }}\n      label: a b\n"
                    ),
                    called: CALLED.replace("      root:\n", "      label:\n        type: string\n      root:\n")
                    .replace(CONDITION, "inputs.label == 'a  b' && " + CONDITION),
                },
                UNRUN | {"crates/covered/wasm"},
            ),
            # Bash deletes a backslash-newline.
            "a command split over 2 lines": (
                {caller: CALLER.replace(TOOL_TEST, "          cargo +1.99 test --lock\\\n          ed\n")}, UNRUN,
            ),
            "a coverage run with a floor": (
                {caller: CALLER.replace(TOOL_TEST, "          cargo llvm-cov --locked --fail-under-lines 80\n")},
                UNRUN,
            ),
            # pytest skips these directories.
            "a test file under a default norecursedirs name": (
                {"app/tests/build/test_built.py": ""}, UNRUN | {"app/tests/build/test_built.py"},
            ),
            "a test file in a virtual environment": (
                {"app/tests/env/pyvenv.cfg": "", "app/tests/env/test_env.py": ""},
                UNRUN | {"app/tests/env/test_env.py"},
            ),
        }
        for case, (files, unrun) in cases.items():
            with self.subTest(case=case):
                self.assertEqual(self.unrun(files), unrun)

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
            # A step's condition cannot fix an output: its job passes either way.
            "a step output its job leaves open": (
                {called: CALLED.replace("  check:\n", SETUP_RUN + "  check:\n    needs: setup\n")
                 .replace(CONDITION, "needs.setup.outputs.run == 'true' && " + CONDITION)},
                "leaves open",
            ),
            "an output its job does not declare": (
                {called: CALLED.replace("  check:\n", SETUP + "  check:\n    needs: setup\n")
                 .replace(CONDITION, "needs.setup.outputs.run == 'true' && " + CONDITION)},
                "does not declare",
            ),
            "an escape in a double-quoted condition": (
                {called: CALLED.replace(STEP_IF, "      - if: \"runner.os != 'Lin\\u0075x'\"\n")},
                "an escape in a double-quoted scalar",
            ),
            "an escape in a double-quoted key": (
                {called: CALLED.replace(STEP_IF, "      - \"i\\u0066\": runner.os != 'Linux'\n")},
                "an escape in a double-quoted scalar",
            ),
            # Unset, the shell is PowerShell on Windows, where a failing native command fails
            # the step only if it is the last.
            "several lines on Windows without shell: bash": (
                {called: CALLED.replace(LINUX, TIERS).replace(
                    "      - run: cargo test --locked\n",
                    "      - run: |\n          cargo test --locked\n          cargo fmt --check\n",
                )},
                "need shell: bash",
            ),
            "a needs output of a job it does not need": (
                {called: CALLED.replace(CONDITION, "needs.setup.outputs.run == 'true' && " + CONDITION)},
                "a job it does not need",
            ),
            "a needed job's condition outside the grammar": (
                {called: CALLED.replace("  check:\n", SETUP.replace(
                    "    runs-on:", "    if: github.ref == 'refs/heads/main'\n    runs-on:"
                ) + "  check:\n    needs: setup\n")},
                "outside the condition grammar",
            ),
            "a needed job that may fail": (
                {called: CALLED.replace("  check:\n", SETUP.replace(
                    "    runs-on:", "    continue-on-error: true\n    runs-on:"
                ) + "  check:\n    needs: setup\n")},
                "job setup: continue-on-error",
            ),
            "a test job that may fail": (
                {called: CALLED.replace(LINUX, LINUX + "    continue-on-error: true\n")},
                "job check: continue-on-error",
            ),
            "a needed job that is not there": (
                {called: CALLED.replace("  check:\n", "  check:\n    needs: setup\n")},
                "'setup', which is not a job",
            ),
            "a needs list outside the grammar": (
                {called: CALLED.replace("  check:\n", SETUP.replace(
                    "    runs-on:", "    if: inputs.package == 'neither'\n    runs-on:"
                ) + "  check:\n    needs:\n      - setup: x\n")},
                "needs .* is outside the grammar",
            ),
            "a test step that may fail": (
                {caller: CALLER.replace(
                    "        working-directory: tools/tool\n",
                    "        working-directory: tools/tool\n        continue-on-error: ${{ matrix.experimental }}\n",
                )},
                "step 1: continue-on-error",
            ),
            "a call from a workflow nothing calls": (
                {".github/workflows/nested.yml": (
                    "name: nested\non:\n  workflow_call:\njobs:\n  rust:\n"
                    "    uses: ./.github/workflows/rust.yml\n    with:\n      package: unlisted\n"
                )},
                "a reusable workflow's call",
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
            # Only cargo llvm-cov takes a coverage floor.
            "a coverage floor on cargo test": (
                {caller: CALLER.replace(TOOL_TEST, "          cargo +1.99 test --locked --fail-under-lines 0\n")},
                "outside the step grammar",
            ),
            "an unreadable script behind a false condition": (
                {called: CALLED.replace(CONDITION, "runner.os != 'Linux'").replace(
                    "          cargo test\n          --manifest-path wasm/Cargo.toml --locked\n",
                    "          false && cargo test\n",
                )},
                "outside the step grammar",
            ),
            "an unreadable npm script behind a false condition": (
                {
                    caller: CALLER.replace(
                        "      - name: Test the tool\n", "      - name: Test the tool\n        if: runner.os != 'Linux'\n"
                    ),
                    "tools/tool/package.json": FILES["tools/tool/package.json"].replace(
                        "tests/runtime.test.mjs", "tests/*.test.mjs"
                    ),
                },
                "outside the step grammar",
            ),
            "an unreadable script in a workflow nothing calls": (
                {".github/workflows/unused.yml": (
                    "name: unused\non:\n  workflow_call:\njobs:\n  check:\n" + LINUX
                    + "    steps:\n      - run: false && cargo test\n"
                )},
                "outside the step grammar",
            ),
            "an npm script in a workflow nothing calls": (
                {
                    ".github/workflows/unused.yml": (
                        "name: unused\non:\n  workflow_call:\njobs:\n  check:\n" + LINUX
                        + "    steps:\n      - working-directory: tools/unused\n        run: npm test\n"
                    ),
                    "tools/unused/package.json": '{"scripts": {"test": "false && cargo test"}}',
                },
                "outside the step grammar",
            ),
            "a workflow nothing calls whose steps need inputs": (
                {".github/workflows/unused.yml": (
                    "name: unused\non:\n  workflow_call:\njobs:\n  check:\n" + LINUX
                    + "    steps:\n      - working-directory: ${{ inputs.dir }}\n        run: cargo test --locked\n"
                )},
                "cannot resolve the working directory",
            ),
            # A folded scalar keeps the line breaks around a deeper line and at an empty one.
            "a deeper line in a folded script": (
                {caller: CALLER.replace(
                    "        run: |\n          # npm test runs package.json's test script.\n"
                    + TOOL_TEST + "          npm test\n",
                    "        run: >-\n          npm run\n            test\n",
                )},
                "'npm run' is outside the step grammar",
            ),
            "an empty line in a folded script": (
                {caller: CALLER.replace(
                    "        run: |\n          # npm test runs package.json's test script.\n"
                    + TOOL_TEST + "          npm test\n",
                    "        run: >-\n          cargo +1.99 test\n\n          --locked\n",
                )},
                "'--locked' is outside the step grammar",
            ),
            # Bash reads one comment line; splitlines() would read a command after it.
            "a line separator YAML does not break at": (
                {caller: CALLER.replace(
                    "        run: |\n          # npm test runs package.json's test script.\n"
                    + TOOL_TEST + "          npm test\n",
                    "        run: |\n          # npm test runs package.json's test script.           npm test\n",
                )},
                "breaks a line where YAML does not",
            ),
            # Bash passes a no-break space inside a word; str.split() would split there.
            "a no-break space in a test step": (
                {caller: CALLER.replace("          npm test\n", "          npm test\n")},
                "whitespace bash does not split at",
            ),
            "a line separator in an npm script": (
                {"tools/tool/package.json": FILES["tools/tool/package.json"].replace(
                    "--test-concurrency=1 tests/", "--test-concurrency=1\\u2028tests/"
                )},
                re.escape("(one command)"),
            ),
            # npm's shell carries on after a failing command.
            "an npm script of 2 commands": (
                {"tools/tool/package.json": FILES["tools/tool/package.json"].replace(
                    "tests/runtime.test.mjs", "tests/runtime.test.mjs\\ncargo fmt"
                )},
                re.escape("(one command)"),
            ),
            "an npm pretest script": (
                {"tools/tool/package.json": FILES["tools/tool/package.json"].replace(
                    '{"scripts": {', '{"scripts": {"pretest": "exit 0", '
                )},
                "pretest",
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
            # pytest reads pytest.toml first, and a parent directory's settings without a table.
            "a pytest.toml beside the table": (
                {"app/pytest.toml": "[pytest]\n"}, "pytest table",
            ),
            "an empty pytest table": (
                {"app/pyproject.toml": "[tool.pytest]\n"}, "pytest table",
            ),
            "a norecursedirs setting": (
                {"app/pyproject.toml": FILES["app/pyproject.toml"] + 'norecursedirs = ["skip"]\n'},
                "pytest setting 'norecursedirs'",
            ),
            "a python_files pattern with a directory": (
                {"app/pyproject.toml": FILES["app/pyproject.toml"].replace('"test_*.py"', '"tests/test_*.py"')},
                "python_files pattern 'tests/test_\\*.py'",
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
