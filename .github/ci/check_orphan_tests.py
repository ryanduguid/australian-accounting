"""Fail when a test file or crate exists that no workflow step runs.

usage: python .github/ci/check_orphan_tests.py

The targets are the tracked crates (each directory with a Cargo.toml), Python
test files (test_*.py, *_test.py) and JavaScript test files (*.test.js, .mjs,
.cjs). A target is run when a step of a workflow directly in .github/workflows
runs it. The check reads the workflows in the small grammar below, which is
what main's workflows use. A step that names a test runner (cargo test,
llvm-cov or fuzz, pytest, unittest, npm test or run, node --test) must fit it,
with its job and any caller of its workflow; anything outside it stops the
check, naming the file, job and step. Other steps are ignored.

Steps. The shell is bash (unset, or shell: bash). The env of the step, its job
and its workflow sets only the variables in ENVIRONMENT, which main's test
steps have. With each backslash-newline deleted, each line is blank, a
comment, ci-rust.yml's capture of its coverage floor (name="$(jq ...)") or one
of the commands below. A word is letters, digits and _+=:./-, or a
double-quoted word with $NAME in it; a path or name does not start with -.
Each command runs:

- cargo [+TOOLCHAIN] test or llvm-cov with --locked, --release, --features F,
  --manifest-path P and --fail-under-lines N: that crate, or P's. With
  --test T -- --ignored --nocapture it runs part of the crate: nothing.
- cargo [+TOOLCHAIN] fuzz run [--target T] TARGET [CORPUS] [-- -flag=N ...]:
  the crate in fuzz/.
- uv run [--locked] [--extra or --group G] pytest OPTIONS: the files under the
  testpaths of the pytest table in that directory's pyproject.toml that its
  python_files match. Its options, addopts and PYTEST_ADDOPTS (literal, or
  ${{ TERMS && 'OPTIONS' || '' }} as ci.yml has it) are -q, --cov[=M],
  --cov-branch and --cov-report=R; it takes no paths.
- python -m unittest [-v] FILE.py ...: those files.
- npm test or npm run NAME: that package.json script, read as a step.
- node --test [--test-concurrency=N] FILE.test.js ...: those files.
- cargo build, clippy or fmt, rustup target add or node SCRIPT.mjs: nothing.

Conditions. A job's or step's if: is a && of terms: NAME == 'value' or
NAME != 'value', or a parenthesised || of those, on a string input of a
reusable workflow, a step's runner.os (from runs-on ubuntu-latest,
windows-latest or macos-latest) or needs.JOB.result (success, as a job runs
only when the jobs it needs succeed); or needs.JOB.outputs.NAME == 'value',
which holds unless the caller, job and step require 2 values of it. JOB must
be in the job's needs:. Comparisons ignore case, as GitHub's do.

Matrices. Plain axes, include rows alone or the matrix ci.yml computes with
rust_crates.py; runs-on and a caller's with: values may name a row's values.

A reusable workflow's steps run for each call of it (a job of another workflow
whose uses: names it and whose condition can hold), once for each row of the
caller's matrix, with its with: values. The check does not follow scripts,
what earlier steps write (such as $GITHUB_ENV and $GITHUB_PATH), configuration
files (such as .cargo/config.toml and conftest.py), cargo workspace members,
ignored Rust tests or the tests inside a file.

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
    rf"{CARGO} (?:test|llvm-cov)(?: --locked| --release| --features {NAME}"
    rf'| --manifest-path (?P<manifest>{NAME})| --fail-under-lines (?:\d+|"\$[a-z_]+"))*'
    rf"(?P<part> --test {NAME} -- --ignored --nocapture)?"
)
FUZZ = re.compile(
    rf"{CARGO} fuzz run(?: --target {NAME})? {NAME}(?: {NAME})?(?: --(?: -\w+=\d+)+)?"
)
PYTEST = re.compile(
    rf"uv run(?: --locked| --extra {NAME}| --group {NAME})* pytest(?P<arguments>(?: {PLAIN})*)"
)
PYTEST_OPTION = re.compile(r"-q|--cov(?:=[\w.-]+)?|--cov-branch|--cov-report=[\w.:-]+")
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
# The condition grammar.
BOUND = re.compile(
    r"(?P<name>inputs\.[\w-]+|runner\.os|needs\.[\w-]+\.result) (?P<op>==|!=) '(?P<value>[^']*)'"
)
OUTPUT = re.compile(r"(?P<name>needs\.(?P<job>[\w-]+)\.outputs\.[\w-]+) == '(?P<value>[^']*)'")
CHOICE = re.compile(r"\( ?(.*?) ?\)")
KEY = re.compile(r"""(?P<key>[\w-][\w.-]*|"[^"]*"|'[^']*'):(?:[ \t]+(?P<rest>.*))?""")
QUOTED = re.compile(r"""("(?:[^"\\]|\\.)*"|'(?:[^']|'')*')[ \t]*(?:#.*)?""")
BLOCK_SCALARS = {"|", "|-", "|+", ">", ">-", ">+"}

