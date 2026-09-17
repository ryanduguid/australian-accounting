"""Bounded HTTP, with the guards that keep a request where it was aimed.

- Redirects are never followed. A 3xx is a refusal, because a redirect is the
  provider changing the destination after the allowlist check passed.
- The response is read with a hard byte ceiling, one chunk at a time, so an
  endless body cannot exhaust memory.
- Retries happen only for a transport failure or a 5xx, only on a request the
  provider documents as safe to retry, and never on a 4xx.
- Nothing here logs a request or response body.
"""

from __future__ import annotations

import time
import urllib.error
import urllib.request
from dataclasses import dataclass

from .config import AdapterConfig
from .errors import DisallowedTargetError, TransportError


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
        except UnicodeDecodeError as exc:
            raise TransportError(f"{self.url}: response is not UTF-8") from exc


class _NoRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):  # noqa: ANN001, ANN201
        raise DisallowedTargetError(
            f"{req.full_url}: the provider redirected to {newurl!r}. This adapter does not "
            "follow redirects: the new destination was never checked against the allowlist."
        )


def _opener() -> urllib.request.OpenerDirector:
    # No cookie handling, no proxy auto-detection, no redirects. Building the
    # opener explicitly is what keeps a global default from adding any of them.
    return urllib.request.build_opener(
        _NoRedirects(),
        urllib.request.HTTPSHandler(),
        urllib.request.HTTPHandler(),
    )


def _read_bounded(response, limit: int, url: str) -> bytes:
    chunks: list[bytes] = []
    total = 0
    while True:
        chunk = response.read(min(65536, limit + 1 - total))
        if not chunk:
            break
        total += len(chunk)
        if total > limit:
            raise TransportError(
                f"{url}: response exceeded the {limit} byte ceiling and was abandoned"
            )
        chunks.append(chunk)
    return b"".join(chunks)


def request(
    config: AdapterConfig,
    method: str,
    url: str,
    *,
    body: bytes | None = None,
    content_type: str | None = None,
    retry_safe: bool = False,
    sleep=time.sleep,
) -> RawResponse:
    """One HTTP request, with every bound the config sets.

    Returns a `RawResponse` for any status the provider answers with, including
    4xx and 5xx: deciding what a status means belongs to the caller, not here.
    Raises `TransportError` when there is no response at all, and
    `DisallowedTargetError` when the destination is not allowed.
    """
    config.require_enabled()
    config.check_url(url)
    headers = {
        "Accept": "application/json",
        "User-Agent": config.user_agent,
        **config.extra_headers,
    }
    if content_type:
        headers["Content-Type"] = content_type
    attempts = config.max_attempts if retry_safe else 1
    last: Exception | None = None
    for attempt in range(1, attempts + 1):
        started = time.monotonic()
        request_object = urllib.request.Request(url, data=body, method=method, headers=headers)
        try:
            with _opener().open(request_object, timeout=config.read_timeout) as response:
                payload = _read_bounded(response, config.max_response_bytes, url)
                return RawResponse(
                    status=response.status,
                    headers={key.lower(): value for key, value in response.headers.items()},
                    body=payload,
                    url=url,
                    elapsed_ms=int((time.monotonic() - started) * 1000),
                )
        except urllib.error.HTTPError as error:
            # An HTTP error is still an answer. Read it under the same ceiling
            # and hand it back; the caller preserves the provider's own body.
            payload = _read_bounded(error, config.max_response_bytes, url)
            raw = RawResponse(
                status=error.code,
                headers={key.lower(): value for key, value in error.headers.items()},
                body=payload,
                url=url,
                elapsed_ms=int((time.monotonic() - started) * 1000),
            )
            if error.code >= 500 and attempt < attempts:
                last = error
                sleep(config.retry_backoff_seconds * attempt)
                continue
            return raw
        except DisallowedTargetError:
            raise
        except (urllib.error.URLError, OSError, TimeoutError) as error:
            last = error
            if attempt < attempts:
                sleep(config.retry_backoff_seconds * attempt)
                continue
            # The reason, not the payload: a URLError's reason can carry a
            # socket error, which is safe, while a body could carry anything.
            raise TransportError(
                f"{url}: no response after {attempt} attempt(s) ({error})"
            ) from error
    raise TransportError(f"{url}: no response ({last})")
