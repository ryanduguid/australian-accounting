"""Bounded HTTP, with the guards that keep a request where it was aimed.

- Redirects are never followed. A 3xx is a refusal, because a redirect is the
  provider changing the destination after the allowlist check passed.
- The response is read with a hard byte ceiling, one chunk at a time, so an
  endless body cannot exhaust memory.
- Retries happen only for a transport failure or a 5xx, and never on a 4xx.
  The caller decides which of its own requests may be retried and passes
  `retry_safe`; this module never decides that for it. `client._call` marks
  GET and POST retry-safe because every route it calls is a stateless
  computation the provider documents as its own fault on a 5xx.
- `config.read_timeout` is the only timeout. urllib sets one socket timeout,
  which bounds the connection and each read, so a separate connect timeout
  would be a number nothing enforced.
- Nothing here logs a request or response body.
"""

from __future__ import annotations

import http.client
import sys
import time
import urllib.error
import urllib.request
from collections.abc import Mapping
from dataclasses import dataclass
from typing import BinaryIO, Literal, NoReturn, Protocol

from .errors import ConfigurationError, DisallowedTargetError, TransportError


class _RequestConfig(Protocol):
    @property
    def read_timeout(self) -> float: ...
    @property
    def max_response_bytes(self) -> int: ...
    @property
    def max_attempts(self) -> int: ...
    @property
    def retry_backoff_seconds(self) -> float: ...
    @property
    def user_agent(self) -> str: ...
    @property
    def extra_headers(self) -> Mapping[str, str]: ...
    def require_enabled(self) -> None: ...
    def check_url(self, url: str, *, require_route: bool = True) -> str: ...


@dataclass(frozen=True)
class RawResponse:
    """What came back, before anything interprets it."""

    status: int
    headers: dict[str, str]
    body: bytes
    url: str
    elapsed_ms: int

    def text(self) -> str:
        try:
            return self.body.decode("utf-8")
        except UnicodeDecodeError:
            pass
        del self
        _raise_detached(TransportError("response is not UTF-8"))


def _raise_detached(error: TransportError | DisallowedTargetError) -> NoReturn:
    """Detach ambient caller context as well as handled upstream failures."""
    try:
        raise error
    except (TransportError, DisallowedTargetError):
        error.__cause__ = error.__context__ = None
        raise


_IO_ERRORS = (OSError, http.client.HTTPException, ValueError)


@dataclass(frozen=True, slots=True)
class _AttemptFailure:
    kind: Literal["open", "read", "ceiling", "close", "redirect"]
    status: int | None = None


class _RedirectRefused(Exception):
    def __init__(self, status: int) -> None:
        self.status = status
        super().__init__()


class _ResponseFailed(http.client.HTTPException):
    def __init__(self, status: int | None) -> None:
        self.status = status
        super().__init__()


class _HeaderReader:
    """Require a complete header section without buffering or retaining it."""

    def __init__(self, stream: BinaryIO) -> None:
        self.stream = stream

    def readline(self, size: int = -1) -> bytes:
        line = self.stream.readline(size)
        if not line or not line.endswith(b"\n"):
            raise http.client.HTTPException("incomplete headers")
        return line

    def close(self) -> None:
        self.stream.close()


