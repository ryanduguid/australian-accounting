"""Fabricated regressions for transport failures and status-driven retries."""

from __future__ import annotations

import http.client
import io
import socket
import ssl
import sys
import urllib.error
import urllib.request
from collections.abc import Mapping
from email.message import Message

import pytest

from lodgeitadapter import transport
from lodgeitadapter.client import LodgeitClient, Status
from lodgeitadapter.config import AdapterConfig
from lodgeitadapter.errors import DisallowedTargetError, TransportError

URL = "http://127.0.0.1:9/v1/calculators"
CANARY = "fabricated-private-canary"


def config(**options):
    return AdapterConfig(
        enabled=True, base_url="http://127.0.0.1:9", allow_loopback=True,
        max_attempts=3, retry_backoff_seconds=0.01, **options,
    )


class ErrorBody(io.BytesIO):
    def __init__(self, failure):
        super().__init__(CANARY.encode() * (2 if failure == "oversize" else 1))
        self.failure = failure
        self.closes = 0

    def read(self, size=-1):
        if self.failure == "parser":
            raise http.client.IncompleteRead(CANARY.encode(), 99)
        if self.failure == "value":
            raise ValueError(CANARY)
        if self.failure == "timeout":
            raise TimeoutError(CANARY)
        return super().read(size)

    def close(self):
        self.closes += 1
        super().close()
        if self.failure == "close":
            raise OSError(CANARY)


class ErrorOpener:
    def __init__(self, status, failure):
        self.status, self.failure = status, failure
        self.bodies = []

    def open(self, request, timeout):
        stream = ErrorBody(self.failure)
        self.bodies.append(stream)
        raise urllib.error.HTTPError(URL, self.status, CANARY, Message(), stream)


def assert_detached(error):
    assert error.__cause__ is error.__context__ is None
    assert CANARY not in str(error) + repr(error) + repr(vars(error))

    def carries_canary(value, seen):
        if isinstance(value, (str, bytes)):
            return CANARY in value if isinstance(value, str) else CANARY.encode() in value
        if id(value) in seen:
            return False
        seen.add(id(value))
        if isinstance(value, Mapping):
            return any(carries_canary(item, seen) for pair in value.items() for item in pair)
        if isinstance(value, (list, tuple, set)):
            return any(carries_canary(item, seen) for item in value)
        if isinstance(value, (Exception, transport.RawResponse,
                              urllib.request.Request, AdapterConfig)):
            return carries_canary(vars(value), seen)
        if callable(value):
            return carries_canary(getattr(value, "__dict__", {}), seen) or any(
                carries_canary(cell.cell_contents, seen)
                for cell in (getattr(value, "__closure__", None) or ())
            )
        return False

    assert not carries_canary(error.args, set())
    assert not carries_canary(getattr(error, "__notes__", ()), set())
    trace = error.__traceback__
    while trace:
        if trace.tb_frame.f_globals.get("__name__") == "lodgeitadapter.transport":
            values = dict(trace.tb_frame.f_locals)
            assert CANARY not in repr(values)
            assert not carries_canary(values, set())
        trace = trace.tb_next


@pytest.mark.parametrize("status", [400, 422, 429, 503])
@pytest.mark.parametrize("failure", ["parser", "value", "timeout", "close", "oversize"])
@pytest.mark.parametrize("retry_safe", [False, True])
def test_failed_http_body_respects_status_and_retry_budget(
    monkeypatch, status, failure, retry_safe,
):
    opener = ErrorOpener(status, failure)
    monkeypatch.setattr(transport, "_opener", lambda: opener)
    pauses = []

    def sleep(seconds):
        assert all(stream.closed for stream in opener.bodies), "close before backoff"
        pauses.append(seconds)

    with pytest.raises(TransportError) as error:
        transport.request(
            config(max_response_bytes=4 if failure == "oversize" else 1024,
                   extra_headers={"X-Fabricated": CANARY}), "POST", URL,
            body=CANARY.encode(), bearer_token=CANARY, retry_safe=retry_safe, sleep=sleep,
        )
    attempts = 3 if status == 503 and retry_safe else 1
    assert len(opener.bodies) == attempts
    assert pauses == [0.01 * attempt for attempt in range(1, attempts)]
    assert all(stream.closed for stream in opener.bodies)
    assert_detached(error.value)


@pytest.mark.parametrize("failure", ["parser", "value", "close", "oversize"])
def test_calculator_client_returns_unavailable_for_failed_5xx(monkeypatch, contract, failure):
    opener = ErrorOpener(503, failure)
    monkeypatch.setattr(transport, "_opener", lambda: opener)
    outcome = LodgeitClient(
        config(max_response_bytes=4 if failure == "oversize" else 1024), contract,
    ).discover()
    assert outcome.status is Status.UPSTREAM_UNAVAILABLE
    assert outcome.result is None and outcome.values == {}
    assert len(opener.bodies) == 3
    assert all(CANARY not in finding for finding in outcome.findings)


@pytest.mark.parametrize("retry_safe", [False, True])
@pytest.mark.parametrize("failure", ["bad-status", "bad-chunk", "close", "socket", "value"])
def test_response_failures_are_normalised_without_retaining_diagnostics(
    monkeypatch, failure, retry_safe,
):
    requests = []

    class Socket:
        def makefile(self, mode):
            wire = (
                CANARY.encode() + b"\r\n" if failure == "bad-status" else
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n"
                b"5\r\nhello\r\ninvalid-size\r\n"
            )
            return io.BytesIO(wire)

    class Opener:
        def open(self, request, timeout):
            requests.append(request)
            if failure == "socket":
                raise urllib.error.URLError(CANARY)
            if failure == "value":
                raise ValueError(CANARY)
            if failure == "close":
                class Response(ErrorBody):
                    status = 200
                    headers = Message()

                return Response("close")
            response = http.client.HTTPResponse(Socket())
            response.begin()
            return response

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(TransportError) as error:
        transport.request(
            config(), "POST", URL, body=CANARY.encode(), bearer_token=CANARY,
            retry_safe=retry_safe, sleep=lambda seconds: None,
        )
    assert len(requests) == (3 if retry_safe else 1)
    assert_detached(error.value)


