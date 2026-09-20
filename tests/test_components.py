"""Run every component's existing suite from its own working directory."""

import subprocess
import sys

import pytest

from test_boundaries import APPLICATIONS, ENGINES, ROOT


@pytest.mark.parametrize("component", sorted(ENGINES | APPLICATIONS))
def test_component_suite(component: str) -> None:
    result = subprocess.run(
        [sys.executable, "-m", "pytest"],
        cwd=ROOT / component,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr
