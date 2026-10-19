"""Run every component's existing suite from its own working directory."""

import subprocess
import sys

import pytest

from test_boundaries import APPLICATIONS, DATASETS, ENGINES, ROOT, RUST_ENGINES


@pytest.mark.parametrize("component", sorted(ENGINES | APPLICATIONS | DATASETS))
def test_component_suite(component: str) -> None:
    command = ["cargo", "test", "--locked"] if component in RUST_ENGINES else [sys.executable, "-m", "pytest"]
    result = subprocess.run(
        command,
        cwd=ROOT / component,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr
