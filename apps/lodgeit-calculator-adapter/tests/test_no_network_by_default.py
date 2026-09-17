"""The property the whole package rests on: nothing leaves unless asked.

Sockets are blocked for the duration of each test here, so a request that
should not happen fails loudly rather than reaching a service.
"""

from __future__ import annotations

import socket
import subprocess
import sys
import textwrap
from pathlib import Path

import pytest

from lodgeitadapter import AdapterConfig, LodgeitClient, Status, load_contract
from lodgeitadapter.config import from_environment
from lodgeitadapter.errors import DisallowedTargetError, NotEnabledError


@pytest.fixture
def no_sockets(monkeypatch):
    def refuse(*args, **kwargs):  # noqa: ANN002, ANN003
        raise AssertionError("a socket was opened when none should have been")

    monkeypatch.setattr(socket, "create_connection", refuse)
    monkeypatch.setattr(socket.socket, "connect", refuse)
    return refuse


MODULES = (
    "lodgeitadapter", "lodgeitadapter.client", "lodgeitadapter.config",
    "lodgeitadapter.contract", "lodgeitadapter.decimals", "lodgeitadapter.evidence",
    "lodgeitadapter.transport", "lodgeitadapter.cli", "lodgeitadapter.trials",
    "lodgeitadapter.trials.div7a", "lodgeitadapter.trials.fbt",
    "lodgeitadapter.trials.depreciation", "lodgeitadapter.trials.fano",
)


def test_importing_every_module_opens_no_socket():
    """A fresh interpreter, sockets sabotaged, every module imported.

    In a subprocess rather than in this one: reloading modules here would
    rebind the exception and enum classes other tests compare against, and a
    test that breaks its neighbours is not evidence of anything.
    """
    program = textwrap.dedent(
        f"""
        import socket, sys
        def refuse(*args, **kwargs):
            raise SystemExit("a socket was opened during import")
        socket.socket.connect = refuse
        socket.create_connection = refuse
        for name in {MODULES!r}:
            __import__(name)
        print("clean")
        """
    )
    root = Path(__file__).resolve().parents[1]
    completed = subprocess.run(
        [sys.executable, "-c", program], cwd=root, capture_output=True, text=True, timeout=120,
    )
    assert completed.returncode == 0, completed.stderr
    assert "clean" in completed.stdout


def test_the_default_configuration_is_off(no_sockets):
    config = AdapterConfig()
    assert config.enabled is False
    assert config.base_url is None
    with pytest.raises(NotEnabledError):
        config.require_enabled()


def test_an_environment_without_the_switch_is_still_off(no_sockets):
    config = from_environment({"LODGEIT_ADAPTER_BASE_URL": "https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app"})
    assert config.enabled is False


def test_discovery_offline_refuses_to_send_rather_than_answering(no_sockets, contract):
    client = LodgeitClient(AdapterConfig(), contract)
    outcome = client.discover()
    assert outcome.status is Status.REFUSED_TO_SEND
    assert outcome.result is None
    assert any("not enabled" in finding for finding in outcome.findings)


def test_invocation_offline_refuses_and_produces_no_figure(no_sockets, contract):
    client = LodgeitClient(AdapterConfig(), contract)
    outcome = client.invoke(
        "urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026", {"amalgamated_base": 1},
    )
    assert outcome.status is Status.REFUSED_TO_SEND
    assert outcome.result is None and outcome.values == {}


def test_the_reviewed_snapshot_is_readable_offline(no_sockets):
    contract = load_contract("lodgeit-calculators")
    assert contract.snapshot_id == "lodgeit-calculators-2026-09-18"
    assert "urn:sbrm:calculator:div7a:at" in contract.calculators


def test_enabled_without_a_base_url_still_sends_nothing(no_sockets, contract):
    client = LodgeitClient(AdapterConfig(enabled=True), contract)
    outcome = client.discover()
    assert outcome.status is Status.REFUSED_TO_SEND


def test_a_host_off_the_allowlist_is_refused_before_a_socket_opens(no_sockets):
    config = AdapterConfig(
        enabled=True,
        allowed_hosts=("fbt-calculator-api-qkp3j5bjnq-ts.a.run.app",
    ))
    for url in (
        "https://evil.example/v1/calculators",
        "http://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/v1/calculators",
        "https://169.254.169.254/v1/calculators",
        "https://user:pass@fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/v1/calculators",
        "file:///etc/passwd",
        "https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/../../admin",
        "http://127.0.0.1:9000/v1/calculators",
    ):
        with pytest.raises(DisallowedTargetError):
            config.check_url(url)


def test_a_route_the_adapter_does_not_call_is_refused(no_sockets):
    config = AdapterConfig(enabled=True)
    with pytest.raises(DisallowedTargetError):
        config.check_url("https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/admin/reset")
    # A schema URL lifted out of a response body is the case this closes.
    with pytest.raises(DisallowedTargetError):
        config.check_url("https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/schemas/anything.json")


def test_loopback_needs_its_own_switch(no_sockets):
    off = AdapterConfig(enabled=True)
    with pytest.raises(DisallowedTargetError):
        off.check_url("http://127.0.0.1:8000/v1/calculators")
    on = AdapterConfig(enabled=True, allow_loopback=True)
    assert on.check_url("http://127.0.0.1:8000/v1/calculators")
