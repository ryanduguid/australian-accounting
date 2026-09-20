"""One version string, so a citation trail cannot understate the engine that ran."""

from __future__ import annotations

import importlib.metadata
from pathlib import Path

import atobenchmark


def test_module_version_matches_distribution_metadata() -> None:
    # The project version supplies installed metadata and the runtime value.
    assert atobenchmark.__version__ == importlib.metadata.version("ato-benchmark-compare")
    root = Path(__file__).resolve().parents[1]
    pyproject = (root / "pyproject.toml").read_text(encoding="utf-8")
    assert f'version = "{atobenchmark.__version__}"' in pyproject
