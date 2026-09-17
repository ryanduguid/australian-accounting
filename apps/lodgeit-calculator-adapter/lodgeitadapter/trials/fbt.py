"""FBT: one benefit category from the provider into the local aggregate worksheet.

One category is wired: the car statutory formula. The other eighteen the
provider registers are not, because a category is wired when its schema and its
scope have been read, and eighteen have not been.

The shape of the join, and the two ways it goes wrong:

    provider                               local
    --------                               -----
    taxable_value            ---->         austaxcalc.fbt(type_one_value=...,
    (category taxable value)                            type_two_value=...)
    grossed_up_taxable_value  X            -> grossed-up amounts and FBT payable
    fbt_payable               X

Only `taxable_value` crosses. `grossed_up_taxable_value` and `fbt_payable` are
already grossed up, and the local worksheet grosses up whatever it is given, so
passing either produces a figure roughly 1.9 or 2.1 times too large.
`prepare_aggregate_input` is the only function that builds the local call, it
takes the category taxable value alone, and `tests/test_trials_fbt.py` proves
that feeding it a grossed-up figure is refused.

The second way it goes wrong is classification. Type 1 and type 2 differ by
whether the employer was entitled to an input tax credit for the acquisition,
which is a professional judgement about the acquisition, not a property of the
benefit the calculator can see. The provider defaults to type 2 engine-side
when `fbtType` is omitted. This module refuses to inherit that default: the
classification is a required, reviewed input, and `"unknown"` is a refusal.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from decimal import Decimal
from typing import Any

from ..client import LodgeitClient, Status
from ..errors import ContractError
from . import Comparison, Evaluation

CALC_URI = "urn:sbrm:calculator:fbt:car-statutory-formula"
PERIOD_URI = "urn:sbrm:period:fbt:fy2026"

#: The provider's FBT year against the local worksheet's `year_ended`. The
#: provider's fy2026 is the FBT year ending 31 March 2026; the local worksheet
#: takes year_ended=2026 for the same year and supports no other.
PERIOD_MAP = {PERIOD_URI: 2026}

#: Fields that are already grossed up. None of them may reach the local
#: worksheet, which grosses up what it is given.
ALREADY_GROSSED_UP = ("grossed_up_taxable_value", "fbt_payable")


class ClassificationRequired(ContractError):
    """The type 1 or type 2 classification was not established."""


@dataclass(frozen=True)
class CarBenefit:
    """A fabricated car benefit, with its reviewed classification."""

    case_id: str
    base_value: Decimal
    days_available: int
    accessories: Decimal
    employee_contribution: Decimal
    fbt_type: str  # "Type 1", "Type 2" or "unknown"
    period_uri: str = PERIOD_URI

    def request(self) -> dict[str, Any]:
        if self.fbt_type not in ("Type 1", "Type 2"):
            raise ClassificationRequired(
                f"{self.case_id}: fbt_type is {self.fbt_type!r}. Type 1 and type 2 turn on "
                "whether the employer was entitled to an input tax credit for the "
                "acquisition, which is a reviewed judgement. The provider defaults to Type 2 "
                "when the field is omitted; this trial will not inherit that default."
            )
        return {
            "baseValue": self.base_value,
            "daysAvailable": self.days_available,
            "accessories": self.accessories,
            "employeeContribution": self.employee_contribution,
            "fbtType": self.fbt_type,
        }


def prepare_aggregate_input(
    taxable_value: Decimal,
    fbt_type: str,
    *,
    field_name: str = "taxable_value",
    response_values: Mapping[str, Decimal] | None = None,
) -> dict[str, Decimal]:
    """Turn one category taxable value into the local worksheet's arguments.

    Refuses anything that is not a category taxable value. This is the single
    door between the two engines and it is deliberately narrow.

    Two checks, because the first one alone was a check on a name. A caller
    that read the wrong field and passed the default `field_name` got a
    grossed-up amount grossed up again, which is what the README says cannot
    happen. So the value itself is compared against the figures the provider
    reported as already grossed up, and a match is refused whatever it was
    called.
    """
    if field_name in ALREADY_GROSSED_UP:
        raise ContractError(
            f"{field_name} is already grossed up. Passing it to the aggregate worksheet would "
            "gross it up a second time. Use the category taxable_value."
        )
    for name in ALREADY_GROSSED_UP:
        recorded = (response_values or {}).get(name)
        if recorded is not None and recorded == taxable_value and taxable_value != 0:
            raise ContractError(
                f"the value supplied as {field_name} is the same figure the provider returned "
                f"as {name}, which is already grossed up. Grossing it up again would overstate "
                "the liability. Use the category taxable_value."
            )
    if fbt_type == "Type 1":
        return {"type_one_value": taxable_value, "type_two_value": Decimal("0.00")}
    if fbt_type == "Type 2":
        return {"type_one_value": Decimal("0.00"), "type_two_value": taxable_value}
    raise ClassificationRequired(
        f"fbt_type {fbt_type!r} is not an established classification; the aggregate worksheet "
        "cannot be called until it is."
    )


def run_local_aggregate(arguments: dict[str, Decimal], year_ended: int) -> dict[str, Decimal]:
    """The local aggregate worksheet on established taxable values."""
    from austaxcalc.calculations import fbt as local_fbt  # noqa: PLC0415

    result = local_fbt(
        type_one_value=arguments["type_one_value"],
        type_two_value=arguments["type_two_value"],
        year_ended=year_ended,
        scope_confirmed=True,
    )
    return {name: Decimal(value) for name, value in result["amounts"].items()}


def evaluate(benefit: CarBenefit, client: LodgeitClient | None = None) -> Comparison:
    """Fetch one category taxable value and carry it into the local worksheet."""
    common: dict[str, Any] = {"case_id": benefit.case_id}
    if benefit.period_uri not in PERIOD_MAP:
        return Comparison(
            evaluation=Evaluation.UNSUPPORTED_PERIOD,
            reasons=(f"{benefit.period_uri} has no local counterpart; the local worksheet "
                     f"supports only the FBT years in PERIOD_MAP",),
            **common,
        )
    try:
        body = benefit.request()
    except ClassificationRequired as exc:
        return Comparison(evaluation=Evaluation.SCOPE_MISMATCH, reasons=(str(exc),), **common)
    if client is None:
        return Comparison(evaluation=Evaluation.NOT_RUN,
                          reasons=("No client supplied; the provider was not called.",), **common)
    outcome = client.invoke(CALC_URI, benefit.period_uri, body)
    return evaluate_outcome(benefit, outcome)


def evaluate_outcome(benefit: CarBenefit, outcome) -> Comparison:
    common: dict[str, Any] = {"case_id": benefit.case_id, "upstream_status": str(outcome.status)}
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
    if "taxable_value" not in values:
        return Comparison(
            evaluation=Evaluation.CONTRACT_FAILURE,
            reasons=("the response carries no taxable_value, so there is nothing to carry into "
                     "the aggregate worksheet",) + outcome.findings,
            **common,
        )
    category_value = values["taxable_value"]
    arguments = prepare_aggregate_input(
        category_value, benefit.fbt_type, response_values=values,
    )
    local = run_local_aggregate(arguments, PERIOD_MAP[benefit.period_uri])
    reasons = list(outcome.findings)

    # The provider's own grossed-up figures are recorded and compared, never
    # consumed. A difference here is a gross-up rate difference and is worth
    # seeing; it is not a reason to prefer either figure.
    upstream = {"taxable_value": category_value}
    for name in ALREADY_GROSSED_UP:
        if name in values:
            upstream[name] = values[name]
    # Both derived figures are compared, not just the gross-up. The FBT
    # payable is the one that reaches a return, so a trial that reported MATCH
    # without looking at it was reporting agreement it had not established.
    # The two engines name them differently; the map is the whole translation.
    local_grossed_key = (
        "type_one_grossed_up" if benefit.fbt_type == "Type 1" else "type_two_grossed_up"
    )
    compared: dict[str, tuple[Decimal, Decimal]] = {}
    for upstream_name, local_name in (
        ("grossed_up_taxable_value", local_grossed_key),
        ("fbt_payable", "fbt_estimate"),
    ):
        if upstream_name in upstream and local_name in local:
            compared[upstream_name] = (local[local_name], upstream[upstream_name])
        elif upstream_name in upstream:
            reasons.append(
                f"{upstream_name}: the provider returned a figure and the local worksheet "
                f"produced no {local_name}"
            )
        elif local_name in local:
            reasons.append(
                f"{local_name}: the local worksheet produced a figure and the provider "
                f"returned no {upstream_name}"
            )

    differences: dict[str, Decimal] = {}
    for name, (local_value, upstream_value) in sorted(compared.items()):
        difference = local_value - upstream_value
        if difference.copy_abs() > Decimal("0.01"):
            differences[name] = difference
            reasons.append(
                f"{name}: local {local_value} against provider {upstream_value}. Check the "
                "gross-up rate and the FBT rate each side used."
            )
    if differences:
        return Comparison(evaluation=Evaluation.NUMERIC_DIFFERENCE, local=local,
                          upstream=upstream, differences=differences,
                          reasons=tuple(reasons), **common)
    if not compared:
        reasons.append(
            "the provider returned neither a grossed-up amount nor an FBT payable, so the "
            "only figure that crossed was never checked against anything"
        )
        return Comparison(evaluation=Evaluation.SCOPE_MISMATCH, local=local,
                          upstream=upstream, reasons=tuple(reasons), **common)
    if len(compared) < 2:
        return Comparison(evaluation=Evaluation.SCOPE_MISMATCH, local=local,
                          upstream=upstream, reasons=tuple(reasons), **common)
    reasons.append(
        f"only taxable_value crossed into the aggregate worksheet, as a {benefit.fbt_type} amount"
    )
    return Comparison(evaluation=Evaluation.MATCH, local=local, upstream=upstream,
                      reasons=tuple(reasons), **common)
