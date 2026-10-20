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
from dataclasses import dataclass, fields
from datetime import date, timedelta
from typing import Any

from ._version import __version__
from .holidays import COVERS_FROM, COVERS_TO, HOLIDAY_PROVISION, HOLIDAY_SOURCE, roll

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

_ATO = "https://www.ato.gov.au"
_PROGRAM = (_ATO + "/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/"
            "registered-agent-lodgment-program")
_BY_TYPE = _PROGRAM + "/due-dates-for-tax-returns-by-client-type"
_BY_MONTH = _PROGRAM + "/due-dates-by-month"

#: Each rule's source: the ATO page and the provision or table on it.
SOURCES: dict[str, tuple[str, str]] = {
    "bas": (
        _ATO + "/businesses-and-organisations/preparing-lodging-and-paying/"
        "business-activity-statements-bas/due-dates-for-lodging-and-paying-your-bas",
        "Lodgment due dates: Monthly reporting; Quarterly reporting, table 'Due date for "
        "each quarter'; Payment due date (last updated 17 September 2026)",
    ),
    "generate": (
        _ATO + "/businesses-and-organisations/preparing-lodging-and-paying/"
        "business-activity-statements-bas/instalment-notices-for-gst-and-payg-instalments/"
        "activity-statement-generate-dates",
        "Tables 'Dates for 2026–27 quarterly activity statement' and 'Dates for 2026–27 "
        "monthly activity statement', column 'Legislative due date' (last updated 16 June "
        "2026)",
    ),
    "two_week": (
        _ATO + "/online-services/businesses-and-organisations-online-services/"
        "lodgments-in-online-services-for-business/"
        "two-week-lodgment-concession-terms-and-conditions",
        "Eligibility for lodgment concession; What isn't included (last updated 18 August "
        "2025)",
    ),
    "bas_agent": (
        _ATO + "/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/"
        "bas-agent-lodgment-program-2026-27",
        "Lodgment and payment due dates; Table 1: BAS agent due dates – monthly lodgment "
        "obligation; Table 2: BAS agent due dates – quarterly lodgment obligation (last "
        "updated 1 July 2026)",
    ),
    "agent_as": (
        _PROGRAM + "/due-dates-by-obligation-type/activity-statements",
        "When lodgment program due dates are applied; Eligible activity statements; "
        "Quarterly activity statements, table 'Due dates for registered agents'; Quarterly "
        "PAYG instalment activity statement – head companies of consolidated groups (last "
        "updated 1 July 2026)",
    ),
    "about": (
        _PROGRAM + "/about-the-lodgment-program",
        "Lodgment and payment due dates (last updated 1 July 2026)",
    ),
    "overdue": (
        _PROGRAM + "/taxpayers-with-overdue-tax-returns",
        "Taxpayers with overdue tax returns: lodge by 31 October 2026; payment by 1 December "
        "2026 for companies and super funds; normal program once all are lodged (last "
        "updated 1 July 2026)",
    ),
    "october": (
        _BY_MONTH + "/october-2026",
        "31 October: final date to add new clients to your client list; tax returns for "
        "all entities with prior year returns outstanding (last updated 1 July 2026)",
    ),
    "ind_trust": (
        _BY_TYPE + "/individuals-and-trusts",
        "Lodgment due dates table; Concession for tax returns due 15 May; Payment of "
        "individual and trust tax returns due 15 May; Payment of individual and trust tax "
        "returns not due 15 May (last updated 1 July 2026)",
    ),
    "co_super": (
        _BY_TYPE + "/companies-and-super-funds",
        "Table 'Lodgment and payment due dates for 30 June balancing companies and super "
        "funds' (last updated 1 July 2026)",
    ),
    "large_medium": (
        _BY_TYPE + "/large-and-medium-taxpayers",
        "Large or medium taxpayer for tax return lodgment purposes; Lodgment due dates "
        "(last updated 1 July 2026)",
    ),
    "partnerships": (
        _BY_TYPE + "/partnerships",
        "Partnerships page (last updated 1 July 2026)",
    ),
    "consolidated": (
        _BY_TYPE + "/consolidated-groups",
        "31 January lodgment date; 28 February lodgment date; Due dates for other "
        "consolidated groups (last updated 1 July 2026)",
    ),
    "february": (
        _BY_MONTH + "/february-2027",
        "28 February: head companies of consolidated groups (last updated 1 July 2026)",
    ),
    "may": (
        _BY_MONTH + "/may-2027",
        "15 May: 2026 tax returns for all remaining entities; payment for companies and "
        "super funds (last updated 1 July 2026)",
    ),
    "june": (
        _BY_MONTH + "/june-2027",
        "5 June concession (last updated 1 July 2026)",
    ),
    "super": (
        _PROGRAM + "/due-dates-by-obligation-type/super-lodgment",
        "Table 'Key dates for super providers and for employers with super guarantee "
        "obligations'; Newly registered SMSFs (last updated 1 July 2026)",
    ),
    "self_individual": (
        _ATO + "/individuals-and-families/your-tax-return/before-you-prepare-your-tax-return/"
        "preparing-your-tax-return",
        "Due dates for your tax return (last updated 27 April 2026)",
    ),
    "self_business": (
        _ATO + "/businesses-and-organisations/preparing-lodging-and-paying/reports-and-returns/"
        "income-tax-return",
        "When to lodge: partnerships, trusts and companies (last updated 27 May 2026)",
    ),
    "self_smsf": (
        _ATO + "/print/section/e18658ad-de6e-49f3-b050-095fdf695f83",
        "Self-managed superannuation fund annual return 2026 instructions: Lodgment due "
        "dates, table 'Dates and types of lodgment for self-preparers' (last updated 30 May "
        "2026)",
    ),
    "holidays": (HOLIDAY_SOURCE, HOLIDAY_PROVISION),
}

#: What each condition means, as the ATO pages put it. ``None`` is "not supplied".
FACT_MEANINGS: dict[str, str] = {
    "prior_returns_outstanding": "one or more prior year tax returns were outstanding at "
    "30 June 2026",
    "overdue_returns_lodged_by_31_october": "every return outstanding at 30 June 2026 was "
    "lodged by 31 October 2026, which restores the program date for the 2026 return",
    "on_agent_client_list_by_cutoff": "the client was on the registered agent's client list "
    "by 31 October 2026, so the program covers the 2026 return",
    "large_or_medium": "a large or medium taxpayer as the ATO defines it for return "
    "lodgment (for example annual total income over $10 million; for a super fund, also "
    "investments over $50 million); for a head company, the group under the single entity "
    "rule",
    "member_large_or_medium": "a member of the head company's consolidated group was large "
    "or medium in its latest year lodged",
    "new_registrant": "a new registrant with no earlier return",
    "prior_year_return_taxable": "the 2025 return (the latest year lodged) was taxable",
    "current_year_return_taxable": "the 2026 return being lodged will be taxable",
    "latest_tax_liability_20k_or_more": "the latest return lodged resulted in a tax "
    "liability of $20,000 or more",
    "total_income_over_2m": "total income was over $2 million both in 2024–25 and in the "
    "latest year lodged (for a head company, a member's); null when the 2 differ or are not "
    "known, because ATO pages name each year",
    "prior_year_non_taxable_or_refund": "the prior year return was non-taxable or resulted "
    "in a refund",
    "current_year_non_taxable_or_refund": "the 2026 return will be non-taxable or result in "
    "a refund",
    "consolidated_head_company": "the company is the head company of a consolidated group",
    "first_consolidated_return": "this is the group's first consolidated return",
    "lodges_electronically": "the statement is lodged electronically: for a self-lodger, "
    "received and lodged online; for an agent, lodged through Online services for agents or "
    "the practitioner lodgment service",
    "previous_statement_lodged_electronically": "each statement generates after the client's "
    "previous activity statement was lodged electronically; false for a first statement, a "
    "paper one or one lodged after the next statement generated",
    "includes_monthly_payg_obligation": "the quarterly statement also carries a monthly PAYG "
    "withholding or PAYG instalment obligation",
    "annual_turnover_up_to_10m": "annual turnover is up to $10 million",
    "deferred_gst_registered": "the business is registered for deferred GST",
}


@dataclass(frozen=True)
class Facts:
    """Conditions the program dates depend on. ``None`` means not supplied."""

    prior_returns_outstanding: bool | None = None
    overdue_returns_lodged_by_31_october: bool | None = None
    on_agent_client_list_by_cutoff: bool | None = None
    large_or_medium: bool | None = None
    member_large_or_medium: bool | None = None
    new_registrant: bool | None = None
    prior_year_return_taxable: bool | None = None
    current_year_return_taxable: bool | None = None
    latest_tax_liability_20k_or_more: bool | None = None
    total_income_over_2m: bool | None = None
    prior_year_non_taxable_or_refund: bool | None = None
    current_year_non_taxable_or_refund: bool | None = None
    consolidated_head_company: bool | None = None
    first_consolidated_return: bool | None = None
    lodges_electronically: bool | None = None
    previous_statement_lodged_electronically: bool | None = None
    includes_monthly_payg_obligation: bool | None = None
    annual_turnover_up_to_10m: bool | None = None
    deferred_gst_registered: bool | None = None


@dataclass(frozen=True)
class Rule:
    """One published rule. ``unresolved`` says why the pages cannot settle the date.

    ``payment`` is ``"with_lodgment"`` when the same date covers payment, ``"none"``
    when there is no payment obligation, ``"unknown"`` when the pages leave it open,
    or the separate payment rule.
    """

    id: str
    summary: str
    source: str
    nominal: date | None = None
    unresolved: str | None = None
    payment: Rule | str = "with_lodgment"


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
) -> dict[str, Any]:
    """Return each lodgement, payment and concession date with its rule and status.

    ``financial_year`` is the year of the periods (activity statements) or the income
    year the return covers (income tax returns), in the form 2026-27.
    """
    facts = Facts() if facts is None else facts
    _check(obligation, entity_type, financial_year, channel, reporting_cycle, facts)
    rows: list[dict[str, Any]] = []
    if obligation == "income_tax_return":
        label = f"Income tax return {financial_year}"
        if financial_year != RETURN_YEAR:
            rows += _rows(label, financial_year, _not_recorded(financial_year, RETURN_YEAR), [])
        else:
            rule, missing, concessions = _return_rule(entity_type, channel, facts)
            # A partnership pays no income tax, whatever its lodgement date turns out to be.
            rows += _rows(label, financial_year, rule, missing,
                          family=_family(entity_type, channel),
                          pending_payment="none" if entity_type == "partnership" else "unknown")
            for concession, needed in concessions:
                rows.append(_row(label, financial_year, "concession", concession, needed))
    elif financial_year != ACTIVITY_YEAR:
        rows += _rows(f"{obligation.upper()} {financial_year}", financial_year,
                      _not_recorded(financial_year, ACTIVITY_YEAR), [])
    else:
        for period, label, rule, missing in _statement_rules(
                obligation, entity_type, channel, reporting_cycle, facts):
            rows += _rows(label, period, rule, missing, family=_family(obligation, channel))
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
        "holiday_calendar": {
            "definition": "Saturdays, Sundays and days that are a public holiday for the "
            "whole of any state or territory move a due date to the next business day, for "
            "taxpayers in every state.",
            "source_url": HOLIDAY_SOURCE,
            "source_provision": HOLIDAY_PROVISION,
            "covers_from": COVERS_FROM.isoformat(),
            "covers_to": COVERS_TO.isoformat(),
        },
        "not_decided": NOT_DECIDED,
        "warnings": WARNINGS,
        "source_checked": SOURCE_CHECKED,
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
        if value is not None and type(value) is not bool:
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


def _rows(label: str, period: str, rule: Rule | None, missing: list[str], *,
          family: str = "", pending_payment: str = "unknown") -> list[dict[str, Any]]:
    if rule is None or missing:
        pending = Rule(family, "The date depends on facts not supplied: "
                       + "; ".join(FACT_MEANINGS[name] for name in missing) + ".", "about")
        return [_row(label, period, "lodgment", pending, missing, payment=pending_payment)]
    payment = rule.payment
    rows = [_row(label, period, "lodgment", rule, [],
                 payment=payment if isinstance(payment, str) else "separate")]
    if isinstance(payment, Rule):
        rows.append(_row(label, period, "payment", payment, []))
    return rows


