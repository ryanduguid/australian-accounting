"""Fail when a test file or crate exists that no workflow step runs.

usage: python .github/ci/check_orphan_tests.py

The targets are the tracked crates (each directory with a Cargo.toml), Python
test files (test_*.py, *_test.py) and JavaScript test files (*.test.js, .mjs,
.cjs). A target is credited when a step of a workflow directly in
.github/workflows runs it. The check guarantees that a credited target runs in
any CI run in which the crediting job runs and passes. A failing step turns CI
red, which puts it outside that scope only when continue-on-error is absent,
so continue-on-error stops the check.

The check reads the workflows in the small grammar below, which is what main's
workflows use. Every step that names a test runner (cargo test, llvm-cov or
fuzz, pytest, unittest, npm test or run, node --test) must fit it, with its
job and any caller of its workflow, whether or not its conditions can hold;
anything outside it stops the check, naming the file, job and step. Other
steps are ignored. The YAML reader takes block YAML and one-line quoted
scalars. It reads block scalars (| or >, with - or +) and '' between single
quotes as YAML 1.2 does, and stops on a double-quoted scalar with a
backslash, whose escapes it does not decode.

Steps. The shell is bash: shell: bash, or unset on a Linux or macOS runner,
where GitHub runs bash -e. Unset on another runner it is PowerShell, where
GitHub fails a step on a terminating PowerShell error or a failing last native
command, but not on an earlier native command, so there the script is one
line. The step has no continue-on-error. The env of the step, its job and its
workflow sets only the variables in ENVIRONMENT, which main's test steps have.
With each backslash-newline deleted, each line is blank, a comment,
ci-rust.yml's capture of its coverage floor (name="$(jq ...)") or one of the
commands below. A word is letters, digits and _+=:./-, or a double-quoted word
with $NAME in it; a path or name does not start with -. Each command runs:

- cargo [+TOOLCHAIN] test with --locked, --release, --features F and
  --manifest-path P: that crate, or P's. With --test T -- --ignored
  --nocapture it runs part of the crate: nothing.
- cargo [+TOOLCHAIN] llvm-cov with --locked and --fail-under-lines N: that
  crate.
- cargo [+TOOLCHAIN] fuzz run [--target T] TARGET [CORPUS] [-- -flag=N ...]:
  the crate in fuzz/.
- uv run [--locked] [--extra or --group G] pytest OPTIONS: the files under the
  testpaths of the pytest table in that directory's pyproject.toml whose names
  its python_files match, outside the directories pytest skips: those its
  default norecursedirs names (regardless of case), __pycache__ and virtual
  environments. No pytest.toml, .pytest.toml, pytest.ini or .pytest.ini is
  beside it, and the table sets only addopts, python_files, pythonpath and
  testpaths. Its options, addopts and PYTEST_ADDOPTS (literal, or
  ${{ TERMS && 'OPTIONS' || '' }} as ci.yml has it) are -q, --cov[=M],
  --cov-branch and --cov-report=R; it takes no paths.
- python -m unittest [-v] FILE.py ...: those files.
- npm test or npm run NAME: that package.json script, which is one command
  in this list, as npm's shell carries on after a failing command; the
  package has no preNAME or postNAME script.
- node --test [--test-concurrency=N] FILE.test.js ...: those files.
- cargo build, clippy or fmt, rustup target add or node SCRIPT.mjs: nothing.

Conditions. A job's or step's if: is a && of terms: NAME == 'value' or
NAME != 'value', or a parenthesised || of those, on a string input of a
reusable workflow, a step's runner.os (from runs-on ubuntu-latest,
windows-latest or macos-latest) or needs.JOB.result (success, as a job runs
only when the jobs it needs succeed); or needs.JOB.outputs.NAME == 'value' for
an output JOB declares. A job's or caller's output test fixes that value in
any run in which that job runs; a step's may test only a value fixed so, as
its job runs and passes whatever the output. JOB must be in the job's needs:.
Comparisons ignore case but not spaces, as GitHub's do. A job can run only if
its condition can hold and each job it needs can run in the same call; none of
them has continue-on-error.

Matrices. Plain axes, include rows alone or the matrix ci.yml computes with
rust_crates.py; runs-on and a caller's with: values may name a row's values.

A reusable workflow's steps run for each call of it (a job of a workflow that
is not reusable itself, whose uses: names it and which can run), once for each
row of the caller's matrix, with its with: values. One that nothing calls is
read without inputs, so a step that needs one stops the check. The check does
not follow scripts, what earlier steps write (such as $GITHUB_ENV and
$GITHUB_PATH), configuration files (such as .cargo/config.toml and
conftest.py), cargo workspace members, ignored Rust tests or the tests inside
a file.

Deliberate exceptions go in orphan_tests_allowed.txt beside this script, one
'<path> <reason>' per line. The check fails on an entry without a reason. It
also fails on an entry naming no target or one a step now runs.
"""

