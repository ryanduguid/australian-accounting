"""Hold `au-accounting ENGINE` to each engine's own command, byte for byte.

Every input in each engine's frozen corpora (tests/data/python-*.json), plus
edge-case JSON the corpora rarely carry, runs through the engine's standalone
binary and through `au-accounting`, each in its own scratch folder; exit codes,
stdout, stderr and (for The WIP Tally's file runs) the files left behind must
match, including the line and column of JSON input errors.

Standard library only. usage:
    python -I apps/au-accounting/tests/parity.py BIN_DIR [--cap N]
BIN_DIR holds au-accounting and the 8 engine commands; --cap samples at most N
corpus rows per engine and kind (default: every row).
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EXE = ".exe" if os.name == "nt" else ""
# subcommand: (crate directory under packages/, standalone command)
ENGINES = {
    "benchmarks": ("ato-benchmark-compare", "ato-benchmark-compare"),
    "div7a": ("div7a-loan-review", "div7a-loan-review"),
    "exchequer": ("the-exchequer-tally", "the-exchequer-tally"),
    "gst-review": ("gst-review", "gst-review"),
    "payday-super": ("payday-super-checker", "payday-super-check"),
    "solomons": ("solomons-sword", "solomons-sword"),
    "tax": ("australian-tax-calculators", "australian-tax-calculators"),
    "wip": ("the-wip-tally", "wip-tally"),
}
NUMBERS = ["0", "-0", "1", "-1", "1.0", "1e2", "1E2", "0.1", "2.5e-3", "-0.0",
           "18446744073709551615", "18446744073709551616", "-9223372036854775808",
           "-9223372036854775809", "123456789012345678901234567890", "1e308", "1e309",
           "-1e400", "4.9e-324", "1e-400", "0.30000000000000004"]
NUMBERS += ["51.248178375505404"]
EDGES = (["{}", "[]", "null", "", "{", '{"b":1,"a":2}', '{"a":1,"a":2}'] + NUMBERS
         + [f'{{"value":{n},"amount":{n},"kind":"money"}}' for n in NUMBERS]
         + [f'{{"z":{{"y":{n},"x":[{n}]}},"a":{n}}}' for n in NUMBERS[:6]])
EDGES += [
    '{"$serde_json::private::Number":"2026"}',
    '{"financial_year":{"$serde_json::private::Number":"2026"}}',
    '{"financial_year":{"$serde_json::private::Number":2026}}',
    '{"financial_year":{"$serde_json::private::Number":"bad","extra":true}}',
    '{"financial_year":2026,"ignored":{"$serde_json::private::Number":"bad"}}',
    '{"a":1e400,"a":2}',
    '{"12[{":"\\\\\\\"[123{","a":[null,true,false,-1.2e+30,{"n":2}]}',
    '[' * 127 + '0' + ']' * 127,
    '[' * 128 + '0' + ']' * 128,
]


def corpus(package: str) -> list[dict]:
    rows = []
    if package == "gst-review":
        return json.loads((ROOT / "packages/gst-review/tests/fixtures/python-contracts.json").read_text(encoding="utf-8"))
    for path in sorted((ROOT / "packages" / package / "tests/data").glob("python-*.json")):
        data = json.loads(path.read_text(encoding="utf-8"))
        rows.extend(data["cases"] if isinstance(data, dict) else data)
    return rows


def sample(rows: list[dict], cap: int) -> list[dict]:
    if not cap or len(rows) <= cap:
        return rows
    return [rows[i * len(rows) // cap] for i in range(cap)]


def environment(row: dict) -> dict[str, str]:
    env = dict(os.environ)
    for key, value in (row.get("env") or {}).items():
        if value is None:
            env.pop(key, None)
        else:
            env[key] = value
    return env


def scrub(data: bytes, folder: str) -> bytes:
    forms = {text.encode() for form in {folder, os.path.realpath(folder)}
             for text in (form, form.replace("\\", "\\\\"))}
    # macOS /var/... aliases /private/var/...; replacing the shorter path first
    # would leave /private<TMP> and create a false difference between commands.
    for form in sorted(forms, key=len, reverse=True):
        data = data.replace(form, b"<TMP>")
    return data


def run(argv: list[str], args: list[str], stdin: bytes, folder: str, env=None):
    p = subprocess.run(argv + args, input=stdin, capture_output=True, cwd=folder,
                       env=env, timeout=300)
    return p.returncode, scrub(p.stdout, folder), scrub(p.stderr, folder)


def file_run(argv: list[str], row: dict):
    """One of The WIP Tally's corpus runs: files written, then its steps in order."""
    with tempfile.TemporaryDirectory() as folder:
        for rel, data in row["files"].items():
            path = Path(folder, rel)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(bytes.fromhex(data))
        results = [run(argv, step, b"", folder, environment(row)) for step in row["steps"]]
        files = sorted((p.relative_to(folder).as_posix(), p.read_bytes())
                       for p in Path(folder).rglob("*") if p.is_file())
        return results, files


def check(name: str, bin_dir: Path, cap: int) -> tuple[str, int, list[str]]:
    package, command = ENGINES[name]
    standalone = [str(bin_dir / f"{command}{EXE}")]
    umbrella = [str(bin_dir / f"au-accounting{EXE}"), name]
    rows = corpus(package)
    api = sample([r for r in rows if "command" in r], cap)
    cli = sample([r for r in rows if "args" in r], cap)
    files = sample([r for r in rows if "steps" in r], cap)
    help_text = subprocess.run(standalone + ["--api-help"], capture_output=True).stdout
    commands = help_text.decode().splitlines()[0].removeprefix("JSON stdin commands: ").split(", ")
    cases = [([r["command"]], json.dumps(r["input"]).encode(), r, r.get("test")) for r in api]
    cases += [(list(r["args"]), b"", r, r.get("test")) for r in cli]
    cases += [([c], text.encode(), {}, f"edge {text!r}") for c in commands for text in EDGES]
    # Routing an engine's own main() adds outside its library must reach
    # au-accounting too; derive and verify are such commands on some branches.
    cases += [(args, stdin, {}, "routing") for args, stdin in
              ((["--api-help"], b""), (["--help"], b""), (["--version"], b""), ([], b""),
               (["--nope"], b""), (["derive"], b"{}"), (["verify"], b"{}"),
               (["verify", "missing.json"], b""))]
    failures = []
    with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
        for args, stdin, row, label in cases:
            env = environment(row)
            left = run(standalone, args, stdin, a, env)
            right = run(umbrella, args, stdin, b, env)
            if left != right:
                failures.append(f"{name} {args} ({label}):\n  own: {left!r:.300}\n  au-accounting: {right!r:.300}")
    for row in files:
        if file_run(standalone, row) != file_run(umbrella, row):
            failures.append(f"{name} file run {row['steps']} ({row.get('test')})")
    return name, len(cases) + len(files), failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("bin_dir", type=Path)
    parser.add_argument("--cap", type=int, default=0)
    options = parser.parse_args()
    # Each command runs inside a scratch folder; resolve binaries before chdir.
    options.bin_dir = options.bin_dir.resolve()
    with ThreadPoolExecutor(max_workers=len(ENGINES)) as pool:
        results = list(pool.map(lambda n: check(n, options.bin_dir, options.cap), ENGINES))
    failed = False
    for name, count, failures in results:
        print(f"{name}: {count} cases, {len(failures)} differences")
        for failure in failures[:10]:
            print(failure)
        failed |= bool(failures)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
