# Imports

Source repositories enter this repository as single verified snapshots created with
`git subtree add --prefix=packages/<destination> <url> <commit> --squash`. Historical
tags and releases stay in the source repositories, which remain the authoritative
location for pre-consolidation history.

Tracked-tree SHA-256 is the SHA-256 of the NUL-delimited bytes of
`git ls-tree -r --full-tree -z HEAD` in a fresh clone of the source at the recorded
commit. The imported subtree must have the same tree id as the source commit.

## Anchor

| Repository | Commit | Tree | Tracked-tree SHA-256 | Latest release | Location |
|---|---|---|---|---|---|
| `https://github.com/ryanduguid/aus-accounting-mcp.git` | `d6bf940cc6850ecb97c07035519167ed86e151ad` | `53718483bad344783dc9728ad5f81ad14a81d723` | `fd5600a4e36b8e090abdc165993e3e7344247b08cdca9bff5f0e5870613f6bae` | v0.1.6 (2026-08-31) | moved to `apps/aus-accounting-mcp/` |

## Sources

| Repository | Commit | Tree | Tracked-tree SHA-256 | Latest release | Destination | Status |
|---|---|---|---|---|---|---|
| `https://github.com/ryanduguid/ato-benchmark-compare.git` | `d290c8b77d5cc47346d7a41e843642c2c7908748` | `2405cd169c5731298851f96d7c992715c49c3c6f` | `29468715ce198635375c944c865c3b15578b84f3ac9945fb53a9731d77dea189` | v0.1.5 (2026-08-31) | `packages/ato-benchmark-compare/` | imported: squash `965739bf4348b74987bd8541a996599bdaeba53b`, merge `f4289747daa83a79ee318d4405fd45c3ea9d6cd8` |
| `https://github.com/ryanduguid/payday-super-checker.git` | `5ffe1d48ef4262bb6aecb34122b314fec7c437c6` | `89631a8992225fd11ca637e726968838667d5846` | `3bc9819cc9230709630fac33f1276944a744d3310d23d005c84b429f6291368e` | v0.1.2 prerelease (2026-08-23) | `packages/payday-super-checker/` | imported: squash `3fa513e9e2bd8d7a19021a92a12fc5e734e15c8b`, merge `89ffab414954ac93edbfa14f1a9a5e7dfc392a50` |
| `https://github.com/ryanduguid/div7a-loan-review.git` | `753e7d630cba0f3b4d5b97f29141c685fc47dd09` | `0d65c3cac7799ffd83f835223bed0253bbf6b212` | `baea80653779c9270c70751a2418b278bcadef1ee7cc10b1eccdf2113e163680` | v0.1.0 (2026-08-31) | `packages/div7a-loan-review/` | imported: squash `4de3b6f092c2499485efb4b3ce128f03f3d5ee35`, merge `18b889ed7db11219bf7b82bf4db51a43cac526d2` |
| `https://github.com/ryanduguid/TheExchequerTally.git` | `1e89aebc9611f1e87114290dc13f3434ac6f5d88` | `a6c50adda17a8ef97f2f439bb64da5761c712880` | `b099a7cddaf55c14ad042445d28b507a3a697e0e47c217a44c508967b08e20ca` | v0.1.2 (2026-08-22) | `packages/the-exchequer-tally/` | imported: squash `02245924342f78dcce110c9872ca576999f1fb2b`, merge `089eda33cc5f5a97ac052491d7d831a30ab2d196` |
| `https://github.com/ryanduguid/SolomonsSword.git` | `af988a45f777559116ec3e59d5abdb0ee7771f90` | `66422d183637058701546a7a1d7ac8aa1254206b` | `b66aa69e69e1589c7864d4d01b60e5f5314c36a5eac344a3749b4faf4cdf4a3e` | v0.1.2 (2026-08-22) | `packages/solomons-sword/` | imported: squash `cb757df10d70be0e490a6f9e5303de9c2394a9d1`, merge `849e8ffffa7e7b7b3e848f9129f14ca0a2a93f1b` |
| `https://github.com/ryanduguid/TheWIPTally.git` | `f6dcdd702d9344745e95174c8783c0b77b5f9dd2` | `578a0419d959801c36ba429969c96d2585f7ab93` | `9549f0ce2f08063cfc5a39ce1febe630dcb0ebbf60a70bcabef3a3e484c1548a` | none | `packages/the-wip-tally/` | imported: squash `36b535c3ea5d096e72a4a29b11168a0721885bc3`, merge `6c8acfecfa770d1df4c3390c92763a2ee577e7c4` |

