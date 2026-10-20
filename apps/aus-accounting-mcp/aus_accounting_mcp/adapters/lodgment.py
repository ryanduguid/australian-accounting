"""Thin adapter over lodgment-calendar.

The delegated engine owns the program dates, conditions, holiday table, sources and
UNKNOWN outcomes. This module only carries the MCP arguments into the engine's types.
"""

from __future__ import annotations

from datetime import date
from typing import Annotated, Any

from lodgmentcalendar import Facts, due_dates
from lodgmentcalendar.due_dates import FACT_MEANINGS
from lodgmentcalendar.holidays import MAX_EXTRA_HOLIDAYS
from pydantic import BaseModel, ConfigDict, Field

from aus_accounting_mcp.errors import InputError

DISCLAIMER = (
    "Preparation aid, not a lodgement or a determination of any due date. The operator "
    "supplies the facts, and none of them is verified. The ATO calls its program list a "
    "guide only: confirm each client's dates in Online services for agents, the "
    "practitioner lodgment service or on the statement before acting."
)


Day = Annotated[str, Field(max_length=10, pattern=r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$",
                           description="YYYY-MM-DD.", json_schema_extra={"format": "date"})]


def _fact(name: str) -> Any:
    meaning = FACT_MEANINGS[name]
    return Field(default=None, description=f"True if {meaning}. Omit or null when not "
                 "established.")


class LodgmentFactsInput(BaseModel):
    """Conditions the program dates depend on; omit any that is not established."""

    model_config = ConfigDict(extra="forbid", strict=True)

    prior_returns_outstanding: bool | None = _fact("prior_returns_outstanding")
    overdue_returns_lodged_by_31_october: bool | None = _fact(
        "overdue_returns_lodged_by_31_october")
    on_agent_client_list_by_cutoff: bool | None = _fact("on_agent_client_list_by_cutoff")
    large_or_medium: bool | None = _fact("large_or_medium")
    member_large_or_medium: bool | None = _fact("member_large_or_medium")
    new_registrant: bool | None = _fact("new_registrant")
    prior_year_return_taxable: bool | None = _fact("prior_year_return_taxable")
    current_year_return_taxable: bool | None = _fact("current_year_return_taxable")
    latest_tax_liability_20k_or_more: bool | None = _fact("latest_tax_liability_20k_or_more")
    total_income_over_2m: bool | None = _fact("total_income_over_2m")
    prior_year_non_taxable_or_refund: bool | None = _fact("prior_year_non_taxable_or_refund")
    current_year_non_taxable_or_refund: bool | None = _fact(
        "current_year_non_taxable_or_refund")
    consolidated_head_company: bool | None = _fact("consolidated_head_company")
    first_consolidated_return: bool | None = _fact("first_consolidated_return")
    lodges_electronically: bool | None = _fact("lodges_electronically")
    previous_statement_lodged_electronically: bool | None = _fact(
        "previous_statement_lodged_electronically")
    includes_monthly_payg_obligation: bool | None = _fact("includes_monthly_payg_obligation")
    annual_turnover_up_to_10m: bool | None = _fact("annual_turnover_up_to_10m")
    deferred_gst_registered: bool | None = _fact("deferred_gst_registered")


class HolidayInput(BaseModel):
    """A public holiday for the whole of a state or territory, missing from the ATO table."""

    model_config = ConfigDict(extra="forbid", strict=True, str_strip_whitespace=True)

    date: Day
    name: Annotated[str, Field(min_length=1, max_length=120)]


HolidayList = Annotated[list[HolidayInput], Field(max_length=MAX_EXTRA_HOLIDAYS)]


def _day(value: str, field: str) -> date:
    try:
        return date.fromisoformat(value)
    except ValueError as exc:
        raise InputError(f"{field}: {value!r} is not a YYYY-MM-DD date") from exc


def lodgment_due_dates(
    obligation: str,
    entity_type: str,
    financial_year: str,
    channel: str,
    *,
    reporting_cycle: str | None = None,
    facts: LodgmentFactsInput | None = None,
    extra_holidays: list[HolidayInput] | None = None,
    holidays_complete_until: str | None = None,
) -> dict[str, Any]:
    supplied = Facts() if facts is None else Facts(**facts.model_dump())
    holidays: dict[date, str] = {}
    for item in extra_holidays or []:
        day = _day(item.date, "extra_holidays.date")
        if day in holidays:
            raise InputError(f"extra_holidays: {item.date} appears more than once")
        holidays[day] = item.name
    complete = (None if holidays_complete_until is None
                else _day(holidays_complete_until, "holidays_complete_until"))
    try:
        result = due_dates(obligation, entity_type, financial_year, channel,
                           reporting_cycle=reporting_cycle, facts=supplied,
                           extra_holidays=holidays, holidays_complete_until=complete)
    except ValueError as exc:
        raise InputError(str(exc)) from exc
    return {**result, "disclaimer": DISCLAIMER}