@pytest.mark.parametrize("status", [301, 302, 303, 307, 308])
@pytest.mark.parametrize("location", ["https://[", "https://outside.invalid/" + CANARY])
@pytest.mark.parametrize("close_failure", [False, True])
def test_redirect_refusal_never_retries_or_retains_location(
    monkeypatch, status, location, close_failure,
):
    requests, streams = [], []

    class Body(io.BytesIO):
        def read(self, size=-1):
            pytest.fail("a refused redirect body must not be drained")

        def close(self):
            super().close()
            if close_failure:
                raise OSError(CANARY)

    class Opener:
        def open(self, request, timeout):
            requests.append(request)
            stream = Body(CANARY.encode())
            streams.append(stream)
            headers = Message()
            headers["Location"] = location
            method = getattr(transport._NoRedirects(), "http_error_" + str(status))
            return method(request, stream, status, CANARY, headers)

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(DisallowedTargetError) as error:
        transport.request(config(), "POST", URL, retry_safe=True, sleep=lambda seconds: None)
    assert len(requests) == 1
    assert all(stream.closed for stream in streams)
    assert_detached(error.value)


@pytest.mark.parametrize("close_failure", [False, True])
def test_successful_oversized_response_is_not_retried(monkeypatch, close_failure):
    requests = []

    class Response(ErrorBody):
        status = 200
        headers = Message()

        def close(self):
            super().close()
            if close_failure:
                raise OSError(CANARY)

    class Opener:
        def open(self, request, timeout):
            requests.append(request)
            return Response("oversize")

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(TransportError, match="byte ceiling") as error:
        transport.request(config(max_response_bytes=len(CANARY)), "GET", URL, retry_safe=True)
    assert len(requests) == 1
    assert_detached(error.value)


@pytest.mark.parametrize("failure", [KeyboardInterrupt, SystemExit])
def test_process_cancellation_is_not_converted_or_retried(monkeypatch, failure):
    requests = []

    class Opener:
        def open(self, request, timeout):
            requests.append(request)
            raise failure()

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(failure):
        transport.request(config(), "GET", URL, retry_safe=True)
    assert len(requests) == 1


@pytest.mark.parametrize("status", [302, 400, 422, 429, 503])
@pytest.mark.parametrize("retry_safe", [False, True])
def test_readable_error_response_preserves_provider_data(monkeypatch, status, retry_safe):
    opener = ErrorOpener(status, "none")
    monkeypatch.setattr(transport, "_opener", lambda: opener)
    raw = transport.request(config(), "GET", URL, retry_safe=retry_safe, sleep=lambda _: None)
    assert raw.status == status and raw.body == CANARY.encode() and raw.url == URL
    assert len(opener.bodies) == (3 if status == 503 and retry_safe else 1)
    assert all(stream.closed for stream in opener.bodies)


@pytest.mark.parametrize("failure", ["none", "parser", "value", "close", "oversize"])
def test_failed_5xx_recovers_to_success_within_budget(monkeypatch, failure):
    first = ErrorOpener(503, failure)
    replies = []
    pauses = []

    class Response(io.BytesIO):
        status = 200
        headers = {"X-Provider": CANARY}

    class Opener:
        def open(self, request, timeout):
            if not first.bodies:
                return first.open(request, timeout)
            response = Response(b"{}")
            replies.append(response)
            return response

    def sleep(seconds):
        assert first.bodies[0].closed
        pauses.append(seconds)

    monkeypatch.setattr(transport, "_opener", Opener)
    raw = transport.request(
        config(max_response_bytes=4 if failure == "oversize" else 1024), "POST", URL,
        body=CANARY.encode(), retry_safe=True, sleep=sleep,
    )
    assert raw.status == 200 and raw.body == b"{}"
    assert raw.headers == {"x-provider": CANARY}
    assert len(first.bodies) == len(replies) == 1 and replies[0].closed
    assert pauses == [0.01]


@pytest.mark.parametrize("failure", ["parser", "value", "timeout", "close", "oversize"])
def test_known_redirect_with_failed_body_is_never_retried(monkeypatch, failure):
    opener = ErrorOpener(302, failure)
    monkeypatch.setattr(transport, "_opener", lambda: opener)
    with pytest.raises(TransportError) as error:
        transport.request(config(max_response_bytes=4 if failure == "oversize" else 1024),
                          "GET", URL,
                          retry_safe=True, sleep=lambda _: pytest.fail("redirect retry"))
    assert len(opener.bodies) == 1 and opener.bodies[0].closed
    assert_detached(error.value)


@pytest.mark.parametrize("stage", ["read", "close"])
@pytest.mark.parametrize("cancellation", [KeyboardInterrupt, SystemExit])
def test_response_cancellation_propagates_after_cleanup(monkeypatch, stage, cancellation):
    replies = []

    class Response(io.BytesIO):
        status = 200
        headers = {}

        def read(self, size=-1):
            if stage == "read":
                raise cancellation()
            return super().read(size)

        def close(self):
            super().close()
            if stage == "close":
                raise cancellation()
            raise OSError(CANARY)

    class Opener:
        def open(self, request, timeout):
            reply = Response(CANARY.encode())
            replies.append(reply)
            return reply

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(cancellation):
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("cancellation retry"))
    assert len(replies) == 1 and replies[0].closed


def test_invalid_utf8_error_does_not_retain_response():
    raw = transport.RawResponse(200, {"x-provider": CANARY},
                                CANARY.encode() + b"\xff", URL, 0)
    with pytest.raises(TransportError, match="not UTF-8") as error:
        raw.text()
    assert_detached(error.value)