## Import records

### ato-benchmark-compare

- Imported 2026-09-02 from `https://github.com/ryanduguid/ato-benchmark-compare.git` at commit
  `d290c8b77d5cc47346d7a41e843642c2c7908748` (tree `2405cd169c5731298851f96d7c992715c49c3c6f`,
  tracked-tree SHA-256 `29468715ce198635375c944c865c3b15578b84f3ac9945fb53a9731d77dea189`,
  latest source release v0.1.5).
- Command: `git subtree add --prefix=packages/ato-benchmark-compare https://github.com/ryanduguid/ato-benchmark-compare.git d290c8b77d5cc47346d7a41e843642c2c7908748 --squash`.
- Squash commit `965739bf4348b74987bd8541a996599bdaeba53b`; merge commit
  `f4289747daa83a79ee318d4405fd45c3ea9d6cd8`. `git rev-parse <merge>:packages/ato-benchmark-compare`
  equals the source tree.
- Imported files edited for location: none. The nested `.github/workflows/ci.yml`, `codeql.yml`
  and `release.yml`, `.github/dependabot.yml` and `tools/build_dataset.py` are inert records of the
  source repository; only root workflows are active.
- Checks run from `packages/ato-benchmark-compare/` immediately after import, as the source
  `ci.yml` defines: `uv run --locked --extra dev pytest -q` (279 passed);
  scoped branch coverage run and `coverage xml`; `pip-audit --local --strict` (no known
  vulnerabilities); `python -m build`; clean-wheel `show` and `compare` smoke (`31% to 38%` and
  `32.00%` present); shipped-sdist `pip install -e ".[dev]"` and `pytest -q`
  (276 passed, 3 skipped); `ruff check atobenchmark tests`; `mypy atobenchmark`;
  `uv lock --check`. All passed.
- Migration-context exception: `diff-cover coverage.xml --compare-branch=origin/main
  --branch-coverage --fail-under=100` exits 1 here because `origin/main` of this repository
  (`d6bf940cc6850ecb97c07035519167ed86e151ad`) predates the import, so every line of
  `atobenchmark/mapping.py` counts as changed and the whole file must reach 100 percent.
  Measured 96 percent branch-inclusive over the whole file (`coverage report`: 227 statements,
  5 missed, 100 branches, 4 partial, 97 percent). The check is unchanged; once `main` contains
  the package the comparison covers only changed lines again, as in the source repository.

### payday-super-checker

- Imported 2026-09-02 from `https://github.com/ryanduguid/payday-super-checker.git` at commit
  `5ffe1d48ef4262bb6aecb34122b314fec7c437c6` (tree `89631a8992225fd11ca637e726968838667d5846`,
  tracked-tree SHA-256 `3bc9819cc9230709630fac33f1276944a744d3310d23d005c84b429f6291368e`,
  latest source release v0.1.2 prerelease).
- Command: `git subtree add --prefix=packages/payday-super-checker https://github.com/ryanduguid/payday-super-checker.git 5ffe1d48ef4262bb6aecb34122b314fec7c437c6 --squash`.
- Squash commit `3fa513e9e2bd8d7a19021a92a12fc5e734e15c8b`; merge commit
  `89ffab414954ac93edbfa14f1a9a5e7dfc392a50`. `git rev-parse <merge>:packages/payday-super-checker`
  equals the source tree.
