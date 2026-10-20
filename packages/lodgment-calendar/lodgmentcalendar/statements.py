"""Activity statement rules: monthly and quarterly BAS and IAS for 2026–27."""

from __future__ import annotations

from datetime import date, timedelta

from .rules import Ask, Facts, Rule

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


def statement_rules(obligation: str, entity_type: str, channel: str, cycle: str | None,
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
        ask = Ask(facts)
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


def _agent_quarter(ask: Ask, quarter: int, original: Rule, program_due: date) -> Rule | None:
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
    ask = Ask(facts)
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