from __future__ import annotations

import fnmatch
import itertools
import json
import posixpath
import re
import subprocess
import sys
import tomllib
from pathlib import Path

from rust_crates import crates

ROOT = Path(__file__).resolve().parents[2]
ALLOWED = Path(__file__).with_name("orphan_tests_allowed.txt")
WORKFLOWS = ".github/workflows"
TEST_FILES = ("test_*.py", "*_test.py", "*.test.js", "*.test.mjs", "*.test.cjs")
PYTEST_FILES = ["test_*.py", "*_test.py"]
# The pytest settings main uses; the others can change what pytest collects.
PYTEST_SETTINGS = {"addopts", "python_files", "pythonpath", "testpaths"}
# The directories pytest does not collect from: its default norecursedirs, and __pycache__.
PYTEST_SKIPPED = [
    "*.egg", ".*", "_darcs", "build", "CVS", "dist", "node_modules", "venv", "{arch}",
    "__pycache__",
]
# Config files that pytest reads before pyproject.toml.
PYTEST_FIRST = ("pytest.toml", ".pytest.toml", "pytest.ini", ".pytest.ini")
# ci.yml's crates job prints this matrix with rust_crates.py.
COMPUTED_MATRICES = {
    "${{ fromJSON(needs.crates.outputs.ci-matrix) }}": lambda root: crates(root, ci_yml=True),
}
# runner.os on the GitHub-hosted images main's workflows use.
HOSTED = {"ubuntu-latest": "Linux", "windows-latest": "Windows", "macos-latest": "macOS"}
EXPRESSION = re.compile(r"\$\{\{\s*(inputs|matrix)\.([\w-]+)\s*\}\}")
FLOW_ITEM = re.compile(r"""\s*(["']?)([\w.-]+)\1\s*""")
# The step grammar. TESTS finds the steps that name a test runner.
TESTS = re.compile(
    r"\bcargo\s+(?:\+\S+\s+)?(?:test|llvm-cov|fuzz)\b|\bpytest\b|\bunittest\b"
    r"|\bnpm\s+(?:test|run)\b|\bnode\b.*--test\b"
)
PLAIN = r"[\w+=:./-]+"
VARIABLE = r'"(?:[\w+=:./-]|\$[A-Za-z_]\w*)+"'
CAPTURE = re.compile(rf"""[a-z_]+="\$\(jq(?: {PLAIN}| {VARIABLE}| '[^']*')*\)\"""")
OTHER = re.compile(
    rf"(?:cargo (?:build|clippy|fmt)|rustup target add|node {PLAIN}\.mjs)(?: {PLAIN}| {VARIABLE})*"
)
CARGO = r"cargo(?: \+[\w.-]+)?"
# A path or name, which an option cannot pass for.
NAME = r"[\w.][\w./-]*"
CARGO_TEST = re.compile(
    rf"{CARGO} test(?: --locked| --release| --features {NAME}"
    rf"| --manifest-path (?P<manifest>{NAME}))*"
    rf"(?P<part> --test {NAME} -- --ignored --nocapture)?"
)
LLVM_COV = re.compile(rf'{CARGO} llvm-cov(?: --locked| --fail-under-lines (?:\d+|"\$[a-z_]+"))*')
FUZZ = re.compile(
    rf"{CARGO} fuzz run(?: --target {NAME})? {NAME}(?: {NAME})?(?: --(?: -\w+=\d+)+)?"
)
PYTEST = re.compile(
    rf"uv run(?: --locked| --extra {NAME}| --group {NAME})* pytest(?P<arguments>(?: {PLAIN})*)"
)
PYTEST_OPTION = re.compile(r"-q|--cov(?:=[\w.-]+)?|--cov-branch|--cov-report=[\w.:-]+")
# A line break, or whitespace a shell does not split words at but str.split() does.
UNSPLIT = re.compile(r"[^\S \t]")
# The environment main's test steps have; another variable could change what a runner runs.
ENVIRONMENT = {
    "ACTIONLINT_SHA256", "ACTIONLINT_VERSION", "AU_ACCOUNTING_PARITY_BIN",
    "AU_ACCOUNTING_PARITY_CAP", "BINARY", "CARGO_BUILD_JOBS", "CRATE", "ENGINE", "PYTEST_ADDOPTS",
    "SHELLCHECK_SHA256", "SHELLCHECK_VERSION", "UV_PYTHON", "UV_VERSION", "WRANGLER_SEND_METRICS",
}
# A comparison is true or false, so this yields the options or nothing.
ADDOPTS = re.compile(r"\$\{\{ (?:[\w.-]+ == '[^']*' && )+'(?P<options>[^']*)' \|\| '' \}\}")
UNITTEST = re.compile(rf"python -m unittest(?: -v)?(?P<files>(?: {NAME}\.py)+)")
NODE = re.compile(rf"node --test(?: --test-concurrency=\d+)?(?P<files>(?: {NAME}\.test\.[cm]?js)+)")
NPM = re.compile(r"npm (?:test|run (?P<script>[\w:-]+))")
COMMANDS = (CARGO_TEST, LLVM_COV, FUZZ, PYTEST, UNITTEST, NODE, NPM)
# The condition grammar.
BOUND = re.compile(
    r"(?P<name>inputs\.[\w-]+|runner\.os|needs\.[\w-]+\.result) (?P<op>==|!=) '(?P<value>[^']*)'"
)
OUTPUT = re.compile(r"(?P<name>needs\.(?P<job>[\w-]+)\.outputs\.[\w-]+) == '(?P<value>[^']*)'")
CHOICE = re.compile(r"\( ?(.*?) ?\)")
# A quoted literal, whose spaces count, or whitespace outside one.
LITERAL_OR_SPACE = re.compile(r"('[^']*')|\s+")
KEY = re.compile(r"""(?P<key>[\w-][\w.-]*|"[^"]*"|'[^']*'):(?:[ \t]+(?P<rest>[^ \t].*)?)?""")
QUOTED = re.compile(r"""("(?:[^"\\]|\\.)*"|'(?:[^']|'')*')[ \t]*(?:#.*)?""")
BLOCK_SCALARS = {"|", "|-", "|+", ">", ">-", ">+"}

