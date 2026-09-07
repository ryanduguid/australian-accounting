# Development entrypoint for the whole repository.
#
# These recipes are a convenience, not a new authority. Each component is still
# tested and released from its own directory with its own commands, and CI runs
# those commands per component; `just test` runs the same checks over every
# component in one pass so a fresh clone can be verified from the top.
#
# Requires just and uv:
#     uv tool install rust-just
#
# Component directory : the import package it owns. Adding a component means
# adding it here and to tests/test_boundaries.py.
components := trim(replace('''
packages/ato-benchmark-compare:atobenchmark
packages/payday-super-checker:paydaysuper
packages/div7a-loan-review:div7aloan
packages/solomons-sword:louisgoldberg
packages/the-exchequer-tally:edwinnixon
packages/the-wip-tally:wiptally
apps/aus-accounting-mcp:aus_accounting_mcp
''', "\n", " "))

# List the available recipes.
default:
    @just --list

# Install every component and the shared toolchain into one workspace .venv.
setup:
    uv sync

# Run every component's test suite, plus the repository boundary checks.
test: setup
    #!/usr/bin/env bash
    set -euo pipefail
    echo "==> repository boundaries"
    uv run --no-sync python -m unittest -v tests/test_boundaries.py
    for entry in {{ components }}; do
        directory="${entry%%:*}"
        echo "==> ${directory}"
        (cd "${directory}" && uv run --no-sync pytest -q)
    done

# Lint every component.
lint: setup
    #!/usr/bin/env bash
    set -euo pipefail
    for entry in {{ components }}; do
        directory="${entry%%:*}"
        echo "==> ${directory}"
        (cd "${directory}" && uv run --no-sync ruff check .)
    done

# Type-check every component.
typecheck: setup
    #!/usr/bin/env bash
    set -euo pipefail
    for entry in {{ components }}; do
        directory="${entry%%:*}"
        echo "==> ${directory}"
        (cd "${directory}" && uv run --no-sync mypy)
    done

# Not a CI equivalent. The component workflows also check each engine's own
# lockfile, run a dependency audit, a distribution build, installed-wheel and
# sdist smoke tests, and changed-line coverage, and none of those run here. A
# green `just check` is not a green CI.

# The fast local pass: lint, type-check and test every component.
check: lint typecheck test
