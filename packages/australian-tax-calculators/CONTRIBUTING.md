# Contributing

Keep statutory methods in this component. Use independently constructed synthetic
tests and official sources. Preserve scope refusals and source metadata.

Run the engine commands documented in the root CONTRIBUTING.md from this directory:

```sh
uv run --locked --extra dev pytest --cov --cov-branch --cov-report=term-missing --cov-report=xml
uv run --locked --extra dev ruff check .
uv run --locked --extra dev mypy
uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict
uv run --locked --extra dev --python 3.12 python -m build
```

Retain the standalone lockfile as well as the root workspace lock. Release this
component through its namespaced release workflow after the applicable gates pass.
