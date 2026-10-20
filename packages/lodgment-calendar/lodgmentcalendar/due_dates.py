"""Lodgment and payment due dates under the ATO's 2026–27 lodgment program.

The calendar reads the facts the caller supplies, applies the dates the ATO
publishes for self-lodgers and for registered tax and BAS agents, and moves each
date to the next business day under the program's public holiday rule. A date
that depends on a fact the caller did not supply, or that the ATO pages leave
open, is UNKNOWN and names what is missing. It contacts no service and lodges
nothing.
"""

from __future__ import annotations

import re
from collections.abc import Mapping
from dataclasses import fields
from datetime import date
from typing import Any

from ._version import __version__
from .holidays import COVERS_FROM, HOLIDAY_PROVISION, HOLIDAY_SOURCE, calendar, roll
from .returns import return_rule
from .rules import FACT_MEANINGS, SOURCES, Facts, Rule
from .statements import statement_rules

SOURCE_CHECKED = "2026-10-20"
PROGRAM_YEAR = "2026-27"
# The program dates cover activity statements for periods in 2026–27 and income
# tax returns for the 2025–26 income year (the 2026 return).
ACTIVITY_YEAR = "2026-27"
RETURN_YEAR = "2025-26"

DETERMINED, UNKNOWN = "DETERMINED", "UNKNOWN"
OBLIGATIONS = ("bas", "ias", "income_tax_return")
ENTITY_TYPES = ("individual", "trust", "partnership", "company", "smsf")
CHANNELS = ("self", "tax_agent", "bas_agent")
CYCLES = ("monthly", "quarterly")

NOT_DECIDED = [
    "Dates the ATO sets for one client override these: a notice requiring earlier "
    "lodgement, a prosecution letter, a deferral, an advised first-year date for a new "
    "SMSF, or the due date shown on the activity statement or in Online services for "
    "agents.",
    "Whether a fact supplied is true. Large or medium status, taxable status, income and "
    "liability thresholds, registration and lodgement history are taken as given.",
    "Substituted accounting periods, companies that are not full self-assessment "
    "taxpayers, APRA-regulated funds, members exiting a consolidated group, annual GST "
    "returns, instalment notices (forms R, S, T and N), PAYG instalments paid in 2 "
    "instalments or annually, deferred BAS payers (who pay monthly PAYG instalments on the "
    "28th) and every other obligation type.",
    "Whether a lodgement or payment was on time, and any penalty or general interest charge.",
    "Disaster or other deferrals, and holidays proclaimed after the ATO published its "
    "2026–27 table.",
]

WARNINGS = [
    "The ATO says its list of lodgment program due dates is a guide only. Confirm each "
    "client's dates in Online services for agents or the practitioner lodgment service.",
    "Retain the status and missing conditions of every UNKNOWN row; do not replace one "
    "with a nominal date.",
]

_YEAR = re.compile(r"([0-9]{4})-([0-9]{2})")


def due_dates(
    obligation: str,
    entity_type: str,
    financial_year: str,
    channel: str,
    *,
    reporting_cycle: str | None = None,
    facts: Facts | None = None,
    extra_holidays: Mapping[date, str] | None = None,
    holidays_complete_until: date | None = None,
) -> dict[str, Any]:
    """Return each lodgement, payment and concession date with its rule and status.

    ``financial_year`` is the year of the periods (activity statements) or the income
    year the return covers (income tax returns), in the form 2026-27.

    ``extra_holidays`` adds public holidays for the whole of a state or territory that
    the ATO table does not list, by date and name. ``holidays_complete_until`` states
    that the ATO table and those holidays hold every such holiday to that date, which
    lets dates after 30 June 2027 be settled.
    """
    facts = Facts() if facts is None else facts
    _check(obligation, entity_type, financial_year, channel, reporting_cycle, facts)
    cal = calendar(extra_holidays, holidays_complete_until)
    if obligation == "income_tax_return":
        rows = _return_rows(entity_type, financial_year, channel, facts, cal)
    else:
        rows = _statement_rows(obligation, entity_type, financial_year, channel,
                               reporting_cycle, facts, cal)
    return {
        "ok": True,
        "engine": "lodgment-calendar",
        "engine_version": __version__,
        "calendar": "lodgment_due_dates",
        "program_year": PROGRAM_YEAR,
        "request": {"obligation": obligation, "entity_type": entity_type,
                    "financial_year": financial_year, "channel": channel,
                    "reporting_cycle": reporting_cycle},
        "facts_used": {field.name: getattr(facts, field.name) for field in fields(facts)},
        "obligations": rows,
        "counts": {status: sum(1 for row in rows if row["status"] == status)
                   for status in (DETERMINED, UNKNOWN)},
        "holiday_calendar": _holiday_calendar(cal, holidays_complete_until),
        "not_decided": NOT_DECIDED,
        "warnings": WARNINGS,
        "source_checked": SOURCE_CHECKED,
    }


def _return_rows(entity_type: str, financial_year: str, channel: str, facts: Facts,
                 cal: Calendar) -> list[dict[str, Any]]:
    label = f"Income tax return {financial_year}"
    if financial_year != RETURN_YEAR:
        return _rows(label, financial_year, _not_recorded(financial_year, RETURN_YEAR), [],
                     cal=cal)
    rule, missing, concessions = return_rule(entity_type, channel, facts)
    # A partnership pays no income tax, whatever its lodgement date turns out to be.
    rows = _rows(label, financial_year, rule, missing, family=_family(entity_type, channel),
                 pending_payment="none" if entity_type == "partnership" else "unknown",
                 cal=cal)
    for concession, needed in concessions:
        rows.append(_row(label, financial_year, "concession", concession, needed, cal=cal))
    return rows


