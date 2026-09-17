"""Local, digest-bound evidence of one remote call.

An evidence file is what an offline consumer reads instead of the network. It
has to carry enough to decide whether to trust the figure, and it has to be
separable into the part that is deterministic and the part that is not:

- `calculation` is the deterministic content. The same facts, the same
  provider ruleset and the same rate snapshot produce the same bytes here.
  Its digest is `calculation_sha256`, and that is what a consumer pins.
- `observation` is everything that changes between two identical calls: when
  it was captured, how long it took, which base URL answered. It is recorded
  and it is excluded from the digest, because a pack that fails when the
  network was slower is a pack that teaches people to ignore it.

`upstream` holds the provider's own response unaltered, with its manifest and
advisory. Normalisation lives beside it under `normalised`, never over it.
"""

from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone
from decimal import Decimal
from pathlib import Path
from typing import Any

from .client import Outcome, Status

SCHEMA = "lodgeit-calculation-evidence/1"


def _canonical(payload: Any) -> bytes:
    """Canonical bytes for hashing: sorted keys, no spaces, UTF-8, no NaN."""
    return json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False,
                      allow_nan=False).encode("utf-8")


def _decimals_as_strings(values: dict[str, Decimal]) -> dict[str, str]:
    return {key: format(value, "f") for key, value in sorted(values.items())}


def _json_safe(value: Any) -> Any:
    """Render Decimals as their exact digits so the record is plain JSON.

    The bytes actually sent are kept separately in `input_json`, so nothing is
    lost: this is the readable copy, and it carries the same digits.
    """
    if isinstance(value, Decimal):
        return format(value, "f")
    if isinstance(value, dict):
        return {key: _json_safe(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_json_safe(item) for item in value]
    return value


def build(
    outcome: Outcome,
    *,
    label: str,
    synthetic: bool,
    engine: dict[str, str] | None = None,
    local_result: dict | None = None,
    notes: tuple[str, ...] = (),
) -> dict:
    """Assemble an evidence record from one outcome.

    `synthetic` must be true for anything committed or shared: this package is
    for fabricated inputs, and the flag travels with the file so a reader never
    has to guess. A record whose input was not synthetic is still built, so the
    caller can keep it locally, but it says so.
    """
    calculation = {
        "schema": SCHEMA,
        "label": label,
        "synthetic_input": bool(synthetic),
        "provider": {
            "name": "lodgeit-labs",
            "contract_snapshot": outcome.contract_snapshot,
            "contract_sha256": outcome.contract_sha256,
        },
        "call": {
            "calculator": outcome.calculator,
            "period": outcome.period,
            "status": str(outcome.status),
            "http_status": outcome.http_status,
        },
        "input": _json_safe(outcome.request_body),
        "input_json": outcome.request_json,
        "upstream": {
            "response": outcome.raw_response,
            "manifest": outcome.manifest,
            "advisory": outcome.advisory,
            "refusal_class": outcome.upstream_refusal_class,
        },
        "normalised": {
            "values": _decimals_as_strings(outcome.values),
            "note": "Parsed from the response's own decimal strings. The response above is "
                    "unaltered.",
        },
        "validation": {
            "findings": list(outcome.findings),
            "accepted": outcome.status is Status.COMPUTED,
        },
        "engine": engine,
        "local_result": _json_safe(local_result),
        "notes": list(notes),
        "boundary": [
            "Calculated on supplied facts. Not advice, not a lodgement and not a review sign-off.",
            "A digest proves the bytes are the bytes. It proves nothing about whether the "
            "figure is right.",
        ],
    }
    digest = hashlib.sha256(_canonical(calculation)).hexdigest()
    return {
        "schema": SCHEMA,
        "calculation_sha256": digest,
        "calculation": calculation,
        "observation": {
            "captured_at": (
                outcome.observed_at
                or datetime.now(timezone.utc).isoformat(timespec="seconds")
            ),
            "elapsed_ms": outcome.elapsed_ms,
            "base_url": outcome.base_url,
            "note": "Observation metadata is excluded from calculation_sha256 on purpose: it "
                    "changes between two identical calls.",
        },
    }


def digest_of(record: dict) -> str:
    """Recompute the digest from the record's own calculation block."""
    return hashlib.sha256(_canonical(record["calculation"])).hexdigest()


def verify(record: dict) -> list[str]:
    """Findings against an evidence record. Empty means it hangs together."""
    findings: list[str] = []
    if record.get("schema") != SCHEMA:
        findings.append(f"unknown evidence schema {record.get('schema')!r}")
        return findings
    if "calculation" not in record or not isinstance(record["calculation"], dict):
        findings.append("no calculation block")
        return findings
    recorded = record.get("calculation_sha256")
    try:
        actual = digest_of(record)
    except (TypeError, ValueError) as exc:
        findings.append(f"calculation block cannot be canonicalised: {exc}")
        return findings
    if recorded != actual:
        findings.append(f"calculation_sha256 {recorded} does not match the calculation block "
                         "({actual})")
    calculation = record["calculation"]
    if calculation.get("schema") != SCHEMA:
        findings.append("the calculation block names a different schema from the record")
    if calculation.get("call", {}).get("status") == str(Status.COMPUTED):
        if not calculation.get("upstream", {}).get("manifest"):
            findings.append("a computed result with no upstream manifest")
        if not calculation.get("upstream", {}).get("advisory"):
            findings.append("a computed result with no upstream advisory")
    return findings


def write(record: dict, path: Path) -> Path:
    """Write an evidence record as pretty JSON with a trailing newline."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2, ensure_ascii=False, allow_nan=False) + "\n",
                    encoding="utf-8")
    return path


def read(path: Path) -> dict:
    return json.loads(Path(path).read_text(encoding="utf-8"))
