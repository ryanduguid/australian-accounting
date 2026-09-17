"""An optional, explicitly enabled adapter for the LodgeiT Labs calculators.

Nothing in this package contacts a service on import. A request is sent only
when remote access has been switched on, a base URL is configured, and the
target is on the allowlist. There is no fallback path: no local engine in this
repository calls it, and a local calculation that fails never reaches out.

Start here:

    from lodgeitadapter import AdapterConfig, LodgeitClient, load_contract

    config = AdapterConfig(enabled=True, base_url="https://...")
    client = LodgeitClient(config, load_contract("lodgeit-calculators"))
    outcome = client.invoke(calc_uri, period_uri, body)
    if outcome.computed:
        ...
"""

from __future__ import annotations

from .client import LodgeitClient, Outcome, Status
from .config import AdapterConfig, from_environment
from .contract import Contract
from .contract import compare as compare_contract
from .contract import load as load_contract
from .errors import (
    AdapterError,
    ConfigurationError,
    ContractError,
    DisallowedTargetError,
    NotEnabledError,
    TransportError,
    UpstreamError,
)

__version__ = "0.1.0"

__all__ = [
    "AdapterConfig",
    "AdapterError",
    "ConfigurationError",
    "Contract",
    "ContractError",
    "DisallowedTargetError",
    "LodgeitClient",
    "NotEnabledError",
    "Outcome",
    "Status",
    "TransportError",
    "UpstreamError",
    "__version__",
    "compare_contract",
    "from_environment",
    "load_contract",
]
