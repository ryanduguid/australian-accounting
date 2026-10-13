"""Fabricated Core catalogue and transport cases; no tenant or secret is accessed."""

from __future__ import annotations

import http.client
import io
import json
import urllib.error
from dataclasses import asdict, fields, is_dataclass
from email.message import Message
from pathlib import Path
from uuid import UUID

import pytest

from lodgeitadapter import transport
from lodgeitadapter.config import AdapterConfig
from lodgeitadapter.core import (
    CORE_URL,
    CoreCategory,
    CoreClient,
    CoreConfig,
    CoreContinuation,
    CoreError,
)
from lodgeitadapter.errors import ConfigurationError, DisallowedTargetError

ID = "00000000-0000-0000-0000-000000000001"
TOKEN = "fabricated-token-for-offline-test"


def item(**changes):
    return {
        "id": ID,
        "code": "DEMO-090",
        "archived": False,
        "private": True,
        "profile": None,
    } | changes


class Response(io.BytesIO):
    status = 200
    headers = {"Content-Type": "application/json; charset=utf-8"}


class Opener:
    def __init__(self, body, status=200, content_type="application/json"):
        self.body = body
        self.status = status
        self.content_type = content_type
        self.requests = []

    def open(self, request, timeout):
        self.requests.append(request)
        if self.status != 200:
            raise urllib.error.HTTPError(
                CORE_URL, self.status, "fabricated", {}, io.BytesIO(self.body)
            )
        response = Response(self.body)
        response.headers = {"Content-Type": self.content_type}
        return response


def stub(monkeypatch, payload, **options):
    body = payload if isinstance(payload, bytes) else json.dumps(payload).encode("utf-8")
    opener = Opener(body, **options)
    monkeypatch.setattr(transport, "_opener", lambda: opener)
    return opener


def test_default_and_environment_cannot_enable_core(monkeypatch):
    monkeypatch.setenv("LODGEIT_ADAPTER_ENABLED", "1")
    monkeypatch.setenv("LODGEIT_ADAPTER_BASE_URL", "https://evil.example")
    opener = stub(monkeypatch, {"items": []})
    with pytest.raises(CoreError) as error:
        CoreClient().list_clients(bearer_token=TOKEN)
    assert error.value.category is CoreCategory.DISABLED
    assert not opener.requests


def test_exact_request_projection_and_sensitive_representations(monkeypatch):
    token = "https://evil.example/../../admin"
    opener = stub(monkeypatch, {"items": [item()], "nextPageToken": token})
    client = CoreClient(CoreConfig(enabled=True))
    page = client.list_clients(bearer_token=TOKEN, page_size=5)
    assert page.items[0].id == ID and page.items[0].code == "DEMO-090"
    assert page.items[0].private is True
    assert page.continuation.value == token
    assert len(opener.requests) == 1
    request = opener.requests[0]
    assert request.full_url == CORE_URL and request.get_method() == "POST"
    assert request.get_header("Authorization") == "Bearer " + TOKEN
    assert json.loads(request.data) == {
        "archived": False,
        "expansion": None,
        "paging": {"size": 5, "token": None},
    }
    assert TOKEN not in repr(client) + repr(client.config) + repr(page)
    assert ID not in repr(page.items[0]) and token not in str(page.continuation)
    client.list_clients(bearer_token="another-fabricated-token", continuation=page.continuation)
    assert opener.requests[1].full_url == CORE_URL
    assert json.loads(opener.requests[1].data)["paging"]["token"] == token
    assert opener.requests[0].get_header("Authorization") == "Bearer " + TOKEN
    assert opener.requests[1].get_header("Authorization") == "Bearer another-fabricated-token"


@pytest.mark.parametrize(
    "url",
    [
        CORE_URL + "/",
        CORE_URL + "?x=1",
        CORE_URL + "#x",
        CORE_URL.replace("https", "http"),
        CORE_URL.replace("api.lodgeit.com", "api.lodgeit.com:443"),
        CORE_URL.replace("list", "modify-profile"),
        "http://127.0.0.1/core/clients/v1-preview/list",
    ],
)
def test_only_the_exact_read_route_is_admitted(url):
    with pytest.raises(DisallowedTargetError):
        CoreConfig(enabled=True).check_url(url)
    with pytest.raises(DisallowedTargetError):
        AdapterConfig().check_url(CORE_URL)