def _row(label: str, period: str, kind: str, rule: Rule, missing: list[str], *,
         payment: str | None = None) -> dict[str, Any]:
    url, provision = SOURCES[rule.source]
    needed = [f"{name}: {FACT_MEANINGS[name]}" for name in missing]
    due: date | None = None
    holiday_roll = None
    if rule.unresolved is not None:
        needed.append(rule.unresolved)
    elif rule.nominal is not None and not needed:
        due, skipped, open_question = roll(rule.nominal)
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
    row: dict[str, Any] = {
        "obligation": label,
        "period": period,
        "kind": kind,
        "status": UNKNOWN if needed else DETERMINED,
        "due_date": None if needed or due is None else due.isoformat(),
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


class _Ask:
    """Read facts along one decision path, remembering each one not supplied."""

    def __init__(self, facts: Facts) -> None:
        self.facts = facts
        self.missing: list[str] = []

    def __call__(self, name: str) -> bool | None:
        value: bool | None = getattr(self.facts, name)
        if value is None and name not in self.missing:
            self.missing.append(name)
        return value


# Activity statements -------------------------------------------------------------

# Quarter: months covered, original due date, head-company PAYG instalment due date
# (21st of the following month) and the registered agent program date.
_QUARTERS = {
    1: ("July to September 2026", date(2026, 10, 28), date(2026, 10, 21), date(2026, 11, 25)),
    2: ("October to December 2026", date(2027, 2, 28), date(2027, 1, 21), None),
    3: ("January to March 2027", date(2027, 4, 28), date(2027, 4, 21), date(2027, 5, 26)),
    4: ("April to June 2027", date(2027, 7, 28), date(2027, 7, 21), date(2027, 8, 25)),
}
_MONTHS = ("January", "February", "March", "April", "May", "June", "July", "August",
           "September", "October", "November", "December")
_TWO_WEEKS = timedelta(days=14)

Statement = tuple[str, str, Rule | None, list[str]]


def _statement_rules(obligation: str, entity_type: str, channel: str, cycle: str | None,
                     facts: Facts) -> list[Statement]:
    name = obligation.upper()
    source = "bas" if obligation == "bas" else "generate"
    statements: list[Statement] = []
    if cycle == "monthly":
        for offset in range(12):
            month, year = (6 + offset) % 12 + 1, 2026 + (6 + offset) // 12
            due = date(year + month // 12, month % 12 + 1, 21)
            standard = Rule("AS-M-21", f"Monthly {name}: lodge and pay by the 21st day of "
                            "the following month. No program or two-week concession applies.",
                            source, due)
            rule, missing = standard, []
            if obligation == "bas" and month == 12 and channel != "self":
                rule, missing = _december(facts, standard)
            statements.append((f"{year}-{month:02d}", f"{name} {_MONTHS[month - 1]} {year}",
                               rule, missing))
        return statements
    for quarter, (months, original_due, head_due, program_due) in _QUARTERS.items():
        ask = _Ask(facts)
        original = Rule("AS-Q-28", f"Quarterly {name}: lodge and pay by the 28th day of the "
                        "month after the quarter, or 28 February for quarter 2, which already "
                        "includes a one-month extension.", source, original_due)
        rule: Rule | None = original
        if entity_type == "company" and ask("consolidated_head_company"):
            rule = _head_company_statement(obligation, head_due)
        elif program_due is None:
            # Quarter 2 already includes a one-month extension, so no concession applies.
            rule = original
        elif channel == "self":
            online = ask("lodges_electronically")
            rule = (Rule("AS-Q-ONLINE", "Quarterly statement received and lodged online: "
                         "lodge and pay 2 weeks after the original due date. Not available "
                         "for quarter 2.", "two_week", original_due + _TWO_WEEKS)
                    if online else original if online is False else None)
        else:
            rule = _agent_quarter(ask, quarter, original, program_due)
        statements.append((f"2026-27 Q{quarter}", f"{name} quarter {quarter} 2026-27 ({months})",
                           rule, ask.missing))
    return statements


def _head_company_statement(obligation: str, head_due: date) -> Rule:
    if obligation == "ias":
        return Rule("AS-Q-HEAD", "Quarterly PAYG instalment activity statement of a "
                    "consolidated group's head company: lodge and pay by the 21st day of the "
                    "month after the quarter. Not eligible for program dates.", "agent_as",
                    head_due)
    return Rule("AS-Q-HEAD-BAS", "A head company's quarterly PAYG instalment statement is due "
                "on the 21st and excluded from program dates; whether this BAS is that "
                "statement is not settled by the facts.", "agent_as",
                unresolved="whether this BAS is the head company's PAYG instalment activity "
                "statement (due on the 21st) or a separate statement due on the 28th")


def _agent_quarter(ask: _Ask, quarter: int, original: Rule, program_due: date) -> Rule | None:
    if ask("includes_monthly_payg_obligation"):
        return Rule("AS-Q-AGENT-MONTHLY-PAYG", "Quarterly statement that also carries a "
                    "monthly PAYG obligation: not eligible for program dates, so the original "
                    "due date stands.", "agent_as", original.nominal)
    electronic = ask("lodges_electronically")
    if electronic is False:
        return Rule("AS-Q-AGENT-PAPER", "Quarterly statement lodged on paper: no program date "
                    "applies, so the original due date stands.", "agent_as", original.nominal)
    if electronic is None:
        return None
    previous = ask("previous_statement_lodged_electronically")
    if previous is False:
        return Rule("AS-Q-AGENT-FIRST-OR-PAPER", "Program dates apply to statements that "
                    "generate after the previous one was lodged electronically. For a first "
                    "statement, or one after a paper lodgement, an earlier date may apply.",
                    "agent_as", original.nominal,
                    unresolved="the due date for a first statement or one following a paper "
                    "lodgement: the ATO says the client may need to lodge by an earlier date "
                    "than the program date")
    if previous is None or ask.missing:
        return None
    rule = Rule("AS-Q-AGENT", "Eligible quarterly statement lodged electronically by a "
                "registered tax or BAS agent: lodge and pay by the program date.", "bas_agent",
                program_due)
    if quarter == 4:
        return Rule(rule.id, rule.summary, rule.source, program_due,
                    unresolved="confirmation of the quarter 4, 2026–27 program date: the ATO "
                    "says 25 August 2027 is to be confirmed when the 2027–28 program is "
                    "developed")
    return rule


def _december(facts: Facts, standard: Rule) -> tuple[Rule | None, list[str]]:
    ask = _Ask(facts)
    turnover = ask("annual_turnover_up_to_10m")
    deferred = ask("deferred_gst_registered")
    electronic = ask("lodges_electronically")
    if turnover is False or deferred or electronic is False:
        return standard, []
    previous = ask("previous_statement_lodged_electronically")
    if ask.missing:
        return None, ask.missing
    if not previous:
        return Rule("AS-M-DEC-AGENT-FIRST-OR-PAPER", "The 21 February program date applies to "
                    "December statements that generate after the previous one was lodged "
                    "electronically; otherwise an earlier date may apply.", "agent_as",
                    standard.nominal,
                    unresolved="the due date for a first statement or one following a paper "
                    "lodgement: the ATO says the client may need to lodge by an earlier date "
                    "than the program date"), []
    return Rule("AS-M-DEC-AGENT", "December monthly BAS of a business with up to $10 million "
                "turnover that reports GST monthly, not registered for deferred GST, lodged "
                "electronically by a registered agent: lodge and pay by 21 February.",
                "bas_agent", date(2027, 2, 21)), []


# Income tax returns for 2025–26 (the 2026 return) ----------------------------------

_OCT31, _DEC1, _JAN31 = date(2026, 10, 31), date(2026, 12, 1), date(2027, 1, 31)
_FEB28, _MAR31, _MAY15, _JUN5 = (date(2027, 2, 28), date(2027, 3, 31), date(2027, 5, 15),
                                 date(2027, 6, 5))
_NOA = ("the notice of assessment, which sets the payment due date: for a return lodged "
        "on time, 21 days after the later of the lodgment due date and the day the notice is "
        "deemed received (7 business days after issue)")


def _pay_dec1(source: str) -> Rule:
    return Rule("PAY-1DEC-2026", "Pay any tax by 1 December 2026.", source, _DEC1)


_PAY_NOA = Rule("PAY-NOA", "Individuals and trusts pay as advised on the notice of "
                "assessment; for a late return, 21 days after the lodgment due date.",
                "ind_trust", unresolved=_NOA)
_PAY_IT_15MAY = Rule(
    "PAY-IT-15MAY", "Staggered payment for returns due 15 May 2027: lodged by 12 February "
    "2027, pay by 21 March 2027; lodged 13 February to 12 March 2027, by 21 April 2027; "
    "lodged from 13 March 2027, by 5 June 2027. These are the latest dates if processing "
    "takes 2 weeks; a later notice of assessment sets a later date.", "ind_trust",
    unresolved="the lodgement date and the notice of assessment, which set the payment due "
    "date")

_SELF = {
    "individual": Rule(
        "ITR-SELF-IND", "Self-lodged individual return: lodge by 31 October; on a weekend, "
        "the next business day.", "self_individual", _OCT31,
        payment=Rule("PAY-SELF-IND", "A self-lodged return with a tax bill lodged by 31 "
                     "October: pay by 21 November, or 21 days after the assessment issues if "
                     "that is after 31 October. Lodged late: pay by 21 November.",
                     "self_individual", date(2026, 11, 21),
                     unresolved="the lodgement date and the assessment issue date: for a "
                     "return lodged by 31 October whose assessment issues after 31 October, "
                     "payment is 21 days after issue")),
    "trust": Rule(
        "ITR-SELF-TRUST", "Self-lodged trust return: lodge by 31 October.", "self_business",
        _OCT31, payment=Rule("PAY-SELF-TRUST", "Any tax assessed to the trustee is payable "
                             "as advised on the notice of assessment.", "self_business",
                             unresolved="the notice of assessment: the self-lodger page "
                             "gives no payment date for a trust")),
    "partnership": Rule(
        "ITR-SELF-PARTNERSHIP", "Self-lodged partnership return: lodge by 31 October. A "
        "partnership pays no income tax; the partners pay on their own returns.",
        "self_business", _OCT31, payment="none"),
}
_SELF_CO_HEAD = Rule("ITR-SELF-CO-HEAD", "The ATO's self-lodger pages publish no date for "
                     "the head company of a consolidated group.", "self_business",
                     unresolved="a self-lodger due date for a consolidated group's head "
                     "company, which the ATO pages do not publish", payment="unknown")
_SELF_CO_OVERDUE = Rule(
    "ITR-SELF-CO-OVERDUE", "Self-lodged company with prior year returns outstanding: lodge "
    "by 31 October.", "self_business", _OCT31,
    payment=Rule("PAY-SELF-CO-OVERDUE", "The self-lodger page gives no payment date for "
                 "this case.", "self_business", unresolved="whether the 1 December 2026 "
                 "payment date on the agent program's overdue returns page applies to a "
                 "self-lodger: that page addresses registered agents and the self-lodger page "
                 "gives no payment date"))
_SELF_CO = Rule("ITR-SELF-CO", "Self-lodged company with no prior year returns "
                "outstanding: the ATO says the date for small companies is generally 28 "
                "February and asks self-lodgers to check their date.", "self_business",
                _FEB28, unresolved="which companies the self-lodger page's 28 February date "
                "covers: it applies 'generally' to 'small companies', which the page does not "
                "define", payment="unknown")
_SELF_SMSF_PAY = Rule("PAY-SELF-SMSF-1DEC", "Pay any tax by 1 December 2026.", "self_smsf",
                      _DEC1)
_SELF_SMSF_OCT = Rule("ITR-SELF-SMSF-31OCT", "Self-preparing SMSF that is a new registrant "
                      "or has an annual return overdue at 30 June 2026 (unless deferred): "
                      "lodge by 31 October 2026. The earliest applicable date applies.",
                      "self_smsf", _OCT31, payment=_SELF_SMSF_PAY)
_SELF_SMSF_JAN = Rule("ITR-SELF-SMSF-31JAN", "Self-preparing SMSF that was a taxable large or "
                      "medium entity in 2025–26: lodge by 31 January 2027.", "self_smsf",
                      _JAN31, payment=_SELF_SMSF_PAY)
_SELF_SMSF_FEB = Rule("ITR-SELF-SMSF-28FEB", "All other self-preparing SMSFs: lodge and pay "
                      "by 28 February 2027.", "self_smsf", _FEB28)

_NOT_COVERED = Rule("ITR-AGENT-NOT-COVERED", "A client added to the agent's client list after "
                    "31 October 2026 is not covered by the program for the 2026 return.",
                    "october", unresolved="the due date for a client the program does not "
                    "cover, which the ATO pages do not state", payment="unknown")


def _overdue(entity_type: str) -> Rule:
    summary = ("Prior year returns outstanding at 30 June 2026 and not all lodged by 31 "
               "October 2026: lodge the 2026 return by 31 October 2026.")
    if entity_type in {"individual", "trust"}:
        return Rule("ITR-AGENT-OVERDUE", summary, "ind_trust", _OCT31, payment=_PAY_NOA)
    if entity_type == "partnership":
        return Rule("ITR-AGENT-OVERDUE", summary, "october", _OCT31, payment="none")
    source = "super" if entity_type == "smsf" else "co_super"
    return Rule("ITR-AGENT-OVERDUE", summary, source, _OCT31, payment=_pay_dec1("overdue"))


_IT_31MAR = Rule("ITR-AGENT-IT-31MAR", "Individual or trust (other than a large or medium "
                 "trust) whose latest return had a tax liability of $20,000 or more: lodge by "
                 "31 March 2027.", "ind_trust", _MAR31, payment=_PAY_NOA)
_IT_15MAY = Rule("ITR-AGENT-IT-15MAY", "All remaining individuals and trusts, including new "
                 "registrants: lodge by 15 May 2027.", "ind_trust", _MAY15,
                 payment=_PAY_IT_15MAY)
_CON_IT = Rule("CON-IT-5JUN", "Concession for returns due 15 May 2027: lodged with any "
               "payment by 5 June 2027, no penalty applies.", "ind_trust", _JUN5)
_TRUST_LM = {
    "new": Rule("ITR-AGENT-TRUST-LM-NEW", "New registrant large or medium trust: lodge by 28 "
                "February 2027.", "ind_trust", _FEB28, payment=_PAY_NOA),
    True: Rule("ITR-AGENT-TRUST-LM-31JAN", "Large or medium trust, taxable in the latest year "
               "lodged: lodge by 31 January 2027.", "ind_trust", _JAN31, payment=_PAY_NOA),
    False: Rule("ITR-AGENT-TRUST-LM-28FEB", "Large or medium trust, non-taxable in the latest "
                "year lodged: lodge by 28 February 2027.", "ind_trust", _FEB28,
                payment=_PAY_NOA),
}
_PART_15MAY = Rule("ITR-AGENT-PART-15MAY", "Partnership not required to lodge earlier: lodge "
                   "by 15 May 2027, progressively so the partners can meet their own dates. "
                   "A partnership pays no income tax.", "may", _MAY15, payment="none")
_CON_PART = Rule("CON-PART-5JUN", "Concession for partnership returns due 15 May 2027: "
                 "lodged with any payment by 5 June 2027, no penalty applies.", "ind_trust",
                 _JUN5)
_CO_LM = {
    "new": Rule("ITR-AGENT-CO-LM-NEW", "New registrant large or medium company: lodge and "
                "pay by 28 February 2027.", "co_super", _FEB28),
    True: Rule("ITR-AGENT-CO-LM-31JAN", "Large or medium company whose 2025 return was "
               "taxable: lodge by 31 January 2027.", "large_medium", _JAN31,
               payment=_pay_dec1("large_medium")),
    False: Rule("ITR-AGENT-CO-LM-28FEB", "Large or medium company whose 2025 return was "
                "non-taxable or not necessary: lodge and pay (if required) by 28 February "
                "2027.", "large_medium", _FEB28),
}
_CO_31MAR = Rule("ITR-AGENT-CO-31MAR", "Company (not large or medium) with total income over "
                 "$2 million in the latest year lodged: lodge and pay by 31 March 2027.",
                 "co_super", _MAR31)
_CO_15MAY = Rule("ITR-AGENT-CO-15MAY", "All remaining companies that are tax agent clients, "
                 "including new registrants: lodge and pay by 15 May 2027.", "co_super", _MAY15)
_CON_CO = Rule("CON-CO-5JUN", "Concession for a company or super fund return due 15 May 2027 "
               "that is non-taxable or a refund in both the prior and current year: lodged by "
               "5 June 2027, no penalty applies.", "co_super", _JUN5)
_HEAD_FIRST = Rule("ITR-AGENT-HEAD-FIRST", "A head company's first consolidated return is due "
                   "on the latest date the group's notification of formation can be lodged.",
                   "consolidated", unresolved="the latest date the group's Notification of "
                   "formation of an income tax consolidated group can be lodged", payment="unknown")
_HEAD_LM = {
    True: Rule("ITR-AGENT-HEAD-LM-31JAN", "Head company (including a new registrant) of a "
               "group with a large or medium member, or large or medium under the single "
               "entity rule, whose 2026 return will be taxable: lodge by 31 January 2027.",
               "consolidated", _JAN31, payment=_pay_dec1("consolidated")),
    False: Rule("ITR-AGENT-HEAD-LM-28FEB", "Head company (including a new registrant) of a "
                "group with a large or medium member, or large or medium under the single "
                "entity rule, whose 2026 return will be non-taxable: lodge by 28 February "
                "2027.", "consolidated", _FEB28, payment="none"),
}
_HEAD_SPLIT = Rule("ITR-AGENT-HEAD-LM-SPLIT", "Head company with a large or medium "
                   "member or a large or medium group, but not both.", "consolidated",
                   unresolved="whether the 31 January or 28 February date applies when only one "
                   "of a large or medium member and a large or medium group holds: the "
                   "consolidated groups page lists both conditions without saying whether "
                   "each is needed", payment="unknown")
_HEAD_NEW = Rule("ITR-AGENT-HEAD-NEW", "New registrant head company without a large or medium "
                 "member.", "co_super", unresolved="the date for a new registrant head company "
                 "without a large or medium member: the companies and super funds table gives "
                 "28 February 2027 and the consolidated groups page gives 31 March or 15 May "
                 "2027", payment="unknown")
_HEAD_31MAR = Rule("ITR-AGENT-HEAD-31MAR", "Head company without a large or medium member, "
                   "with a member whose total income exceeded $2 million in the latest year "
                   "lodged: lodge and pay by 31 March 2027.", "consolidated", _MAR31)
_HEAD_15MAY = Rule("ITR-AGENT-HEAD-15MAY", "All other head companies: lodge and pay (if "
                   "required) by 15 May 2027. The 5 June concession excludes head companies.",
                   "may", _MAY15)
_SMSF_LM = {
    True: Rule("ITR-AGENT-SMSF-LM-31JAN", "Large or medium super fund, taxable in the latest "
               "year lodged: lodge by 31 January 2027.", "super", _JAN31,
               payment=_pay_dec1("super")),
    False: Rule("ITR-AGENT-SMSF-LM-28FEB", "Large or medium super fund, non-taxable in the "
                "latest year lodged: lodge and pay (if required) by 28 February 2027.",
                "co_super", _FEB28),
}
_SMSF_NEW = Rule("ITR-AGENT-SMSF-NEW", "New registrant SMSF, taxable or not, unless advised "
                 "of a 31 October 2026 date at its registration review: lodge and pay by 28 "
                 "February 2027.", "super", _FEB28)
_SMSF_31MAR = Rule("ITR-AGENT-SMSF-31MAR", "Super fund (not large or medium) with total "
                   "income over $2 million in the latest year lodged: lodge and pay by 31 "
                   "March 2027.", "super", _MAR31)
_SMSF_15MAY = Rule("ITR-AGENT-SMSF-15MAY", "Fund return not required earlier: lodge and pay "
                   "by 15 May 2027.", "super", _MAY15)
_CON_SMSF = Rule("CON-SMSF-5JUN", "Concession for a fund return due 15 May 2027 that was "
                 "non-taxable or a refund in the latest year lodged and will be in the current "
                 "year: lodged by 5 June 2027, no penalty applies.", "super", _JUN5)

Concessions = list[tuple[Rule, list[str]]]


def _return_rule(entity_type: str, channel: str,
                 facts: Facts) -> tuple[Rule | None, list[str], Concessions]:
    ask = _Ask(facts)
    if channel == "self":
        return _self_return(entity_type, ask), ask.missing, []
    if ask("on_agent_client_list_by_cutoff") is False:
        return _NOT_COVERED, [], []
    rule, concession = _agent_return(entity_type, ask)
    concessions: Concessions = []
    if concession is not None and not ask.missing:
        if concession in (_CON_CO, _CON_SMSF):
            eligible = _Ask(facts)
            both = (eligible("prior_year_non_taxable_or_refund"),
                    eligible("current_year_non_taxable_or_refund"))
            if False not in both:
                concessions.append((concession, eligible.missing))
        else:
            concessions.append((concession, []))
    return rule, ask.missing, concessions


def _self_return(entity_type: str, ask: _Ask) -> Rule | None:
    if entity_type in _SELF:
        return _SELF[entity_type]
    if entity_type == "company":
        if ask("consolidated_head_company"):
            return _SELF_CO_HEAD
        if ask("prior_returns_outstanding"):
            return _SELF_CO_OVERDUE
        return _SELF_CO
    new, overdue = ask("new_registrant"), ask("prior_returns_outstanding")
    if new or overdue:
        # The earliest date that applies wins, so a fact not supplied cannot change it.
        ask.missing.clear()
        return _SELF_SMSF_OCT
    if ask.facts.current_year_return_taxable is False or not ask("large_or_medium"):
        return _SELF_SMSF_FEB
    taxable = ask("current_year_return_taxable")
    return _SELF_SMSF_JAN if taxable else None


def _agent_return(entity_type: str, ask: _Ask) -> tuple[Rule | None, Rule | None]:
    if entity_type == "company" and ask("consolidated_head_company"):
        return _head_company_return(ask), None
    if _still_overdue(ask):
        return _overdue(entity_type), None
    if entity_type == "partnership":
        return _PART_15MAY, _CON_PART
    # A new registrant has no earlier return, so a test on the latest return cannot apply.
    new = ask.facts.new_registrant is True
    if entity_type in {"individual", "trust"}:
        if entity_type == "trust" and ask("large_or_medium"):
            return _large_or_medium(ask, _TRUST_LM), None
        if not new and ask("latest_tax_liability_20k_or_more"):
            return _IT_31MAR, None
        return _IT_15MAY, _CON_IT
    if entity_type == "smsf" and ask("new_registrant"):
        return _SMSF_NEW, None
    if ask("large_or_medium"):
        return _large_or_medium(ask, _CO_LM if entity_type == "company" else _SMSF_LM), None
    if not new and ask("total_income_over_2m"):
        return (_CO_31MAR if entity_type == "company" else _SMSF_31MAR), None
    if entity_type == "company":
        return _CO_15MAY, _CON_CO
    return _SMSF_15MAY, _CON_SMSF


def _still_overdue(ask: _Ask) -> bool | None:
    """Whether the 31 October 2026 overdue-return date applies to an agent client."""
    if not ask("prior_returns_outstanding"):
        return ask.facts.prior_returns_outstanding
    cleared = ask("overdue_returns_lodged_by_31_october")
    return None if cleared is None else not cleared


def _large_or_medium(ask: _Ask, table: dict[Any, Rule]) -> Rule | None:
    if "new" in table and ask("new_registrant"):
        return table["new"]
    taxable = ask("prior_year_return_taxable")
    return None if taxable is None else table[taxable]


def _head_company_return(ask: _Ask) -> Rule | None:
    if ask("first_consolidated_return"):
        return _HEAD_FIRST
    if _still_overdue(ask):
        return _overdue("company")
    member = ask("member_large_or_medium")
    if member:
        taxable = ask("current_year_return_taxable")
        if taxable is False:
            # The February 2027 page: a non-taxable head company with such a member.
            return _HEAD_LM[False]
        group = ask("large_or_medium")
        if taxable is None or group is None:
            return None
        return _HEAD_LM[True] if group else _HEAD_SPLIT
    if ask("large_or_medium"):
        return None if member is None else _HEAD_SPLIT
    if ask("new_registrant"):
        return _HEAD_NEW
    if ask("total_income_over_2m"):
        return _HEAD_31MAR
    return _HEAD_15MAY