@pytest.mark.parametrize("status", [503, 600])
@pytest.mark.parametrize("retry_safe", [False, True])
def test_normally_returned_server_error_uses_status_for_retry(monkeypatch, status, retry_safe):
    replies = []

    class Response(ErrorBody):
        headers = {"X-Provider": CANARY}

    class Opener:
        def open(self, request, timeout):
            reply = Response("none")
            reply.status = status
            replies.append(reply)
            return reply

    monkeypatch.setattr(transport, "_opener", Opener)
    raw = transport.request(config(), "GET", URL, retry_safe=retry_safe, sleep=lambda _: None)
    assert len(replies) == (3 if retry_safe else 1)
    assert all(reply.closed for reply in replies)
    assert raw.status == status and raw.body == CANARY.encode()
    assert raw.headers == {"x-provider": CANARY}


@pytest.mark.parametrize("as_http_error", [False, True])
def test_readable_5xx_then_open_failure_never_returns_stale_response(monkeypatch, as_http_error):
    replies, requests = [], []

    class Response(ErrorBody):
        status = 503
        headers = {}

    class Opener:
        def open(self, request, timeout):
            requests.append(request)
            if replies:
                raise urllib.error.URLError(CANARY)
            reply = Response("none")
            replies.append(reply)
            if as_http_error:
                raise urllib.error.HTTPError(URL, 503, CANARY, Message(), reply)
            return reply

    def sleep(seconds):
        assert replies[0].closed
        frame = sys._getframe(1)
        assert frame.f_globals["__name__"] == "lodgeitadapter.transport"
        assert not any(isinstance(value, (transport.RawResponse, Exception))
                       for value in frame.f_locals.values())

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(TransportError, match="no response after 3") as error:
        transport.request(config(), "GET", URL, retry_safe=True, sleep=sleep)
    assert len(requests) == 3 and replies[0].closed
    assert_detached(error.value)


@pytest.mark.parametrize("target", ["request", "text"])
@pytest.mark.parametrize("closure", [False, True])
def test_final_error_detaches_ambient_context_and_opaque_callback(monkeypatch, target, closure):
    class Callback:
        def __init__(self):
            self.secret = CANARY

        def __call__(self, seconds):
            pass

    class Opener:
        def open(self, request, timeout):
            raise urllib.error.URLError(CANARY)

    monkeypatch.setattr(transport, "_opener", Opener)
    hidden = CANARY
    callback = (lambda _: hidden) if closure else Callback()
    try:
        raise RuntimeError(CANARY)
    except RuntimeError:
        with pytest.raises(TransportError) as error:
            if target == "request":
                transport.request(config(), "GET", URL, sleep=callback)
            else:
                transport.RawResponse(200, {}, b"\xff", URL, 0).text()
    assert_detached(error.value)


@pytest.mark.parametrize("exception", [KeyboardInterrupt, SystemExit, TypeError, RuntimeError])
def test_backoff_exceptions_propagate_after_closure(monkeypatch, exception):
    opener = ErrorOpener(503, "none")
    monkeypatch.setattr(transport, "_opener", lambda: opener)

    def sleep(seconds):
        assert opener.bodies[0].closed
        raise exception()

    with pytest.raises(exception):
        transport.request(config(), "GET", URL, retry_safe=True, sleep=sleep)
    assert len(opener.bodies) == 1


@pytest.mark.parametrize("exception", [TypeError, RuntimeError])
def test_unrelated_open_errors_are_not_converted(monkeypatch, exception):
    class Opener:
        def open(self, request, timeout):
            raise exception()

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(exception):
        transport.request(config(), "GET", URL, retry_safe=True)


@pytest.mark.parametrize(("status", "failure"), [(400, "parser"), (503, "parser"),
                                                (503, "oversize")])
def test_close_failure_preserves_primary_body_failure(monkeypatch, status, failure):
    replies = []

    class Body(ErrorBody):
        def close(self):
            super().close()
            raise OSError(CANARY)

    class Opener:
        def open(self, request, timeout):
            reply = Body(failure)
            replies.append(reply)
            raise urllib.error.HTTPError(URL, status, CANARY, Message(), reply)

    monkeypatch.setattr(transport, "_opener", Opener)
    message = "byte ceiling" if failure == "oversize" else f"HTTP {status} body could not be read"
    with pytest.raises(TransportError, match=message) as error:
        transport.request(config(max_response_bytes=4 if failure == "oversize" else 1024),
                          "GET", URL, retry_safe=True, sleep=lambda _: None)
    assert len(replies) == (3 if status == 503 else 1)
    assert all(reply.closed for reply in replies)
    assert_detached(error.value)


@pytest.fixture
def wire_sockets(monkeypatch):
    peers, sockets = [], []

    class Reader:
        def __init__(self, stream, close_failure, read_failure):
            self.stream = stream
            self.close_failure = close_failure
            self.read_failure = read_failure
            self.lines = 0
            self.reads = 0
            self.closes = 0

        def read(self, size=-1):
            self.reads += 1
            assert size >= 0, "the socket reader must never receive an unbounded read"
            return self.stream.read(size)

        def readline(self, size=-1):
            self.lines += 1
            if self.read_failure is not None and self.lines == 2:
                raise self.read_failure(CANARY)
            return self.stream.readline(size)

        def close(self):
            self.closes += 1
            self.stream.close()
            if self.close_failure:
                failure = OSError if self.close_failure is True else self.close_failure
                raise failure(CANARY)

        def flush(self):
            self.stream.flush()

        @property
        def closed(self):
            return self.stream.closed

    class Socket:
        def __init__(self, sock, close_failure, read_failure, socket_close_failure):
            self.sock = sock
            self.streams = []
            self.close_failure = close_failure
            self.read_failure = read_failure
            self.socket_close_failure = socket_close_failure

        def makefile(self, mode):
            stream = Reader(self.sock.makefile(mode), self.close_failure, self.read_failure)
            self.streams.append(stream)
            return stream

        def sendall(self, data):
            self.sock.sendall(data)

        def close(self):
            self.sock.close()
            if self.socket_close_failure:
                failure = (OSError if self.socket_close_failure is True
                           else self.socket_close_failure)
                raise failure(CANARY)

        @property
        def released(self):
            return self.sock.fileno() == -1 and all(stream.closed for stream in self.streams)

    def install(wire, *, close_failure=False, read_failure=None, socket_close_failure=False):
        def connect(connection):
            client, peer = socket.socketpair()
            client.settimeout(1)
            peer.settimeout(1)
            peers.append(peer)
            connection.sock = Socket(client, close_failure, read_failure, socket_close_failure)
            sockets.append(connection.sock)
            peer.sendall(wire)
            peer.shutdown(socket.SHUT_WR)

        monkeypatch.setattr(http.client.HTTPConnection, "connect", connect)
        monkeypatch.setattr(http.client.HTTPSConnection, "connect", connect)
        return sockets

    yield install
    for sock in sockets:
        for stream in sock.streams:
            stream.stream.close()
        sock.sock.close()
    for peer in peers:
        peer.close()


