"""Select application checks on PRs and validate their aggregate results."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

from select_package import is_shared, parse_changed_paths

# Root pytest checks these lists against the manifests. Its required
# boundaries-gates owner runs on every change, including a skipped app.
DEPENDENCIES = {
    "aus-accounting-mcp": frozenset({
        "australian-tax-calculators", "ato-benchmark-compare",
        "payday-super-checker", "div7a-loan-review",
    }),
    "lodgeit-calculator-adapter": frozenset({
        "australian-tax-calculators", "div7a-loan-review",
    }),
}
PACKAGES = frozenset().union(*DEPENDENCIES.values(), {
    "solomons-sword", "the-exchequer-tally", "the-wip-tally",
})
APP_JOBS = frozenset({"test", "lint", "dependency-audit"})
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


def check_results(results: object, event: str) -> None:
    if not isinstance(results, dict):
        raise ValueError("Job results must be an object")
    changes = results.get("changes")
    if not isinstance(changes, dict) or changes.get("result") != "success":
        raise ValueError("Application selection did not succeed")
    outputs = changes.get("outputs")
    run = outputs.get("run") if isinstance(outputs, dict) else None
    if run not in ("true", "false") or (event != "pull_request" and run != "true"):
        raise ValueError("Invalid application selection")
    if not APP_JOBS <= results.keys():
        raise ValueError("Missing application job results")
    expected_app = "success" if run == "true" else "skipped"
    failed = [
        name for name, job in results.items()
        if not isinstance(job, dict)
        or job.get("result") != (expected_app if name in APP_JOBS else "success")
    ]
    if failed:
        raise ValueError("Jobs did not satisfy application selection: " + ", ".join(failed))


def main(argv: list[str]) -> int:
    event = os.environ.get("GITHUB_EVENT_NAME", "")
    try:
        if argv[1:] == ["gate"]:
            check_results(json.loads(os.environ.get("RESULTS", "")), event)
        elif len(argv) == 3 and argv[1] == "select" and argv[2] in DEPENDENCIES:
            paths = changed_paths(Path.cwd()) if event == "pull_request" else []
            run = event != "pull_request" or should_run(argv[2], paths)
            reason = "relevant or unknown change" if run else "only known unrelated paths"
            print(f"Application checks: {reason}", file=sys.stderr)
            print(f"run={'true' if run else 'false'}")
        else:
            print(f"usage: {argv[0]} select <app> | gate", file=sys.stderr)
            return 2
    except (ValueError, TypeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