- Imported files edited for location: none. The nested `.github/workflows/ci.yml`,
  `publish-pypi.yml`, `release.yml` (the bespoke experimental prerelease workflow) and
  `verify.yml`, `.github/dependabot.yml`, `tools/demo.tape`, `tools/generate_calendar.py`,
  `tools/release.py` and `tools/render_quick_proof.py` are inert records of the source
  repository; only root workflows are active. The component's own test suite runs
  `tools/generate_calendar.py` and `tools/render_quick_proof.py --check` in subprocesses and
  imports `tools/release.py` with fabricated inputs, exactly as the source repository's tests
  define; none of that touches a remote. Its `git check-ignore` assertions skip themselves
  because `packages/payday-super-checker/.git` does not exist.
- Checks run from `packages/payday-super-checker/` immediately after import, as the source
  `ci.yml` and `verify.yml` define: `uv sync --locked --extra dev --python 3.12`;
  `uv run --locked --extra dev pytest -q` (616 passed, 5 skipped); scoped branch coverage
  run and `coverage xml`; `pip-audit --local --strict` (no known vulnerabilities);
  `python -m build`; clean-wheel sample run (exit 2), `import` flow (exit 0) and the report from
  the imported contributions (exit 2); shipped-sdist `pip install -e ".[dev]"` and `pytest -q`
  (616 passed, 5 skipped); `ruff check paydaysuper tests`; `mypy paydaysuper`;
  `uv lock --check`. All passed.
- Migration-context exception: `diff-cover coverage.xml --compare-branch=origin/main
  --branch-coverage --fail-under=100` exits 1 here because `origin/main` of this repository
  predates the import, so every line of `paydaysuper/assess.py` and `paydaysuper/report.py`
  counts as changed. Measured 97 percent branch-inclusive over both whole files (`assess.py`
  97.1 percent, `report.py` 97.2 percent; `coverage report` 98 percent each). The check is
  unchanged; once `main` contains the package the comparison covers only changed lines again.

### div7a-loan-review

- Imported 2026-09-02 from `https://github.com/ryanduguid/div7a-loan-review.git` at commit
  `753e7d630cba0f3b4d5b97f29141c685fc47dd09` (tree `0d65c3cac7799ffd83f835223bed0253bbf6b212`,
  tracked-tree SHA-256 `baea80653779c9270c70751a2418b278bcadef1ee7cc10b1eccdf2113e163680`,
  latest source release v0.1.0).
- Command: `git subtree add --prefix=packages/div7a-loan-review https://github.com/ryanduguid/div7a-loan-review.git 753e7d630cba0f3b4d5b97f29141c685fc47dd09 --squash`.
- Squash commit `4de3b6f092c2499485efb4b3ce128f03f3d5ee35`; merge commit
  `18b889ed7db11219bf7b82bf4db51a43cac526d2`. `git rev-parse <merge>:packages/div7a-loan-review`
  equals the source tree.
- Imported files edited for location: none. The nested `.github/workflows/ci.yml` and
  `publish-pypi.yml` and `.github/dependabot.yml` are inert records of the source repository;
  only root workflows are active.
- Checks run from `packages/div7a-loan-review/` immediately after import in a fresh CPython
  3.12 virtual environment, as the source `ci.yml` defines: `python -m pip install --upgrade
  pip`; `python -m pip install ".[dev]"`; `python -m ruff check div7aloan tests`;
  `python -m mypy`; `python -m pytest -q` (292 passed, 6 skipped); the documented sample
  registers (`sample_loans_myr_met.csv` exit 0, `sample_loans_mixed.csv` exit 2); the
  no-network grep over `div7aloan/*.py`; `python -m build`; clean-wheel
  `rate --year 2026-27 --format json`. All passed. The component has no lockfile, so there is
  no `uv lock --check`.