@pytest.mark.parametrize(
    "token", [None, 1, "", "Bearer token", "a\nb", "a\rb", " a", "a b", "é", "a" * 8193]
)
def test_invalid_tokens_never_open_transport(monkeypatch, token):
    opener = stub(monkeypatch, {"items": []})
    with pytest.raises(CoreError) as error:
        CoreClient(CoreConfig(enabled=True)).list_clients(bearer_token=token)
    assert error.value.category is CoreCategory.INVALID_INPUT
    assert error.value.__context__ is None
    assert not opener.requests


@pytest.mark.parametrize("size", [0, 501, True, "100", 1.5])
def test_invalid_page_size_never_sends(monkeypatch, size):
    opener = stub(monkeypatch, {"items": []})
    with pytest.raises(CoreError):
        CoreClient(CoreConfig(enabled=True)).list_clients(bearer_token=TOKEN, page_size=size)
    assert not opener.requests


@pytest.mark.parametrize(
    "continuation",
    ["raw-string", CoreContinuation(""), CoreContinuation("a" * 8193), CoreContinuation("\ud800")],
)
def test_invalid_continuation_never_sends(monkeypatch, continuation):
    opener = stub(monkeypatch, {"items": []})
    with pytest.raises(CoreError) as error:
        CoreClient(CoreConfig(enabled=True)).list_clients(
            bearer_token=TOKEN, continuation=continuation
        )
    assert error.value.category is CoreCategory.INVALID_INPUT
    assert not opener.requests


def test_captured_contract_pins_the_read_request_and_required_projection():
    snapshot = json.loads(
        (Path(__file__).resolve().parents[1] / "contracts" / "lodgeit-core-clients.json").read_text(
            encoding="utf-8"
        )
    )
    path = "/core/clients/v1-preview/list"
    assert set(snapshot["paths"][path]) == {"post"}
    assert "read:core.clients" in str(snapshot["paths"][path]["post"]["security"])
    assert snapshot["schemas"]["ClientV1"]["required"] == ["id", "archived", "private", "profile"]
    assert snapshot["schemas"]["ClientPagingArgsV1"]["properties"]["size"]["maximum"] == 500
    assert len(snapshot["source_sha256"]) == 64


@pytest.mark.parametrize(
    "payload",
    [
        {"items": [item(profile={"sensitive": "canary"})]},
        {"items": [item(groups=[])]},
        {"items": [item(assignments={})]},
        {"items": [item(relations=[])]},
        {"items": [item(unexpected="canary")]},
        {"items": [item(archived=True)]},
        {"items": [item(private=1)]},
        {"items": [item(id="not-a-uuid")]},
        {"items": [item(code=1.2)]},
        {"items": [item(), item()]},
        {"items": [], "nextPageToken": ""},
        {"items": [], "nextPageToken": "a" * 8193},
        b'{"items":[],"items":[]}',
        b'{"items":NaN}',
        b"\xff",
        b'{"items":canary}',
    ],
)
def test_bad_success_is_transactionally_rejected_without_context(monkeypatch, payload):
    stub(monkeypatch, payload)
    with pytest.raises(CoreError) as error:
        CoreClient(CoreConfig(enabled=True)).list_clients(bearer_token=TOKEN)
    assert error.value.category is CoreCategory.PROTOCOL
    assert error.value.__context__ is error.value.__cause__ is None
    assert "canary" not in repr(error.value) + str(error.value) + str(error.value.args)


@pytest.mark.parametrize(
    "status,category",
    [
        (401, CoreCategory.AUTHENTICATION),
        (403, CoreCategory.AUTHORISATION),
        (400, CoreCategory.REQUEST_REJECTED),
        (422, CoreCategory.REQUEST_REJECTED),
        (429, CoreCategory.THROTTLED),
        (503, CoreCategory.UPSTREAM),
        (302, CoreCategory.PROTOCOL),
    ],
)
def test_http_error_bodies_are_discarded_and_never_retried(monkeypatch, status, category):
    opener = stub(monkeypatch, b"private-provider-canary", status=status)
    with pytest.raises(CoreError) as error:
        CoreClient(CoreConfig(enabled=True)).list_clients(bearer_token=TOKEN)
    assert error.value.category is category and error.value.http_status == status
    assert error.value.__context__ is error.value.__cause__ is None
    assert "canary" not in str(error.value) + repr(error.value)
    assert len(opener.requests) == 1