class _HTTPResponse(http.client.HTTPResponse):
    _method: str | None

    def begin(self) -> None:
        original = self.fp
        reader = _HeaderReader(original) if original is not None else None
        if reader is not None:
            self.fp = reader  # type: ignore[assignment]  # Temporary readline/close delegate.
        completed = False
        try:
            super().begin()
            # Method/status framing takes precedence over representation headers.
            if self._method == "HEAD" or self.status in (204, 304) or 100 <= self.status < 200:
                self.chunked = False
                self.length = 0
            elif self.headers.get_all("Transfer-Encoding"):
                codings = [part.strip().lower()
                           for value in self.headers.get_all("Transfer-Encoding", [])
                           for part in value.split(",")]
                if any(not coding for coding in codings) or "chunked" in codings[:-1]:
                    raise ValueError("invalid transfer coding")
                self.chunked = codings[-1] == "chunked"
                self.length = None
                if self.chunked:
                    self.chunk_left = None
                else:
                    self.will_close = True
            else:
                lengths = self.headers.get_all("Content-Length", [])
                if lengths:
                    values = [part.strip() for value in lengths for part in value.split(",")]
                    if any(not value.isascii() or not value.isdecimal() for value in values):
                        raise ValueError("invalid content length")
                    sizes = [int(value) for value in values]
                    if any(size != sizes[0] for size in sizes):
                        raise ValueError("conflicting content lengths")
                    self.length = sizes[0]
            completed = True
        except _IO_ERRORS:
            status = self.status if type(self.status) is int else None
            raise _ResponseFailed(status) from None
        finally:
            self._begin_failed = not completed
            if self.fp is reader:
                self.fp = original

    def close(self) -> None:
        active = sys.exception() if getattr(self, "_begin_failed", False) else None
        preserve_primary = active is not None
        primary_cancelled = preserve_primary and not isinstance(active, Exception)
        del active
        try:
            super().close()
        except Exception:
            # Stdlib getresponse closes while re-raising a failed begin. Keep
            # that primary outcome, including cancellation/programming errors.
            if not preserve_primary:
                raise
        except BaseException:
            # Cancellation raised by cleanup wins unless one is already active.
            if not primary_cancelled:
                raise

    def _read_next_chunk_size(self) -> int:
        max_line = getattr(http.client, "_MAXLINE")
        line = self.fp.readline(max_line + 1)
        if len(line) > max_line:
            raise http.client.LineTooLong("chunk size")
        if not line.endswith(b"\n"):
            raise ValueError("incomplete chunk size")
        token = line.rstrip(b"\r\n").split(b";", 1)[0]
        if not token or any(byte not in b"0123456789abcdefABCDEF" for byte in token):
            raise ValueError("invalid chunk size")
        return int(token, 16)

    def _get_chunk_left(self) -> int | None:
        chunk_left = self.chunk_left
        if not chunk_left:
            if chunk_left is not None:
                delimiter = self._safe_read(2)  # type: ignore[attr-defined]
                if delimiter != b"\r\n":
                    raise http.client.IncompleteRead(b"")
            try:
                chunk_left = self._read_next_chunk_size()
            except ValueError:
                raise http.client.IncompleteRead(b"") from None
            if chunk_left == 0:
                self._read_and_discard_trailer()
                self._close_conn()  # type: ignore[attr-defined]
                chunk_left = None
            self.chunk_left = chunk_left
        return chunk_left

    def _read_and_discard_trailer(self) -> None:
        max_line = getattr(http.client, "_MAXLINE")
        max_headers = getattr(http.client, "_MAXHEADERS")
        for _ in range(max_headers + 1):
            line = self.fp.readline(max_line + 1)
            if len(line) > max_line:
                raise http.client.LineTooLong("trailer line")
            if not line or not line.endswith(b"\n"):
                raise http.client.IncompleteRead(b"")
            if line in (b"\r\n", b"\n"):
                return
        raise http.client.HTTPException("too many trailers")

    def read(self, amt: int | None = None) -> bytes:
        data = super().read(amt)
        # Sized stdlib reads accept EOF with an unsatisfied Content-Length.
        if not data and amt != 0 and self.length is not None and self.length > 0:
            raise http.client.IncompleteRead(b"", self.length)
        return data


class _HTTPConnection(http.client.HTTPConnection):
    response_class = _HTTPResponse

    def getresponse(self) -> http.client.HTTPResponse:
        response = None
        response_class = self.response_class

        def create_response(*args, **kwargs):
            nonlocal response
            response = response_class(*args, **kwargs)
            return response

        self.response_class = create_response  # type: ignore[assignment]
        try:
            return super().getresponse()
        except _IO_ERRORS:
            status = response.status if response is not None else None
            raise _ResponseFailed(status if type(status) is int else None) from None
        finally:
            self.response_class = response_class


class _HTTPSConnection(_HTTPConnection, http.client.HTTPSConnection):
    def connect(self) -> None:
        super().connect()
        # A TLS truncation must not become the normal EOF of a close-delimited body.
        if self.sock is not None and hasattr(self.sock, "suppress_ragged_eofs"):
            self.sock.suppress_ragged_eofs = False


