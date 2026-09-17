"""Failure kinds, kept apart from one another on purpose.

A caller has to be able to tell "the provider refused" from "the provider is
broken" from "we refused to send". Collapsing them is how a timeout becomes a
zero.
"""

from __future__ import annotations


class AdapterError(Exception):
    """Base for every error this package raises."""


class NotEnabledError(AdapterError):
    """Remote access was not explicitly enabled, so nothing was sent."""


class ConfigurationError(AdapterError):
    """The configuration is unusable: bad host, bad limit, missing endpoint."""


class DisallowedTargetError(AdapterError):
    """A URL was not on the configured allowlist, or was not a permitted shape.

    Raised before any socket is opened. Covers an unexpected redirect, a schema
    URL fetched from a response body, a non-HTTPS scheme outside loopback and a
    host that resolves somewhere the configuration does not allow.
    """


class TransportError(AdapterError):
    """The request could not be completed: timeout, connection, oversized body.

    Never a result. A caller that turns this into a number has lost the
    distinction the class exists for.
    """


class ContractError(AdapterError):
    """A response arrived but does not match the reviewed contract.

    A missing manifest, a missing advisory, an unknown response schema, a money
    field that is not an exact decimal, or a period the snapshot does not
    record. Distinct from an upstream refusal: the provider believes it
    answered, and we do not accept the answer.
    """


class UpstreamError(AdapterError):
    """The provider reported an error. Its own status and body are preserved.

    `status` is the HTTP status. `payload` is the provider's body, unaltered.
    `refusal_class` is the provider's own refusal token when it sent one, never
    a token this package invented.
    """

    def __init__(
        self,
        message: str,
        *,
        status: int,
        payload: object = None,
        refusal_class: str | None = None,
    ) -> None:
        super().__init__(message)
        self.status = status
        self.payload = payload
        self.refusal_class = refusal_class
