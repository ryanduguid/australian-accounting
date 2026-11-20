# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Build the pinned calculators executable when the run supplies none.

The release gate runs plain `pytest` without one, and test modules call the
calculators whilst they are collected, so this runs in pytest_configure, before
collection. An executable in AUS_TAX_CALCULATORS_BIN or on PATH is used as it is;
otherwise the built one's folder goes first on PATH. Either is named by its SHA-256
in AUS_TAX_CALCULATORS_SHA256, as an operator names a source build, unless the run
names one.
"""

import hashlib
import json
import os
import subprocess
from pathlib import Path

import pytest

from aus_accounting_mcp.adapters import tax

PACKAGE = Path(__file__).resolve().parents[3] / "packages" / tax.CALCULATORS
SUPPLY = (
    f"Install Rust (https://rustup.rs) so the tests can build {PACKAGE}, or set "
    f"{tax.BINARY_VARIABLE} to the absolute path of the {tax.CALCULATORS} "
    f"{tax.CALCULATORS_VERSION} executable."
)


def pytest_configure() -> None:
    supplied = os.environ.get(tax.BINARY_VARIABLE) or tax._executable()
    binary = supplied or _build()
    # A missing or relative supplied path is left for the tests that refuse it.
    if os.path.isabs(binary) and os.path.isfile(binary):
        with open(binary, "rb") as file:
            os.environ.setdefault(tax.DIGEST_VARIABLE,
                                  hashlib.file_digest(file, "sha256").hexdigest())
    if supplied:
        return
    try:
        with tax._binary():
            pass
    except tax.CalculatorsUnavailable as exc:
        raise pytest.UsageError(
            f"The {tax.CALCULATORS} built from {PACKAGE} is not the release this server "
            f"pins: {exc}") from exc


def _build() -> str:
    """Build the package, put the executable's folder first on PATH and return its path."""
    command = ["cargo", "build", "--release", "--locked",
               "--message-format=json-render-diagnostics"]
    try:
        # cwd selects the package's rust-toolchain.toml; cargo's diagnostics go to stderr.
        build = subprocess.run(command, cwd=PACKAGE, stdout=subprocess.PIPE, text=True,
                               check=False)
    except OSError as exc:
        raise pytest.UsageError(
            f"No {tax.CALCULATORS} executable was supplied and cargo could not run in "
            f"{PACKAGE}: {exc}. {SUPPLY}") from exc
    if build.returncode != 0:
        raise pytest.UsageError(
            f"No {tax.CALCULATORS} executable was supplied and `cargo build --release "
            f"--locked` in {PACKAGE} exited {build.returncode}; its errors are above. {SUPPLY}")
    # The artefact message names the executable wherever the target directory is.
    [binary] = [message["executable"] for message in map(json.loads, build.stdout.splitlines())
                if message.get("reason") == "compiler-artifact"
                and message["target"]["name"] == tax.CALCULATORS and message["executable"]]
    # PATH, as CI gives it, because test_mcp_evaluation's server inherits PATH but not
    # AUS_TAX_CALCULATORS_BIN.
    os.environ["PATH"] = os.pathsep.join([os.path.dirname(binary), os.environ["PATH"]])
    return binary
