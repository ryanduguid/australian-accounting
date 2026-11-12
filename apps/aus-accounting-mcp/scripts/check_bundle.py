# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Check a packed MCP Bundle against its source, then start it as a host would.

usage: python -I scripts/check_bundle.py BUNDLE.mcpb SOURCE_DIR

The archive must hold exactly the files of the clean SOURCE_DIR, byte for byte,
apart from ``.mcpbignore``, which the packer reads and leaves out. The check
then unpacks it into a temporary folder and runs ``server.mcp_config`` with
``${__dirname}`` set to that folder and each ``${user_config.KEY}`` set to the
manifest's default, as an install with no folders chosen would. Over stdio it
sends an MCP ``initialize`` and ``tools/list``: the server must report the
manifest's version and register exactly the tools the manifest declares.
Exit 0 prints what it checked; any mismatch or silence exits 1.
"""

from __future__ import annotations

import json
import os
import queue
import re
import subprocess
import sys
import tempfile
import threading
import zipfile
from pathlib import Path
from typing import IO, Any

PROTOCOL_VERSION = "2025-06-18"
# The first start installs the release from PyPI, so allow it minutes, not seconds.
READ_TIMEOUT = 300


def _files(folder: Path) -> dict[str, bytes]:
    return {
        path.relative_to(folder).as_posix(): path.read_bytes()
        for path in folder.rglob("*")
        if path.is_file()
    }


def _read(stdout: IO[str], replies: queue.Queue[str]) -> None:
    for line in stdout:
        replies.put(line)
    replies.put("")


def _request(stdin: IO[str], replies: queue.Queue[str], message: dict[str, Any]) -> Any:
    stdin.write(json.dumps(message) + "\n")
    stdin.flush()
    if "id" not in message:
        return {}
    while True:
        try:
            line = replies.get(timeout=READ_TIMEOUT)
        except queue.Empty:
            raise SystemExit(
                f"FAIL no answer to {message['method']} within {READ_TIMEOUT} s"
            ) from None
        if not line:
            raise SystemExit(f"FAIL the server exited before answering {message['method']}")
        reply = json.loads(line)
        if reply.get("id") == message["id"]:
            if "error" in reply:
                raise SystemExit(f"FAIL {message['method']}: {reply['error']}")
            return reply["result"]


def check(bundle: Path, source: Path) -> list[str]:
    with tempfile.TemporaryDirectory() as name:
        folder = Path(name)
        with zipfile.ZipFile(bundle) as archive:
            archive.extractall(folder)
        packed = _files(folder)
        expected = {path: data for path, data in _files(source).items() if path != ".mcpbignore"}
        if packed != expected:
            differ = sorted(
                path for path in packed.keys() | expected.keys()
                if packed.get(path) != expected.get(path)
            )
            raise SystemExit(f"FAIL {bundle.name} differs from {source} in {differ}")

        manifest = json.loads(packed["manifest.json"])
        defaults = {
            key: str(option.get("default", ""))
            for key, option in manifest.get("user_config", {}).items()
        }

        def resolve(value: str) -> str:
            value = value.replace("${__dirname}", name)
            return re.sub(r"\$\{user_config\.([^}]+)\}", lambda m: defaults[m.group(1)], value)

        config = manifest["server"]["mcp_config"]
        command = [config["command"], *(resolve(arg) for arg in config.get("args", []))]
        environment = {
            **os.environ,
            **{key: resolve(value) for key, value in config.get("env", {}).items()},
        }
        with subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                              text=True, encoding="utf-8", env=environment) as process:
            assert process.stdin is not None and process.stdout is not None
            replies: queue.Queue[str] = queue.Queue()
            threading.Thread(target=_read, args=(process.stdout, replies), daemon=True).start()
            try:
                started = _request(process.stdin, replies, {
                    "jsonrpc": "2.0", "id": 1, "method": "initialize",
                    "params": {"protocolVersion": PROTOCOL_VERSION, "capabilities": {},
                               "clientInfo": {"name": "check_bundle", "version": "1"}},
                })
                _request(process.stdin, replies,
                         {"jsonrpc": "2.0", "method": "notifications/initialized"})
                listed = _request(process.stdin, replies,
                                  {"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
                process.stdin.close()
                process.wait(timeout=30)
            finally:
                if process.poll() is None:
                    process.kill()

    version = started["serverInfo"]["version"]
    if version != manifest["version"]:
        raise SystemExit(f"FAIL the server reports {version}, the manifest {manifest['version']}")
    registered = sorted(tool["name"] for tool in listed["tools"])
    declared = sorted(tool["name"] for tool in manifest["tools"])
    if registered != declared:
        raise SystemExit(
            f"FAIL tools differ: registered only {sorted(set(registered) - set(declared))}, "
            f"declared only {sorted(set(declared) - set(registered))}"
        )
    return [
        f"{bundle.name} holds exactly the {len(expected)} files of {source}",
        f"it started with {' '.join(command[:2])} ... and reports version {version}",
        f"tools/list returned the manifest's {len(declared)} tools",
    ]


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    for line in check(Path(sys.argv[1]), Path(sys.argv[2])):
        print(line)
