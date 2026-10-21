use crate::{Ask, Rule, data};
use serde_json::Value;

fn rule(name: &str) -> Rule {
    serde_json::from_value(data()["return_rules"][name].clone()).expect("recorded rule")
}
fn table(name: &str, key: &str) -> Rule {
    serde_json::from_value(data()["return_rules"][name][key].clone()).expect("recorded rule table")
}
fn overdue(entity: &str) -> Rule {
    table("overdue", entity)
}
type Selection = (Option<Rule>, Vec<String>, Vec<(Rule, Vec<String>)>);

pub(crate) fn select(entity: &str, channel: &str, facts: &Value) -> Selection {
    let mut ask = Ask::new(facts);
    if channel == "self" {
        return (self_return(entity, &mut ask), ask.missing, vec![]);
    }
    if ask.get("on_agent_client_list_by_cutoff") == Some(false) {
        return (Some(rule("_NOT_COVERED")), vec![], vec![]);
    }
    let (selected, concession) = agent_return(entity, &mut ask);
    let mut concessions = vec![];
    if let Some(concession) = concession.filter(|_| ask.missing.is_empty()) {
        if ["CON-CO-5JUN", "CON-SMSF-5JUN"].contains(&concession.id.as_str()) {
            let mut eligible = Ask::new(facts);
            let prior = eligible.get("prior_year_non_taxable_or_refund");
            let current = eligible.get("current_year_non_taxable_or_refund");
            if prior != Some(false) && current != Some(false) {
                concessions.push((concession, eligible.missing));
            }
        } else {
            concessions.push((concession, vec![]));
        }
    }
    (selected, ask.missing, concessions)
}

fn self_return(entity: &str, ask: &mut Ask<'_>) -> Option<Rule> {
    if ["individual", "trust", "partnership"].contains(&entity) {
        return Some(table("_SELF", entity));
    }
    if entity == "company" {
        if ask.yes("consolidated_head_company") {
            return Some(rule("_SELF_CO_HEAD"));
        }
        if ask.yes("prior_returns_outstanding") {
            return Some(rule("_SELF_CO_OVERDUE"));
        }
        return Some(rule("_SELF_CO"));
    }
    let new = ask.yes("new_registrant");
    let overdue = ask.yes("prior_returns_outstanding");
    if new || overdue {
        ask.missing.clear();
        return Some(rule("_SELF_SMSF_OCT"));
    }
    if ask.facts["current_year_return_taxable"] == false || !ask.yes("large_or_medium") {
        return Some(rule("_SELF_SMSF_FEB"));
    }
    ask.yes("current_year_return_taxable")
        .then(|| rule("_SELF_SMSF_JAN"))
}

fn still_overdue(ask: &mut Ask<'_>) -> bool {
    if !ask.yes("prior_returns_outstanding") {
        return false;
    }
    ask.get("overdue_returns_lodged_by_31_october") == Some(false)
}

fn large_medium(ask: &mut Ask<'_>, name: &str) -> Option<Rule> {
    if data()["return_rules"][name].get("new").is_some() && ask.yes("new_registrant") {
        return Some(table(name, "new"));
    }
    ask.get("prior_year_return_taxable")
        .map(|b| table(name, if b { "True" } else { "False" }))
}

fn agent_return(entity: &str, ask: &mut Ask<'_>) -> (Option<Rule>, Option<Rule>) {
    if entity == "company" && ask.yes("consolidated_head_company") {
        return (head(ask), None);
    }
    if still_overdue(ask) {
        return (Some(overdue(entity)), None);
    }
    if entity == "partnership" {
        return (Some(rule("_PART_15MAY")), Some(rule("_CON_PART")));
    }
    let new = ask.facts["new_registrant"] == true;
    if ["individual", "trust"].contains(&entity) {
        if entity == "trust" && ask.yes("large_or_medium") {
            return (large_medium(ask, "_TRUST_LM"), None);
        }
        if !new && ask.yes("latest_tax_liability_20k_or_more") {
            return (Some(rule("_IT_31MAR")), None);
        }
        return (Some(rule("_IT_15MAY")), Some(rule("_CON_IT")));
    }
    if entity == "smsf" && ask.yes("new_registrant") {
        return (Some(rule("_SMSF_NEW")), None);
    }
    if ask.yes("large_or_medium") {
        return (
            large_medium(
                ask,
                if entity == "company" {
                    "_CO_LM"
                } else {
                    "_SMSF_LM"
                },
            ),
            None,
        );
    }
    if !new && ask.yes("total_income_over_2m") {
        return (
            Some(rule(if entity == "company" {
                "_CO_31MAR"
            } else {
                "_SMSF_31MAR"
            })),
            None,
        );
    }
    if entity == "company" {
        (Some(rule("_CO_15MAY")), Some(rule("_CON_CO")))
    } else {
        (Some(rule("_SMSF_15MAY")), Some(rule("_CON_SMSF")))
    }
}

fn head(ask: &mut Ask<'_>) -> Option<Rule> {
    if ask.yes("first_consolidated_return") {
        return Some(rule("_HEAD_FIRST"));
    }
    if still_overdue(ask) {
        return Some(overdue("company"));
    }
    let member = ask.get("member_large_or_medium");
    if member == Some(true) {
        let taxable = ask.get("current_year_return_taxable");
        if taxable == Some(false) {
            return Some(table("_HEAD_LM", "False"));
        }
        let group = ask.get("large_or_medium");
        if taxable.is_none() || group.is_none() {
            return None;
        }
        return Some(if group == Some(true) {
            table("_HEAD_LM", "True")
        } else {
            rule("_HEAD_SPLIT")
        });
    }
    if ask.yes("large_or_medium") {
        return member.map(|_| rule("_HEAD_SPLIT"));
    }
    if ask.yes("new_registrant") {
        return Some(rule("_HEAD_NEW"));
    }
    if ask.yes("total_income_over_2m") {
        return Some(rule("_HEAD_31MAR"));
    }
    Some(rule("_HEAD_15MAY"))
}
