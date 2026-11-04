# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: FSL-1.1-ALv2

"""Select application checks on PRs and validate their aggregate results."""

from __future__ import annotations

import fnmatch
import json
import os
import subprocess
import sys
from pathlib import Path

from select_package import is_shared, parse_changed_paths

# Root pytest checks these lists against the manifests. Its required
# boundaries-gates owner runs on every change, including a skipped app.
DEPENDENCIES = {
    # The MCP server takes australian-tax-calculators, div7a-loan-review,
    # ato-benchmark-compare, payday-super-checker and gst-review from PyPI
    # until it links the Rust crates, so their directories do not select it.
    "aus-accounting-mcp": frozenset({
        "lodgment-calendar",
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
    "devcontainer": frozenset({"setup"}),
    "glama-badges": frozenset({"check", "dependency-audit", "worker"}),
    "readme-quick-start": frozenset({"trial"}),
}
# Every selection's jobs, by the name its select and gate commands take.
SELECTED_JOBS = APP_JOBS | PATH_JOBS
KNOWN_PREFIXES = tuple(f"packages/{name}/" for name in PACKAGES) + tuple(
    f"apps/{name}/" for name in DEPENDENCIES
)


def should_run(app: str, paths: list[str]) -> bool:
    relevant = (f"apps/{app}/", "tests/") + tuple(
        f"packages/{name}/" for name in DEPENDENCIES[app]
    )
    if not paths:
        return True
    for path in paths:
        if not path or any(part in {"", ".", ".."} for part in path.split("/")):
            return True
        if is_shared(path) or path.startswith(relevant):
            return True
        if not path.startswith(KNOWN_PREFIXES):
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


def check_results(results: object, event: str, app: str) -> None:
    if not isinstance(results, dict):
        raise ValueError("Job results must be an object")
    changes = results.get("changes")
    if not isinstance(changes, dict) or changes.get("result") != "success":
        raise ValueError("Application selection did not succeed")
    outputs = changes.get("outputs")
    run = outputs.get("run") if isinstance(outputs, dict) else None
    if run not in ("true", "false") or (event != "pull_request" and run != "true"):
        raise ValueError("Invalid application selection")
    app_jobs = SELECTED_JOBS[app]
    if not app_jobs <= results.keys():
        raise ValueError("Missing application job results")
    expected_app = "success" if run == "true" else "skipped"
    failed = [
        name for name, job in results.items()
        if not isinstance(job, dict)
        or job.get("result") != (expected_app if name in app_jobs else "success")
    ]
    if failed:
        raise ValueError("Jobs did not satisfy application selection: " + ", ".join(failed))


def main(argv: list[str]) -> int:
    event = os.environ.get("GITHUB_EVENT_NAME", "")
    try:
        if len(argv) == 3 and argv[1] == "gate" and argv[2] in SELECTED_JOBS:
            check_results(json.loads(os.environ.get("RESULTS", "")), event, argv[2])
        elif len(argv) == 3 and argv[1] == "select" and argv[2] in SELECTED_JOBS:
            paths = changed_paths(Path.cwd()) if event == "pull_request" else []
            if argv[2] in PATH_JOBS:
                selected = path_selected(paths, trigger_paths())
            else:
                selected = should_run(argv[2], paths)
            run = event != "pull_request" or selected
            reason = "relevant or unknown change" if run else "only known unrelated paths"
            print(f"Application checks: {reason}", file=sys.stderr)
            print(f"run={'true' if run else 'false'}")
        else:
            print(f"usage: {argv[0]} select <app> | gate <app>", file=sys.stderr)
            return 2
    except (ValueError, TypeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