Needs = dict[tuple[str, str], str]


class Unreadable(Exception):
    """Something outside the grammar, so the check stops rather than guess."""


def block_scalar(body: list[str], indicator: str, broken: bool) -> str:
    """A block scalar's value as YAML 1.2 gives it, from its lines without their
    indentation; broken says whether the last line ends in a line break, as the
    end of a file may not. Folded (>), the break between 2 lines that start with
    no space is a space, or dropped when empty lines between them stay as
    breaks; every other break stays, as in a literal (|) scalar. Then - drops
    the breaks after the last line with content, + keeps them all and no sign
    keeps its own, if any."""
    end = len(body)
    while end and not body[end - 1]:
        end -= 1
    value, previous, empty = "", None, 0
    for line in body[:end]:
        if not line:
            empty += 1
            continue
        if previous is None:
            value += "\n" * empty
        elif indicator[0] == ">" and previous[:1] not in " \t" and line[:1] not in " \t":
            value += "\n" * empty if empty else " "
        else:
            value += "\n" * (empty + 1)
        value += line
        previous, empty = line, 0
    # The breaks after the last line with content: its own and each empty line's.
    breaks = max(0, len(body) - end + (previous is not None) - (not broken))
    if indicator[1:] == "+":
        return value + "\n" * breaks
    return value + "\n" if indicator[1:] == "" and previous is not None and breaks else value


