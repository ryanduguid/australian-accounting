# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""The root first screen and llms.txt must agree with the MCP application they describe."""

import re
import shutil
import subprocess
import tomllib
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
PROJECT = tomllib.loads((ROOT / "apps/aus-accounting-mcp/pyproject.toml").read_text(encoding="utf-8"))["project"]


def _text(name: str) -> str:
    return (ROOT / name).read_text(encoding="utf-8")


def test_root_readme_install_command_matches_the_application_readme() -> None:
    command = re.compile(
        r"claude mcp add --scope user aus-accounting -- "
        r"uvx --from 'aus-accounting-mcp==[0-9.]+' aus-accounting-mcp"
    )
    # The application README's pin moves to a release only once it is published.
    assert command.findall(_text("README.md")) == command.findall(
        _text("apps/aus-accounting-mcp/README.md")
    )
    assert command.search(_text("README.md"))


def test_root_docs_state_the_python_floor_the_package_requires() -> None:
    floor = re.fullmatch(r">=(\d+\.\d+)", PROJECT["requires-python"])
    assert floor is not None
    assert f"Python {floor.group(1)}+" in _text("README.md")
    assert f"Python {floor.group(1)} or later" in _text("llms.txt")


def test_root_docs_do_not_call_the_rust_engines_python() -> None:
    # lodgment-calendar keeps its Python source beside its Rust crate until MCP
    # cutover, so it counts as the Python engine until then. No other package
    # may have both manifests.
    packages = list((ROOT / "packages").iterdir())
    both = {p.name for p in packages if (p / "Cargo.toml").is_file() and (p / "pyproject.toml").is_file()}
    assert both == {"lodgment-calendar"}
    rust_engines = sum(
        (path / "Cargo.toml").is_file() and path.name not in {"au-tax-rates-data", "lodgment-calendar"}
        for path in packages
    )
    python_engines = sum(
        (path / "pyproject.toml").is_file() for path in (ROOT / "packages").iterdir()
    )
    assert (rust_engines, python_engines) == (8, 1)
    claim = "Nine engines (8 in Rust, 1 in Python)"
    for name in ("README.md", "llms.txt"):
        text = " ".join(_text(name).replace("> ", "").split())
        assert claim in text
        assert (
            "published Python releases of 5 of them and, from 0.10.0, the "
            "`australian-tax-calculators` executable"
        ) in text
        assert (
            "native build in this repository links the Rust crates of all 6 instead "
            "and is not released yet"
        ) in text


def test_just_test_is_the_sum_of_parts_a_runner_can_queue_apart() -> None:
    # A runner that caps each job's time cannot finish the whole of `just test`,
    # so its parts stay separate recipes, and the Rust part takes crate directories.
    justfile = _text("justfile")
    assert re.search(r"^test: setup rust-test python-test\n\n", justfile, flags=re.MULTILINE)
    assert re.search(r"^python-test: setup\n", justfile, flags=re.MULTILINE)
    assert re.search(r"^rust-test \*crates:\n", justfile, flags=re.MULTILINE)
    assert "just python-test" in _text("CONTRIBUTING.md")


@pytest.mark.skipif(shutil.which("just") is None, reason="needs just")
def test_rust_test_refuses_a_directory_that_is_not_a_crate() -> None:
    result = subprocess.run(
        ["just", "rust-test", "tests"], cwd=ROOT, capture_output=True, text=True, check=False)
    assert result.returncode != 0
    assert "no Cargo.toml in tests" in result.stderr
