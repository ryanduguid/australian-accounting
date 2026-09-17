"""Configuration, and the rule that nothing leaves the machine by default.

Three separate things have to be true before a request is sent:

1. remote access is explicitly enabled (`enabled=True`, or
   `LODGEIT_ADAPTER_ENABLED=1` when building from the environment);
2. a base URL is configured;
3. that base URL's host is on the allowlist.

None of them has a default that reaches the network. Importing this package,
constructing a config, running discovery in offline mode and recovering from an
error all do nothing over the network, and the test suite asserts that.
"""

from __future__ import annotations

import math
import os
from dataclasses import dataclass, field, replace
from urllib.parse import urlsplit

from .errors import ConfigurationError, DisallowedTargetError

#: Hosts this adapter is built to talk to. Configuration may narrow this set,
#: never widen it beyond what the caller passes as `allowed_hosts`.
KNOWN_HOSTS = (
    "fbt-calculator-api-qkp3j5bjnq-ts.a.run.app",
    "fano-engine-79859053141.australia-southeast1.run.app",
    "fano-engine-afmurhqkaq-ts.a.run.app",
)
LOOPBACK_HOSTS = ("127.0.0.1", "localhost", "::1", "[::1]")

#: Route prefixes a caller may reach. A path outside these is refused before a
#: socket is opened, so a URL taken from a response body cannot become a
#: request.
ALLOWED_PATH_PREFIXES = (
    "/v1/calculators",
    "/v1/rates",
    "/openapi.json",
    "/healthz",
    "/livez",
    "/v1/info",
    "/ingest/trial_balance",
)


@dataclass(frozen=True)
class AdapterConfig:
    """Everything the transport is allowed to do."""

    enabled: bool = False
    base_url: str | None = None
    allowed_hosts: tuple[str, ...] = KNOWN_HOSTS
    allow_loopback: bool = False
    connect_timeout: float = 5.0
    read_timeout: float = 20.0
    max_response_bytes: int = 2 * 1024 * 1024
    max_attempts: int = 2
    retry_backoff_seconds: float = 1.0
    user_agent: str = "lodgeit-calculator-adapter (https://github.com/ryanduguid/australian-accounting)"
    extra_headers: dict[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        for name in ("connect_timeout", "read_timeout", "retry_backoff_seconds"):
            value = getattr(self, name)
            if (not isinstance(value, (int, float)) or not math.isfinite(value)
                    or value <= 0):
                raise ConfigurationError(f"{name} must be a positive number")
        if self.max_response_bytes <= 0:
            raise ConfigurationError("max_response_bytes must be positive")
        if not 1 <= self.max_attempts <= 5:
            raise ConfigurationError("max_attempts must be between 1 and 5")
        if self.base_url is not None:
            # A base URL carries no route of its own, so the route check is for
            # the request URLs built from it, not for this.
            self.check_url(self.base_url, require_route=False)

    def with_base_url(self, base_url: str) -> "AdapterConfig":
        return replace(self, base_url=base_url)

    def require_enabled(self) -> None:
        from .errors import NotEnabledError

        if not self.enabled:
            raise NotEnabledError(
                "Remote access is not enabled. This adapter never contacts a service "
                "unless it is switched on deliberately: pass enabled=True, or set "
                "LODGEIT_ADAPTER_ENABLED=1."
            )
        if not self.base_url:
            raise ConfigurationError("No base URL is configured; there is nothing to call.")

    def check_url(self, url: str, *, require_route: bool = True) -> str:
        """Refuse a URL the configuration does not allow. No network here.

        The checks are on the URL itself, before resolution: scheme, host,
        port, path prefix, and the absence of credentials. A host is compared
        literally against the allowlist, so an attacker-supplied name that
        happens to resolve to an allowed address is still refused.
        """
        parts = urlsplit(url)
        if parts.username or parts.password:
            raise DisallowedTargetError(f"{url}: credentials in a URL are refused")
        host = (parts.hostname or "").lower()
        if not host:
            raise DisallowedTargetError(f"{url}: no host")
        loopback = host in LOOPBACK_HOSTS
        if loopback:
            if not self.allow_loopback:
                raise DisallowedTargetError(
                    f"{url}: loopback is only reachable when allow_loopback is set, which is "
                    "for tests against a local stub."
                )
        else:
            if parts.scheme != "https":
                raise DisallowedTargetError(f"{url}: only https is allowed off loopback")
            if host not in {name.lower() for name in self.allowed_hosts}:
                raise DisallowedTargetError(
                    f"{url}: host {host} is not on the configured allowlist "
                    f"({', '.join(self.allowed_hosts) or 'empty'})"
                )
        if parts.scheme not in ("https", "http"):
            raise DisallowedTargetError(f"{url}: scheme {parts.scheme!r} is refused")
        if parts.scheme == "http" and not loopback:
            raise DisallowedTargetError(f"{url}: plaintext http is refused off loopback")
        path = parts.path or "/"
        if require_route and not any(path.startswith(prefix) for prefix in ALLOWED_PATH_PREFIXES):
            raise DisallowedTargetError(
                f"{url}: path {path} is not one of the routes this adapter calls. A URL "
                "found in a response body is data, not a destination."
            )
        return url


def from_environment(environ: dict[str, str] | None = None) -> AdapterConfig:
    """Build a config from the environment. Still disabled unless asked."""
    env = os.environ if environ is None else environ
    hosts = env.get("LODGEIT_ADAPTER_ALLOWED_HOSTS")
    allowed: tuple[str, ...] = KNOWN_HOSTS
    if hosts:
        allowed = tuple(part.strip() for part in hosts.split(",") if part.strip())

    def number(name: str, fallback: float) -> float:
        raw = env.get(name)
        if raw is None or raw == "":
            return fallback
        try:
            return float(raw)
        except ValueError as exc:
            raise ConfigurationError(f"{name}={raw!r} is not a number") from exc

    return AdapterConfig(
        enabled=env.get("LODGEIT_ADAPTER_ENABLED") == "1",
        base_url=env.get("LODGEIT_ADAPTER_BASE_URL") or None,
        allowed_hosts=allowed,
        allow_loopback=env.get("LODGEIT_ADAPTER_ALLOW_LOOPBACK") == "1",
        connect_timeout=number("LODGEIT_ADAPTER_CONNECT_TIMEOUT", 5.0),
        read_timeout=number("LODGEIT_ADAPTER_READ_TIMEOUT", 20.0),
        max_response_bytes=int(number("LODGEIT_ADAPTER_MAX_RESPONSE_BYTES", 2 * 1024 * 1024)),
        max_attempts=int(number("LODGEIT_ADAPTER_MAX_ATTEMPTS", 2)),
    )
