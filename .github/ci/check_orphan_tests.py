"""Fail when a test file or crate exists that no workflow step runs.

usage: python .github/ci/check_orphan_tests.py

The targets are the tracked crates (each directory with a Cargo.toml), Python
test files (test_*.py, *_test.py) and JavaScript test files (*.test.js, .mjs,
.cjs). A target is run when a step of a workflow directly in .github/workflows
runs one of these commands from its working directory (the step's, else the
job's or the workflow's default):

- cargo [+toolchain] test or llvm-cov, without --no-run: that crate, or the one
  --manifest-path names;
- cargo [+toolchain] fuzz run: the crate in fuzz/;
- pytest, bare or after uv run or python -m: each file it names, and under the
  directories it names (else the testpaths of the pytest table in the
  pyproject.toml there, else that directory) the files its python_files match;
- python -m unittest: the files it names;
- npm test or npm run NAME: that package.json script's commands;
- node --test: the files and single-level globs it names.

A reusable workflow's steps run once for each call of it, with the caller's
with: values for each row of the caller's matrix: its include rows, the product
of its axes or, for the one matrix CI computes, rust_crates.py's rows.

A caller job, job or step runs when it has no condition, or one that is a && of
terms, each NAME == 'value', NAME != 'value' or a parenthesised || of
inputs.NAME == 'value'. Inputs, matrix values and runner.os (from an ubuntu,
windows or macos runs-on label) are decided together for each call and matrix
row. Other names, such as needs and github, are known only at run time, so
their terms are taken to hold on some run unless they compare one name with 2
values, or with a value they also exclude. Any other condition, a literal false
or a negation included, runs nothing.

The model is narrow and fails closed. A command it does not recognise runs
nothing, so its targets are reported. These stop the check: YAML it cannot
read; a test step whose working directory it cannot resolve, or that changes
directory, uses shell control flow, && or ||, a function or a heredoc
(quoted text is not shell syntax); pytest's --ignore,
--ignore-glob, --deselect, -k or -m, on the command line or in addopts; and
node --test with a ** glob. It does not follow scripts, cargo workspace
members, ignored Rust tests or the tests inside a file.

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
import shlex
import subprocess
import sys
import tomllib
from pathlib import Path, PurePosixPath

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
RUNNER = re.compile(r"\b(?:cargo|pytest|unittest|npm|node)\b")
EXPRESSION = re.compile(r"\$\{\{\s*(inputs|matrix)\.([\w-]+)\s*\}\}")
# The condition terms the check decides: a comparison with a literal, and a
# parenthesised choice of input values.
COMPARISON = re.compile(r"(?P<name>[a-z]+(?:\.[\w-]+)+) (?P<op>==|!=) '(?P<value>[^']*)'")
CHOICE = re.compile(r"\(\s*inputs\.[\w-]+ == '[^']*'(?:\s*\|\|\s*inputs\.[\w-]+ == '[^']*')*\s*\)")
CHOSEN = re.compile(r"(inputs\.[\w-]+) == '([^']*)'")
# runner.os for the image a GitHub-hosted runs-on label names.
SYSTEMS = {"ubuntu": "Linux", "windows": "Windows", "macos": "macOS"}
NARROWING = ("--ignore", "--ignore-glob", "--deselect")
KEY = re.compile(r"""(?P<key>[\w-][\w.-]*|"[^"]*"|'[^']*'):(?:[ \t]+(?P<rest>.*))?""")
QUOTED = re.compile(r"""("(?:[^"\\]|\\.)*"|'(?:[^']|'')*')[ \t]*(?:#.*)?""")
BLOCK_SCALARS = {"|", "|-", "|+", ">", ">-", ">+"}
# Control flow can skip a command or run it more than once, so a test step
# that uses it, or && or ||, stops the check rather than being read line by line.
CONTROL = {"if", "then", "else", "elif", "fi", "case", "esac", "for", "select", "while", "until",
           "do", "done", "function"}
PREFIXES = {"!", "{", "time"}
OPERATORS = "();&|<>"


class Unreadable(Exception):
    """Something the check does not model, so it stops rather than guess."""


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


