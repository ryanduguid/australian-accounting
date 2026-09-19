"""Regressions for the shared command-line options and the transport boundary.

Everything here is offline: the transport tests replace the opener with a fake
that raises the HTTP error itself, and no socket is opened.
"""

from __future__ import annotations

import email.message
import urllib.error
import urllib.request
from pathlib import Path

import pytest

from lodgeitadapter import transport
from lodgeitadapter.cli import build_parser
from lodgeitadapter.config import AdapterConfig
from lodgeitadapter.contract import compare
from lodgeitadapter.errors import TransportError

SHARED = ["--contract", "custom", "--enable-network", "--base-url", "http://127.0.0.1:9",
          "--allow-loopback", "--evidence-out", "out.json", "--not-synthetic"]
INVOKE = ["invoke", "--calculator", "div7a", "--period", "fy2026", "--body", "body.json"]


def _assert_shared(args) -> None:
    assert args.contract == "custom"
    assert args.enable_network is True
    assert args.base_url == "http://127.0.0.1:9"
    assert args.allow_loopback is True
    assert args.evidence_out == Path("out.json")
    assert args.not_synthetic is True
    assert args.command == "invoke" and args.calculator == "div7a"


@pytest.mark.parametrize("command", ["contract", "discover", "drift", INVOKE])
def test_shared_options_before_the_command_word_survive_the_subcommand(command) -> None:
    """`--not-synthetic invoke ...` used to record synthetic=True."""
    words = command if isinstance(command, list) else [command]
    args = build_parser().parse_args([*SHARED, *words])
    assert args.not_synthetic is True
    assert args.evidence_out == Path("out.json")
    assert args.enable_network is True
    assert args.contract == "custom"


def test_shared_options_after_the_command_word_still_work() -> None:
    _assert_shared(build_parser().parse_args([*INVOKE, *SHARED]))


def test_shared_options_split_around_the_command_word_are_all_kept() -> None:
    before, after = SHARED[:5], SHARED[5:]
    _assert_shared(build_parser().parse_args([*before, *INVOKE, *after]))


def test_absent_shared_options_keep_their_defaults() -> None:
    args = build_parser().parse_args(INVOKE)
    assert args.contract == "lodgeit-calculators"
    assert args.enable_network is False
    assert args.base_url is None
    assert args.allow_loopback is False
    assert args.evidence_out is None
    assert args.not_synthetic is False


def test_an_option_typed_in_both_positions_takes_the_later_value() -> None:
    args = build_parser().parse_args(["--contract", "first", *INVOKE, "--contract", "second"])
    assert args.contract == "second"


def _live_from(contract) -> list[dict]:
    return [
        {"calc_uri": uri, "supported_periods": list(spec.get("supported_periods", [])),
         "input_schema_ref": spec.get("input_schema_ref")}
        for uri, spec in contract.calculators.items()
    ]


def test_a_matching_live_listing_still_agrees(contract) -> None:
    assert compare(contract, _live_from(contract)) == []


@pytest.mark.parametrize("conflict_first", [True, False])
def test_a_repeated_calc_uri_is_drift_in_either_order(contract, conflict_first) -> None:
    """A conflicting duplicate ahead of a matching one used to print agreement."""
    live = _live_from(contract)
    uri = live[0]["calc_uri"]
    conflicting = {**live[0], "supported_periods": ["urn:fabricated:period"]}
    duplicate = [conflicting, live[0]] if conflict_first else [live[0], conflicting]
    findings = compare(contract, [*duplicate, *live[1:]])
    assert any(f"live lists {uri} more than once" in finding for finding in findings)
    identical = compare(contract, [live[0], live[0], *live[1:]])
    assert identical != [], "an identical repeat is still drift"
    assert not any(f"snapshot {contract.snapshot_id} records {uri}, which live no longer lists"
                   in finding for finding in identical)


@pytest.mark.parametrize("current_ref", [None, "", 7])
def test_a_lost_schema_reference_is_drift(contract, current_ref) -> None:
    live = _live_from(contract)
    with_ref = next(entry for entry in live if entry["input_schema_ref"])
    with_ref["input_schema_ref"] = current_ref
    findings = compare(contract, live)
    assert any("no longer states input schema ref" in finding for finding in findings)


def test_the_opener_ignores_environment_proxies(monkeypatch) -> None:
    """An https_proxy variable used to install a route the allowlist never saw.

    build_opener adds an environment-reading ProxyHandler unless one is
    supplied. The empty one the transport supplies registers no scheme, so an
    opener that ignores proxies is one with no ProxyHandler in its chain at
    all; the defective opener carried one whose proxies named this variable.
    """
    monkeypatch.setenv("https_proxy", "http://127.0.0.1:9")
    monkeypatch.setenv("HTTPS_PROXY", "http://127.0.0.1:9")
    handlers = transport._opener().handlers
    assert not any(isinstance(h, urllib.request.ProxyHandler) for h in handlers)
    default = urllib.request.build_opener().handlers
    assert any(isinstance(h, urllib.request.ProxyHandler) and h.proxies for h in default), (
        "the control: a default opener in this environment does pick the variable up"
    )


class _TimingOutBody:
    def read(self, _size: int = -1) -> bytes:
        raise TimeoutError("timed out")

    def close(self) -> None:
        pass


class _FakeOpener:
    def __init__(self, status: int) -> None:
        self.status = status
        self.opens = 0

    def open(self, request_object, timeout):  # noqa: ANN001, ANN201
        self.opens += 1
        raise urllib.error.HTTPError(
            request_object.full_url, self.status, "fabricated", email.message.Message(),
            _TimingOutBody(),
        )


def _request(monkeypatch, status: int, attempts: int) -> _FakeOpener:
    opener = _FakeOpener(status)
    monkeypatch.setattr(transport, "_opener", lambda: opener)
    config = AdapterConfig(enabled=True, base_url="http://127.0.0.1:9", allow_loopback=True,
                           max_attempts=attempts, retry_backoff_seconds=0.01)
    with pytest.raises(TransportError, match=f"HTTP {status} body could not be read"):
        transport.request(config, "GET", "http://127.0.0.1:9/v1/calculators",
                          retry_safe=True, sleep=lambda _seconds: None)
    return opener


def test_a_timed_out_5xx_body_is_retried_then_normalised(monkeypatch) -> None:
    """A raw TimeoutError used to escape after one attempt despite max_attempts=2."""
    assert _request(monkeypatch, 503, attempts=2).opens == 2


def test_a_timed_out_4xx_body_is_not_retried(monkeypatch) -> None:
    assert _request(monkeypatch, 400, attempts=3).opens == 1