### the-exchequer-tally

- Imported 2026-09-02 from `https://github.com/ryanduguid/TheExchequerTally.git` at commit
  `1e89aebc9611f1e87114290dc13f3434ac6f5d88` (tree `a6c50adda17a8ef97f2f439bb64da5761c712880`,
  tracked-tree SHA-256 `b099a7cddaf55c14ad042445d28b507a3a697e0e47c217a44c508967b08e20ca`,
  latest source release v0.1.2).
- Command: `git subtree add --prefix=packages/the-exchequer-tally https://github.com/ryanduguid/TheExchequerTally.git 1e89aebc9611f1e87114290dc13f3434ac6f5d88 --squash`.
- Squash commit `02245924342f78dcce110c9872ca576999f1fb2b`; merge commit
  `089eda33cc5f5a97ac052491d7d831a30ab2d196`. `git rev-parse <merge>:packages/the-exchequer-tally`
  equals the source tree.
- Imported files edited for location: none. The nested `.github/workflows/ci.yml` and
  `codeql.yml` and `.github/dependabot.yml` are inert records of the source repository; only
  root workflows are active.
- Checks run from `packages/the-exchequer-tally/` immediately after import, as the source
  `ci.yml` defines: `uv run --locked --extra dev ruff check edwinnixon tests`;
  `uv run --locked --extra dev mypy edwinnixon`; `uv run --locked --extra dev pytest -q`
  (26 passed); `python -m build`; clean-wheel `the-exchequer-tally --help`;
  `uv lock --check`. All passed.

### solomons-sword

- Imported 2026-09-02 from `https://github.com/ryanduguid/SolomonsSword.git` at commit
  `af988a45f777559116ec3e59d5abdb0ee7771f90` (tree `66422d183637058701546a7a1d7ac8aa1254206b`,
  tracked-tree SHA-256 `b66aa69e69e1589c7864d4d01b60e5f5314c36a5eac344a3749b4faf4cdf4a3e`,
  latest source release v0.1.2).
- Command: `git subtree add --prefix=packages/solomons-sword https://github.com/ryanduguid/SolomonsSword.git af988a45f777559116ec3e59d5abdb0ee7771f90 --squash`.
- Squash commit `cb757df10d70be0e490a6f9e5303de9c2394a9d1`; merge commit
  `849e8ffffa7e7b7b3e848f9129f14ca0a2a93f1b`. `git rev-parse <merge>:packages/solomons-sword`
  equals the source tree.
- Imported files edited for location: none. The nested `.github/workflows/ci.yml` and
  `codeql.yml` and `.github/dependabot.yml` are inert records of the source repository; only
  root workflows are active.
- Checks run from `packages/solomons-sword/` immediately after import, as the source `ci.yml`
  defines: `uv run --locked --extra dev ruff check louisgoldberg tests`;
  `uv run --locked --extra dev mypy louisgoldberg`; `uv run --locked --extra dev pytest -q`
  (22 passed); `python -m build`; clean-wheel `solomons-sword --help`;
  `uv lock --check`. All passed.

### the-wip-tally

- Imported 2026-09-02 from `https://github.com/ryanduguid/TheWIPTally.git` at commit
  `f6dcdd702d9344745e95174c8783c0b77b5f9dd2` (tree `578a0419d959801c36ba429969c96d2585f7ab93`,
  tracked-tree SHA-256 `9549f0ce2f08063cfc5a39ce1febe630dcb0ebbf60a70bcabef3a3e484c1548a`,
  no source release or tag).
- Command: `git subtree add --prefix=packages/the-wip-tally https://github.com/ryanduguid/TheWIPTally.git f6dcdd702d9344745e95174c8783c0b77b5f9dd2 --squash`.
- Squash commit `36b535c3ea5d096e72a4a29b11168a0721885bc3`; merge commit
  `6c8acfecfa770d1df4c3390c92763a2ee577e7c4`. `git rev-parse <merge>:packages/the-wip-tally`
  equals the source tree.
