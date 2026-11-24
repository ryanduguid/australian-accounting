# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Select application checks on PRs and validate their aggregate results."""

from __future__ import annotations

import fnmatch
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from rust_crates import ROOT, UMBRELLA, crate_dependencies, crates, linked
from select_package import is_shared, parse_changed_paths

# Root pytest checks these lists against the manifests. Its required
# boundaries-gates owner runs on every change, including a skipped app.
DEPENDENCIES = {
    # The unreleased native server links these Rust engines by path, which is
    # what selects this application's checks. The published Python server still
    # takes div7a-loan-review, ato-benchmark-compare, payday-super-checker and
    # gst-review from PyPI, and its tests run the australian-tax-calculators
    # executable built from this commit.
    "aus-accounting-mcp": frozenset({
        "ato-benchmark-compare", "australian-tax-calculators", "div7a-loan-review",
        "gst-review", "lodgment-calendar", "payday-super-checker",
    }),
    # The adapter's trials link these Rust engines by path.
    "lodgeit-calculator-adapter": frozenset({
        "australian-tax-calculators", "div7a-loan-review",
    }),
}
PACKAGES = frozenset().union(*DEPENDENCIES.values(), {
    "solomons-sword", "the-exchequer-tally", "the-wip-tally", "australian-tax-calculators",
    "div7a-loan-review", "ato-benchmark-compare", "payday-super-checker", "gst-review",
})
# The jobs each application's selection runs or skips. The adapter's one job
# calls ci-rust.yml.
APP_JOBS = {
    "aus-accounting-mcp": frozenset({"test", "lint", "dependency-audit"}),
    "lodgeit-calculator-adapter": frozenset({"rust"}),
}
# Workflows that once filtered pull requests by path, and the jobs that filter
# ran. Each one's changes job passes the former trigger paths in CI_PATHS.
PATH_JOBS = {
    "aus-accounting-connector": frozenset({"core", "dependency-audit", "worker"}),
    "devcontainer": frozenset({"setup"}),
    "glama-badges": frozenset({"check", "dependency-audit", "worker"}),
    "readme-quick-start": frozenset({"trial"}),
}
# ci.yml's crates job selects the jobs that check the crates rust_crates.py lists
# for it. Those crates link only each other by path, which root pytest checks.
CRATE_JOBS = {"crates": frozenset({"rust", "umbrella-parity"})}
# The job whose run output holds a selection, where it is not changes.
SELECTORS = {"crates": "crates"}
# Every selection's jobs, by the name its select and gate commands take.
SELECTED_JOBS = APP_JOBS | PATH_JOBS | CRATE_JOBS
KNOWN_PREFIXES = tuple(f"packages/{name}/" for name in PACKAGES) + tuple(
    f"apps/{name}/" for name in DEPENDENCIES
)
# Root Python tests and the tool workspaces, which none of those crates reads.
CRATE_UNRELATED_PREFIXES = ("tests/", "tools/")


def should_run(app: str, paths: list[str]) -> bool:
    relevant = (f"apps/{app}/", "tests/") + tuple(
        f"packages/{name}/" for name in DEPENDENCIES[app]
    )
    return change_selects(paths, relevant, KNOWN_PREFIXES)


@dataclass(frozen=True)
class CrateSelection:
    crates: list[dict[str, str]]
    run: bool
    parity_run: bool
    parity_engines: list[str]
    all_parity: bool


def _direct_crates(
    paths: list[str],
    crate_prefixes: tuple[tuple[str, str], ...],
    unrelated_prefixes: tuple[str, ...],
) -> set[str] | None:
    """Return directly changed crate names, or None when paths fail closed."""
    direct: set[str] = set()
    for path in paths:
        if not path or any(part in {"", ".", ".."} for part in path.split("/")) or is_shared(path):
            return None
        matched = next((pkg for prefix, pkg in crate_prefixes if path.startswith(prefix)), None)
        if matched is not None:
            direct.add(matched)
        elif not path.startswith(unrelated_prefixes):
            return None
    return direct


def _reverse_closure(direct: set[str], deps: dict[str, set[str]]) -> set[str]:
    """Expand directly changed crates to all reverse transitive dependents."""
    reverse: dict[str, set[str]] = {name: set() for name in deps}
    for pkg, pkg_deps in deps.items():
        for dep in pkg_deps:
            reverse[dep].add(pkg)
    selected = set(direct)
    queue = list(direct)
    while queue:
        for dependent in reverse[queue.pop()]:
            if dependent not in selected:
                selected.add(dependent)
                queue.append(dependent)
    return selected


