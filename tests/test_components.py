# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Run each component's existing suite from its own working directory.

ci.yml's rust job runs the same cargo test for the crates rust_crates.py lists
for it, on Linux, Windows and macOS, so this file leaves those crates out.
"""

import runpy
import subprocess
import sys

import pytest

from test_boundaries import APPLICATIONS, DATASETS, ENGINES, ROOT, RUST_APPLICATIONS, RUST_ENGINES

CI_YML_CRATES = {
    f"{crate['root']}/{crate['package']}"
    for crate in runpy.run_path(str(ROOT / ".github/ci/rust_crates.py"))["crates"](ci_yml=True)
}


@pytest.mark.parametrize(
    "component",
    sorted((ENGINES | APPLICATIONS | RUST_APPLICATIONS | DATASETS).keys() - CI_YML_CRATES),
)
def test_component_suite(component: str) -> None:
    rust = component in RUST_ENGINES or component in RUST_APPLICATIONS or component in DATASETS
    command = ["cargo", "test", "--locked"] if rust else [sys.executable, "-m", "pytest"]
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