def calls(root: Path, flows: dict[str, dict]) -> dict[str, list[dict[str, str]]]:
    """The inputs of every call of each workflow from a job of another."""
    found: dict[str, list[dict[str, str]]] = {name: [] for name in flows}
    for flow in flows.values():
        for job in flow["jobs"].values():
            uses = at(job, "uses")
            name = uses.removeprefix("./.github/workflows/") if isinstance(uses, str) else ""
            if name == uses or name not in flows:
                continue
            specs = at(flows[name], "on", "workflow_call", "inputs")
            defaults = {
                key: str(at(spec, "default") or "")
                for key, spec in (specs.items() if isinstance(specs, dict) else [])
            }
            given = at(job, "with")
            for row, _ in satisfying(at(job, "if"), contexts(root, job, {})):
                found[name].append(defaults | {
                    key: substitute(str(text), {}, row)
                    for key, text in (given.items() if isinstance(given, dict) else [])
                })
    return found


def matrix_rows(root: Path, job: object) -> list[dict] | None:
    """A job's matrix rows: its include rows, the product of its axes or a
    computed matrix; None for any other matrix."""
    matrix = at(job, "strategy", "matrix")
    if matrix is None:
        return [{}]
    if isinstance(matrix, str):
        return COMPUTED_MATRICES[matrix](root) if matrix in COMPUTED_MATRICES else None
    if not isinstance(matrix, dict):
        return None
    if list(matrix) == ["include"]:
        rows = matrix["include"]
        return rows if isinstance(rows, list) and all(isinstance(r, dict) for r in rows) else None
    axes = []
    for values in matrix.values():
        if isinstance(values, str) and values[:1] == "[" and values[-1:] == "]":
            values = [value.strip().strip("'\"") for value in values[1:-1].split(",")]
        if not isinstance(values, list) or not all(isinstance(value, str) for value in values):
            return None
        axes.append(values)
    return [dict(zip(matrix, combination)) for combination in itertools.product(*axes)]


def contexts(root: Path, job: object, inputs: dict[str, str]) -> list[tuple[dict, dict]]:
    """Each matrix row of a job, with the values a condition can test under it:
    the inputs, the row and the runner.os of its runs-on label."""
    rows = matrix_rows(root, job)
    found = []
    for row in [{}] if rows is None else rows:
        names = {f"inputs.{key}": value for key, value in inputs.items()}
        names |= {f"matrix.{key}": str(value) for key, value in row.items()}
        label = at(job, "runs-on")
        if isinstance(label, str):
            system = SYSTEMS.get(substitute(label, inputs, row).split("-", 1)[0])
            if system:
                names["runner.os"] = system
        found.append((row, names))
    return found


def satisfying(condition: object, options: list[tuple[dict, dict]]) -> list[tuple[dict, dict]]:
    """The contexts under which a job or step condition can hold."""
    if not isinstance(condition, str):
        return options
    text = condition.strip()
    if text.startswith("${{") and text.endswith("}}"):
        text = text[3:-2].strip()
    terms, depth, start = [], 0, 0
    for index, char in enumerate(text):
        depth += (char == "(") - (char == ")")
        if depth == 0 and text.startswith("&&", index):
            terms.append(text[start:index].strip())
            start = index + 2
    terms.append(text[start:].strip())
    tests: list[list[tuple[str, str, str]]] = []
    # Other names, such as needs and github, are known only at run time, so a
    # term on one holds on some run unless the condition contradicts it.
    free: dict[str, tuple[set[str], set[str]]] = {}
    for term in terms:
        match = COMPARISON.fullmatch(term)
        if CHOICE.fullmatch(term):
            tests.append([(name, "==", value) for name, value in CHOSEN.findall(term)])
        elif match is None:
            return []
        elif match["name"].startswith(("inputs.", "matrix.")) or match["name"] == "runner.os":
            tests.append([(match["name"], match["op"], match["value"])])
        else:
            equal, unequal = free.setdefault(match["name"], (set(), set()))
            (equal if match["op"] == "==" else unequal).add(match["value"])
    if any(len(equal) > 1 or equal & unequal for equal, unequal in free.values()):
        return []
    return [
        (row, names) for row, names in options
        if all(
            any(name in names and (names[name] == value) == (op == "==") for name, op, value in test)
            for test in tests
        )
    ]


