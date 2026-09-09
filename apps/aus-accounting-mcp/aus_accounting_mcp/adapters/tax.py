"""Validate worksheet facts and delegate all arithmetic to austaxcalc."""

from decimal import Decimal, InvalidOperation
from typing import Annotated, Any, Literal

from austaxcalc import calculations
from pydantic import BaseModel, ConfigDict, Field

from ..errors import InputError
from ..money import parse_amount

Money = Annotated[str, Field(max_length=60, description="Non-negative AUD decimal string, "
                            "at most 2dp and 1000000000000.00.")]
Ratio = Annotated[str, Field(max_length=30, description="Finite decimal string, not a percentage.")]


class Worksheet(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)
    scope_confirmed: bool = Field(description="Pass true only after establishing every scope "
        "condition in aus-accounting://scope calculation_worksheets for this kind. "
        "Missing or uncertain scope must be resolved first; false is refused.")


class GstFacts(Worksheet):
    kind: Literal["gst"]
    year: Literal["2025-26"]
    amount: Money
    gst_inclusive: bool = Field(description="Whether the established taxable price includes GST.")


class ResidentTaxFacts(Worksheet):
    kind: Literal["resident_tax"]
    year: Literal["2024-25", "2025-26", "2026-27"]
    taxable_income: Money


class CapitalGainsFacts(Worksheet):
    kind: Literal["capital_gains"]
    year: Literal["2025-26"]
    other_gains: Money
    discount_gains: Money
    current_losses: Money
    prior_losses: Money


class FbtFacts(Worksheet):
    kind: Literal["fbt"]
    year_ended: Annotated[int, Field(ge=2026, le=2026,
        description="FBT year ended 31 March 2026, not an income year.")]
    type_one_value: Money
    type_two_value: Money


class DepreciationFacts(Worksheet):
    kind: Literal["depreciation"]
    year: Literal["2025-26"]
    cost: Money
    effective_life: Ratio
    days: Annotated[int, Field(ge=0, le=365,
        description="Days from first use or installation ready for use in 2025-26.")]
    taxable_use: Ratio
    method: Literal["prime_cost", "diminishing_value"]


class SgFacts(Worksheet):
    kind: Literal["quarterly_sg"]
    year: Literal["2025-26"]
    quarter: Annotated[int, Field(ge=1, le=4,
        description="1 is Jul-Sep, 2 Oct-Dec, 3 Jan-Mar, 4 Apr-Jun.")]
    ordinary_time_earnings: Money
    qualifying_contributions: Money


TaxFacts = Annotated[
    GstFacts | ResidentTaxFacts | CapitalGainsFacts | FbtFacts | DepreciationFacts | SgFacts,
    Field(discriminator="kind"),
]
MONEY_FIELDS = {
    "amount", "taxable_income", "other_gains", "discount_gains", "current_losses",
    "prior_losses", "type_one_value", "type_two_value", "cost", "ordinary_time_earnings",
    "qualifying_contributions",
}


def calculate(facts: TaxFacts) -> dict[str, Any]:
    arguments = facts.model_dump()
    kind = arguments.pop("kind")
    for name in arguments.keys() & MONEY_FIELDS:
        arguments[name] = parse_amount(arguments[name], name)
    try:
        for name in ("effective_life", "taxable_use"):
            if name in arguments:
                arguments[name] = Decimal(arguments[name])
        functions = {
            "gst": calculations.gst, "resident_tax": calculations.resident_tax,
            "capital_gains": calculations.capital_gains, "fbt": calculations.fbt,
            "depreciation": calculations.depreciation, "quarterly_sg": calculations.quarterly_sg,
        }
        return functions[kind](**arguments)
    except (ValueError, InvalidOperation) as exc:
        raise InputError(str(exc)) from exc