def load_yaml(text: str) -> object:
    """Parse the block YAML workflows use into dicts, lists and strings."""
    # YAML and bash treat only space, tab, LF and CR as whitespace; str methods
    # such as splitlines(), split() and strip() also treat others as whitespace.
    odd = re.search(r"[^\S \t\n\r]", text)
    if odd:
        line = text.count("\n", 0, odd.start()) + 1
        raise Unreadable(f"line {line}: {odd.group()!r} is whitespace other than space and tab")
    lines = text.splitlines()
    terminated = text.endswith(("\n", "\r"))
    pos = 0

    def fail(why: str) -> Unreadable:
        return Unreadable(f"line {pos + 1}: {why}")

    def peek() -> tuple[int, str] | None:
        nonlocal pos
        while pos < len(lines):
            body = lines[pos].lstrip(" ")
            if body and not body.startswith("#"):
                return len(lines[pos]) - len(body), body
            pos += 1
        return None

    def is_item(body: str) -> bool:
        return body == "-" or body.startswith("- ")

    def unquote(scalar: str) -> str:
        """A one-line scalar as YAML 1.2 reads it: between single quotes, '' is ';
        between double quotes, a backslash starts an escape, which this reader
        does not decode."""
        if scalar[:1] == "'":
            return scalar[1:-1].replace("''", "'")
        if scalar[:1] == '"':
            if "\\" in scalar:
                raise fail("an escape in a double-quoted scalar")
            return scalar[1:-1]
        return scalar

    def mapping(indent: int) -> dict[str, object]:
        nonlocal pos
        found: dict[str, object] = {}
        while (head := peek()) is not None and head[0] == indent and not is_item(head[1]):
            match = KEY.fullmatch(head[1])
            if match is None:
                raise fail("not a key")
            key = unquote(match["key"])
            pos += 1
            found[key] = value(indent, match["rest"] or "")
        if head is not None and head[0] > indent:
            raise fail("unexpected indentation")
        return found

    def sequence(indent: int) -> list[object]:
        nonlocal pos
        found: list[object] = []
        while (head := peek()) is not None and head[0] == indent and is_item(head[1]):
            rest = head[1][1:].lstrip(" ")
            if KEY.fullmatch(rest):
                # "- key: value" starts a mapping at the key's column.
                column = len(lines[pos]) - len(rest)
                lines[pos] = " " * column + rest
                found.append(mapping(column))
            else:
                pos += 1
                found.append(value(indent, rest))
        if head is not None and head[0] > indent:
            raise fail("unexpected indentation")
        return found

    def value(indent: int, rest: str) -> object:
        nonlocal pos
        if rest[:1] in ("'", '"'):
            match = QUOTED.fullmatch(rest)
            if match is None:
                raise fail("a quoted scalar over several lines")
            return unquote(match[1])
        text = re.split(r"[ \t]#", rest, maxsplit=1)[0].rstrip()
        if text[:1] in ("&", "*", "!", "?", "%", "@", "`"):
            raise fail("an anchor, alias, tag or reserved indicator")
        if text[:1] in ("|", ">"):
            if text not in BLOCK_SCALARS:
                raise fail("a block scalar indicator")
            start = pos
            while pos < len(lines) and (
                not lines[pos].strip() or len(lines[pos]) - len(lines[pos].lstrip(" ")) > indent
            ):
                pos += 1
            body = lines[start:pos]
            # Without a line with content, every line is empty.
            margin = next(
                (len(line) - len(line.lstrip(" ")) for line in body if line.strip()),
                max(map(len, body), default=0),
            )
            broken = pos < len(lines) or terminated
            return block_scalar([line[margin:] for line in body], text, broken)
        if text:
            return text
        head = peek()
        if head is not None and (head[0] > indent or (head[0] == indent and is_item(head[1]))):
            return sequence(head[0]) if is_item(head[1]) else mapping(head[0])
        return ""

    head = peek()
    return mapping(head[0]) if head is not None else {}


def at(node: object, *keys: str) -> object:
    for key in keys:
        node = node.get(key) if isinstance(node, dict) else None
    return node


def workflows(root: Path) -> dict[str, dict]:
    found = {}
    for path in sorted((root / WORKFLOWS).iterdir()):
        if path.is_file() and path.suffix in (".yml", ".yaml"):
            try:
                flow = load_yaml(path.read_text(encoding="utf-8"))
            except Unreadable as error:
                raise Unreadable(f"{WORKFLOWS}/{path.name}: {error}") from None
            if not isinstance(flow, dict) or not isinstance(flow.get("jobs"), dict):
                raise Unreadable(f"{WORKFLOWS}/{path.name}: no jobs mapping")
            found[path.name] = flow
    return found


def substitute(text: str, inputs: dict[str, str], matrix: dict[str, str]) -> str:
    def replace(match: re.Match[str]) -> str:
        return str((inputs if match[1] == "inputs" else matrix).get(match[2], match[0]))

    return EXPRESSION.sub(replace, text)