Needs = dict[tuple[str, str], str]


class Unreadable(Exception):
    """Something outside the grammar, so the check stops rather than guess."""


def load_yaml(text: str) -> object:
    """Parse the block YAML workflows use into dicts, lists and strings."""
    lines = text.splitlines()
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

    def mapping(indent: int) -> dict[str, object]:
        nonlocal pos
        found: dict[str, object] = {}
        while (head := peek()) is not None and head[0] == indent and not is_item(head[1]):
            match = KEY.fullmatch(head[1])
            if match is None:
                raise fail("not a key")
            pos += 1
            found[match["key"].strip("\"'")] = value(indent, match["rest"] or "")
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
            return match[1][1:-1]
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
            margin = next((len(line) - len(line.lstrip(" ")) for line in body if line.strip()), 0)
            body = [line[margin:] for line in body]
            return " ".join(filter(None, body)) if text[0] == ">" else "\n".join(body)
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


def needed(job: object) -> dict[str, str]:
    """needs.JOB.result for each job this one needs: success, as a job with no
    status function in its condition runs only when they succeed."""
    needs = at(job, "needs")
    jobs = [needs] if isinstance(needs, str) and needs[:1] != "[" else items(needs) or []
    return {f"needs.{name}.result": "success" for name in jobs}


def holds(
    condition: object, where: str, flow: str, names: dict[str, str], free: Needs
) -> Needs | None:
    """Whether a job's or step's condition can hold with the values in names:
    if so, free with the needs outputs it requires added; if not, None."""
    if condition is None:
        return free
    text = " ".join(str(condition).split())
    if text.startswith("${{") and text.endswith("}}"):
        text = text[3:-2].strip()
    free, possible = dict(free), True
    for term in text.split(" && "):
        if output := OUTPUT.fullmatch(term):
            if f"needs.{output['job']}.result" not in names:
                raise Unreadable(f"{where}: {term!r} names a job it does not need")
            value = output["value"].casefold()
            possible &= free.setdefault((flow, output["name"]), value) == value
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


def calls(root: Path, flows: dict[str, dict], name: str) -> list[tuple[dict[str, str], Needs]]:
    """The string inputs of each call of a reusable workflow, once for each row
    of the caller's matrix, with the needs outputs the caller's condition requires."""
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
            free = holds(at(job, "if"), where, caller, needed(job), {})
            if free is None:
                continue
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
    """Yield (where, working directory, script, PYTEST_ADDOPTS) for each step
    that names a test runner, for each call and matrix row it runs under."""
    for name, flow in flows.items():
        for job_id, job in flow["jobs"].items():
            where = f"{WORKFLOWS}/{name} job {job_id}"
            steps = []
            for index, step in enumerate(at(job, "steps") or [], 1):
                script = at(step, "run")
                if not isinstance(script, str) or not TESTS.search(script.replace("\\\n", "")):
                    continue
                here = f"{where} step {index}"
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
                steps.append((here, step, script, directory, str(env.get("PYTEST_ADDOPTS", ""))))
            if not steps:
                continue
            rows = matrix_rows(root, job, where)
            called = "workflow_call" in (flow.get("on") or "")
            for inputs, free in calls(root, flows, name) if called else [({}, {})]:
                names = {f"inputs.{key}": value for key, value in inputs.items()} | needed(job)
                running = holds(at(job, "if"), where, name, names, free)
                if running is None:
                    continue
                for row, (here, step, script, directory, addopts) in itertools.product(rows, steps):
                    label = at(job, "runs-on")
                    label = substitute(label, inputs, row) if isinstance(label, str) else ""
                    runner = {"runner.os": HOSTED[label]} if label in HOSTED else {}
                    if holds(at(step, "if"), here, name, names | runner, running) is not None:
                        yield here, substitute(directory, inputs, {}), script, addopts