@pytest.mark.parametrize(
    "wire",
    [
        b"private-upstream-canary\r\n",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n"
        b"Transfer-Encoding: chunked\r\n\r\n5\r\ncanary\r\ninvalid-size\r\n",
    ],
)
def test_http_parser_failures_have_no_payload_or_bearer_context(monkeypatch, wire):
    class Socket:
        def makefile(self, mode):
            return io.BytesIO(wire)

    class ProtocolOpener:
        def open(self, request, timeout):
            response = http.client.HTTPResponse(Socket())
            response.begin()
            return response

    monkeypatch.setattr(transport, "_opener", ProtocolOpener)
    with pytest.raises(CoreError) as error:
        CoreClient(CoreConfig(enabled=True)).list_clients(bearer_token=TOKEN)
    assert error.value.category is CoreCategory.TRANSPORT
    assert error.value.__context__ is error.value.__cause__ is None
    assert "canary" not in str(error.value) + repr(error.value) + repr(vars(error.value))
    trace = error.value.__traceback__
    while trace:
        if trace.tb_frame.f_code.co_name == "list_clients":
            assert trace.tb_frame.f_locals["bearer_token"] == ""
        trace = trace.tb_next


def test_http_error_close_failure_is_sanitised(monkeypatch):
    requests = []

    class CloseFailsOnce(io.BytesIO):
        def __init__(self, value):
            super().__init__(value)
            self.failed = False

        def close(self):
            if not self.failed:
                self.failed = True
                raise OSError("private-close-canary")
            return super().close()

    class CloseFailureOpener:
        def open(self, request, timeout):
            requests.append(request)
            headers = Message()
            headers["Content-Type"] = "application/problem+json"
            headers["X-Private"] = "private-header-canary"
            raise urllib.error.HTTPError(
                CORE_URL,
                503,
                "fabricated",
                headers,
                CloseFailsOnce(b'{"private":"response-canary"}'),
            )

    monkeypatch.setattr(transport, "_opener", CloseFailureOpener)
    with pytest.raises(CoreError) as error:
        CoreClient(CoreConfig(enabled=True)).list_clients(
            bearer_token="private-bearer-canary",
            continuation=CoreContinuation("private-continuation-canary"),
        )
    assert len(requests) == 1 and requests[0].full_url == CORE_URL
    assert requests[0].get_method() == "POST"
    assert error.value.category is CoreCategory.TRANSPORT
    assert error.value.http_status is None
    assert error.value.__cause__ is error.value.__context__ is None
    assert "canary" not in str(error.value) + repr(error.value) + repr(vars(error.value))
    trace = error.value.__traceback__
    while trace:
        if trace.tb_frame.f_globals.get("__name__", "").startswith("lodgeitadapter."):
            assert "canary" not in repr(dict(trace.tb_frame.f_locals))
        if trace.tb_frame.f_code.co_name == "list_clients":
            assert trace.tb_frame.f_locals["bearer_token"] == ""
            assert trace.tb_frame.f_locals["continuation"] is None
            assert trace.tb_frame.f_locals["body"] is None
            assert trace.tb_frame.f_locals["raw"] is None
        trace = trace.tb_next


@pytest.mark.parametrize(
    "stage", ["disabled", "size", "continuation", "bearer", "transport", "status", "malformed"]
)
def test_every_core_failure_clears_sensitive_frame_locals(monkeypatch, stage):
    token, continuation = "fabricated-bearer-canary", "fabricated-continuation-canary"
    options = {"bearer_token": token, "continuation": CoreContinuation(continuation)}
    if stage == "size":
        options["page_size"] = 0
    elif stage == "continuation":
        options["continuation"] = continuation
    elif stage == "bearer":
        options["bearer_token"] = token + "\n"
    opener = stub(
        monkeypatch,
        b"malformed" if stage == "malformed" else {"items": []},
        status=503 if stage == "status" else 200,
    )
    if stage == "transport":

        def failed(request, timeout):
            raise transport.TransportError("fabricated transport failure")

        monkeypatch.setattr(opener, "open", failed)

    def contains_secret(value):
        if isinstance(value, str):
            return token in value or continuation in value
        if isinstance(value, bytes):
            return token.encode() in value or continuation.encode() in value
        if is_dataclass(value) and not isinstance(value, type):
            return any(contains_secret(getattr(value, field.name)) for field in fields(value))
        if isinstance(value, dict):
            return any(contains_secret(key) or contains_secret(item) for key, item in value.items())
        if isinstance(value, (tuple, list)):
            return any(contains_secret(item) for item in value)
        return False

    with pytest.raises(CoreError) as error:
        CoreClient(CoreConfig(enabled=stage != "disabled")).list_clients(**options)
    assert error.value.__cause__ is error.value.__context__ is None
    trace = error.value.__traceback__
    while trace:
        if trace.tb_frame.f_globals.get("__name__", "").startswith("lodgeitadapter."):
            assert not contains_secret(dict(trace.tb_frame.f_locals))
        trace = trace.tb_next