def items(value: object) -> list[str] | None:
    """A sequence of plain values, in block or [a, b] form; None for anything else."""
    if isinstance(value, str) and value[:1] == "[" and value[-1:] == "]":
        value = [
            (match := FLOW_ITEM.fullmatch(item)) and match[2] for item in value[1:-1].split(",")
        ]
    if isinstance(value, list) and value and all(isinstance(item, str) for item in value):
        return value
    return None


def matrix_rows(root: Path, job: object, where: str) -> list[dict[str, str]]:
    """A job's matrix rows: one empty row without a matrix, the computed matrix's
    rows, the include rows or the product of the axes."""
    matrix = at(job, "strategy", "matrix")
    if matrix is None:
        return [{}]
    if isinstance(matrix, str) and matrix in COMPUTED_MATRICES:
        return COMPUTED_MATRICES[matrix](root)
    if isinstance(matrix, dict) and list(matrix) == ["include"]:
        rows = matrix["include"]
        if isinstance(rows, list) and rows and all(
            isinstance(row, dict) and all(isinstance(value, str) for value in row.values())
            for row in rows
        ):
            return rows
    elif isinstance(matrix, dict) and matrix.keys().isdisjoint({"include", "exclude"}):
        axes = [items(values) for values in matrix.values()]
        if all(axes):
            return [dict(zip(matrix, row)) for row in itertools.product(*axes)]
    raise Unreadable(f"{where}: the matrix is outside the matrix grammar")


def reusable(flow: dict) -> bool:
    return "workflow_call" in (flow.get("on") or "")


def needed(job: object, where: str) -> list[str]:
    needs = at(job, "needs")
    if needs is None:
        return []
    jobs = [needs] if isinstance(needs, str) and needs[:1] != "[" else items(needs)
    if jobs is None:
        raise Unreadable(f"{where}: needs {needs!r} is outside the grammar")
    return jobs


def context(inputs: dict[str, str], job: object, jobs: dict, where: str) -> dict[str, str]:
    """The names a job's conditions can test: its call's string inputs;
    needs.JOB.result for each job it needs, which is success, as a job with no
    status function in its condition runs only when they succeed; and
    needs.JOB.outputs.NAME for each output those jobs declare, with its
    declared expression, as its value is not known."""
    names = {f"inputs.{key}": value for key, value in inputs.items()}
    for need in needed(job, where):
        if need not in jobs:
            raise Unreadable(f"{where}: needs {need!r}, which is not a job")
        names[f"needs.{need}.result"] = "success"
        outputs = at(jobs[need], "outputs")
        for output, expression in (outputs.items() if isinstance(outputs, dict) else []):
            names[f"needs.{need}.outputs.{output}"] = str(expression)
    return names


def holds(
    condition: object,
    where: str,
    flow: str,
    names: dict[str, str],
    free: Needs | None,
    step: bool = False,
) -> Needs | None:
    """Whether a job's or step's condition can hold with the values in names,
    given the needs outputs in free (None when it cannot run anyway): if so, free
    with the needs outputs it requires added; if not, None. Either way the
    condition must fit the grammar. A job runs only with the outputs its
    condition requires, but a step's condition cannot set one: its job runs and
    passes either way, so the step may test only an output that free sets."""
    if condition is None:
        return free
    text = LITERAL_OR_SPACE.sub(lambda match: match[1] or " ", str(condition)).strip()
    if text.startswith("${{") and text.endswith("}}"):
        text = text[3:-2].strip()
    known = free is not None
    possible, free = known, dict(free or {})
    for term in text.split(" && "):
        if output := OUTPUT.fullmatch(term):
            if f"needs.{output['job']}.result" not in names:
                raise Unreadable(f"{where}: {term!r} names a job it does not need")
            if output["name"] not in names:
                raise Unreadable(f"{where}: {term!r} names an output its job does not declare")
            key, value = (flow, output["name"]), output["value"].casefold()
            if step and known and key not in free:
                raise Unreadable(
                    f"{where}: {term!r} tests an output its job's condition leaves open"
                )
            possible &= free.setdefault(key, value) == value
            continue
        parts = choice[1].split(" || ") if (choice := CHOICE.fullmatch(term)) else [term]
        tests = [test for part in parts if (test := BOUND.fullmatch(part))]
        if len(tests) < len(parts):
            raise Unreadable(f"{where}: {term!r} is outside the condition grammar")
        for test in tests:
            value = names.get(test["name"])
            if value is None or "${{" in value:
                raise Unreadable(f"{where}: cannot decide {term!r} without a known {test['name']}")
        possible &= any(
            (names[test["name"]].casefold() == test["value"].casefold()) == (test["op"] == "==")
            for test in tests
        )
    return free if possible else None


