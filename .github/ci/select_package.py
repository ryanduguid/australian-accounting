"""Decide whether one engine's CI gates run for a set of changed paths.

`ci.yml` calls `ci-package.yml` once per engine, unconditionally, so that every
gate keeps a stable check name. The path filter therefore lives here instead of
on the workflow trigger: a change under `packages/<name>/` runs that engine, a
change to a root policy file or to anything under `.github/` runs every engine,
and anything else runs none of them.

Read the changed paths from standard input, one per line, and write the
GitHub Actions output line to standard output:

    git diff --name-only "$BASE" "$HEAD" | python3 .github/ci/select_package.py wiptally

Empty input means the caller could not work out what changed, so every gate
runs rather than one being skipped by accident.
"""

from __future__ import annotations

import sys

# A change to any of these runs every engine, because they govern all of them.
SHARED_PATHS = frozenset(
    {
        "AGENTS.md",
        "CONTRIBUTING.md",
        "README.md",
        "SECURITY.md",
        "IMPORTS.md",
        ".editorconfig",
        ".gitignore",
        ".gitattributes",
        ".mailmap",
    }
)
SHARED_PREFIXES = (".github/",)
PACKAGE_ROOT = "packages/"


def is_shared(path: str) -> bool:
    """True when ``path`` governs every engine rather than one of them."""
    return path in SHARED_PATHS or path.startswith(SHARED_PREFIXES)


def should_run(package: str, changed_paths: list[str]) -> bool:
    """True when ``package`` must run its gates for this set of changed paths."""
    if not changed_paths:
        return True
    prefix = f"{PACKAGE_ROOT}{package}/"
    return any(
        is_shared(path) or path.startswith(prefix) for path in changed_paths
    )


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(f"usage: {argv[0]} <package>", file=sys.stderr)
        return 2
    changed_paths = [line.strip() for line in sys.stdin if line.strip()]
    run = should_run(argv[1], changed_paths)
    print(f"run={'true' if run else 'false'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