@pytest.mark.parametrize("status", [200, 400, 503, 600])
@pytest.mark.parametrize("retry_safe", [False, True])
@pytest.mark.parametrize("framing", ["negative-chunk", "huge-negative-chunk", "truncated"])
def test_invalid_framing_uses_real_opener_and_status_budget(
    wire_sockets, status, retry_safe, framing,
):
    body = CANARY.encode()
    if framing == "truncated":
        headers = f"Content-Length: {len(body) + 1}\r\n".encode()
    else:
        headers = b"Transfer-Encoding: chunked\r\n"
        size = b"-1" if framing == "negative-chunk" else b"-1000000000000000000000000000000"
        body = f"{len(body):x}\r\n".encode() + body + b"\r\n" + size + b"\r\n"
    sockets = wire_sockets(f"HTTP/1.1 {status} Test\r\n".encode() + headers + b"\r\n" + body)
    pauses = []

    def sleep(seconds):
        assert all(sock.released for sock in sockets), "release socket before backoff"
        pauses.append(seconds)

    with pytest.raises(TransportError) as error:
        transport.request(
            config(), "POST", URL, body=CANARY.encode(), bearer_token=CANARY,
            retry_safe=retry_safe, sleep=sleep,
        )
    attempts = 3 if retry_safe and status != 400 else 1
    assert len(sockets) == attempts
    assert pauses == [0.01 * attempt for attempt in range(1, attempts)]
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)


@pytest.mark.parametrize("status", [302, 400, 422, 429, 503, 600])
@pytest.mark.parametrize("retry_safe", [False, True])
def test_status_parsed_before_bad_headers_controls_retry(wire_sockets, status, retry_safe):
    sockets = wire_sockets(
        f"HTTP/1.1 {status} Test\r\n".encode() + (b"X-Fabricated: " + CANARY.encode() + b"\r\n")
        * 101 + b"\r\n"
    )
    pauses = []

    def sleep(seconds):
        assert all(sock.released for sock in sockets)
        pauses.append(seconds)

    with pytest.raises(TransportError) as error:
        transport.request(config(), "GET", URL, retry_safe=retry_safe, sleep=sleep)
    attempts = 3 if retry_safe and status >= 500 else 1
    assert len(sockets) == attempts
    assert pauses == [0.01 * attempt for attempt in range(1, attempts)]
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)


@pytest.mark.parametrize(("status", "method", "headers", "body", "expected"), [
    (200, "GET", b"Content-Length: 2\r\n", b"[]", b"[]"),
    (200, "GET", b"", b"[]", b"[]"),
    (200, "GET", b"Transfer-Encoding: chunked\r\n", b"2\r\n[]\r\n0\r\n\r\n", b"[]"),
    (200, "HEAD", b"Content-Length: 20\r\n", b"", b""),
    (204, "GET", b"Content-Length: 20\r\n", b"", b""),
    (304, "GET", b"Content-Length: 20\r\n", b"", b""),
])
def test_effective_framing_preserves_valid_and_bodyless_responses(
    wire_sockets, status, method, headers, body, expected,
):
    sockets = wire_sockets(f"HTTP/1.1 {status} Test\r\n".encode() + headers + b"\r\n" + body)
    raw = transport.request(config(max_response_bytes=2), method, URL)
    assert raw.status == status and raw.body == expected
    assert len(sockets) == 1 and sockets[0].released