def runnable(
    flows: dict[str, dict], name: str, job_id: str, inputs: dict[str, str], free: Needs | None
) -> Needs | None:
    """As holds, for a job in a call with these inputs: whether its condition
    can hold, and in turn that of each job it needs, as it runs only when they
    succeed. No such job may have continue-on-error."""
    where = f"{WORKFLOWS}/{name} job {job_id}"
    jobs = flows[name]["jobs"]
    job = jobs[job_id]
    if at(job, "continue-on-error") is not None:
        raise Unreadable(f"{where}: continue-on-error is outside the grammar")
    names = context(inputs, job, jobs, where)
    for need in needed(job, where):
        free = runnable(flows, name, need, inputs, free)
    return holds(at(job, "if"), where, name, names, free)


def calls(
    root: Path, flows: dict[str, dict], name: str
) -> list[tuple[dict[str, str], Needs | None]]:
    """The string inputs of each call of a reusable workflow, once for each row
    of the caller's matrix, with the needs outputs the caller requires to run,
    or None when it cannot. A caller must not be reusable itself."""
    specs = at(flows[name], "on", "workflow_call", "inputs")
    # Conditions compare strings; GitHub compares a boolean or number otherwise.
    defaults = {
        key: str(at(spec, "default") or "")
        for key, spec in (specs.items() if isinstance(specs, dict) else [])
        if at(spec, "type") == "string"
    }
    found = []
    for caller, flow in flows.items():
        for job_id, job in flow["jobs"].items():
            if at(job, "uses") != f"./{WORKFLOWS}/{name}":
                continue
            where = f"{WORKFLOWS}/{caller} job {job_id}"
            if reusable(flow):
                raise Unreadable(
                    f"{where}: a reusable workflow's call of another is outside the grammar"
                )
            free = runnable(flows, caller, job_id, {}, {})
            given = at(job, "with")
            given = given if isinstance(given, dict) else {}
            for row in matrix_rows(root, job, where):
                inputs = defaults | {
                    key: substitute(str(given[key]), {}, row) for key in given.keys() & defaults
                }
                found.append((inputs, free))
    return found


def setting(flow: dict, job: object, step: object, key: str) -> object:
    """A step's run setting, else its job's default, else its workflow's."""
    return at(step, key) or at(job, "defaults", "run", key) or at(flow, "defaults", "run", key)


def test_steps(root: Path, flows: dict[str, dict]):
    """Yield (where, working directory, commands, PYTEST_ADDOPTS, whether it can
    run) for each step that names a test runner, for each call and matrix row,
    whether or not it can run there."""
    for name, flow in flows.items():
        for job_id, job in flow["jobs"].items():
            where = f"{WORKFLOWS}/{name} job {job_id}"
            steps = []
            for index, step in enumerate(at(job, "steps") or [], 1):
                script = at(step, "run")
                if not isinstance(script, str) or not TESTS.search(script.replace("\\\n", "")):
                    continue
                here = f"{where} step {index}"
                if at(step, "continue-on-error") is not None:
                    raise Unreadable(f"{here}: continue-on-error is outside the step grammar")
                shell = setting(flow, job, step, "shell")
                if shell not in (None, "bash"):
                    raise Unreadable(f"{here}: shell {shell!r} is outside the step grammar")
                # A step's env overrides its job's, which overrides its workflow's.
                env = {}
                for variables in (at(flow, "env"), at(job, "env"), at(step, "env")):
                    unknown = variables
                    if isinstance(variables, dict):
                        unknown = sorted(variables.keys() - ENVIRONMENT)
                    if unknown:
                        raise Unreadable(f"{here}: env {unknown!r} is outside the step grammar")
                    env |= variables or {}
                directory = str(setting(flow, job, step, "working-directory") or ".")
                addopts = str(env.get("PYTEST_ADDOPTS", ""))
                several = shell is None and len(script.strip().splitlines()) > 1
                steps.append((here, step, commands(here, script), directory, addopts, several))
            if not steps:
                continue
            rows = matrix_rows(root, job, where)
            calling = calls(root, flows, name) if reusable(flow) else [({}, {})]
            # Without a call, its steps are read without inputs, in a run that cannot happen.
            for inputs, free in calling or [({}, None)]:
                running = runnable(flows, name, job_id, inputs, free)
                names = context(inputs, job, flow["jobs"], where)
                for row, (here, step, found, directory, addopts, several) in itertools.product(
                    rows, steps
                ):
                    label = at(job, "runs-on")
                    label = substitute(label, inputs, row) if isinstance(label, str) else ""
                    runner = {"runner.os": HOSTED[label]} if label in HOSTED else {}
                    # Unset off Linux and macOS, the shell is PowerShell, where a failing
                    # native command fails the step only if it is the last.
                    if several and runner.get("runner.os") not in ("Linux", "macOS"):
                        raise Unreadable(f"{here}: several lines on {label!r} need shell: bash")
                    reached = holds(at(step, "if"), here, name, names | runner, running, step=True)
                    yield (
                        here, substitute(directory, inputs, {}), found, addopts, reached is not None
                    )


