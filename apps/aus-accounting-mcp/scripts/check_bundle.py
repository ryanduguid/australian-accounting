# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Start a packed MCP Bundle the way its manifest tells a host to, and check it.

usage: python -I scripts/check_bundle.py BUNDLE.mcpb

Unpacks the bundle into a temporary folder, runs ``server.mcp_config`` with
``${__dirname}`` set to that folder and each ``${user_config.KEY}`` set to the
manifest's default, as an install with no folders chosen would, then sends an
MCP ``initialize`` and ``tools/list`` over stdio. The server must report the
manifest's version and register exactly the tools the manifest declares.
Exit 0 prints what it checked; any mismatch exits 1.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path
from typing import IO, Any

PROTOCOL_VERSION = "2025-06-18"


def _request(stdin: IO[str], stdout: IO[str], message: dict[str, Any]) -> dict[str, Any]:
    stdin.write(json.dumps(message) + "\n")
    stdin.flush()
    if "id" not in message:
        return {}
    for line in stdout:
        reply = json.loads(line)
        if reply.get("id") == message["id"]:
            if "error" in reply:
                raise SystemExit(f"FAIL {message['method']}: {reply['error']}")
            return reply["result"]
    raise SystemExit(f"FAIL the server exited before answering {message['method']}")


def check(bundle: Path) -> list[str]:
    with tempfile.TemporaryDirectory() as folder:
        with zipfile.ZipFile(bundle) as archive:
            archive.extractall(folder)
        manifest = json.loads(Path(folder, "manifest.json").read_text(encoding="utf-8"))
        defaults = {key: str(option.get("default", "")) for key, option in
                    manifest.get("user_config", {}).items()}

        def resolve(value: str) -> str:
            value = value.replace("${__dirname}", folder)
            return re.sub(r"\$\{user_config\.([^}]+)\}", lambda m: defaults[m.group(1)], value)

        config = manifest["server"]["mcp_config"]
        command = [config["command"], *(resolve(arg) for arg in config.get("args", []))]
        environment = {**os.environ,
                       **{name: resolve(value) for name, value in config.get("env", {}).items()}}
        with subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                              text=True, encoding="utf-8", env=environment) as process:
            assert process.stdin is not None and process.stdout is not None
            pipes = (process.stdin, process.stdout)
            started = _request(*pipes, {
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": PROTOCOL_VERSION, "capabilities": {},
                           "clientInfo": {"name": "check_bundle", "version": "1"}},
            })
            _request(*pipes, {"jsonrpc": "2.0", "method": "notifications/initialized"})
            listed = _request(*pipes, {"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
            process.stdin.close()
            process.wait(timeout=30)

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
        f"{bundle.name} started with {' '.join(command[:2])} ... and reports version {version}",
        f"tools/list returned the manifest's {len(declared)} tools",
    ]


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    for line in check(Path(sys.argv[1])):
        print(line)