def _open(connection_class, req, **connection_options):
    """Own opener cleanup so a secondary I/O error cannot replace its primary."""
    connection = connection_class(req.host, timeout=req.timeout, **connection_options)
    headers = dict(req.unredirected_hdrs)
    headers.update(req.headers)
    headers["Connection"] = "close"
    headers = {name.title(): value for name, value in headers.items()}
    response = None
    try:
        connection.request(req.get_method(), req.selector, req.data, headers,
                           encode_chunked=req.has_header("Transfer-encoding"))
        response = connection.getresponse()
        if connection.sock is not None:
            connection.sock.close()
            connection.sock = None
    except BaseException as error:
        # Cleanup only: cancellation and unrelated programming errors propagate.
        # An ordinary cleanup failure never replaces the primary; cancellation
        # raised by cleanup does, unless the primary is already a cancellation.
        cancellation = None
        releases = (connection.close,) if response is None else (connection.close, response.close)
        for release in releases:
            try:
                release()
            except Exception:
                pass
            except BaseException as secondary:
                if cancellation is None and isinstance(error, Exception):
                    cancellation = secondary
        if cancellation is not None:
            raise cancellation
        if response is not None and isinstance(error, _IO_ERRORS):
            status = response.status if type(response.status) is int else None
            raise _ResponseFailed(status) from None
        raise
    response.url = req.get_full_url()
    response.msg = response.reason
    return response


class _HTTPHandler(urllib.request.HTTPHandler):
    def http_open(self, req):  # noqa: ANN001, ANN201
        return _open(_HTTPConnection, req)


class _HTTPSHandler(urllib.request.HTTPSHandler):
    def https_open(self, req):  # noqa: ANN001, ANN201
        return _open(_HTTPSConnection, req, context=self._context)


class _NoRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):  # noqa: ANN001, ANN201
        raise _RedirectRefused(code)

    def http_error_302(self, req, fp, code, msg, headers):  # noqa: ANN001, ANN201
        try:
            super().http_error_302(req, fp, code, msg, headers)
        except (_RedirectRefused, ValueError):
            pass
        # Refusal also owns a response without Location or URI. Inherited
        # HTTPError paths still hand theirs to the bounded consumer.
        try:
            fp.close()
        except Exception:
            pass
        raise _RedirectRefused(code)

    http_error_301 = http_error_302
    http_error_303 = http_error_302
    http_error_307 = http_error_302
    http_error_308 = http_error_302


def _opener() -> urllib.request.OpenerDirector:
    # No cookie handling, no proxy, no redirects. build_opener adds a
    # ProxyHandler that reads the environment unless one is supplied, so the
    # empty one below is what makes "no proxy" true: without it an https_proxy
    # variable routed the call through a host the allowlist never saw.
    return urllib.request.build_opener(
        urllib.request.ProxyHandler({}),
        _NoRedirects(),
        _HTTPSHandler(),
        _HTTPHandler(),
    )


def _read_bounded(response, limit: int) -> bytes | None:
    chunks: list[bytes] = []
    total = 0
    while True:
        chunk = response.read(min(65536, limit + 1 - total))
        if not chunk:
            break
        total += len(chunk)
        if total > limit:
            return None
        chunks.append(chunk)
    return b"".join(chunks)


def _consume(response, limit: int, url: str, started: float) -> RawResponse | _AttemptFailure:
    """Finish and close an owned response before returning data or scalar failure."""
    status: int | None = None
    failure: _AttemptFailure | None = None
    payload: bytes | None = None
    headers: dict[str, str] = {}
    preserve_primary = primary_cancelled = False
    try:
        try:
            status = response.status
            payload = _read_bounded(response, limit)
            if payload is None:
                failure = _AttemptFailure("ceiling", status)
            else:
                headers = {key.lower(): value for key, value in response.headers.items()}
        except _IO_ERRORS:
            failure = _AttemptFailure("read", status)
    except Exception:
        preserve_primary = True
        raise
    except BaseException:
        preserve_primary = primary_cancelled = True
        raise
    finally:
        try:
            response.close()
        except _IO_ERRORS:
            if failure is None:
                failure = _AttemptFailure("close", status)
        except Exception:
            # A recorded read or ceiling failure is the primary outcome.
            if not preserve_primary and failure is None:
                raise
        except BaseException:
            # Cancellation raised by close wins unless one is already active.
            if not primary_cancelled:
                raise
    if failure is not None:
        return failure
    if status is None or payload is None:
        return _AttemptFailure("read", status)
    return RawResponse(status, headers, payload, url, int((time.monotonic() - started) * 1000))