def join(base: str, path: str) -> str:
    return posixpath.normpath(posixpath.join(base, path))


def inside(path: str, directory: str) -> bool:
    return directory == "." or path == directory or path.startswith(directory + "/")


def listed(value: str | list[str]) -> list[str]:
    """A pytest setting as a list; the ini form separates entries with spaces."""
    return value.split() if isinstance(value, str) else value


def collected(path: str, base: str, tracked: set[str]) -> bool:
    """Whether pytest reaches a file from base: no directory on the way is one
    that PYTEST_SKIPPED names (in any case, as on Windows) or a virtual
    environment, which holds pyvenv.cfg or conda-meta/history."""
    parts = posixpath.relpath(path, base).split("/")
    for index, part in enumerate(parts[:-1], 1):
        directory = join(base, "/".join(parts[:index]))
        if any(fnmatch.fnmatchcase(part.casefold(), name.casefold()) for name in PYTEST_SKIPPED):
            return False
        if {f"{directory}/pyvenv.cfg", f"{directory}/conda-meta/history"} & tracked:
            return False
    return True


def pytest_runs(
    root: Path, files: list[str], where: str, base: str, arguments: list[str], addopts: str
) -> set[str]:
    """The files pytest collects from base: those under its testpaths that its
    python_files match and it reaches. It takes no paths, so every argument is
    an option."""
    pyproject = root / base / "pyproject.toml"
    table = None
    if pyproject.is_file():
        table = at(tomllib.loads(pyproject.read_text(encoding="utf-8")), "tool", "pytest")
    # pytest reads PYTEST_FIRST before pyproject.toml. Without them or a pytest
    # table (an empty [tool.pytest] counts as none), it reads tox.ini, setup.cfg
    # or a parent directory's settings.
    first = any((root / base / name).is_file() for name in PYTEST_FIRST)
    if first or not isinstance(table, dict) or not table:
        raise Unreadable(f"{where}: pytest in {base} must read its pyproject.toml's pytest table")
    config = table["ini_options"] if list(table) == ["ini_options"] else table
    unknown = sorted(config.keys() - PYTEST_SETTINGS)
    if unknown:
        raise Unreadable(f"{where}: pytest setting {unknown[0]!r} is outside the step grammar")
    if "${{" in addopts:
        expression = ADDOPTS.fullmatch(" ".join(addopts.split()))
        if expression is None:
            raise Unreadable(f"{where}: PYTEST_ADDOPTS {addopts!r} is outside the step grammar")
        addopts = expression["options"]
    for argument in [*arguments, *listed(config.get("addopts", [])), *addopts.split()]:
        if not PYTEST_OPTION.fullmatch(argument):
            raise Unreadable(f"{where}: pytest argument {argument!r} is outside the step grammar")
    patterns = listed(config.get("python_files", PYTEST_FILES))
    for pattern in patterns:
        # pytest matches a pattern with a separator against the whole path.
        if "/" in pattern or "\\" in pattern:
            raise Unreadable(
                f"{where}: python_files pattern {pattern!r} is outside the step grammar"
            )
    roots = [join(base, path) for path in listed(config.get("testpaths", ["."]))]
    tracked = set(files)
    return {
        path for path in files
        if any(inside(path, root) for root in roots)
        and any(fnmatch.fnmatchcase(posixpath.basename(path), pattern) for pattern in patterns)
        and collected(path, base, tracked)
    }


