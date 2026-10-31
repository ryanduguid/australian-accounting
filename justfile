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
packages/lodgment-calendar:lodgmentcalendar
apps/aus-accounting-mcp:aus_accounting_mcp
''', "\n", " "))

# List the available recipes.
default:
    @just --list

# Install every component and the shared toolchain into one workspace .venv.
setup:
    uv sync

# Run every component's test suite, plus the repository boundary checks.
test: setup rust-test
    #!/usr/bin/env bash
    set -euo pipefail
    echo "==> repository boundaries"
    uv run --no-sync python -m unittest -v tests/test_boundaries.py tests/test_shared_blocks.py tests/test_release_script.py
    for entry in {{ components }}; do
        directory="${entry%%:*}"
        echo "==> ${directory}"
        (cd "${directory}" && uv run --no-sync pytest -q)
    done

# Lint every component.
lint: setup rust-lint
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
        (cd "${directory}" && uv run --no-sync pyright)
    done

# Regenerate the root lock and every component's own release lock. Run it after
# any dependency change, including on a Dependabot branch before merging. Inside
# the workspace `uv lock` writes only the root lock, so each component is relocked
# from a copy outside the checkout, as CONTRIBUTING.md describes.
relock:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "==> root"
    uv lock
    for lock in packages/*/uv.lock apps/*/uv.lock; do
        directory="$(dirname "${lock}")"
        [ -f "${directory}/pyproject.toml" ] || continue
        echo "==> ${directory}"
        standalone="$(mktemp -d)"
        tar -C "${directory}" --exclude=.venv -cf - . | tar -C "${standalone}" -xf -
        (cd "${standalone}" && uv lock)
        cp "${standalone}/uv.lock" "${lock}"
        rm -rf "${standalone}"
    done

# Release the named components from main's head, in order, one tag per push,
# each after the previous release run succeeds; nothing is tagged if any check
# refuses. Keep main frozen until it finishes. `--dry-run` checks and tags nothing:
#     just release --dry-run payday-super-checker solomons-sword
release +args:
    uv run --no-project python .github/ci/release.py {{ args }}

# Not a CI equivalent. The component workflows also check each engine's own
# lockfile, run a dependency audit, a distribution build, installed-wheel and
# sdist smoke tests, and changed-line coverage, and none of those run here. A
# green `just check` is not a green CI.

# The fast local pass: lint, type-check and test every component.
check: lint typecheck test

# Every standalone Rust crate (the engines, the au-accounting command and the
# rates dataset's validator), outside the virtual Python workspace. CI lists them
# the same way, in .github/ci/rust_crates.py.
rust_manifests := "packages/*/Cargo.toml apps/*/Cargo.toml"

rust-test:
    #!/usr/bin/env bash
    set -euo pipefail
    for manifest in {{ rust_manifests }}; do
        (cd "${manifest%/Cargo.toml}" && cargo test --locked)
    done

rust-lint:
    #!/usr/bin/env bash
    set -euo pipefail
    for manifest in {{ rust_manifests }}; do
        (cd "${manifest%/Cargo.toml}" && cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings)
    done
