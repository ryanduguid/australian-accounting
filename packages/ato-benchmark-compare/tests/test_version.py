"""One version string, so a citation trail cannot understate the engine that ran."""

from __future__ import annotations

import importlib.metadata
import subprocess
import sys
from pathlib import Path

import atobenchmark
import pytest


def test_module_version_matches_distribution_metadata() -> None:
    # Keep the runtime copy aligned with the authoritative project version.
    assert atobenchmark.__version__ == importlib.metadata.version("ato-benchmark-compare")
    root = Path(__file__).resolve().parents[1]
    pyproject = (root / "pyproject.toml").read_text(encoding="utf-8")
    assert f'version = "{atobenchmark.__version__}"' in pyproject


@pytest.mark.parametrize("installed_version", ["0.0.1", None])
def test_checkout_version_ignores_unrelated_metadata(installed_version: str | None) -> None:
    code = f"""
from importlib.metadata import PackageNotFoundError
from unittest.mock import patch

with patch("importlib.metadata.version", return_value={installed_version!r},
           side_effect=PackageNotFoundError if {installed_version!r} is None else None):
    import atobenchmark
    print(atobenchmark.__version__)
"""
    result = subprocess.run(
        [sys.executable, "-c", code],
        cwd=Path(__file__).resolve().parents[1],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == atobenchmark.__version__