def steps(root: Path, flows: dict[str, dict]):
    """Yield (where, working directory, script) for every run step and call."""
    called = calls(root, flows)
    for name, flow in flows.items():
        on = flow.get("on") or ""
        for inputs in called[name] if "workflow_call" in on else [{}]:
            for job_id, job in flow["jobs"].items():
                running = satisfying(at(job, "if"), contexts(root, job, inputs))
                for step in at(job, "steps") or []:
                    if not isinstance(at(step, "run"), str) or not satisfying(at(step, "if"), running):
                        continue
                    directory = (
                        at(step, "working-directory")
                        or at(job, "defaults", "run", "working-directory")
                        or at(flow, "defaults", "run", "working-directory")
                        or "."
                    )
                    where = f"{WORKFLOWS}/{name} job {job_id}"
                    yield where, substitute(str(directory), inputs, {}), step["run"]


def shell_words(line: str) -> list[tuple[str, str]]:
    """Split a shell line into (text, kind) pairs: a word without its quotes
    ("word", or "quoted" if any of it was quoted) or a run of unquoted operator
    characters ("operator"). An unquoted # starting a word ends the line."""
    found: list[tuple[str, str]] = []
    word, quoted, quote, index = "", False, "", 0
    while index < len(line):
        char = line[index]
        if quote:
            if char == quote:
                quote = ""
            elif char == "\\" and quote == '"' and line[index + 1 : index + 2] in ('"', "\\", "$", "`"):
                index += 1
                word += line[index]
            else:
                word += char
        elif char in "'\"":
            quote, quoted = char, True
        elif char == "\\":
            index += 1
            word, quoted = word + line[index : index + 1], True
        elif char == "#" and not word and not quoted:
            break
        elif char.isspace() or char in OPERATORS:
            if word or quoted:
                found.append((word, "quoted" if quoted else "word"))
                word, quoted = "", False
            if char in OPERATORS and found and found[-1][1] == "operator" and line[index - 1] in OPERATORS:
                found[-1] = (found[-1][0] + char, "operator")
            elif char in OPERATORS:
                found.append((char, "operator"))
        else:
            word += char
        index += 1
    if quote:
        raise ValueError("an unclosed quote")
    if word or quoted:
        found.append((word, "quoted" if quoted else "word"))
    return found


def shell_commands(script: str, where: str) -> tuple[list[list[str]], bool]:
    """Split a step's script into commands, one line at a time, and say whether
    it uses control flow, && or ||, a function or a heredoc."""
    commands = []
    control = False
    for line in script.replace("\\\n", " ").splitlines():
        try:
            words = shell_words(line)
        except ValueError:
            if RUNNER.search(line):
                raise Unreadable(f"{where}: cannot split {line.strip()!r}") from None
            continue
        command: list[str] = []
        previous = ""
        for text, kind in [*words, (";", "operator")]:
            if kind == "operator":
                function = text == "()" or (previous == "(" and text == ")")
                if function or text in ("&&", "||") or text.startswith("<<"):
                    control = True
                if command:
                    commands.append(command)
                command = []
            elif not command and kind == "word" and text in CONTROL:
                control = True
            elif command or not (
                (kind == "word" and text in PREFIXES) or re.match(r"[A-Za-z_]\w*=", text)
            ):
                command.append(text)
            previous = text if kind == "operator" else ""
        if command:
            commands.append(command)
    return commands, control


def join(base: str, path: str) -> str:
    return posixpath.normpath(posixpath.join(base, path))


def inside(path: str, directory: str) -> bool:
    return directory == "." or path == directory or path.startswith(directory + "/")


def listed(value: str | list[str]) -> list[str]:
    """A pytest setting as a list; the ini form separates entries with spaces."""
    return value.split() if isinstance(value, str) else value