def test_truncated_valid_json_discovery_cannot_be_computed(wire_sockets, contract):
    sockets = wire_sockets(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\n[]")
    outcome = LodgeitClient(config(), contract).discover()
    assert outcome.status is Status.UPSTREAM_UNAVAILABLE
    assert outcome.result is None and outcome.values == {}
    assert len(sockets) == 3 and all(sock.released for sock in sockets)


@pytest.mark.parametrize("headers,body", [
    (b"Content-Length: 3\r\n", b"abc"),
    (b"", b"abc"),
    (b"Transfer-Encoding: chunked\r\n", b"3\r\nabc\r\n0\r\n\r\n"),
])
def test_real_opener_ceiling_stops_without_retry(wire_sockets, headers, body):
    sockets = wire_sockets(b"HTTP/1.1 200 OK\r\n" + headers + b"\r\n" + body)
    with pytest.raises(TransportError, match="byte ceiling"):
        transport.request(config(max_response_bytes=2), "GET", URL, retry_safe=True)
    assert len(sockets) == 1 and sockets[0].released


def test_real_opener_pre_status_failure_still_retries(wire_sockets):
    sockets = wire_sockets(CANARY.encode() + b"\r\n")
    with pytest.raises(TransportError, match="no response after 3") as error:
        transport.request(config(), "GET", URL, retry_safe=True, sleep=lambda _: None)
    assert len(sockets) == 3 and all(sock.released for sock in sockets)
    assert_detached(error.value)


@pytest.mark.parametrize("wire", [
    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n-1\r\n",
    b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\n[]",
    b"HTTP/1.1 429 Throttled\r\n" + b"X-Fabricated: ignored\r\n" * 101 + b"\r\n",
])
def test_https_installs_framing_guards_with_verified_tls_defaults(monkeypatch, wire_sockets, wire):
    sockets = wire_sockets(wire)
    connect = http.client.HTTPSConnection.connect

    def checked_connect(connection):
        assert connection.response_class is transport._HTTPResponse
        assert connection._context.verify_mode == ssl.CERT_REQUIRED
        assert connection._context.check_hostname is True
        connect(connection)

    monkeypatch.setattr(http.client.HTTPSConnection, "connect", checked_connect)
    target = "https://127.0.0.1:9"
    with pytest.raises(TransportError) as error:
        transport.request(
            AdapterConfig(enabled=True, base_url=target, allow_loopback=True, max_attempts=1),
            "GET", target + "/v1/calculators", bearer_token=CANARY,
        )
    assert len(sockets) == 1 and sockets[0].released
    assert_detached(error.value)
    assert http.client.HTTPConnection.response_class is http.client.HTTPResponse
    assert http.client.HTTPSConnection.response_class is http.client.HTTPResponse


@pytest.mark.parametrize("status", [200, 400, 503, 600])
@pytest.mark.parametrize("retry_safe", [False, True])
@pytest.mark.parametrize("tail", [b"0\r\n", b"0", b"0\r\nX-Trailer: unfinished"])
def test_premature_chunk_trailer_eof_is_not_completion(wire_sockets, status, retry_safe, tail):
    sockets = wire_sockets(
        f"HTTP/1.1 {status} Test\r\nTransfer-Encoding: chunked\r\n\r\n".encode()
        + b"2\r\n[]\r\n" + tail
    )
    pauses = []

    def sleep(seconds):
        assert all(sock.released for sock in sockets)
        pauses.append(seconds)

    with pytest.raises(TransportError) as error:
        transport.request(config(), "GET", URL, retry_safe=retry_safe, sleep=sleep)
    attempts = 3 if retry_safe and status != 400 else 1
    assert len(sockets) == attempts
    assert pauses == [0.01 * attempt for attempt in range(1, attempts)]
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)


@pytest.mark.parametrize("status", [200, 400, 503])
@pytest.mark.parametrize("length", [
    b"2, 3", b"2\r\nContent-Length: 3", b"-2", b"two", b"", b"3, 3",
    b"3\r\nContent-Length: 3",
])
def test_invalid_length_is_not_close_delimited_success(wire_sockets, status, length):
    sockets = wire_sockets(
        f"HTTP/1.1 {status} Test\r\nContent-Length: ".encode() + length + b"\r\n\r\n[]"
    )
    with pytest.raises(TransportError) as error:
        transport.request(config(), "GET", URL, retry_safe=True, sleep=lambda _: None)
    assert len(sockets) == (1 if status == 400 else 3)
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)


@pytest.mark.parametrize("status", [200, 429, 503])
def test_initial_header_eof_is_not_completion(wire_sockets, status):
    sockets = wire_sockets(f"HTTP/1.1 {status} Test\r\nContent-Length: 0\r\n".encode())
    with pytest.raises(TransportError) as error:
        transport.request(config(), "GET", URL, retry_safe=True, sleep=lambda _: None)
    assert len(sockets) == (1 if status == 429 else 3)
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)


@pytest.mark.parametrize("status", [302, 400, 422, 429, 503, 600])
@pytest.mark.parametrize("socket_close_failure", [False, True])
def test_known_status_survives_secondary_opener_cleanup(
    wire_sockets, status, socket_close_failure,
):
    sockets = wire_sockets(
        f"HTTP/1.1 {status} Test\r\n".encode() + b"X-Fabricated: ignored\r\n" * 101,
        close_failure=True, socket_close_failure=socket_close_failure,
    )
    pauses = []

    def sleep(seconds):
        assert all(sock.released for sock in sockets)
        pauses.append(seconds)

    with pytest.raises(TransportError) as error:
        transport.request(config(), "GET", URL, retry_safe=True, sleep=sleep)
    attempts = 3 if status >= 500 else 1
    assert len(sockets) == attempts
    assert pauses == [0.01 * attempt for attempt in range(1, attempts)]
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)


@pytest.mark.parametrize("failure", [KeyboardInterrupt, SystemExit, TypeError, RuntimeError])
@pytest.mark.parametrize("socket_close_failure", [False, True])
def test_primary_exception_survives_secondary_opener_cleanup(
    wire_sockets, failure, socket_close_failure,
):
    sockets = wire_sockets(
        b"HTTP/1.1 200 OK\r\n\r\n", close_failure=True, read_failure=failure,
        socket_close_failure=socket_close_failure,
    )
    pauses = []
    with pytest.raises(failure, match=CANARY):
        transport.request(config(), "GET", URL, retry_safe=True, sleep=pauses.append)
    assert len(sockets) == 1 and sockets[0].released
    assert pauses == []


@pytest.mark.parametrize("status", [200, 429, 503])
def test_sole_socket_close_failure_retains_parsed_status(wire_sockets, status):
    sockets = wire_sockets(
        f"HTTP/1.1 {status} Test\r\nContent-Length: 2\r\n\r\n[]".encode(),
        socket_close_failure=True,
    )
    with pytest.raises(TransportError) as error:
        transport.request(config(), "GET", URL, retry_safe=True, sleep=lambda _: None)
    assert len(sockets) == (1 if status == 429 else 3)
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)


