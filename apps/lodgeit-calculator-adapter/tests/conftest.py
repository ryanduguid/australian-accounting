"""Shared fixtures: a loopback stub, and the contract snapshots on disk.

Every test in this suite runs offline. The only socket any of them opens is to
a `http.server` on 127.0.0.1 started inside the test, which is what
`allow_loopback` exists for. No test contacts a LodgeiT service, and
`test_no_network_by_default.py` proves the package cannot without being asked.
"""

from __future__ import annotations

import json
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer

import pytest

from lodgeitadapter.config import AdapterConfig
from lodgeitadapter.contract import load as load_contract


@pytest.fixture(scope="session")
def contract():
    return load_contract("lodgeit-calculators")


@pytest.fixture(scope="session")
def fano_contract():
    return load_contract("fano-classifier")


class StubState:
    """What the stub should do next, and what it saw."""

    def __init__(self) -> None:
        self.status = 200
        self.body: bytes = b"{}"
        self.headers: dict[str, str] = {"Content-Type": "application/json"}
        self.requests: list[dict] = []
        self.delay = 0.0

    def respond(self, payload, status: int = 200, *, raw: bytes | None = None) -> None:
        self.status = status
        self.body = raw if raw is not None else json.dumps(payload).encode("utf-8")


@pytest.fixture
def stub():
    """A loopback HTTP stub. Returns (base_url, state)."""
    state = StubState()

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args) -> None:  # noqa: ANN002 - silence the default stderr log
            return

        def _serve(self) -> None:
            length = int(self.headers.get("Content-Length") or 0)
            body = self.rfile.read(length) if length else b""
            state.requests.append({
                "method": self.command,
                "path": self.path,
                "headers": dict(self.headers),
                "body": body,
            })
            if state.delay:
                import time

                time.sleep(state.delay)
            self.send_response(state.status)
            for key, value in state.headers.items():
                self.send_header(key, value)
            self.send_header("Content-Length", str(len(state.body)))
            self.end_headers()
            self.wfile.write(state.body)

        do_GET = _serve
        do_POST = _serve

    server = HTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_address[1]}", state
    finally:
        server.shutdown()
        server.server_close()


@pytest.fixture
def stub_config(stub):
    base_url, _ = stub
    return AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1)