def pytest_runs(root: Path, files: list[str], base: str, args: list[str]) -> set[str]:
    if any((root / base / name).is_file() for name in ("pytest.ini", ".pytest.ini")):
        raise Unreadable(f"{base}: pytest reads pytest.ini, which the check does not model")
    pyproject = root / base / "pyproject.toml"
    table = {}
    if pyproject.is_file():
        table = at(tomllib.loads(pyproject.read_text(encoding="utf-8")), "tool", "pytest") or {}
    config = table.get("ini_options", table)
    addopts = config.get("addopts", [])
    options = [*args, *(shlex.split(addopts) if isinstance(addopts, str) else addopts)]
    for option in options:
        if option.split("=", 1)[0] in NARROWING or option[:2] in ("-k", "-m"):
            raise Unreadable(f"{base}: pytest {option} would narrow what it collects")
    patterns = listed(config.get("python_files", PYTEST_FILES))
    named = [join(base, arg) for arg in args if not arg.startswith("-")]
    roots = named or [join(base, path) for path in listed(config.get("testpaths", ["."]))]
    return {
        path for path in files
        if path in named or (
            any(inside(path, root) for root in roots)
            and any(fnmatch.fnmatchcase(posixpath.basename(path), p) for p in patterns)
        )
    }


def runs(root: Path, files: list[str], where: str, directory: str, argv: list[str]) -> set[str]:
    """The targets one command runs from its working directory."""
    name, args = argv[0], argv[1:]
    if name == "cargo" and args[:1] and args[0].startswith("+"):
        args = args[1:]
    cargo_test = name == "cargo" and args[:1] in (["test"], ["llvm-cov"]) and "--no-run" not in args
    fuzz = name == "cargo" and args[:2] == ["fuzz", "run"]
    at_pytest = argv.index("pytest") if "pytest" in argv else -1
    pytest = at_pytest == 0 or (at_pytest > 0 and (
        (name == "uv" and args[:1] == ["run"] and argv[at_pytest - 1] != "--with")
        or (name in ("python", "python3") and argv[at_pytest - 1] == "-m")
    ))
    unittest = name in ("python", "python3") and args[:2] == ["-m", "unittest"]
    npm = name == "npm" and (args[:1] == ["test"] or (args[:1] == ["run"] and len(args) > 1))
    node = name == "node" and "--test" in args
    if not (cargo_test or fuzz or pytest or unittest or npm or node):
        return set()
    if "${{" in directory:
        raise Unreadable(f"{where}: cannot resolve the working directory {directory!r}")
    base = posixpath.normpath(directory)
    if cargo_test:
        manifest = next(
            (args[i + 1] for i, arg in enumerate(args[:-1]) if arg == "--manifest-path"),
            next((arg.split("=", 1)[1] for arg in args if arg.startswith("--manifest-path=")), None),
        )
        return {posixpath.dirname(join(base, manifest)) if manifest else base}
    if fuzz:
        return {join(base, "fuzz")}
    if pytest:
        return pytest_runs(root, files, base, argv[at_pytest + 1 :])
    if unittest:
        return {join(base, arg) for arg in args[2:] if not arg.startswith("-")}
    if npm:
        package = root / base / "package.json"
        scripts = json.loads(package.read_text(encoding="utf-8")).get("scripts", {})
        script = scripts.get("test" if args[0] == "test" else args[1])
        return step_runs(root, files, where, directory, script) if isinstance(script, str) else set()
    patterns = [join(base, arg) for arg in args if not arg.startswith("-")]
    if not patterns:
        raise Unreadable(f"{where}: node --test without file arguments")
    # A shell without globstar reads ** as *, and node expands a quoted one itself.
    if any("**" in pattern for pattern in patterns):
        raise Unreadable(f"{where}: node --test with a recursive glob")
    return {path for path in files if any(PurePosixPath(path).full_match(p) for p in patterns)}


def step_runs(root: Path, files: list[str], where: str, directory: str, script: str) -> set[str]:
    commands, control = shell_commands(script, where)
    found = set().union(*(runs(root, files, where, directory, argv) for argv in commands))
    if found and any(argv[0] in ("cd", "pushd") for argv in commands):
        raise Unreadable(f"{where}: a test step changes directory; set its working-directory")
    if found and control:
        raise Unreadable(f"{where}: a test step uses shell control flow or a heredoc")
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
    run = set().union(*(step_runs(root, files, *step) for step in steps(root, flows)))
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