def join(base: str, path: str) -> str:
    return posixpath.normpath(posixpath.join(base, path))


def inside(path: str, directory: str) -> bool:
    return directory == "." or path == directory or path.startswith(directory + "/")


def listed(value: str | list[str]) -> list[str]:
    """A pytest setting as a list; the ini form separates entries with spaces."""
    return value.split() if isinstance(value, str) else value


def pytest_runs(
    root: Path, files: list[str], where: str, base: str, arguments: list[str], addopts: str
) -> set[str]:
    """The files pytest collects from base: those under its testpaths that its
    python_files match. It takes no paths, so every argument is an option."""
    pyproject = root / base / "pyproject.toml"
    table = None
    if pyproject.is_file():
        table = at(tomllib.loads(pyproject.read_text(encoding="utf-8")), "tool", "pytest")
    # pytest.ini wins over pyproject.toml. Without either, pytest reads tox.ini,
    # setup.cfg or a parent directory's settings.
    ini = any((root / base / name).is_file() for name in ("pytest.ini", ".pytest.ini"))
    if ini or not isinstance(table, dict):
        raise Unreadable(f"{where}: pytest in {base} must read its pyproject.toml's pytest table")
    config = table.get("ini_options", table)
    if "${{" in addopts:
        expression = ADDOPTS.fullmatch(" ".join(addopts.split()))
        if expression is None:
            raise Unreadable(f"{where}: PYTEST_ADDOPTS {addopts!r} is outside the step grammar")
        addopts = expression["options"]
    for argument in [*arguments, *listed(config.get("addopts", [])), *addopts.split()]:
        if not PYTEST_OPTION.fullmatch(argument):
            raise Unreadable(f"{where}: pytest argument {argument!r} is outside the step grammar")
    patterns = listed(config.get("python_files", PYTEST_FILES))
    roots = [join(base, path) for path in listed(config.get("testpaths", ["."]))]
    return {
        path for path in files
        if any(inside(path, root) for root in roots)
        and any(fnmatch.fnmatchcase(posixpath.basename(path), pattern) for pattern in patterns)
    }


def runs(
    root: Path, files: list[str], where: str, directory: str, line: str, addopts: str
) -> set[str]:
    """The targets one test command runs from its working directory."""
    if "${{" in directory:
        raise Unreadable(f"{where}: cannot resolve the working directory {directory!r}")
    base = posixpath.normpath(directory)
    if cargo := CARGO_TEST.fullmatch(line):
        if cargo["part"]:
            return set()
        return {posixpath.dirname(join(base, cargo["manifest"])) if cargo["manifest"] else base}
    if FUZZ.fullmatch(line):
        return {join(base, "fuzz")}
    if pytest := PYTEST.fullmatch(line):
        return pytest_runs(root, files, where, base, pytest["arguments"].split(), addopts)
    if named := UNITTEST.fullmatch(line) or NODE.fullmatch(line):
        return {join(base, path) for path in named["files"].split()}
    if npm := NPM.fullmatch(line):
        package = root / base / "package.json"
        scripts = {}
        if package.is_file():
            scripts = json.loads(package.read_text(encoding="utf-8")).get("scripts", {})
        script = scripts.get(npm["script"] or "test")
        if isinstance(script, str) and TESTS.search(script):
            return step_runs(root, files, where, directory, script, addopts)
        return set()
    raise Unreadable(f"{where}: {line!r} is outside the step grammar")


def step_runs(
    root: Path, files: list[str], where: str, directory: str, script: str, addopts: str
) -> set[str]:
    """The targets a step's script runs, read a line at a time once bash has
    deleted each backslash-newline."""
    found: set[str] = set()
    for line in script.replace("\\\n", "").splitlines():
        line = " ".join(line.split())
        if not line or line.startswith("#") or CAPTURE.fullmatch(line) or OTHER.fullmatch(line):
            continue
        found |= runs(root, files, where, directory, line, addopts)
    return found


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
    run = set().union(*(step_runs(root, files, *step) for step in test_steps(root, flows)))
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
