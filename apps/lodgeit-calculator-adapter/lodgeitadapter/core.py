"""Opt-in, single-page client catalogue reads against the pinned Core preview."""

from __future__ import annotations

import http.client
import json
from collections.abc import Mapping
from dataclasses import dataclass, field
from enum import Enum
from types import MappingProxyType
from typing import Any
from uuid import UUID

from . import transport
from .errors import ConfigurationError, DisallowedTargetError, NotEnabledError, TransportError

CORE_URL = "https://api.lodgeit.com/core/clients/v1-preview/list"
CONTRACT_SNAPSHOT = "lodgeit-core-clients-2026-10-09"


class CoreCategory(str, Enum):
    DISABLED = "DISABLED"
    INVALID_INPUT = "INVALID_INPUT"
    TRANSPORT = "TRANSPORT"
    AUTHENTICATION = "AUTHENTICATION"
    AUTHORISATION = "AUTHORISATION"
    REQUEST_REJECTED = "REQUEST_REJECTED"
    THROTTLED = "THROTTLED"
    UPSTREAM = "UPSTREAM"
    PROTOCOL = "PROTOCOL"


class CoreError(Exception):
    """Fixed category and status only; no upstream body or exception context."""

    def __init__(self, category: CoreCategory, http_status: int | None = None) -> None:
        self.category = category
        self.http_status = http_status
        super().__init__(category.value)


@dataclass(frozen=True, slots=True)
class CoreConfig:
    enabled: bool = False
    read_timeout: float = 20.0
    max_response_bytes: int = 1024 * 1024

    def __post_init__(self) -> None:
        if (
            type(self.enabled) is not bool
            or type(self.read_timeout) not in (int, float)
            or not 0 < self.read_timeout <= 60
            or type(self.max_response_bytes) is not int
            or not 1 <= self.max_response_bytes <= 1024 * 1024
        ):
            raise ConfigurationError("Invalid Core safety limits.")

    @property
    def max_attempts(self) -> int:
        return 1

    @property
    def retry_backoff_seconds(self) -> float:
        return 1.0

    @property
    def user_agent(self) -> str:
        return "lodgeit-core-client-preview (australian-accounting)"

    @property
    def extra_headers(self) -> Mapping[str, str]:
        return MappingProxyType({})

    def require_enabled(self) -> None:
        if not self.enabled:
            raise NotEnabledError("Core access is disabled.")

    def check_url(self, url: str, *, require_route: bool = True) -> str:
        if url != CORE_URL:
            raise DisallowedTargetError("Core destination is outside the fixed read route.")
        return url


@dataclass(frozen=True, slots=True, repr=False)
class CoreContinuation:
    value: str = field(repr=False)

    def __repr__(self) -> str:
        return "CoreContinuation(<redacted>)"


@dataclass(frozen=True, slots=True, repr=False)
class CoreClientSummary:
    id: str
    code: str | None
    archived: bool
    private: bool

    def __repr__(self) -> str:
        return "CoreClientSummary(<redacted>)"


@dataclass(frozen=True, slots=True, repr=False)
class CoreClientPage:
    items: tuple[CoreClientSummary, ...]
    continuation: CoreContinuation | None

    def __repr__(self) -> str:
        return "CoreClientPage(<redacted>)"


def _text(value: Any, limit: int) -> bool:
    if type(value) is not str:
        return False
    try:
        return len(value.encode("utf-8")) <= limit
    except UnicodeError:
        return False


