"""The adapter itself: discovery and bounded invocation, with honest outcomes.

Every call returns an `Outcome` whose `status` says what happened. There is no
path from a timeout, a malformed body, a missing advisory or an unreviewed
schema to a number. `Outcome.result` is populated only for `COMPUTED`.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from datetime import datetime, timezone
from decimal import Decimal
from enum import Enum
from typing import Any

from . import decimals, transport
from .config import AdapterConfig
from .contract import Contract
from .decimals import DecimalWireError
from .errors import (
    ConfigurationError,
    ContractError,
    DisallowedTargetError,
    NotEnabledError,
    TransportError,
)


class Status(str, Enum):
    """What happened to one call. Not an accounting verdict."""

    COMPUTED = "COMPUTED"
    UPSTREAM_REFUSED = "UPSTREAM_REFUSED"        # provider 400, its own refusal_class
    UPSTREAM_REJECTED = "UPSTREAM_REJECTED"      # provider 422, our request was malformed
    UPSTREAM_NOT_FOUND = "UPSTREAM_NOT_FOUND"    # provider 404, unknown calculator or period
    UPSTREAM_THROTTLED = "UPSTREAM_THROTTLED"    # provider 429
    UPSTREAM_UNAVAILABLE = "UPSTREAM_UNAVAILABLE"  # 5xx, timeout, connection failure
    CONTRACT_FAILURE = "CONTRACT_FAILURE"        # answered, but not in a shape we accept
    REFUSED_TO_SEND = "REFUSED_TO_SEND"          # not enabled, or the target was not allowed

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True)
class Outcome:
    """One call, with everything needed to audit it later."""

    status: Status
    calculator: str | None = None
    period: str | None = None
    request_body: dict | None = None
    request_json: str | None = None
    raw_response: object = None
    http_status: int | None = None
    result: dict | None = None
    values: dict[str, Decimal] = field(default_factory=dict)
    manifest: dict | None = None
    advisory: dict | None = None
    upstream_refusal_class: str | None = None
    findings: tuple[str, ...] = ()
    observed_at: str | None = None
    elapsed_ms: int | None = None
    contract_snapshot: str | None = None
    contract_sha256: str | None = None
    base_url: str | None = None

    @property
    def computed(self) -> bool:
        return self.status is Status.COMPUTED


def _now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def _refuse_constant(name: str) -> Any:
    raise ValueError(f"response contains the JSON-invalid constant {name}")


class LodgeitClient:
    """A thin, bounded client for one provider surface.

    Constructing one does nothing over the network. Every method that would
    send checks `config.require_enabled()` first, through the transport.
    """

    def __init__(self, config: AdapterConfig, contract: Contract) -> None:
        self.config = config
        self.contract = contract

    # -- discovery -------------------------------------------------------

    def discover(self) -> Outcome:
        """Fetch the live calculator listing and compare it with the snapshot.

        Offline, this raises `NotEnabledError` through the transport rather
        than returning a cached listing dressed up as a live one. The snapshot
        is readable without this call; see `contract.load`.
        """
        return self._call("GET", "/v1/calculators", retry_safe=True, expect_contract=False)

    def drift(self) -> tuple[Outcome, list[str]]:
        """Discovery plus the findings against the reviewed snapshot."""
        from .contract import compare

        outcome = self.discover()
        if not outcome.computed or not isinstance(outcome.result, list):
            return outcome, []
        return outcome, compare(self.contract, outcome.result)

    # -- invocation ------------------------------------------------------

    def invoke(self, calc_uri: str, period_uri: str, body: dict[str, Any]) -> Outcome:
        """Call one calculator for one period, with the contract checks around it."""
        try:
            self.contract.check_period(calc_uri, period_uri)
        except ContractError as exc:
            return Outcome(
                status=Status.CONTRACT_FAILURE, calculator=calc_uri, period=period_uri,
                findings=(str(exc),), observed_at=_now(),
                contract_snapshot=self.contract.snapshot_id, contract_sha256=self.contract.sha256,
            )
        route = self.contract.calculator(calc_uri).get("route")
        if not route:
            return Outcome(
                status=Status.CONTRACT_FAILURE, calculator=calc_uri, period=period_uri,
                findings=(f"snapshot records no route for {calc_uri}",), observed_at=_now(),
                contract_snapshot=self.contract.snapshot_id, contract_sha256=self.contract.sha256,
            )
        path = route.replace("{period_uri}", period_uri).replace("{calc_uri}", calc_uri)
        return self._call("POST", path, body=body, calc_uri=calc_uri, period_uri=period_uri)

    # -- the one place a request is made ---------------------------------

    def _call(
        self,
        method: str,
        path: str,
        *,
        body: dict | None = None,
        calc_uri: str | None = None,
        period_uri: str | None = None,
        retry_safe: bool = False,
        expect_contract: bool = True,
    ) -> Outcome:
        base = (self.config.base_url or "").rstrip("/")
        url = f"{base}{path}"
        # One bag of fields every Outcome on this path carries, so a branch
        # cannot quietly drop the provenance. Typed as Any because it is
        # splatted into a dataclass with mixed field types.
        common: dict[str, Any] = dict(
            calculator=calc_uri, period=period_uri, observed_at=_now(),
            contract_snapshot=self.contract.snapshot_id, contract_sha256=self.contract.sha256,
            base_url=self.config.base_url,
        )
        encoded = None
        payload_bytes = None
        if body is not None:
            try:
                encoded = decimals.dumps(body)
            except DecimalWireError as exc:
                return Outcome(status=Status.CONTRACT_FAILURE, request_body=body,
                               findings=(
                                   f"request could not be serialised exactly: {exc}",
                                   ), **common,
                               )
            payload_bytes = encoded.encode("utf-8")
        common["request_body"] = body
        common["request_json"] = encoded

        try:
            raw = transport.request(
                self.config, method, url, body=payload_bytes,
                content_type="application/json" if payload_bytes else None,
                # These calculators are stateless computations and the provider
                # documents a 5xx as its own fault and reasonable to retry, so a
                # POST is retried on a 5xx or a transport failure. The transport
                # never retries a 4xx: that is the provider saying the request
                # itself is wrong, and repeating it cannot change the answer.
                retry_safe=retry_safe or method in ("GET", "POST"),
            )
        except (NotEnabledError, ConfigurationError, DisallowedTargetError) as exc:
            return Outcome(status=Status.REFUSED_TO_SEND, findings=(str(exc),), **common)
        except TransportError as exc:
            return Outcome(status=Status.UPSTREAM_UNAVAILABLE, findings=(str(exc),), **common)

        common["elapsed_ms"] = raw.elapsed_ms
        common["http_status"] = raw.status
        try:
            # parse_float and parse_int keep the provider's own digits: a JSON
            # number becomes the text it was written as, so a money field
            # declared as a number still reaches parse_decimal exactly.
            # parse_constant refuses NaN and Infinity, which are not JSON and
            # have no place in a money field.
            parsed = json.loads(
                raw.text(), parse_float=str, parse_int=str,
                parse_constant=_refuse_constant,
            )
        except (TransportError, json.JSONDecodeError, ValueError) as exc:
            return Outcome(
                status=Status.CONTRACT_FAILURE, raw_response=raw.body[:2048].decode(
                    "utf-8",
                    "replace"),
                findings=(f"response body is not JSON: {exc}",), **common,
            )
        common["raw_response"] = parsed

        if raw.status >= 500:
            return Outcome(status=Status.UPSTREAM_UNAVAILABLE,
                           findings=(f"provider returned {raw.status}",), **common)
        if raw.status == 429:
            return Outcome(status=Status.UPSTREAM_THROTTLED,
                           findings=(f"provider returned 429; Retry-After: "
                                     f"{raw.headers.get('retry-after', 'absent')}",), **common)
        if raw.status == 404:
            return Outcome(status=Status.UPSTREAM_NOT_FOUND,
                           findings=(f"provider returned 404 for {path}",), **common)
        if raw.status == 422:
            return Outcome(status=Status.UPSTREAM_REJECTED,
                           findings=("provider returned 422: our request was malformed. "
                                     "Do not retry unchanged.",), **common)
        if raw.status == 400:
            refusal = parsed.get("refusal_class") if isinstance(parsed, dict) else None
            return Outcome(
                status=Status.UPSTREAM_REFUSED, upstream_refusal_class=refusal,
                findings=(
                    f"provider refused: {refusal or 'no refusal_class in the body'}",
                    ), **common,
            )
        if raw.status != 200:
            return Outcome(status=Status.CONTRACT_FAILURE,
                           findings=(f"unexpected status {raw.status}",), **common)

        if not expect_contract:
            return Outcome(status=Status.COMPUTED, result=parsed, **common)
        return self._accept(parsed, calc_uri, common)

    def _accept(self, parsed: object, calc_uri: str | None, common: dict) -> Outcome:
        """Check a 200 against the reviewed contract before calling it a result."""
        findings: list[str] = []
        if not isinstance(parsed, dict):
            return Outcome(status=Status.CONTRACT_FAILURE,
                           findings=("a 200 body that is not a JSON object",), **common)
        rules = self.contract.response_contract
        recorded = self.contract.calculators.get(calc_uri or "", {})

        manifest = parsed.get("manifest")
        advisory = parsed.get("advisory")
        if rules.get("requires_manifest", True) and not isinstance(manifest, dict):
            findings.append("no manifest block: the response does not name what it consumed")
        if rules.get("requires_advisory", True):
            if not isinstance(advisory, dict) or not advisory.get("notes"):
                findings.append(
                    "no advisory block: the provider's own boundary statement is missing, so "
                    "the figure would travel without the framing it needs"
                )

        unknown = sorted(set(parsed) - set(rules.get("known_top_level_fields", [])))
        if unknown and rules.get("refuse_unknown_fields", False):
            listed = ", ".join(unknown)
            findings.append(f"response carries fields the snapshot does not record: {listed}")
        elif unknown:
            findings.append(f"note: response carries unrecorded fields: {', '.join(unknown)}")

        # A required field's absence is a contract failure. An optional field
        # is parsed when it is there and ignored when it is not, because some
        # of the provider's fields only appear in some outcomes: a Division 7A
        # response has no deemed dividend when nothing fell short.
        values: dict[str, Decimal] = {}
        required = recorded.get("required_decimal_fields", [])
        for name in [*required, *recorded.get("optional_decimal_fields", [])]:
            if name not in parsed:
                if name in required:
                    findings.append(f"required decimal field {name} is absent")
                continue
            try:
                values[name] = decimals.parse_decimal(parsed[name], name)
            except DecimalWireError as exc:
                findings.append(str(exc))

        blocking = [
            item for item in findings
            if not item.startswith("note:")
        ]
        if blocking:
            return Outcome(status=Status.CONTRACT_FAILURE, result=parsed,
                           manifest=manifest if isinstance(manifest, dict) else None,
                           advisory=advisory if isinstance(advisory, dict) else None,
                           findings=tuple(findings), **common)
        return Outcome(
            status=Status.COMPUTED, result=parsed, values=values,
            manifest=manifest if isinstance(manifest, dict) else None,
            advisory=advisory if isinstance(advisory, dict) else None,
            findings=tuple(findings), **common,
        )
