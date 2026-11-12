# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""The bundle check: what it must refuse before a bundle reaches a release."""

from __future__ import annotations

import importlib.util
import json
import sys
import zipfile
from pathlib import Path

import pytest

APP = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("check_bundle", APP / "scripts" / "check_bundle.py")
assert SPEC is not None and SPEC.loader is not None
check_bundle = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check_bundle)

# A stand-in server: answers initialize with FAKE_VERSION and lists one tool.
FAKE_SERVER = """\
import json, os, sys
for line in sys.stdin:
    message = json.loads(line)
    if message.get("method") == "initialize":
        result = {"serverInfo": {"name": "fake", "version": os.environ["FAKE_VERSION"]}}
    elif message.get("method") == "tools/list":
        result = {"tools": [{"name": "get_data_freshness"}]}
    else:
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": message["id"], "result": result}), flush=True)
"""


def bundle(tmp_path: Path, server_version: str) -> tuple[Path, Path]:
    source = tmp_path / "source"
    source.mkdir()
    manifest = {
        "version": "1.0.0",
        "server": {"mcp_config": {
            "command": sys.executable,
            "args": ["${__dirname}/fake.py"],
            "env": {"FAKE_VERSION": server_version},
        }},
        "tools": [{"name": "get_data_freshness"}],
    }
    (source / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    (source / "fake.py").write_text(FAKE_SERVER, encoding="utf-8")
    (source / ".mcpbignore").write_text(".venv/\n", encoding="utf-8")
    packed = tmp_path / "fake.mcpb"
    with zipfile.ZipFile(packed, "w") as archive:
        for name in ("manifest.json", "fake.py"):
            archive.write(source / name, name)
    return packed, source


def test_a_bundle_matching_its_source_and_manifest_passes(tmp_path: Path) -> None:
    lines = check_bundle.check(*bundle(tmp_path, "1.0.0"))
    assert lines[0] == f"fake.mcpb holds exactly the 2 files of {tmp_path / 'source'}"
    assert lines[-1] == "tools/list returned the manifest's 1 tools"


def test_a_server_reporting_another_version_is_refused(tmp_path: Path) -> None:
    with pytest.raises(SystemExit, match="reports 9.9.9, the manifest 1.0.0"):
        check_bundle.check(*bundle(tmp_path, "9.9.9"))


def test_an_archive_that_differs_from_its_source_is_refused(tmp_path: Path) -> None:
    packed, source = bundle(tmp_path, "1.0.0")
    (source / "fake.py").write_text(FAKE_SERVER + "# edited\n", encoding="utf-8")
    (source / "extra.txt").write_text("not packed\n", encoding="utf-8")
    with pytest.raises(SystemExit, match=r"differs from .* in \['extra.txt', 'fake.py'\]"):
        check_bundle.check(packed, source)
