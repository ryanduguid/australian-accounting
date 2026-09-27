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

On a push to `main` the caller adds `--on-main`. A package whose pyproject
version has no `<package>/v<version>` tag yet then runs as well, whatever
changed, so its release can be tagged on whichever commit is `main`'s head: the
release gate needs this package's push checks on that exact commit. Once every
version is tagged this adds nothing.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

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
        # The root workspace files: the root uv.lock is what `uv run --locked`
        # validates from inside every component directory, so a change to it
        # changes what every engine resolves.
        "pyproject.toml",
        "uv.lock",
        "justfile",
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


def project_version(package: str, root: Path = ROOT) -> str:
    """The static ``version`` in the package's ``[project]`` table."""
    text = (root / PACKAGE_ROOT / package / "pyproject.toml").read_text(encoding="utf-8")
    table = re.search(r"(?ms)^\[project\]\s*$(.*?)(?=^\[|\Z)", text)
    # TOML allows a basic ("...") or a literal ('...') string here.
    match = table and re.search(r"""(?m)^version\s*=\s*(["'])([^"'\n]+)\1""", table.group(1))
    if not match:
        raise ValueError(f"{package}: no static [project] version")
    return match.group(2)


def release_pending(package: str, tags: list[str], root: Path = ROOT) -> bool:
    """True when this commit's version of ``package`` has no release tag yet."""
    return f"{package}/v{project_version(package, root)}" not in tags


def main(argv: list[str]) -> int:
    args = [arg for arg in argv[1:] if arg != "--on-main"]
    if len(args) != 1:
        print(f"usage: {argv[0]} <package> [--on-main]", file=sys.stderr)
        return 2
    package = args[0]
    changed_paths = [line.strip() for line in sys.stdin if line.strip()]
    run = should_run(package, changed_paths)
    if not run and "--on-main" in argv:
        tags = subprocess.run(
            ["git", "tag", "--list", f"{package}/v*"],
            capture_output=True, text=True, check=True, cwd=ROOT,
        ).stdout.split()
        run = release_pending(package, tags)
    print(f"run={'true' if run else 'false'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
