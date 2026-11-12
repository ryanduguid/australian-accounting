// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! FBT return rules: the 2027 return, for the FBT year ending 31 March 2027.
use crate::{Payment, Rule, date};
use serde_json::Value;

const FACTS: [&str; 2] = ["lodges_electronically", "on_fbt_client_list_by_21_may"];

/// The rule for this channel, or the facts on its path that were not supplied.
pub(crate) fn select(channel: &str, facts: &Value) -> (Option<Rule>, Vec<String>) {
    let may = Some(date(2027, 5, 21));
    if channel == "self" {
        let summary = "FBT return not lodged by a tax agent: lodge and pay by 21 May.";
        return (
            Some(Rule::new("FBT-21-MAY", summary, "fbt_self", may)),
            vec![],
        );
    }
    if channel == "bas_agent" {
        let summary = "The ATO's FBT pages give program dates to tax agents and the BAS agent program lists no FBT date.";
        let mut rule = Rule::new("FBT-BAS-AGENT", summary, "fbt_agent", may).unresolved(
            "whether a BAS agent can lodge an FBT return: the ATO's FBT pages name only tax agents, and its BAS agent program page lists no FBT date",
        );
        rule.payment = Payment::Mode("unknown".into());
        return (Some(rule), vec![]);
    }
    let electronic = facts["lodges_electronically"].as_bool();
    let listed = facts["on_fbt_client_list_by_21_may"].as_bool();
    // Either fact alone can settle 21 May, so neither is asked before the other.
    let rule = match (electronic, listed) {
        (Some(false), _) => Rule::new(
            "FBT-AGENT-PAPER",
            "FBT return a tax agent lodges by paper: lodge and pay by 21 May.",
            "fbt_agent",
            may,
        ),
        (_, Some(false)) => Rule::new(
            "FBT-AGENT-NOT-LISTED",
            "Client not on the tax agent's FBT client list by 21 May, so outside the lodgment program: lodge and pay by 21 May.",
            "fbt_agent",
            may,
        ),
        (Some(true), Some(true)) => Rule::new(
            "FBT-AGENT-ELECTRONIC",
            "FBT return a tax agent lodges electronically, for a client on its FBT client list by 21 May: generally lodge and pay by 25 June.",
            "fbt_agent",
            Some(date(2027, 6, 25)),
        ),
        _ => {
            let missing = FACTS.iter().filter(|name| facts[**name].is_null());
            return (None, missing.map(|name| name.to_string()).collect());
        }
    };
    (Some(rule), vec![])
}
