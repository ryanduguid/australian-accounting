# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""The MCP Bundle manifest must describe the server this source tree releases."""

import asyncio
import importlib.util
import json
import tomllib
from pathlib import Path
from types import SimpleNamespace

import pytest

from aus_accounting_mcp import doctor
from aus_accounting_mcp.server import mcp

APP = Path(__file__).resolve().parents[1]
BUNDLE = APP / "mcpb"


def test_bundle_pins_the_release_version_licence_and_python() -> None:
    project = tomllib.loads((APP / "pyproject.toml").read_text(encoding="utf-8"))["project"]
    bundle = tomllib.loads((BUNDLE / "pyproject.toml").read_text(encoding="utf-8"))["project"]
    manifest = json.loads((BUNDLE / "manifest.json").read_text(encoding="utf-8"))

    assert manifest["name"] == project["name"]
    assert manifest["version"] == bundle["version"] == project["version"]
    assert bundle["dependencies"] == [f"{project['name']}=={project['version']}"]
    assert bundle["requires-python"] == project["requires-python"]
    assert manifest["compatibility"]["runtimes"]["python"] == project["requires-python"]
    assert manifest["license"] == project["license"]


def test_bundle_declares_exactly_the_registered_tools() -> None:
    manifest = json.loads((BUNDLE / "manifest.json").read_text(encoding="utf-8"))
    declared = [tool["name"] for tool in manifest["tools"]]
    registered = [tool.name for tool in asyncio.run(mcp.list_tools())]
    assert sorted(declared) == sorted(registered)


def test_every_retrieval_folder_is_an_optional_directory_setting() -> None:
    manifest = json.loads((BUNDLE / "manifest.json").read_text(encoding="utf-8"))
    server = manifest["server"]
    env = server["mcp_config"]["env"]

    assert server["type"] == "uv"
    assert server["entry_point"] in server["mcp_config"]["args"]
    assert (BUNDLE / server["entry_point"]).is_file()
    assert env == {
        "AUS_ACCOUNTING_LIBRARY_ROOT": "${user_config.library_root}",
        "AUS_ACCOUNTING_CORPUS_ROOT": "${user_config.corpus_root}",
        "AUS_ACCOUNTING_RULINGS_ROOT": "${user_config.rulings_root}",
    }
    assert sorted(env) == sorted(name for name, _, _ in doctor.ROOTS)
    settings = manifest["user_config"]
    assert sorted(settings) == ["corpus_root", "library_root", "rulings_root"]
    for option in settings.values():
        assert option["type"] == "directory"
        assert option["default"] == ""
        assert not option.get("required", False)


@pytest.mark.parametrize(("platform", "machine", "filename"), [
    ("linux", "x86_64", "australian-tax-calculators"),
    ("win32", "AMD64", "australian-tax-calculators.exe"),
    ("darwin", "x86_64", None),
    ("linux", "aarch64", None),
])
def test_the_launcher_selects_a_supported_binary_and_preserves_an_override(
    monkeypatch, tmp_path, platform, machine, filename,
) -> None:
    spec = importlib.util.spec_from_file_location("bundle_launcher", BUNDLE / "src/server.py")
    assert spec is not None and spec.loader is not None
    launcher = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(launcher)
    monkeypatch.setattr(launcher, "sys", SimpleNamespace(platform=platform))
    monkeypatch.setattr(launcher, "platform", SimpleNamespace(machine=lambda: machine))
    (tmp_path / "src").mkdir()
    monkeypatch.setattr(launcher, "__file__", str(tmp_path / "src/server.py"))
    variable = "AUS_TAX_CALCULATORS_BIN"
    environment: dict[str, str] = {}
    monkeypatch.setattr(launcher, "os", SimpleNamespace(environ=environment))

    launcher.configure_calculators()
    assert variable not in environment
    if filename is None:
        return

    binary = tmp_path / "bin" / filename
    binary.parent.mkdir()
    binary.touch()
    launcher.configure_calculators()
    assert environment.get(variable) == str(binary)
    operator_calculator = str(tmp_path / "operator-calculator")
    environment[variable] = operator_calculator
    launcher.configure_calculators()
    assert environment.get(variable) == operator_calculator