@pytest.mark.parametrize(("status", "method", "headers", "body", "expected"), [
    (200, "GET", b"Content-Length: 0\r\n", b"", b""),
    (200, "GET", b"Content-Length: 2, 2\r\n", b"[]", b"[]"),
    (200, "GET", b"Content-Length: 2\r\nContent-Length: 2\r\n", b"[]", b"[]"),
    (200, "GET", b"Transfer-Encoding: chunked\r\n", b"2\r\n[]\r\n0\r\n\r\n", b"[]"),
    (200, "GET", b"Transfer-Encoding: chunked\r\n",
     b"2\r\n[]\r\n0\r\nX-Trailer: ignored\r\n\r\n", b"[]"),
    (304, "GET", b"Transfer-Encoding: chunked\r\n", b"", b""),
    (200, "HEAD", b"Transfer-Encoding: chunked\r\n", b"", b""),
    (204, "GET", b"Transfer-Encoding: chunked\r\n", b"", b""),
    (304, "GET", b"Content-Length: invalid\r\n", b"", b""),
    (200, "GET", b"Transfer-Encoding: chunked\r\nContent-Length: invalid\r\n",
     b"2\r\n[]\r\n0\r\n\r\n", b"[]"),
])
def test_completed_framing_and_bodyless_controls(
    wire_sockets, status, method, headers, body, expected,
):
    sockets = wire_sockets(f"HTTP/1.1 {status} Test\r\n".encode() + headers + b"\r\n" + body)
    raw = transport.request(config(max_response_bytes=2), method, URL)
    assert raw.status == status and raw.body == expected
    if b"Transfer-Encoding" in headers:
        assert raw.headers["transfer-encoding"] == "chunked"
    if b"Content-Length: 2, 2" in headers:
        assert raw.headers["content-length"] == "2, 2"
    assert len(sockets) == 1 and sockets[0].released


@pytest.mark.parametrize("wire", [
    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n[]\r\n0\r\n",
    b"HTTP/1.1 200 OK\r\nContent-Length: 2, 3\r\n\r\n[]",
])
def test_invalid_message_discovery_cannot_be_computed(wire_sockets, contract, wire):
    sockets = wire_sockets(wire)
    outcome = LodgeitClient(config(), contract).discover()
    assert outcome.status is Status.UPSTREAM_UNAVAILABLE
    assert outcome.result is None and outcome.values == {}
    assert len(sockets) == 3 and all(sock.released for sock in sockets)


@pytest.mark.parametrize("ragged", [False, True])
@pytest.mark.parametrize("framing", ["close", "length", "chunked"])
@pytest.mark.parametrize("status", [200, 429, 503])
def test_https_eof_suppression_branch(monkeypatch, ragged, framing, status):
    """Exercise actual SSLSocket reads with fabricated TLS outcomes, without a handshake."""
    sockets, peers = [], []
    body = b"[]"
    headers = b""
    if framing == "length":
        headers = b"Content-Length: 2\r\n"
    elif framing == "chunked":
        headers = b"Transfer-Encoding: chunked\r\n"
        body = b"2\r\n[]\r\n0\r\n\r\n"
    wire = f"HTTP/1.1 {status} Test\r\n".encode() + headers + b"\r\n" + body

    class FabricatedTLS:
        def __init__(self):
            self.remaining = wire

        def write(self, data):
            return len(data)

        def read(self, size, buffer=None):
            if not self.remaining and ragged:
                raise ssl.SSLEOFError(ssl.SSL_ERROR_EOF, CANARY)
            data, self.remaining = self.remaining[:size], self.remaining[size:]
            if buffer is None:
                return data
            buffer[:len(data)] = data
            return len(data)

    def connect(connection):
        assert connection._context.verify_mode == ssl.CERT_REQUIRED
        assert connection._context.check_hostname is True
        client, peer = socket.socketpair()
        peers.append(peer)
        sock = connection._context.wrap_socket(
            client, server_hostname=connection.host, do_handshake_on_connect=False,
        )
        assert sock.suppress_ragged_eofs is True
        sock._sslobj = FabricatedTLS()
        connection.sock = sock
        sockets.append(sock)

    def released():
        return all(sock.fileno() == -1 and sock._io_refs == 0 for sock in sockets)

    pauses = []

    def sleep(seconds):
        assert released()
        pauses.append(seconds)

    monkeypatch.setattr(http.client.HTTPSConnection, "connect", connect)
    target = "https://127.0.0.1:9"
    settings = AdapterConfig(
        enabled=True, base_url=target, allow_loopback=True, max_attempts=3,
        retry_backoff_seconds=0.01,
    )
    try:
        if ragged and framing == "close":
            with pytest.raises(TransportError) as error:
                transport.request(settings, "GET", target + "/v1/calculators",
                                  retry_safe=True, sleep=sleep)
            assert_detached(error.value)
        else:
            raw = transport.request(settings, "GET", target + "/v1/calculators",
                                    retry_safe=True, sleep=sleep)
            assert raw.status == status and raw.body == b"[]"
        failed_close = ragged and framing == "close"
        attempts = 1 if status == 429 or (status == 200 and not failed_close) else 3
        assert len(sockets) == attempts
        assert pauses == [0.01 * attempt for attempt in range(1, attempts)]
        assert released()
    finally:
        for sock in sockets:
            sock.close()
        for peer in peers:
            peer.close()


@pytest.mark.parametrize("status", [200, 400, 503])
@pytest.mark.parametrize("body", [
    b"2\r\n[]XX0\r\n\r\n", b"+2\r\n[]\r\n0\r\n\r\n",
    b"0x2\r\n[]\r\n0\r\n\r\n", b"-0\r\n\r\n", b" 2\r\n[]\r\n0\r\n\r\n",
])
def test_invalid_chunk_transition_is_not_completion(wire_sockets, status, body):
    sockets = wire_sockets(
        f"HTTP/1.1 {status} Test\r\nTransfer-Encoding: chunked\r\n\r\n".encode() + body
    )
    with pytest.raises(TransportError) as error:
        transport.request(config(), "GET", URL, retry_safe=True, sleep=lambda _: None)
    assert len(sockets) == (1 if status == 400 else 3)
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)


