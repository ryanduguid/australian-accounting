"""The ATO pages each rule comes from, the facts a date can depend on, and the
rule and fact-reading types the statement and return modules share."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import date

from .holidays import HOLIDAY_PROVISION, HOLIDAY_SOURCE

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


class Ask:
    """Read facts along one decision path, remembering each one not supplied."""

    def __init__(self, facts: Facts) -> None:
        self.facts = facts
        self.missing: list[str] = []

    def __call__(self, name: str) -> bool | None:
        value: bool | None = getattr(self.facts, name)
        if value is None and name not in self.missing:
            self.missing.append(name)
        return value
