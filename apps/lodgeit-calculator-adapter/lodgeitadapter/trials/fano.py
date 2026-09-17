"""Fano: mapping suggestions, and nothing that follows from them.

Experimental and opt-in, and separate from every deterministic calculator in
this package. Fano has a model in the path. What it returns is a suggestion
about which LodgeiT chart code a ledger line might belong to, produced partly
by a classifier, and the separation is recorded on every line it touches.

Three rules this module enforces, because each one is a way a suggestion turns
into a posting:

1. **Nothing is applied.** The original account, the supplied entity structure
   and topology, the proposed code, the confidence, the reason, the classifier
   identity and the provenance are all preserved side by side. Nothing is
   overwritten and no mapping is written anywhere.
2. **A code means nothing outside its own chart.** LodgeiT codes are LodgeiT's
   chart. Applying one to a different chart of accounts needs a reviewed
   crosswalk, and `require_crosswalk` refuses without one.
3. **`accepted_fact` is not approval.** It is the classifier's verdict on its
   own prediction. `draft_fact` is the classifier declining to guess, which is
   a correct outcome, not a failure. Neither authorises a posting, and neither
   fills in a human approval field.
"""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Any

from ..client import LodgeitClient, Status
from ..errors import ContractError

ROUTE_CALC_URI = "urn:lodgeit:classifier:fano:trial-balance"
PERIOD_URI = "urn:lodgeit:period:fano:unscoped"
ENTITY_STRUCTURES = ("company", "trust", "partnership", "sole_trader", "super_fund")
TOPOLOGIES = (
    "current_assets", "non_current_assets", "current_liabilities",
    "non_current_liabilities", "equity", "revenue", "expenses",
)
MAX_LINES = 500
MAX_BODY_BYTES = 524288


@dataclass(frozen=True)
class Line:
    """One fabricated trial-balance line."""

    description: str
    source_topology: str
    amount: Decimal
    predicted_code: str = "sbrm_0000"
    # Decimal, not float: the serialiser refuses a float outright, because a
    # float has already lost whatever digits the caller meant.
    confidence: Decimal = Decimal("0")

    def payload(self) -> dict[str, Any]:
        if self.source_topology not in TOPOLOGIES:
            raise ContractError(f"{self.source_topology!r} is not one of {TOPOLOGIES}")
        return {
            "description": self.description,
            "source_topology": self.source_topology,
            "predicted_code": self.predicted_code,
            "confidence": self.confidence,
            "amount": self.amount,
        }


@dataclass(frozen=True)
class Suggestion:
    """One line's suggestion, with everything needed to review it."""

    description: str
    supplied_topology: str
    supplied_code: str
    supplied_amount: Decimal
    proposed_code: str | None
    confidence: str | None
    fano_status: str | None
    reason: str | None
    cascade_topology: str | None
    classifier: str | None
    findings: tuple[str, ...] = ()

    @property
    def needs_review(self) -> bool:
        """Every suggestion needs review. The property exists to be read."""
        return True

    @property
    def topology_disagrees(self) -> bool:
        return bool(self.cascade_topology) and self.cascade_topology != self.supplied_topology

    def to_json_dict(self) -> dict:
        return {
            "original": {
                "description": self.description,
                "topology": self.supplied_topology,
                "code": self.supplied_code,
                "amount": format(self.supplied_amount, "f"),
            },
            "suggestion": {
                "proposed_code": self.proposed_code,
                "confidence": self.confidence,
                "fano_status": self.fano_status,
                "reason": self.reason,
                "cascade_topology": self.cascade_topology,
                "classifier": self.classifier,
            },
            "review": {
                "applied": False,
                "approved_by": None,
                "topology_disagreement": self.topology_disagrees,
                "findings": list(self.findings),
                "boundary": "A suggestion from a model. accepted_fact is the classifier's "
                            "verdict on its own prediction, not an approval, and it authorises "
                            "no posting. A LodgeiT code needs a reviewed crosswalk before it "
                            "means anything in another chart of accounts.",
            },
        }