def _parity_scope(
    direct: set[str],
    selected: set[str],
    umbrella_name: str,
    linked_engines: list[str],
) -> tuple[bool, list[str], bool]:
    """Determine parity run flag, engine list and full-parity marker."""
    linked_set = set(linked_engines)
    if umbrella_name in direct:
        # au-accounting itself changed: rebuild and parity-test every linked engine.
        selected.update(linked_set)
        return True, linked_engines, True
    affected = sorted(selected & linked_set)
    if affected:
        return True, affected, len(affected) == len(linked_engines)
    return False, [], False


def select_crates(paths: list[str], root: Path = ROOT) -> CrateSelection:
    """Select the Rust crates and parity engines a pull request must check."""
    all_crates = crates(root, ci_yml=False)
    ci_crates = crates(root, ci_yml=True)
    linked_engines = linked(root)
    full = CrateSelection(ci_crates, True, True, linked_engines, True)

    deps = crate_dependencies(root, ci_yml=True) if paths else None
    if deps is None:
        return full

    by_name = {crate["package"]: crate for crate in ci_crates}
    crate_prefixes = tuple((f"{c['root']}/{c['package']}/", c["package"]) for c in ci_crates)
    own_workflow_prefixes = tuple(
        f"{c['root']}/{c['package']}/" for c in all_crates if c["package"] not in by_name
    )
    direct = _direct_crates(paths, crate_prefixes, own_workflow_prefixes + CRATE_UNRELATED_PREFIXES)
    if direct is None:
        return full
    if not direct:
        return CrateSelection([], False, False, [], False)

    selected = _reverse_closure(direct, deps)
    parity_run, parity_engines, all_parity = _parity_scope(
        direct, selected, Path(UMBRELLA).name, linked_engines
    )
    return CrateSelection(
        crates=[crate for crate in ci_crates if crate["package"] in selected],
        run=True,
        parity_run=parity_run,
        parity_engines=parity_engines,
        all_parity=all_parity,
    )


def crates_should_run(paths: list[str], root: Path = ROOT) -> bool:
    return select_crates(paths, root=root).run


def change_selects(paths: list[str], relevant: tuple[str, ...], known: tuple[str, ...]) -> bool:
    """True unless every path is known and outside the shared and relevant paths."""
    if not paths:
        return True
    for path in paths:
        if not path or any(part in {"", ".", ".."} for part in path.split("/")):
            return True
        if is_shared(path) or path.startswith(relevant):
            return True
        if not path.startswith(known):
            return True
    return False


def trigger_paths() -> list[str]:
    """The CI_PATHS patterns, one per line; none is an error rather than a skip."""
    lines = os.environ.get("CI_PATHS", "").splitlines()
    patterns = [line.strip() for line in lines if line.strip()]
    if not patterns:
        raise ValueError("CI_PATHS names no paths")
    return patterns


def path_selected(paths: list[str], patterns: list[str]) -> bool:
    """True when a changed path matches a pattern, or what changed is unknown."""
    # fnmatch's * also matches /, unlike GitHub's. CI_PATHS holds only literal paths
    # and a trailing /**, which select the same paths in both.
    return not paths or any(
        fnmatch.fnmatchcase(path, pattern) for path in paths for pattern in patterns
    )


def mcp_python_matrix(event: str, ref: str, *, selected: bool = True) -> list[dict[str, str]]:
    """Always produce Linux coverage, expanding the matrix for relevant changes."""
    rows = [{"os": "ubuntu-latest", "python": "3.14"}]
    if not selected:
        return rows
    rows.extend([
        {"os": "ubuntu-latest", "python": "3.15"},
        {"os": "windows-latest", "python": "3.14"},
    ])
    if event == "schedule" or (event == "push" and ref == "refs/heads/main"):
        rows.append({"os": "macos-latest", "python": "3.14"})
    return rows


def changed_paths(root: Path) -> list[str]:
    """Use both sides of a PR merge; uncertainty selects all checks."""
    try:
        parents = subprocess.run(
            ["git", "cat-file", "-p", "HEAD"], cwd=root,
            check=True, capture_output=True,
        ).stdout.split(b"\n\n", 1)[0].splitlines()
        if sum(line.startswith(b"parent ") for line in parents) != 2:
            return []
        diff = subprocess.run(
            ["git", "diff", "--name-only", "--no-renames", "-z", "HEAD^1..HEAD", "--"],
            cwd=root, check=True, capture_output=True,
        ).stdout
        return parse_changed_paths(diff)
    except (OSError, subprocess.CalledProcessError):
        return []