def commands(where: str, script: str) -> list[re.Match[str]]:
    """A script's test commands, read a line at a time once bash has deleted
    each backslash-newline. Each line must fit the step grammar."""
    found = []
    for line in script.replace("\\\n", "").splitlines():
        line = " ".join(line.split())
        if not line or line.startswith("#") or CAPTURE.fullmatch(line) or OTHER.fullmatch(line):
            continue
        if (command := as_command(line)) is None:
            raise Unreadable(f"{where}: {line!r} is outside the step grammar")
        found.append(command)
    return found


def as_command(line: str) -> re.Match[str] | None:
    return next(filter(None, (pattern.fullmatch(line) for pattern in COMMANDS)), None)


def runs(
    root: Path,
    files: list[str],
    where: str,
    directory: str,
    found: list[re.Match[str]],
    addopts: str,
) -> set[str]:
    """The targets a step's commands run from its working directory."""
    if "${{" in directory:
        raise Unreadable(f"{where}: cannot resolve the working directory {directory!r}")
    base = posixpath.normpath(directory)
    run: set[str] = set()
    for command in found:
        if command.re is CARGO_TEST:
            if not command["part"]:
                manifest = command["manifest"]
                run.add(posixpath.dirname(join(base, manifest)) if manifest else base)
        elif command.re is LLVM_COV:
            run.add(base)
        elif command.re is FUZZ:
            run.add(join(base, "fuzz"))
        elif command.re is PYTEST:
            run |= pytest_runs(root, files, where, base, command["arguments"].split(), addopts)
        elif command.re is NPM:
            name = command["script"] or "test"
            package = root / base / "package.json"
            scripts = {}
            if package.is_file():
                scripts = json.loads(package.read_text(encoding="utf-8")).get("scripts", {})
            # npm runs preNAME before NAME and postNAME after it.
            for hook in (f"pre{name}", f"post{name}"):
                if hook in scripts:
                    raise Unreadable(f"{where}: npm script {hook!r} is outside the step grammar")
            script = scripts.get(name)
            if isinstance(script, str) and TESTS.search(script):
                # npm's shell does not stop at a failing command, so a script is one command.
                single = None if UNSPLIT.search(script) else as_command(" ".join(script.split()))
                if single is None:
                    raise Unreadable(
                        f"{where}: npm script {name!r} {script!r} is outside the step grammar"
                        " (one command)"
                    )
                run |= runs(root, files, where, directory, [single], addopts)
        else:
            run |= {join(base, path) for path in command["files"].split()}
    return run


def targets(files: list[str]) -> set[str]:
    found = set()
    for path in files:
        name = posixpath.basename(path)
        if name == "Cargo.toml":
            found.add(posixpath.dirname(path) or ".")
        elif any(fnmatch.fnmatchcase(name, pattern) for pattern in TEST_FILES):
            found.add(path)
    return found


def load_allowed(path: Path) -> dict[str, str]:
    allowed = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.lstrip().startswith("#"):
            entry, _, reason = line.strip().partition(" ")
            allowed[entry] = reason.strip()
    return allowed


def problems(root: Path, files: list[str], allowed: dict[str, str]) -> list[str]:
    """Name each orphan and each allow-list entry that no longer applies."""
    wanted = targets(files)
    flows = workflows(root)
    run = set()
    for where, directory, commands_run, addopts, can_run in test_steps(root, flows):
        # Every step is read, so a condition cannot hide one outside the grammar.
        ran = runs(root, files, where, directory, commands_run, addopts)
        if can_run:
            run |= ran
    found = [
        f"{path}: no workflow step runs this {'test file' if path in files else 'crate'}"
        for path in sorted(wanted - run - set(allowed))
    ]
    name = ALLOWED.name
    for path, reason in sorted(allowed.items()):
        if not reason:
            found.append(f"{name}: give {path} a reason")
        elif path not in wanted:
            found.append(f"{name}: {path} is not a test file or crate")
        elif path in run:
            found.append(f"{name}: a workflow step runs {path} now; remove its entry")
    return found


def main() -> None:
    listing = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, check=True)
    files = [path for path in listing.stdout.decode("utf-8").split("\0") if path]
    try:
        found = problems(ROOT, files, load_allowed(ALLOWED))
    except Unreadable as error:
        sys.exit(f"{error}; the orphan test check needs updating")
    if found:
        sys.exit(
            "\n".join(found)
            + f"\nRun each in a workflow step, or list it in .github/ci/{ALLOWED.name} with a reason."
        )
    print("Every test file and crate runs in a workflow step.")


if __name__ == "__main__":
    main()
