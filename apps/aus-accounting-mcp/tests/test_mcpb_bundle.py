# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""The MCP Bundle manifest must describe the server this source tree releases."""

import asyncio
import json
import tomllib
from pathlib import Path

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
    assert sorted(env) == sorted(name for name, _, _ in doctor.ROOTS)
    settings = manifest["user_config"]
    assert sorted(env.values()) == sorted(f"${{user_config.{key}}}" for key in settings)
    for option in settings.values():
        # An empty default reaches the server as an unset folder, not a literal placeholder.
        assert option["type"] == "directory"
        assert option["default"] == ""
        assert not option.get("required", False)