def test_bad_chunk_delimiter_cannot_be_computed(wire_sockets, contract):
    sockets = wire_sockets(
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n[]XX0\r\n\r\n"
    )
    outcome = LodgeitClient(config(), contract).discover()
    assert outcome.status is Status.UPSTREAM_UNAVAILABLE
    assert outcome.result is None and outcome.values == {}
    assert len(sockets) == 3 and all(sock.released for sock in sockets)


def test_valid_hex_sizes_and_extensions(wire_sockets):
    sockets = wire_sockets(
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n"
        b"A;fabricated=value\r\n0123456789\r\n0\r\n\r\n"
    )
    raw = transport.request(config(), "GET", URL)
    assert raw.status == 200 and raw.body == b"0123456789"
    assert len(sockets) == 1 and sockets[0].released


@pytest.mark.parametrize("headers,body,expected", [
    (b"Transfer-Encoding: gzip\r\nContent-Length: 2\r\n", b"[]x", b"[]x"),
    (b"Transfer-Encoding: gzip, chunked\r\nContent-Length: invalid\r\n",
     b"2\r\n[]\r\n0\r\n\r\n", b"[]"),
])
def test_transfer_coding_takes_precedence_over_length(wire_sockets, headers, body, expected):
    sockets = wire_sockets(b"HTTP/1.1 200 OK\r\n" + headers + b"\r\n" + body)
    raw = transport.request(config(), "GET", URL)
    assert raw.body == expected
    assert len(sockets) == 1 and sockets[0].released


def test_transfer_coding_cannot_hide_bytes_above_ceiling(wire_sockets):
    sockets = wire_sockets(
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\nContent-Length: 2\r\n\r\n[]x"
    )
    with pytest.raises(TransportError, match="byte ceiling"):
        transport.request(config(max_response_bytes=2), "GET", URL, retry_safe=True)
    assert len(sockets) == 1 and sockets[0].released


@pytest.mark.parametrize("status", [301, 302, 303, 307, 308])
@pytest.mark.parametrize("close_failure", [False, True])
def test_redirect_without_location_releases_unread_body(wire_sockets, status, close_failure):
    sockets = wire_sockets(
        f"HTTP/1.1 {status} Test\r\nContent-Length: {len(CANARY)}\r\n\r\n".encode()
        + CANARY.encode(),
        close_failure=close_failure,
    )
    with pytest.raises(DisallowedTargetError) as error:
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("redirect retry"))
    assert len(sockets) == 1 and sockets[0].released
    assert all(stream.reads == 0 and stream.closes >= 1 for stream in sockets[0].streams)
    assert_detached(error.value)


@pytest.mark.parametrize("stage", ["headers", "request", "detach"])
@pytest.mark.parametrize("failure", [KeyboardInterrupt, SystemExit, TypeError])
def test_non_io_cleanup_preserves_primary_opener_exception(
    monkeypatch, wire_sockets, stage, failure,
):
    primary = failure("primary fabricated failure")
    closes = 0

    def socket_failure(message):
        nonlocal closes
        closes += 1
        return primary if stage == "detach" and closes == 1 else RuntimeError(message)

    sockets = wire_sockets(
        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n[]",
        read_failure=(lambda _: primary) if stage == "headers" else None,
        close_failure=RuntimeError,
        socket_close_failure=socket_failure,
    )
    if stage == "request":
        original = http.client.HTTPConnection.request

        def send_then_fail(connection, *args, **kwargs):
            original(connection, *args, **kwargs)
            raise primary

        monkeypatch.setattr(http.client.HTTPConnection, "request", send_then_fail)
    with pytest.raises(failure) as error:
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("primary exception retry"))
    assert error.value is primary
    assert len(sockets) == 1 and sockets[0].released
    assert closes >= 1
    assert all(stream.closes >= 1 for stream in sockets[0].streams)


@pytest.mark.parametrize("failure", [KeyboardInterrupt, SystemExit, TypeError])
def test_non_io_cleanup_preserves_primary_body_exception(monkeypatch, failure):
    primary = failure("primary fabricated failure")
    replies = []

    class Response(io.BytesIO):
        status = 200
        headers = {}

        def read(self, size=-1):
            raise primary

        def close(self):
            super().close()
            raise RuntimeError("secondary fabricated failure")

    class Opener:
        def open(self, request, timeout):
            reply = Response(b"[]")
            replies.append(reply)
            return reply

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(failure) as error:
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("primary exception retry"))
    assert error.value is primary
    assert len(replies) == 1 and replies[0].closed


@pytest.mark.parametrize("failure", [RuntimeError, KeyboardInterrupt])
def test_sole_non_io_body_close_exception_propagates(monkeypatch, failure):
    primary = failure("standalone fabricated failure")
    replies = []

    class Response(io.BytesIO):
        status = 200
        headers = {}

        def close(self):
            super().close()
            raise primary

    class Opener:
        def open(self, request, timeout):
            reply = Response(b"[]")
            replies.append(reply)
            return reply

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(failure) as error:
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("standalone exception retry"))
    assert error.value is primary
    assert len(replies) == 1 and replies[0].closed


@pytest.mark.parametrize(("status", "failure", "retry_safe", "attempts"), [
    (429, "parser", True, 1),
    (503, "parser", False, 1),
    (503, "parser", True, 3),
    (200, "oversize", True, 1),
])
def test_non_io_close_failure_preserves_recorded_body_failure(
    monkeypatch, status, failure, retry_safe, attempts,
):
    replies = []

    class Response(ErrorBody):
        headers = Message()

        def close(self):
            super().close()
            raise RuntimeError(CANARY)

    class Opener:
        def open(self, request, timeout):
            reply = Response(failure)
            reply.status = status
            replies.append(reply)
            return reply

    def sleep(seconds):
        assert all(reply.closed for reply in replies), "close before backoff"

    monkeypatch.setattr(transport, "_opener", Opener)
    message = "byte ceiling" if failure == "oversize" else f"HTTP {status} body could not be read"
    with pytest.raises(TransportError, match=message) as error:
        transport.request(config(max_response_bytes=len(CANARY)), "GET", URL,
                          retry_safe=retry_safe, sleep=sleep)
    assert len(replies) == attempts
    assert all(reply.closed for reply in replies)
    assert_detached(error.value)