def _attempt(
    method: str, url: str, body: bytes | None, headers: dict[str, str],
    timeout: float, limit: int,
) -> RawResponse | _AttemptFailure:
    started = time.monotonic()
    try:
        request_object = urllib.request.Request(url, data=body, method=method, headers=headers)
        response = _opener().open(request_object, timeout=timeout)
    except urllib.error.HTTPError as error:
        return _consume(error, limit, url, started)
    except _RedirectRefused as refusal:
        return _AttemptFailure("redirect", refusal.status)
    except _ResponseFailed as failure:
        return _AttemptFailure("open", failure.status)
    except _IO_ERRORS:
        return _AttemptFailure("open")
    return _consume(response, limit, url, started)


def request(
    config: _RequestConfig,
    method: str,
    url: str,
    *,
    body: bytes | None = None,
    content_type: str | None = None,
    retry_safe: bool = False,
    bearer_token: str | None = None,
    sleep=time.sleep,
) -> RawResponse:
    """One HTTP request, with every bound the config sets.

    Returns a `RawResponse` for any status the provider answers with, including
    4xx and 5xx: deciding what a status means belongs to the caller, not here.
    Raises `TransportError` when a response cannot be completed, and
    `DisallowedTargetError` when the destination is not allowed.
    """
    config.require_enabled()
    config.check_url(url)
    headers = {
        "Accept": "application/json",
        "User-Agent": config.user_agent,
        **config.extra_headers,
    }
    if bearer_token is not None:
        import re

        if (
            type(bearer_token) is not str
            or len(bearer_token) > 8192
            or not re.fullmatch(r"[A-Za-z0-9._~+/-]+=*", bearer_token)
            or any(key.lower() == "authorization" for key in config.extra_headers)
        ):
            raise ConfigurationError("Invalid per-call bearer authentication.")
        headers["Authorization"] = "Bearer " + bearer_token
    if content_type:
        headers["Content-Type"] = content_type
    attempts = config.max_attempts if retry_safe else 1
    limit = config.max_response_bytes
    for attempt in range(1, attempts + 1):
        result = _attempt(method, url, body, headers, config.read_timeout, limit)
        status = result.status
        server_error = status is not None and status >= 500
        if isinstance(result, RawResponse):
            if not server_error or attempt == attempts:
                return result
            # A discarded server response must not survive backoff or a later
            # failed attempt. Only request inputs are needed for another send.
            del result
        else:
            failure = result
            eligible = failure.kind != "redirect" and (
                server_error or status is None
                or (200 <= status < 300 and failure.kind != "ceiling")
            )
            if not eligible or attempt == attempts:
                break
            del result, failure
        sleep(config.retry_backoff_seconds * attempt)
    # The attempt has returned without retaining its exception or resource.
    # Drop direct and indirect request carriers before the public traceback.
    del config, method, url, body, content_type, bearer_token, headers, sleep, result
    if failure.kind == "redirect":
        _raise_detached(DisallowedTargetError("provider redirected; redirects are refused"))
    if failure.kind == "ceiling":
        message = f"response exceeded the {limit} byte ceiling and was abandoned"
    elif failure.status is None:
        message = f"no response after {attempt} attempt(s)"
    elif failure.kind == "close":
        message = f"HTTP {failure.status} response could not be closed after {attempt} attempt(s)"
    else:
        message = f"HTTP {failure.status} body could not be read after {attempt} attempt(s)"
    _raise_detached(TransportError(message))