def _statement_rows(obligation: str, entity_type: str, financial_year: str, channel: str,
                    reporting_cycle: str | None, facts: Facts,
                    cal: Calendar) -> list[dict[str, Any]]:
    if financial_year != ACTIVITY_YEAR:
        return _rows(f"{obligation.upper()} {financial_year}", financial_year,
                     _not_recorded(financial_year, ACTIVITY_YEAR), [], cal=cal)
    rows: list[dict[str, Any]] = []
    for period, label, rule, missing in statement_rules(
            obligation, entity_type, channel, reporting_cycle, facts):
        rows += _rows(label, period, rule, missing, family=_family(obligation, channel),
                      cal=cal)
    return rows


def _holiday_calendar(cal: Calendar, complete_until: date | None) -> dict[str, Any]:
    return {
        "definition": "Saturdays, Sundays and days that are a public holiday for the whole "
        "of any state or territory move a due date to the next business day, for taxpayers "
        "in every state.",
        "source_url": HOLIDAY_SOURCE,
        "source_provision": HOLIDAY_PROVISION,
        "covers_from": COVERS_FROM.isoformat(),
        "covers_to": cal[1].isoformat(),
        "caller_holidays": {day.isoformat(): name for day, name in sorted(cal[0].items())
                            if name.endswith(" (caller-supplied)")},
        "caller_complete_until": None if complete_until is None else complete_until.isoformat(),
    }


def _check(obligation: str, entity_type: str, financial_year: object, channel: str,
           reporting_cycle: str | None, facts: object) -> None:
    for name, value, allowed in (("obligation", obligation, OBLIGATIONS),
                                 ("entity_type", entity_type, ENTITY_TYPES),
                                 ("channel", channel, CHANNELS)):
        if value not in allowed:
            raise ValueError(f"{name} must be one of {', '.join(allowed)}; got {value!r}")
    match = _YEAR.fullmatch(financial_year) if isinstance(financial_year, str) else None
    if match is None or int(match[2]) != (int(match[1]) + 1) % 100:
        raise ValueError(f"financial_year must look like 2026-27; got {financial_year!r}")
    if obligation == "income_tax_return":
        if reporting_cycle is not None:
            raise ValueError("reporting_cycle applies only to bas and ias")
        if channel == "bas_agent":
            raise ValueError("a BAS agent cannot lodge an income tax return; use tax_agent")
    elif reporting_cycle not in CYCLES:
        raise ValueError(f"reporting_cycle must be one of {', '.join(CYCLES)} for "
                         f"{obligation}; got {reporting_cycle!r}")
    if not isinstance(facts, Facts):
        raise ValueError("facts must be a Facts instance")
    for field in fields(facts):
        value = getattr(facts, field.name)
        # Identity, not isinstance: an object can claim bool as its class.
        if value is not None and value is not True and value is not False:
            raise ValueError(f"facts.{field.name} must be true, false or null; got {value!r}")


def _family(kind: str, channel: str) -> str:
    return f"{kind.upper()}-{'SELF' if channel == 'self' else 'AGENT'}"


def _not_recorded(year: str, recorded: str) -> Rule:
    return Rule(
        "YEAR-NOT-RECORDED",
        f"This release records the {PROGRAM_YEAR} lodgment program, which covers {recorded} "
        f"for this obligation; {year} is not recorded.",
        "about",
        unresolved=f"rules for {year}: this release records only {recorded} for this "
        "obligation",
        payment="unknown",
    )


Calendar = tuple[dict[date, str], date]


def _rows(label: str, period: str, rule: Rule | None, missing: list[str], *, cal: Calendar,
          family: str = "", pending_payment: str = "unknown") -> list[dict[str, Any]]:
    if rule is None or missing:
        pending = Rule(family, "The date depends on facts not supplied: "
                       + "; ".join(FACT_MEANINGS[name] for name in missing) + ".", "about")
        return [_row(label, period, "lodgment", pending, missing, payment=pending_payment,
                     cal=cal)]
    payment = rule.payment
    rows = [_row(label, period, "lodgment", rule, [],
                 payment=payment if isinstance(payment, str) else "separate", cal=cal)]
    if isinstance(payment, Rule):
        rows.append(_row(label, period, "payment", payment, [], cal=cal))
    return rows


def _row(label: str, period: str, kind: str, rule: Rule, missing: list[str], *,
         cal: Calendar, payment: str | None = None) -> dict[str, Any]:
    url, provision = SOURCES[rule.source]
    needed = [f"{name}: {FACT_MEANINGS[name]}" for name in missing]
    due: date | None = None
    holiday_roll = None
    if rule.unresolved is not None:
        needed.append(rule.unresolved)
    elif rule.nominal is not None and not needed:
        due, skipped, open_question = roll(rule.nominal, *cal)
        if skipped or due is None:
            holiday_roll = {"skipped": skipped, "source_url": HOLIDAY_SOURCE}
        if open_question is not None:
            needed.append(open_question)
        elif kind == "concession" and skipped:
            needed.append(
                f"whether the next-business-day rule extends this concession: "
                f"{rule.nominal.isoformat()} is not a business day, and the ATO pages call "
                "it both a concessional due date and not a lodgment due date")
            due = None
    if needed:
        due = None
    row: dict[str, Any] = {
        "obligation": label,
        "period": period,
        "kind": kind,
        "status": UNKNOWN if needed else DETERMINED,
        "due_date": None if due is None else due.isoformat(),
        "nominal_due_date": None if rule.nominal is None else rule.nominal.isoformat(),
        "rule_id": rule.id,
        "summary": rule.summary,
        "source_url": url,
        "source_provision": provision,
        "source_checked": SOURCE_CHECKED,
        "holiday_roll": holiday_roll,
        "missing": needed,
    }
    if kind == "lodgment":
        row["payment"] = payment
    return row