@pytest.mark.parametrize("failure", [KeyboardInterrupt, SystemExit])
def test_cancellation_during_close_after_recorded_body_failure_propagates(monkeypatch, failure):
    replies = []

    class Response(ErrorBody):
        status = 503
        headers = Message()

        def close(self):
            super().close()
            raise failure()

    class Opener:
        def open(self, request, timeout):
            reply = Response("parser")
            replies.append(reply)
            return reply

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(failure):
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("cancellation retry"))
    assert len(replies) == 1 and replies[0].closed


@pytest.mark.parametrize(("primary", "secondary", "expected"), [
    (TypeError, KeyboardInterrupt, KeyboardInterrupt),
    (KeyboardInterrupt, SystemExit, KeyboardInterrupt),
])
def test_body_close_cancellation_precedence(monkeypatch, primary, secondary, expected):
    replies = []

    class Response(io.BytesIO):
        status = 200
        headers = {}

        def read(self, size=-1):
            raise primary("primary fabricated failure")

        def close(self):
            super().close()
            raise secondary("secondary fabricated failure")

    class Opener:
        def open(self, request, timeout):
            reply = Response(b"[]")
            replies.append(reply)
            return reply

    monkeypatch.setattr(transport, "_opener", Opener)
    with pytest.raises(expected):
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("cancellation retry"))
    assert len(replies) == 1 and replies[0].closed


@pytest.mark.parametrize("location", [False, True])
@pytest.mark.parametrize("close_failure", [RuntimeError, TypeError])
def test_redirect_close_exception_keeps_refusal(wire_sockets, location, close_failure):
    sockets = wire_sockets(
        b"HTTP/1.1 302 Test\r\n"
        + (b"Location: http://127.0.0.1:9/elsewhere\r\n" if location else b"")
        + f"Content-Length: {len(CANARY)}\r\n\r\n".encode() + CANARY.encode(),
        close_failure=close_failure,
    )
    with pytest.raises(DisallowedTargetError) as error:
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("redirect retry"))
    assert len(sockets) == 1 and sockets[0].released
    assert all(stream.reads == 0 and stream.closes >= 1 for stream in sockets[0].streams)
    assert_detached(error.value)


@pytest.mark.parametrize("location", [False, True])
@pytest.mark.parametrize("failure", [KeyboardInterrupt, SystemExit])
def test_redirect_close_cancellation_propagates(wire_sockets, location, failure):
    sockets = wire_sockets(
        b"HTTP/1.1 302 Test\r\n"
        + (b"Location: http://127.0.0.1:9/elsewhere\r\n" if location else b"")
        + f"Content-Length: {len(CANARY)}\r\n\r\n".encode() + CANARY.encode(),
        close_failure=failure,
    )
    with pytest.raises(failure):
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("cancellation retry"))
    assert len(sockets) == 1 and sockets[0].released


@pytest.mark.parametrize("stage", ["stream", "socket"])
@pytest.mark.parametrize("failure", [KeyboardInterrupt, SystemExit])
def test_header_cleanup_cancellation_propagates(wire_sockets, stage, failure):
    sockets = wire_sockets(
        b"HTTP/1.1 503 Test\r\n" + b"X-Fabricated: ignored\r\n" * 101,
        close_failure=failure if stage == "stream" else False,
        socket_close_failure=failure if stage == "socket" else False,
    )
    with pytest.raises(failure):
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("cancellation retry"))
    assert len(sockets) == 1 and sockets[0].released


@pytest.mark.parametrize(("primary", "secondary", "expected"), [
    (TypeError, KeyboardInterrupt, KeyboardInterrupt),
    (TypeError, SystemExit, SystemExit),
    (KeyboardInterrupt, SystemExit, KeyboardInterrupt),
])
def test_header_cleanup_cancellation_precedence(wire_sockets, primary, secondary, expected):
    sockets = wire_sockets(
        b"HTTP/1.1 200 OK\r\n\r\n", read_failure=primary, close_failure=secondary,
    )
    with pytest.raises(expected, match=CANARY):
        transport.request(config(), "GET", URL, retry_safe=True,
                          sleep=lambda _: pytest.fail("cancellation retry"))
    assert len(sockets) == 1 and sockets[0].released


@pytest.mark.parametrize("failure", [RuntimeError, KeyboardInterrupt])
def test_failed_begin_does_not_suppress_later_standalone_close_exception(wire_sockets, failure):
    sockets = wire_sockets(
        b"HTTP/1.1 200 OK\r\n\r\n", read_failure=TypeError, close_failure=failure,
    )
    connection = transport._HTTPConnection("127.0.0.1", timeout=1)
    connection.connect()
    response = transport._HTTPResponse(connection.sock)
    try:
        with pytest.raises(TypeError):
            response.begin()
        with pytest.raises(failure):
            response.close()
    finally:
        connection.close()
    assert len(sockets) == 1 and sockets[0].released


@pytest.mark.parametrize("status", [429, 503])
def test_non_io_header_cleanup_preserves_known_status(wire_sockets, status):
    sockets = wire_sockets(
        f"HTTP/1.1 {status} Test\r\n".encode() + b"X-Fabricated: ignored\r\n" * 101,
        close_failure=RuntimeError, socket_close_failure=RuntimeError,
    )

    def sleep(seconds):
        assert all(sock.released for sock in sockets)

    with pytest.raises(TransportError) as error:
        transport.request(config(), "GET", URL, retry_safe=True, sleep=sleep)
    assert len(sockets) == (3 if status == 503 else 1)
    assert all(sock.released for sock in sockets)
    assert_detached(error.value)
