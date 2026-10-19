"""Thin adapter over gst-review.

The delegated engine owns the review states, BAS label mapping, attribution,
rounding and refusals. This module only parses MCP arguments into the engine's
types and applies the facade's money and date boundary.
"""

from __future__ import annotations

from collections.abc import Mapping
from datetime import date
from typing import Annotated, Any, Literal

from gstreview import (
    BankLine,
    LedgerLine,
    Period,
    Registration,
    reconcile_bas,
    review_coding,
)
from pydantic import BaseModel, ConfigDict, Field

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.money import MONEY_SCHEMA, parse_amount

DISCLAIMER = (
    "Experimental review aid. Not a BAS, a revised BAS, a lodgement or a GST "
    "determination. The operator supplies registration dates, codes, periods, lodged "
    "figures and the basis, and none of them is verified. Obtain human review before acting."
)

Day = Annotated[str, Field(max_length=10, pattern=r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$",
                           description="YYYY-MM-DD.", json_schema_extra={"format": "date"})]
Money = Annotated[str, Field(max_length=60, json_schema_extra=MONEY_SCHEMA,
                             description="AUD decimal string.")]
Reference = Annotated[str, Field(min_length=1, max_length=120)]
Code = Annotated[str, Field(max_length=120)]
BasLabel = Literal["G1", "G2", "G3", "G10", "G11", "1A", "1B"]
GstTreatment = Literal["taxable", "gst_free", "input_taxed", "outside_gst"]
BasCategory = Literal[
    "taxable", "capital", "gst_free", "capital_gst_free", "export", "input_taxed", "excluded",
]


class _Input(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True, str_strip_whitespace=True)


class BankLineInput(_Input):
    """One bank line; a negative amount is money out."""

    id: Reference = Field(description="Unique line reference.")
    entity: Reference = Field(description="Entity name, matching one registrations entry.")
    date: Day
    amount: Money
    account: Code | None = Field(default=None, description="Account coded on the line.")
    gst_code: Code | None = Field(default=None, description="GST code on the line.")
    confirmed: bool = Field(default=False, description=(
        "True only when a person accepted the coding; needs account and gst_code."))
    suggested_account: Code | None = Field(default=None, description="Unaccepted proposal.")
    suggested_gst_code: Code | None = Field(default=None, description="Unaccepted proposal.")
    supplier_registered: bool | None = Field(default=None, description=(
        "Established registration of the other party; omit or null when unknown."))


class RegistrationInput(_Input):
    """An entity's GST registration, first and last registered day inclusive."""

    entity: Reference
    registered_from: Day | None = Field(description=(
        "Required: the first registered day, or null for not registered at any time "
        "in the review. Do not send null for an unknown date."))
    registered_to: Day | None = Field(default=None, description=(
        "Last registered day; omit or null while still registered."))


class LedgerLineInput(_Input):
    """One ledger line; gross includes GST, and both are negative for a credit note."""

    id: Reference = Field(description="Unique line reference.")
    side: Literal["sale", "purchase"]
    gross: Money
    gst: Money
    gst_code: Code = Field(description="GST code, read through bas_codes or the default table.")
    invoice_date: Day
    payment_date: Day | None = Field(default=None, description="Omit or null while unpaid.")


class BasPeriodInput(_Input):
    """A tax period and, once lodged, the figures lodged for it."""

    label: Reference = Field(description="Unique period label, such as 2026-Q1.")
    start: Day
    end: Day
    lodged: dict[BasLabel, Money] | None = Field(default=None, description=(
        "Lodged whole-dollar figures by label; omit for a period not lodged and omit a "
        "label that was not reported."))


def _day(value: str, field: str) -> date:
    try:
        return date.fromisoformat(value)
    except ValueError as exc:
        raise InputError(f"{field}: {value!r} is not a YYYY-MM-DD date") from exc


def _optional_day(value: str | None, field: str) -> date | None:
    return None if value is None else _day(value, field)


def coding_review(
    lines: list[BankLineInput],
    registrations: list[RegistrationInput],
    *,
    match_window_days: int = 3,
    gst_codes: Mapping[str, str] | None = None,
) -> dict[str, Any]:
    facts: dict[str, Registration] = {}
    for item in registrations:
        if item.entity in facts:
            raise InputError(f"registrations: entity {item.entity!r} appears more than once")
        facts[item.entity] = Registration(
            _optional_day(item.registered_from,
                          f"registrations[{item.entity}].registered_from"),
            _optional_day(item.registered_to, f"registrations[{item.entity}].registered_to"),
        )
    bank_lines = [
        BankLine(
            id=line.id,
            entity=line.entity,
            date=_day(line.date, f"lines[{line.id}].date"),
            amount=parse_amount(line.amount, f"lines[{line.id}].amount"),
            account=line.account,
            gst_code=line.gst_code,
            confirmed=line.confirmed,
            suggested_account=line.suggested_account,
            suggested_gst_code=line.suggested_gst_code,
            supplier_registered=line.supplier_registered,
        )
        for line in lines
    ]
    try:
        result = review_coding(bank_lines, facts, match_window_days=match_window_days,
                               gst_codes=None if gst_codes is None else dict(gst_codes))
    except ValueError as exc:
        raise InputError(str(exc)) from exc
    return {**result, "disclaimer": DISCLAIMER}


def bas_reconciliation(
    lines: list[LedgerLineInput],
    periods: list[BasPeriodInput],
    *,
    basis: str,
    g1_includes_gst: bool = True,
    bas_codes: Mapping[str, str] | None = None,
) -> dict[str, Any]:
    tax_periods = [
        Period(item.label, _day(item.start, f"periods[{item.label}].start"),
               _day(item.end, f"periods[{item.label}].end"))
        for item in periods
    ]
    lodged = {
        item.label: {
            label: parse_amount(amount, f"periods[{item.label}].lodged.{label}")
            for label, amount in item.lodged.items()
        }
        for item in periods if item.lodged is not None
    }
    ledger_lines = [
        LedgerLine(
            id=line.id,
            side=line.side,
            gross=parse_amount(line.gross, f"lines[{line.id}].gross"),
            gst=parse_amount(line.gst, f"lines[{line.id}].gst"),
            gst_code=line.gst_code,
            invoice_date=_day(line.invoice_date, f"lines[{line.id}].invoice_date"),
            payment_date=_optional_day(line.payment_date, f"lines[{line.id}].payment_date"),
        )
        for line in lines
    ]
    try:
        result = reconcile_bas(ledger_lines, tax_periods, lodged, basis=basis,
                               g1_includes_gst=g1_includes_gst,
                               bas_codes=None if bas_codes is None else dict(bas_codes))
    except ValueError as exc:
        raise InputError(str(exc)) from exc
    return {**result, "disclaimer": DISCLAIMER}
