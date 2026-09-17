"""Accounting depreciation: a movement that has to close, or a refusal.

This trial does **not** compare the provider's figure with the local
`austaxcalc.depreciation` worksheet, and that is the point. The provider
returns an AASB 116 accounting carrying amount. The local worksheet returns a
decline in value under ITAA 1997 Division 40, on an effective life and a
taxable-use proportion. They answer different questions, they will not agree,
and a trial that put them side by side and reported a difference would be
inviting exactly the confusion the provider's own advisory warns about.

What is checked instead is the provider's own ledger identity over a range:

    closing_wdv = opening_wdv + cost_additions - range_dep

The provider publishes a limitation, read on 18 September 2026: where
`from_date` precedes `acquisition_date`, the asset's cost enters during the
window and no `cost_additions` field is emitted, so the three published figures
do not close. The provider says `range_dep` and `closing_wdv` are correct and
the roll-forward is not closable from the response alone.

This module takes that at face value and refuses. An unreconcilable movement is
`CONTRACT_FAILURE`, and the addition that would close it is never invented. An
addition may be supplied separately, as evidence the caller holds, and then it
is named as a supplied figure in the reasons.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import date, timedelta
from decimal import Decimal
from typing import Any

from ..client import LodgeitClient, Status
from . import Comparison, Evaluation

RANGE_CALC_URI = "urn:sbrm:calculator:depreciation:range"
AT_CALC_URI = "urn:sbrm:calculator:depreciation:at"
PERIOD_URI = "urn:sbrm:period:depreciation:unscoped"
DAY_COUNTS = ("actual/actual", "actual/365", "monthly")


@dataclass(frozen=True)
class Asset:
    """A fabricated asset. Accounting inputs only; nothing here is tax."""

    case_id: str
    cost: Decimal
    acquisition_date: str
    useful_life_years: int
    method: str = "prime_cost"

    def payload(self) -> dict[str, Any]:
        return {
            "cost": self.cost,
            "acquisition_date": self.acquisition_date,
            "accounting_useful_life_years": self.useful_life_years,
            "accounting_method": self.method,
        }


def range_request(asset: Asset, from_date: str, to_date: str, day_count: str) -> dict[str, Any]:
    if day_count not in DAY_COUNTS:
        raise ValueError(f"day_count must be one of {DAY_COUNTS}")
    return {
        "basis": "accounting",
        "asset": asset.payload(),
        "from_date": from_date,
        "to_date": to_date,
        "day_count": day_count,
    }


def reconcile(
    values: dict[str, Decimal],
    *,
    supplied_additions: Decimal | None = None,
    tolerance: Decimal = Decimal("0.01"),
) -> tuple[bool, tuple[str, ...]]:
    """Does opening + additions - depreciation equal closing?

    `supplied_additions` is the caller's own evidence of an addition inside the
    window. It is named in the reasons when used. Nothing here derives an
    addition from the gap, because a figure computed to make a reconciliation
    pass is not evidence of anything.
    """
    missing = [name for name in ("opening_wdv", "closing_wdv", "range_dep") if name not in values]
    if missing:
        return False, (f"the response is missing {', '.join(missing)}, so the movement cannot be "
                        "tested",)
    additions = supplied_additions if supplied_additions is not None else Decimal("0.00")
    expected_closing = values["opening_wdv"] + additions - values["range_dep"]
    gap = values["closing_wdv"] - expected_closing
    reasons: list[str] = []
    if supplied_additions is not None:
        reasons.append(
            f"cost additions of {supplied_additions} were supplied by the caller as separate "
            "evidence; the provider did not return them"
        )
    if gap.copy_abs() <= tolerance:
        return True, tuple(reasons)
    reasons.append(
        f"movement does not close: opening {values['opening_wdv']} plus additions {additions} "
        f"less depreciation {values['range_dep']} is {expected_closing}, against a closing "
        f"balance of {values['closing_wdv']}, a gap of {gap}. No addition has been invented to "
        "close it."
    )
    return False, tuple(reasons)


def spans_acquisition(asset: Asset, from_date: str) -> bool:
    """True when the window starts before the asset was acquired.

    This is the provider's documented pre-acquisition limitation. Knowing it
    before the call is what lets the trial say "this is the known limitation"
    rather than "the provider is wrong".
    """
    return from_date < asset.acquisition_date


def consecutive_windows(start: str, end: str, boundaries: list[str]) -> list[tuple[str, str]]:
    """Split a window at the given boundaries, inclusive of both ends.

    Consecutive ranges telescope: the depreciation of the parts should sum to
    the depreciation of the whole, and the closing balance of one part should
    be the opening balance of the next.
    """
    windows: list[tuple[str, str]] = []
    current = start
    for boundary in boundaries:
        windows.append((current, boundary))
        current = (date.fromisoformat(boundary) + timedelta(days=1)).isoformat()
    windows.append((current, end))
    return windows


def evaluate_range(
    asset: Asset,
    from_date: str,
    to_date: str,
    day_count: str,
    client: LodgeitClient | None = None,
    *,
    supplied_additions: Decimal | None = None,
    case_id: str | None = None,
) -> Comparison:
    identifier = case_id or f"{asset.case_id}:{from_date}..{to_date}:{day_count}"
    if client is None:
        return Comparison(case_id=identifier, evaluation=Evaluation.NOT_RUN,
                          reasons=("No client supplied; the provider was not called.",))
    outcome = client.invoke(RANGE_CALC_URI, PERIOD_URI,
                            range_request(asset, from_date, to_date, day_count))
    return evaluate_outcome(asset, from_date, identifier, outcome,
                            supplied_additions=supplied_additions)


def evaluate_outcome(
    asset: Asset,
    from_date: str,
    case_id: str,
    outcome,
    *,
    supplied_additions: Decimal | None = None,
) -> Comparison:
    common: dict[str, Any] = {"case_id": case_id, "upstream_status": str(outcome.status)}
    if outcome.status is Status.UPSTREAM_NOT_FOUND:
        return Comparison(
            evaluation=Evaluation.UNSUPPORTED_PERIOD,
            reasons=outcome.findings,
            **common,
        )
    if outcome.status in (
        Status.UPSTREAM_UNAVAILABLE,
        Status.UPSTREAM_THROTTLED,
        Status.REFUSED_TO_SEND):
        return Comparison(
            evaluation=Evaluation.UPSTREAM_UNAVAILABLE,
            reasons=outcome.findings,
            **common,
        )
    if outcome.status is Status.UPSTREAM_REFUSED:
        return Comparison(evaluation=Evaluation.SCOPE_MISMATCH, reasons=outcome.findings, **common)
    if outcome.status is not Status.COMPUTED:
        return Comparison(
            evaluation=Evaluation.CONTRACT_FAILURE,
            reasons=outcome.findings,
            **common,
        )

    values = dict(outcome.values)
    closes, reasons = reconcile(values, supplied_additions=supplied_additions)
    reasons = tuple(outcome.findings) + reasons
    if closes:
        return Comparison(evaluation=Evaluation.MATCH, upstream=values,
                          reasons=reasons + (
                              "accounting carrying amounts only; this is not a Division 40 "
                              "deduction and the tax figure will not equal it",
                          ), **common)
    if spans_acquisition(asset, from_date):
        reasons += (
            "the window starts before the acquisition date, which is the provider's published "
            "pre-acquisition limitation: the cost enters during the window and no cost_additions "
            "field is emitted. Supply the addition as separate evidence or start the window on "
            "or after the acquisition date.",
        )
    return Comparison(
        evaluation=Evaluation.CONTRACT_FAILURE,
        upstream=values,
        reasons=reasons,
        **common,
    )