- Imported files edited for location: none. The nested `.github/workflows/ci.yml`,
  `.github/dependabot.yml` and `.github/PULL_REQUEST_TEMPLATE.md` are inert records of the
  source repository; only root workflows are active.
- Checks run from `packages/the-wip-tally/` immediately after import, as the source `ci.yml`
  defines: `uv run --locked --extra dev pytest -q` (40 passed);
  `pip-audit --local --strict` (no known vulnerabilities); `python -m build`; clean-wheel
  `wip-tally schedule examples/sample_contracts.csv --as-at 2026-08-31 -o <tmp>` (exit 2 by
  design, output contains `221,000.00`); `ruff check wiptally tests`; `mypy wiptally`;
  `uv lock --check`. All passed.

## Release Policy prerequisites at the imported snapshots

Every root release caller calls
`ryanduguid/release-policy/.github/workflows/release-python.yml` at commit
`6ad53a7b030da22fc299cee704c37ba7550ea1d7`. That workflow requires, inside the component
directory, a `RELEASE_NOTES.md` whose first line is `# vX.Y.Z` for the tag, a committed
`uv.lock` (it runs `uv run --locked`), a `dev` extra providing `pytest` and `build`, a
pure-Python wheel, and a version source that is either a static `[project] version` in
`pyproject.toml` or a single-literal `__version__` file selected with
`version-parser: python-literal`. It also requires an annotated tag on the `main` commit,
a clean tree and no existing release for the tag.

| Component | Notes header | Lockfile | dev extra | Pure wheel | Version source | Status at the imported snapshot |
|---|---|---|---|---|---|---|
| aus-accounting-mcp | `# v0.1.6` | `uv.lock` | pytest, build | yes | `pyproject.toml` 0.1.6 | satisfied |
| ato-benchmark-compare | `# v0.1.5` | `uv.lock` | pytest, build | yes | `atobenchmark/__init__.py` 0.1.5 (python-literal) | satisfied |
| payday-super-checker | absent | `uv.lock` | pytest, build | yes | `pyproject.toml` 0.1.2 | fail-closed: no `RELEASE_NOTES.md` (its notes live under `docs/releases/`) |
| div7a-loan-review | absent | absent | pytest, build | yes | `pyproject.toml` 0.1.0 | fail-closed: no `RELEASE_NOTES.md` and no `uv.lock` |
| the-exchequer-tally | `# v0.1.2` | `uv.lock` | pytest, build | yes | `pyproject.toml` 0.1.2 | satisfied |
| solomons-sword | `# v0.1.2` | `uv.lock` | pytest, build | yes | `pyproject.toml` 0.1.2 | satisfied |
| the-wip-tally | absent | `uv.lock` | pytest, build | yes | `wiptally/__init__.py` 0.1.0 (python-literal) | fail-closed: no `RELEASE_NOTES.md` |

"Satisfied" means the file-level prerequisites exist at the imported snapshot; a release
still needs the pinned Release Policy commit to be reachable on GitHub, a bumped version,
an annotated tag on `main`, and the component's environment and trusted publisher. A
fail-closed component cannot release until a later reviewed component change adds the
missing file or files; its caller refuses the tag until then and nothing else changes.

The payday-super-checker bespoke experimental prerelease workflow
(`packages/payday-super-checker/.github/workflows/release.yml`) and every other workflow
under a component directory remain nested and inert: GitHub runs workflows only from the
root `.github/workflows/`, and nothing references the nested files.

the-exchequer-tally, solomons-sword and the-wip-tally have no PyPI project today. Their
root callers are complete, but their `pypi-<component>` environments and trusted
publishers do not exist until the owner creates them, so their `pypi` jobs cannot succeed
before that separate step.
