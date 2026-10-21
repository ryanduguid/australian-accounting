use crate::{Ask, Rule, date};
use chrono::{Days, NaiveDate};
use serde_json::Value;

type Statement = (String, String, Option<Rule>, Vec<String>);
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const EARLIER: &str = "the due date for a first statement or one following a paper lodgement: the ATO says the client may need to lodge by an earlier date than the program date";

pub(crate) fn select(
    obligation: &str,
    entity: &str,
    channel: &str,
    cycle: &str,
    facts: &Value,
) -> Vec<Statement> {
    let name = obligation.to_uppercase();
    let source = if obligation == "bas" {
        "bas"
    } else {
        "generate"
    };
    let mut result = vec![];
    if cycle == "monthly" {
        for offset in 0..12 {
            let month = (6 + offset) % 12 + 1;
            let year = 2026 + (6 + offset) / 12;
            let due = date((year + month / 12) as i32, month % 12 + 1, 21);
            let standard = Rule::new(
                "AS-M-21",
                format!(
                    "Monthly {name}: lodge and pay by the 21st day of the following month. No program or two-week concession applies."
                ),
                source,
                Some(due),
            );
            let (rule, missing) = if obligation == "bas" && month == 12 && channel != "self" {
                december(facts, standard)
            } else {
                (Some(standard), vec![])
            };
            result.push((
                format!("{year}-{month:02}"),
                format!("{name} {} {year}", MONTHS[(month - 1) as usize]),
                rule,
                missing,
            ));
        }
        return result;
    }
    let quarters = [
        (
            "July to September 2026",
            date(2026, 10, 28),
            date(2026, 10, 21),
            Some(date(2026, 11, 25)),
        ),
        (
            "October to December 2026",
            date(2027, 2, 28),
            date(2027, 1, 21),
            None,
        ),
        (
            "January to March 2027",
            date(2027, 4, 28),
            date(2027, 4, 21),
            Some(date(2027, 5, 26)),
        ),
        (
            "April to June 2027",
            date(2027, 7, 28),
            date(2027, 7, 21),
            Some(date(2027, 8, 25)),
        ),
    ];
    for (index, (months, due, head_due, program_due)) in quarters.into_iter().enumerate() {
        let quarter = index + 1;
        let mut ask = Ask::new(facts);
        let original = Rule::new(
            "AS-Q-28",
            format!(
                "Quarterly {name}: lodge and pay by the 28th day of the month after the quarter, or 28 February for quarter 2, which already includes a one-month extension."
            ),
            source,
            Some(due),
        );
        let rule = if entity == "company" && ask.yes("consolidated_head_company") {
            Some(head(obligation, head_due))
        } else if let Some(program_due) = program_due {
            if channel == "self" {
                match ask.get("lodges_electronically") {
                    Some(true) => Some(Rule::new(
                        "AS-Q-ONLINE",
                        "Quarterly statement received and lodged online: lodge and pay 2 weeks after the original due date. Not available for quarter 2.",
                        "two_week",
                        due.checked_add_days(Days::new(14)),
                    )),
                    Some(false) => Some(original),
                    None => None,
                }
            } else {
                agent_quarter(&mut ask, quarter, original, program_due)
            }
        } else {
            Some(original)
        };
        result.push((
            format!("2026-27 Q{quarter}"),
            format!("{name} quarter {quarter} 2026-27 ({months})"),
            rule,
            ask.missing,
        ));
    }
    result
}

fn head(obligation: &str, due: NaiveDate) -> Rule {
    if obligation == "ias" {
        Rule::new(
            "AS-Q-HEAD",
            "Quarterly PAYG instalment activity statement of a consolidated group's head company: lodge and pay by the 21st day of the month after the quarter. Not eligible for program dates.",
            "agent_as",
            Some(due),
        )
    } else {
        Rule::new("AS-Q-HEAD-BAS","A head company's quarterly PAYG instalment statement is due on the 21st and excluded from program dates; whether this BAS is that statement is not settled by the facts.","agent_as",None).unresolved("whether this BAS is the head company's PAYG instalment activity statement (due on the 21st) or a separate statement due on the 28th")
    }
}

fn agent_quarter(
    ask: &mut Ask<'_>,
    quarter: usize,
    original: Rule,
    due: NaiveDate,
) -> Option<Rule> {
    if ask.yes("includes_monthly_payg_obligation") {
        return Some(Rule::new(
            "AS-Q-AGENT-MONTHLY-PAYG",
            "Quarterly statement that also carries a monthly PAYG obligation: not eligible for program dates, so the original due date stands.",
            "agent_as",
            original.nominal,
        ));
    }
    match ask.get("lodges_electronically") {
        Some(false) => {
            return Some(Rule::new(
                "AS-Q-AGENT-PAPER",
                "Quarterly statement lodged on paper: no program date applies, so the original due date stands.",
                "agent_as",
                original.nominal,
            ));
        }
        None => return None,
        Some(true) => {}
    }
    match ask.get("previous_statement_lodged_electronically") {
        Some(false)=>return Some(Rule::new("AS-Q-AGENT-FIRST-OR-PAPER","Program dates apply to statements that generate after the previous one was lodged electronically. For a first statement, or one after a paper lodgement, an earlier date may apply.","agent_as",original.nominal).unresolved(EARLIER)),
        None=>return None, Some(true)=>{},
    }
    if !ask.missing.is_empty() {
        return None;
    }
    let rule = Rule::new(
        "AS-Q-AGENT",
        "Eligible quarterly statement lodged electronically by a registered tax or BAS agent: lodge and pay by the program date.",
        "bas_agent",
        Some(due),
    );
    Some(if quarter == 4 {
        rule.unresolved("confirmation of the quarter 4, 2026–27 program date: the ATO says 25 August 2027 is to be confirmed when the 2027–28 program is developed")
    } else {
        rule
    })
}

fn december(facts: &Value, standard: Rule) -> (Option<Rule>, Vec<String>) {
    let mut ask = Ask::new(facts);
    let turnover = ask.get("annual_turnover_up_to_10m");
    let deferred = ask.get("deferred_gst_registered");
    let electronic = ask.get("lodges_electronically");
    if turnover == Some(false) || deferred == Some(true) || electronic == Some(false) {
        return (Some(standard), vec![]);
    }
    let previous = ask.get("previous_statement_lodged_electronically");
    if !ask.missing.is_empty() {
        return (None, ask.missing);
    }
    if previous != Some(true) {
        return (Some(Rule::new("AS-M-DEC-AGENT-FIRST-OR-PAPER","The 21 February program date applies to December statements that generate after the previous one was lodged electronically; otherwise an earlier date may apply.","agent_as",standard.nominal).unresolved(EARLIER)),vec![]);
    }
    (
        Some(Rule::new(
            "AS-M-DEC-AGENT",
            "December monthly BAS of a business with up to $10 million turnover that reports GST monthly, not registered for deferred GST, lodged electronically by a registered agent: lodge and pay by 21 February.",
            "bas_agent",
            Some(date(2027, 2, 21)),
        )),
        vec![],
    )
}