def _parse_page(body: bytes, size: int) -> CoreClientPage | None:
    def pairs(items: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in items:
            if key in result:
                raise ValueError("Duplicate JSON key.")
            result[key] = value
        return result

    def constant(value: str) -> None:
        raise ValueError("Non-standard JSON number.")

    try:
        data = json.loads(
            body.decode("utf-8"),
            object_pairs_hook=pairs,
            parse_constant=constant,
            parse_float=constant,
        )
        if (
            not isinstance(data, dict)
            or not {"items"}.issubset(data)
            or not set(data).issubset({"items", "nextPageToken"})
            or not isinstance(data["items"], list)
            or len(data["items"]) > size
        ):
            return None
        summaries: list[CoreClientSummary] = []
        identifiers: set[str] = set()
        for item in data["items"]:
            if (
                not isinstance(item, dict)
                or not {"id", "archived", "private", "profile"}.issubset(item)
                or not set(item).issubset(
                    {
                        "id",
                        "code",
                        "archived",
                        "private",
                        "profile",
                        "assignments",
                        "groups",
                        "relations",
                    }
                )
                or type(item["id"]) is not str
                or str(UUID(item["id"])) != item["id"]
                or item["id"] in identifiers
                or item["archived"] is not False
                or type(item["private"]) is not bool
                or any(
                    item.get(key) is not None
                    for key in ("profile", "assignments", "groups", "relations")
                )
                or (item.get("code") is not None and not _text(item["code"], 1024))
            ):
                return None
            identifiers.add(item["id"])
            summaries.append(
                CoreClientSummary(item["id"], item.get("code"), False, item["private"])
            )
        token = data.get("nextPageToken")
        if token is not None and (not _text(token, 8192) or not token):
            return None
        return CoreClientPage(
            tuple(summaries), CoreContinuation(token) if token is not None else None
        )
    except (ValueError, UnicodeError, RecursionError, TypeError):
        return None


class CoreClient:
    def __init__(self, config: CoreConfig | None = None) -> None:
        self.config = CoreConfig() if config is None else config
        if type(self.config) is not CoreConfig:
            raise ConfigurationError("CoreClient requires CoreConfig.")

    def list_clients(
        self,
        *,
        bearer_token: str,
        page_size: int = 100,
        continuation: CoreContinuation | None = None,
    ) -> CoreClientPage:
        body: bytes | None = None
        raw: transport.RawResponse | None = None
        try:
            if not self.config.enabled:
                raise CoreError(CoreCategory.DISABLED)
            if (
                type(bearer_token) is not str
                or type(page_size) is not int
                or not 1 <= page_size <= 500
                or (
                    continuation is not None
                    and (
                        type(continuation) is not CoreContinuation
                        or not _text(continuation.value, 8192)
                        or not continuation.value
                    )
                )
            ):
                raise CoreError(CoreCategory.INVALID_INPUT)
            body = json.dumps(
                {
                    "archived": False,
                    "expansion": None,
                    "paging": {
                        "size": page_size,
                        "token": None if continuation is None else continuation.value,
                    },
                }
            ).encode("utf-8")
            raw = None
            failure = CoreCategory.TRANSPORT
            try:
                raw = transport.request(
                    self.config,
                    "POST",
                    CORE_URL,
                    body=body,
                    content_type="application/json",
                    retry_safe=False,
                    bearer_token=bearer_token,
                )
            except ConfigurationError:
                failure = CoreCategory.INVALID_INPUT
            except (
                TransportError,
                DisallowedTargetError,
                NotEnabledError,
                http.client.HTTPException,
                OSError,
                ValueError,
            ):
                pass
            bearer_token = ""
            if raw is None:
                raise CoreError(failure)
            status = raw.status
            categories = {
                401: CoreCategory.AUTHENTICATION,
                403: CoreCategory.AUTHORISATION,
                400: CoreCategory.REQUEST_REJECTED,
                422: CoreCategory.REQUEST_REJECTED,
                429: CoreCategory.THROTTLED,
            }
            if status != 200:
                failure = categories.get(
                    status, CoreCategory.UPSTREAM if status >= 500 else CoreCategory.PROTOCOL
                )
                raw = None
                raise CoreError(failure, status)
            page = (
                _parse_page(raw.body, page_size)
                if raw.headers.get("content-type", "").split(";", 1)[0].strip().lower()
                == "application/json"
                else None
            )
            raw = None
            if page is None:
                raise CoreError(CoreCategory.PROTOCOL, status)
            return page
        finally:
            bearer_token = ""
            continuation = None
            body = None
            raw = None