@pytest.mark.parametrize("status", [301, 302, 303, 307, 308])
@pytest.mark.parametrize(
    ("location", "category"),
    [
        ("https://outside.invalid/private-redirect-canary", CoreCategory.TRANSPORT),
        ("https://[", CoreCategory.TRANSPORT),
        ("https://[not-an-ip]/", CoreCategory.TRANSPORT),
        ("file:///private-redirect-canary", CoreCategory.PROTOCOL),
    ],
)
def test_actual_redirect_handler_refuses_without_following_or_leaking(
    monkeypatch, status, location, category
):
    requests = []

    class RedirectOpener:
        def open(self, request, timeout):
            requests.append(request)
            handler = transport._NoRedirects()
            method = getattr(handler, "http_error_" + str(status))
            headers = Message()
            headers["Location"] = location
            return method(
                request,
                io.BytesIO(b"private-response-canary"),
                status,
                "redirect",
                headers,
            )

    monkeypatch.setattr(transport, "_opener", RedirectOpener)
    with pytest.raises(CoreError) as error:
        CoreClient(CoreConfig(enabled=True)).list_clients(
            bearer_token="private-bearer-canary",
            continuation=CoreContinuation("private-continuation-canary"),
        )
    assert len(requests) == 1 and requests[0].full_url == CORE_URL
    assert error.value.category is category
    assert error.value.http_status == (status if category is CoreCategory.PROTOCOL else None)
    assert error.value.__cause__ is error.value.__context__ is None
    assert "canary" not in str(error.value) + repr(error.value) + repr(vars(error.value))
    trace = error.value.__traceback__
    while trace:
        if trace.tb_frame.f_globals.get("__name__", "").startswith("lodgeitadapter."):
            values = dict(trace.tb_frame.f_locals)
            assert "canary" not in repr(values)
            assert location not in repr(values)
        if trace.tb_frame.f_code.co_name == "list_clients":
            assert trace.tb_frame.f_locals["bearer_token"] == ""
            assert trace.tb_frame.f_locals["continuation"] is None
            assert trace.tb_frame.f_locals["body"] is None
        trace = trace.tb_next


@pytest.mark.parametrize(
    "timeout",
    [pytest.param(10**400, id="huge-positive"), pytest.param(-(10**400), id="huge-negative")],
)
def test_huge_timeouts_are_configuration_errors(timeout):
    with pytest.raises(ConfigurationError):
        CoreConfig(read_timeout=timeout)


def test_response_ceiling_and_media_type_are_enforced(monkeypatch):
    for options, config in (
        ({"content_type": "text/html"}, CoreConfig(enabled=True)),
        ({}, CoreConfig(enabled=True, max_response_bytes=1)),
    ):
        stub(monkeypatch, {"items": []}, **options)
        with pytest.raises(CoreError):
            CoreClient(config).list_clients(bearer_token=TOKEN)


def test_synthetic_maximum_page_fits_and_contract_projection_is_serialisable(monkeypatch):
    clients = [item(id=str(UUID(int=i + 1)), code="x" * 1024) for i in range(500)]
    stub(monkeypatch, {"items": clients})
    page = CoreClient(CoreConfig(enabled=True)).list_clients(bearer_token=TOKEN, page_size=500)
    assert len(page.items) == 500
    assert asdict(page.items[0])["code"] == "x" * 1024


@pytest.mark.parametrize(
    "options",
    [
        {"enabled": 1},
        {"read_timeout": float("nan")},
        {"read_timeout": 61},
        {"max_response_bytes": True},
        {"max_response_bytes": 1048577},
    ],
)
def test_config_safety_limits(options):
    with pytest.raises(ConfigurationError):
        CoreConfig(**options)


def test_conflicting_authorisation_is_refused_by_shared_transport(monkeypatch):
    opener = stub(monkeypatch, {"items": []})
    config = AdapterConfig(
        enabled=True, allow_loopback=True, extra_headers={"authorization": "fabricated"}
    )
    with pytest.raises(ConfigurationError):
        transport.request(config, "POST", "http://127.0.0.1/v1/calculators", bearer_token=TOKEN)
    assert not opener.requests