def check_results(results: object, event: str, *apps: str) -> None:
    if not isinstance(results, dict):
        raise ValueError("Job results must be an object")
    expected = {}
    for app in apps:
        changes = results.get(SELECTORS.get(app, "changes"))
        if not isinstance(changes, dict) or changes.get("result") != "success":
            raise ValueError("Application selection did not succeed")
        outputs = changes.get("outputs")
        run = outputs.get("run") if isinstance(outputs, dict) else None
        if run not in ("true", "false") or (event != "pull_request" and run != "true"):
            raise ValueError("Invalid application selection")
        app_jobs = SELECTED_JOBS[app]
        if not app_jobs <= results.keys():
            raise ValueError("Missing application job results")
        expected.update(dict.fromkeys(app_jobs, "success" if run == "true" else "skipped"))
        if app == "aus-accounting-mcp":
            expected["test"] = "success"
        elif app == "crates":
            parity_run = outputs.get("parity-run") if isinstance(outputs, dict) else None
            if (
                parity_run not in ("true", "false")
                or (event != "pull_request" and parity_run != "true")
                or (run == "false" and parity_run != "false")
            ):
                raise ValueError("Invalid application selection")
            expected["umbrella-parity"] = "success" if parity_run == "true" else "skipped"
    failed = [
        name for name, job in results.items()
        if not isinstance(job, dict) or job.get("result") != expected.get(name, "success")
    ]
    if failed:
        raise ValueError("Jobs did not satisfy application selection: " + ", ".join(failed))


def main(argv: list[str]) -> int:
    event = os.environ.get("GITHUB_EVENT_NAME", "")
    try:
        if len(argv) >= 3 and argv[1] == "gate" and set(argv[2:]) <= SELECTED_JOBS.keys():
            check_results(json.loads(os.environ.get("RESULTS", "")), event, *argv[2:])
        elif len(argv) == 3 and argv[1] == "select" and argv[2] in SELECTED_JOBS:
            paths = changed_paths(Path.cwd()) if event == "pull_request" else []
            if argv[2] in CRATE_JOBS:
                selection = select_crates(paths if event == "pull_request" else [], root=Path.cwd())
                parity_names = (
                    [*selection.parity_engines, Path(UMBRELLA).name] if selection.parity_run else []
                )
                parity_glob = ("{" + ",".join(parity_names) + "}") if parity_names else ""
                parity_env = (
                    ""
                    if (not selection.parity_run or selection.all_parity)
                    else ",".join(selection.parity_engines)
                )
                selected_names = [crate["package"] for crate in selection.crates]
                reason = (
                    f"selected {len(selected_names)} crate(s): {', '.join(selected_names)}"
                    if selection.run
                    else "only known unrelated paths"
                )
                print(f"Crate checks: {reason}", file=sys.stderr)
                print(f"run={'true' if selection.run else 'false'}")
                print(f"parity-run={'true' if selection.parity_run else 'false'}")
                print("ci-matrix=" + json.dumps({"include": selection.crates}, separators=(",", ":")))
                print(f"parity={parity_glob}")
                print(f"parity-count={len(parity_names)}")
                print(f"parity-engines={parity_env}")
                return 0
            if argv[2] in PATH_JOBS:
                selected = path_selected(paths, trigger_paths())
            else:
                selected = should_run(argv[2], paths)
            run = event != "pull_request" or selected
            reason = "relevant or unknown change" if run else "only known unrelated paths"
            print(f"Application checks: {reason}", file=sys.stderr)
            print(f"run={'true' if run else 'false'}")
        elif argv[1:] == ["mcp-python-matrix"]:
            rows = mcp_python_matrix(
                event,
                os.environ.get("GITHUB_REF", ""),
                selected=os.environ.get("CI_APP_SELECTED") != "false",
            )
            print("matrix=" + json.dumps({"include": rows}, separators=(",", ":")))
        else:
            print(f"usage: {argv[0]} select <app> | gate <app> [<app> ...] | mcp-python-matrix", file=sys.stderr)
            return 2
    except (ValueError, TypeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
