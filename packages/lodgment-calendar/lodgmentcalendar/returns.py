"""Income tax return rules for 2025–26 (the 2026 return)."""

from __future__ import annotations

from datetime import date
from typing import Any

from .rules import Ask, Facts, Rule

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


def return_rule(entity_type: str, channel: str,
                 facts: Facts) -> tuple[Rule | None, list[str], Concessions]:
    ask = Ask(facts)
    if channel == "self":
        return _self_return(entity_type, ask), ask.missing, []
    if ask("on_agent_client_list_by_cutoff") is False:
        return _NOT_COVERED, [], []
    rule, concession = _agent_return(entity_type, ask)
    concessions: Concessions = []
    if concession is not None and not ask.missing:
        if concession in (_CON_CO, _CON_SMSF):
            eligible = Ask(facts)
            both = (eligible("prior_year_non_taxable_or_refund"),
                    eligible("current_year_non_taxable_or_refund"))
            if False not in both:
                concessions.append((concession, eligible.missing))
        else:
            concessions.append((concession, []))
    return rule, ask.missing, concessions


def _self_return(entity_type: str, ask: Ask) -> Rule | None:
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


def _agent_return(entity_type: str, ask: Ask) -> tuple[Rule | None, Rule | None]:
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


def _still_overdue(ask: Ask) -> bool | None:
    """Whether the 31 October 2026 overdue-return date applies to an agent client."""
    if not ask("prior_returns_outstanding"):
        return ask.facts.prior_returns_outstanding
    cleared = ask("overdue_returns_lodged_by_31_october")
    return None if cleared is None else not cleared


def _large_or_medium(ask: Ask, table: dict[Any, Rule]) -> Rule | None:
    if "new" in table and ask("new_registrant"):
        return table["new"]
    taxable = ask("prior_year_return_taxable")
    return None if taxable is None else table[taxable]


def _head_company_return(ask: Ask) -> Rule | None:
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
