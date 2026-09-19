"""Reviewed contract snapshots, and what happens when the live one moves.

A snapshot records what a provider's surface looked like when a person read it:
the routes, the calculators, the periods each accepts, and the response fields
this adapter relies on. It is the thing a live response is checked against.

The rule this module exists to enforce: a contract change is reviewed by a
person, never accepted automatically. `compare` reports drift; nothing here
rewrites a snapshot, and no code path blesses one.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path

from .errors import ContractError

_HERE = Path(__file__).resolve().parent
#: Source checkout first, then the copy a built wheel carries inside the
#: package. A snapshot has to travel with the code that checks against it.
CONTRACTS_DIRS = (_HERE.parent / "contracts", _HERE / "contracts")


def contracts_dir() -> Path:
    for candidate in CONTRACTS_DIRS:
        if candidate.is_dir():
            return candidate
    return CONTRACTS_DIRS[0]


@dataclass(frozen=True)
class Contract:
    """One reviewed snapshot."""

    provider: str
    snapshot_id: str
    read_at: str
    read_by: str
    source_urls: tuple[str, ...]
    calculators: dict[str, dict]
    response_contract: dict
    notes: tuple[str, ...]
    sha256: str
    path: Path

    def calculator(self, calc_uri: str) -> dict:
        try:
            return self.calculators[calc_uri]
        except KeyError:
            raise ContractError(
                f"{calc_uri} is not in reviewed snapshot {self.snapshot_id}. Calling a "
                "calculator nobody has reviewed is how an unknown response shape becomes a "
                "number. Review it and add it to the snapshot first."
            ) from None

    def check_period(self, calc_uri: str, period_uri: str) -> None:
        supported = self.calculator(calc_uri).get("supported_periods", [])
        if period_uri not in supported:
            raise ContractError(
                f"{calc_uri} does not accept {period_uri} in snapshot {self.snapshot_id}. "
                f"It accepts: {', '.join(supported) or 'nothing recorded'}."
            )


def load(name: str, *, directory: Path | None = None) -> Contract:
    path = (directory or contracts_dir()) / f"{name}.json"
    try:
        payload = path.read_bytes()
    except OSError as exc:
        raise ContractError(f"contract snapshot {path} could not be read: {exc}") from exc
    try:
        data = json.loads(payload.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ContractError(f"contract snapshot {path} is not valid JSON: {exc}") from exc
    required = {
        "provider",
        "snapshot_id",
        "read_at",
        "read_by",
        "source_urls",
        "calculators",
        "response_contract",
    }
    missing = required - set(data)
    if missing:
        raise ContractError(f"contract snapshot {path} is missing {', '.join(sorted(missing))}")
    return Contract(
        provider=data["provider"],
        snapshot_id=data["snapshot_id"],
        read_at=data["read_at"],
        read_by=data["read_by"],
        source_urls=tuple(data["source_urls"]),
        calculators=data["calculators"],
        response_contract=data["response_contract"],
        notes=tuple(data.get("notes", [])),
        sha256=hashlib.sha256(payload).hexdigest(),
        path=path,
    )


def compare(contract: Contract, live_listing: list[dict]) -> list[str]:
    """Differences between a reviewed snapshot and a live discovery response.

    Returns findings. It does not raise, does not decide and does not update
    anything: a person reads the findings and edits the snapshot, or does not.
    """
    findings: list[str] = []
    live: dict[str, dict] = {}
    # A calc_uri listed twice is drift whatever the two entries say: a plain
    # dict assignment kept whichever came last, so a conflicting duplicate
    # ahead of a matching one printed agreement and the reverse order did not.
    repeated: set[str] = set()
    for index, entry in enumerate(live_listing):
        if not isinstance(entry, dict):
            findings.append(
                f"live listing entry {index} is {type(entry).__name__}, not an object, and "
                "was not compared"
            )
            continue
        uri = entry.get("calc_uri")
        if not isinstance(uri, str) or not uri:
            # Skipping this silently let a malformed row sit beside a full set
            # of valid ones and the command print agreement.
            findings.append(
                f"live listing entry {index} carries no calc_uri string and was not compared"
            )
            continue
        if uri in live or uri in repeated:
            repeated.add(uri)
            live.pop(uri, None)
            findings.append(
                f"live lists {uri} more than once (again at entry {index}); no entry for it "
                "was compared"
            )
            continue
        live[uri] = entry
    for uri in sorted(set(live) - set(contract.calculators)):
        findings.append(f"live has {uri}, which snapshot {contract.snapshot_id} does not record")
    for uri in sorted(set(contract.calculators) - set(live)):
        findings.append(f"snapshot {contract.snapshot_id} records {uri}, which live no longer "
                         "lists")
    for uri in sorted(set(live) & set(contract.calculators)):
        recorded = set(contract.calculators[uri].get("supported_periods", []))
        periods = live[uri].get("supported_periods")
        if periods is None:
            periods = []
        if not isinstance(periods, list) or not all(isinstance(p, str) for p in periods):
            # set() of a null or of unhashable entries raised TypeError out of
            # a function that promises not to raise.
            findings.append(
                f"{uri}: live supported_periods is not a list of strings; its periods were "
                "not compared"
            )
            continue
        current = set(periods)
        for period in sorted(current - recorded):
            findings.append(f"{uri}: live accepts {period}, which the snapshot does not record")
        for period in sorted(recorded - current):
            findings.append(f"{uri}: the snapshot records {period}, which live no longer accepts")
        recorded_ref = contract.calculators[uri].get("input_schema_ref")
        current_ref = live[uri].get("input_schema_ref")
        if not recorded_ref:
            continue
        if not isinstance(current_ref, str) or not current_ref:
            # The comparison used to be skipped whenever live carried no
            # reference, so a dropped schema printed agreement.
            findings.append(
                f"{uri}: live no longer states input schema ref {recorded_ref} "
                f"(now {current_ref!r})"
            )
        elif recorded_ref != current_ref:
            findings.append(f"{uri}: input schema ref moved from {recorded_ref} to {current_ref}")
    return findings
