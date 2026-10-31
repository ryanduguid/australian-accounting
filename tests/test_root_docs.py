"""The root first screen and llms.txt must agree with the MCP application they describe."""

import re
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROJECT = tomllib.loads((ROOT / "apps/aus-accounting-mcp/pyproject.toml").read_text(encoding="utf-8"))["project"]


def _text(name: str) -> str:
    return (ROOT / name).read_text(encoding="utf-8")


def test_root_readme_install_command_names_the_current_release() -> None:
    command = (
        "claude mcp add --scope user aus-accounting -- "
        f"uvx --from 'aus-accounting-mcp=={PROJECT['version']}' aus-accounting-mcp"
    )
    assert command in _text("README.md")


def test_root_docs_state_the_python_floor_the_package_requires() -> None:
    floor = re.fullmatch(r">=(\d+\.\d+)", PROJECT["requires-python"])
    assert floor is not None
    assert f"Python {floor.group(1)}+" in _text("README.md")
    assert f"Python {floor.group(1)} or later" in _text("llms.txt")


def test_root_docs_do_not_call_the_rust_engines_python() -> None:
    rust_engines = sum(
        (path / "Cargo.toml").is_file() and path.name != "au-tax-rates-data"
        for path in (ROOT / "packages").iterdir()
    )
    python_engines = sum(
        (path / "pyproject.toml").is_file() for path in (ROOT / "packages").iterdir()
    )
    assert (rust_engines, python_engines) == (8, 1)
    claim = "Nine engines (8 in Rust, 1 in Python)"
    assert claim in _text("README.md")
    assert claim in _text("llms.txt")