def build_payload(entity_structure: str, lines: list[Line]) -> dict[str, Any]:
    """The request body, with the provider's own limits checked first."""
    if entity_structure not in ENTITY_STRUCTURES:
        raise ContractError(
            f"entity_structure {entity_structure!r} is not one of "
            f"{sorted(ENTITY_STRUCTURES)}"
        )
    if not lines:
        raise ContractError("a trial balance with no lines has nothing to classify")
    if len(lines) > MAX_LINES:
        raise ContractError(
            f"{len(lines)} lines exceeds the provider's limit of {MAX_LINES}; it answers 413"
        )
    total = sum((line.amount for line in lines), Decimal("0"))
    if total != 0:
        raise ContractError(
            f"the supplied lines net to {total}, not nil. The provider refuses an unbalanced "
            "payload with a 400 on equilibrium, and a trial balance that does not balance is "
            "a bookkeeping problem, not a classification problem."
        )
    return {
        "entity_structure": entity_structure,
        "lines": [line.payload() for line in lines],
    }


def require_crosswalk(crosswalk: dict[str, str] | None, proposed_code: str) -> str:
    """Translate a LodgeiT code into the local chart, or refuse.

    There is no default and no passthrough. A code with no reviewed crosswalk
    entry is a refusal, because the alternative is writing a foreign chart's
    code into an account nobody mapped.
    """
    if not crosswalk:
        raise ContractError(
            "no reviewed crosswalk was supplied. A LodgeiT chart code is meaningless in another "
            "chart of accounts until somebody maps it."
        )
    try:
        return crosswalk[proposed_code]
    except KeyError:
        raise ContractError(
            f"{proposed_code} has no entry in the reviewed crosswalk. Add one, or leave the line "
            "for review."
        ) from None


def classify(
    entity_structure: str,
    lines: list[Line],
    client: LodgeitClient | None = None,
) -> tuple[str, list[Suggestion], tuple[str, ...]]:
    """Send a fabricated trial balance and return suggestions, never mappings."""
    payload = build_payload(entity_structure, lines)
    if client is None:
        return "NOT_RUN", [], ("No client supplied; the classifier was not called.",)
    outcome = client.invoke(ROUTE_CALC_URI, PERIOD_URI, payload)
    return interpret(lines, outcome)


def interpret(lines: list[Line], outcome) -> tuple[str, list[Suggestion], tuple[str, ...]]:
    """Turn one response into suggestions. Offline-testable from a fixture."""
    if outcome.status is not Status.COMPUTED:
        return str(outcome.status), [], outcome.findings
    body = outcome.result if isinstance(outcome.result, dict) else {}
    results = body.get("results")
    if not isinstance(results, list):
        return "CONTRACT_FAILURE", [], ("the response carries no results array",)
    if len(results) != len(lines):
        return "CONTRACT_FAILURE", [], (
            f"the provider returned {len(results)} results for {len(lines)} lines; the rows "
            "cannot be matched up and nothing is proposed",
        )
    if body.get("equilibrium_valid") is False:
        return "CONTRACT_FAILURE", [], ("the provider reports equilibrium_valid false",)

    for index, result in enumerate(results):
        if not isinstance(result, dict):
            return "CONTRACT_FAILURE", [], (
                f"the result at position {index} is {type(result).__name__}, not an object, so "
                "there is nothing in it to propose. A row that cannot be read is not a row "
                "with empty fields.",
            )

    suggestions: list[Suggestion] = []
    for line, result in zip(lines, results):
        findings: list[str] = []
        if result.get("description") != line.description:
            findings.append(
                f"the provider echoed description {result.get('description')!r} against the "
                f"supplied {line.description!r}; the rows may be out of order"
            )
        status = result.get("fano_status")
        if status not in ("accepted_fact", "draft_fact", None):
            findings.append(f"unknown fano_status {status!r}; treat the line as unreviewed")
        suggestions.append(
            Suggestion(
                description=line.description,
                supplied_topology=line.source_topology,
                supplied_code=line.predicted_code,
                supplied_amount=line.amount,
                proposed_code=result.get("predicted_code"),
                confidence=result.get("confidence"),
                fano_status=status,
                reason=result.get("quarantine_reason"),
                cascade_topology=result.get("cascade_topology"),
                classifier=result.get("model_architecture"),
                findings=tuple(findings),
            )
        )
    # The first element is this trial's own status and it stays in this
    # trial's vocabulary. Returning the provider's string verbatim put an
    # unchecked value where a caller reads CONTRACT_FAILURE and NOT_RUN, so a
    # provider that answered "NOT_RUN" or "COMPLETE" would have been read as
    # one of ours. Its status is recorded, unaltered, as a note.
    provider_status = body.get("status")
    notes = tuple(outcome.findings) + (
        f"the provider's own status field read {provider_status!r}",
        "Every line above is a suggestion awaiting review. Nothing has been applied.",
    )
    return "INTERPRETED", suggestions, notes
