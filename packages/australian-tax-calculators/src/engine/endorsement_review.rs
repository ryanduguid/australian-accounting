// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `endorsement_review`: the ATO's 3 endorsement review worksheets, NAT
//! 74110-02.2021 (income tax exempt charities), NAT 11685-02.2021 (Worksheet
//! 1, a DGR endorsed as a whole) and NAT 11686-02.2021 (Worksheet 2, a DGR
//! endorsed for a fund, authority or institution it owns or includes), each
//! answer tied to the provision it tests as the ATO's legal database gave it
//! on 24 November 2026: ITAA 1997 ss 30-15 item 1, 30-125, 30-130, 30-228,
//! 50-5, 50-47, 50-50 and 50-110 and TAA 1953 Sch 1 ss 426-45 and 426-55.
//! `dgr-check` tells a donor whether a recipient is shown as a DGR; this is
//! the entity's own review. The answers are the entity's; nothing is
//! checked against a register.

use serde_json::{Map, Value, json};

use super::{Arg, Result, refuse, result, scope};

const KIND: &str = "endorsement_review";

/// What a 'No' means, most serious first.
const NOT_ENTITLED: &str = "not_entitled";
const ACTION_NEEDED: &str = "action_needed";

/// One question: (answer key, provision, outcome of a 'No', what to do).
type Question = (&'static str, &'static str, &'static str, &'static str);

const NOTICE: &str = "Tell the ATO in writing, before or as soon as practicable after the entity stops being entitled, and give the date it stopped (TAA 1953 Sch 1 s 426-45(1)).";
const ABN_GONE: &str = "Entitlement ended with the ABN; the ATO will have given notice of revocation, and no notice from the entity is needed for this (TAA 1953 Sch 1 s 426-45(3)).";

const CHARITY: [Question; 4] = [
    (
        "abn_current",
        "ITAA 1997 s 50-110(3): the entity must have an ABN",
        NOT_ENTITLED,
        ABN_GONE,
    ),
    (
        "registered_charity",
        "ITAA 1997 s 50-5 item 1.1 and s 50-47: a registered charity, registered with the ACNC",
        NOT_ENTITLED,
        NOTICE,
    ),
    (
        "meets_a_test",
        "ITAA 1997 s 50-50(1): the in Australia test, the DGR test or the prescribed by law test",
        NOT_ENTITLED,
        NOTICE,
    ),
    (
        "meets_both_conditions",
        "ITAA 1997 s 50-50(2): complies with the substantive requirements of its governing rules and applies its income and assets solely for its purpose",
        NOT_ENTITLED,
        NOTICE,
    ),
];

const DGR_WHOLE: [Question; 5] = [
    (
        "abn_current",
        "ITAA 1997 s 30-125(1)(a): the entity must have an ABN",
        NOT_ENTITLED,
        ABN_GONE,
    ),
    (
        "within_category",
        "ITAA 1997 s 30-125(1)(b): the fund, authority or institution falls within the DGR category on the ATO notice",
        NOT_ENTITLED,
        "Tell the ATO in writing and give the date it stopped falling within the category (TAA 1953 Sch 1 s 426-45(1)); if it falls within another DGR category, write to the ATO about that.",
    ),
    (
        "winding_up_clause",
        "ITAA 1997 s 30-125(1)(c), (6) and (7): a law, the constituent documents or the rules must require surplus gifts and deductible contributions, and money received from them, to go to another DGR on winding up or revocation, unless the entity is established by an Act that does not provide for its winding up",
        NOT_ENTITLED,
        "Section 30-125(1)(c) makes the clause a condition of entitlement: tell the ATO in writing and give the date entitlement stopped (TAA 1953 Sch 1 s 426-45(1)), and amend the governing documents to include a winding up and revocation clause, as the worksheet says (the ATO helps on 1300 130 248).",
    ),
    (
        "in_australia",
        "ITAA 1997 s 30-15 item 1, special condition (a): the fund, authority or institution must be in Australia",
        NOT_ENTITLED,
        "Not entitled for the period it was not in Australia: tell the ATO in writing so it can review the endorsement (TAA 1953 Sch 1 s 426-45(1)).",
    ),
    (
        "receipts_correct",
        "ITAA 1997 s 30-228: a receipt for a gift or deductible contribution states the name, the ABN and what it is a receipt for",
        ACTION_NEEDED,
        "Correct how receipts are issued straight away: the Commissioner may revoke the endorsement for a contravention (TAA 1953 Sch 1 s 426-55(1)(c)).",
    ),
];

const DGR_FUND: [Question; 5] = [
    (
        "abn_current",
        "ITAA 1997 s 30-125(2)(a): the entity must have an ABN",
        NOT_ENTITLED,
        ABN_GONE,
    ),
    (
        "within_category",
        "ITAA 1997 s 30-125(2) and (2)(c): the fund, authority or institution falls within the DGR category on the ATO notice",
        NOT_ENTITLED,
        "Tell the ATO in writing and give the date it stopped falling within the category (TAA 1953 Sch 1 s 426-45(1)); if it falls within another DGR category, write to the ATO about that.",
    ),
    (
        "gift_fund_maintained",
        "ITAA 1997 s 30-125(2)(e) and s 30-130: a gift fund for the principal purpose that receives only gifts, deductible contributions and money from them, and is used only for that purpose",
        NOT_ENTITLED,
        "Not entitled for the period the gift fund was not maintained: tell the ATO in writing so the endorsement can be revoked for that period (TAA 1953 Sch 1 s 426-45(1)).",
    ),
    (
        "in_australia",
        "ITAA 1997 s 30-15 item 1, special condition (a): the fund, authority or institution must be in Australia",
        NOT_ENTITLED,
        "Not entitled for the period it was not in Australia: tell the ATO in writing so it can review the endorsement (TAA 1953 Sch 1 s 426-45(1)).",
    ),
    (
        "receipts_correct",
        "ITAA 1997 s 30-228: a receipt states the name of the fund, authority or institution, the DGR's ABN and what it is a receipt for",
        ACTION_NEEDED,
        "Correct how receipts are issued straight away: the Commissioner may revoke the endorsement for a contravention (TAA 1953 Sch 1 s 426-55(1)(c)).",
    ),
];

/// Conditions of entitlement the worksheets do not ask about.
const NOT_ASKED_DGR: [&str; 3] = [
    "ITAA 1997 s 30-125(1)(b)(iii) and (2)(c): the special conditions of the s 30-15 table item and the Subdivision 30-B item",
    "ITAA 1997 s 30-125(1)(d) and (e): a giving or community charity trust fund, or a community charity corporation, and its trustees or directors, comply with the guidelines",
    "ITAA 1997 s 30-125(2)(d) and (6): for an entity endorsed for a fund, authority or institution, the winding up and revocation clause",
];
const NOT_ASKED_CHARITY: [&str; 1] = [
    "ITAA 1997 s 50-110(5)(b): an entity that has not yet carried on activities as a charity is tested on reasonable grounds for believing it will meet the conditions",
];

fn worksheet(value: &Arg) -> Result<(&'static str, &'static [Question], &'static [&'static str])> {
    match value.text() {
        Some("income_tax_exempt_charity") => {
            Ok(("NAT 74110-02.2021", &CHARITY, &NOT_ASKED_CHARITY))
        }
        Some("dgr_whole") => Ok((
            "NAT 11685-02.2021 (Worksheet 1)",
            &DGR_WHOLE,
            &NOT_ASKED_DGR,
        )),
        Some("dgr_fund") => Ok(("NAT 11686-02.2021 (Worksheet 2)", &DGR_FUND, &NOT_ASKED_DGR)),
        _ => refuse(
            "endorsement_review.worksheet_invalid",
            "worksheet must be income_tax_exempt_charity, dgr_whole or dgr_fund.",
        ),
    }
}

/// The worksheet's answers, each a boolean, and no other key.
fn answers(value: &Arg, questions: &[Question]) -> Result<Vec<bool>> {
    let keys: Vec<&str> = questions.iter().map(|q| q.0).collect();
    let bad = || {
        refuse(
            "endorsement_review.answers_invalid",
            format!(
                "answers must be an object with exactly these booleans for the worksheet: {}.",
                keys.join(", ")
            ),
        )
    };
    let Arg::Json(Value::Object(given)) = value else {
        return bad();
    };
    if given.len() != keys.len() {
        return bad();
    }
    keys.iter()
        .map(|k| given.get(*k).and_then(Value::as_bool))
        .collect::<Option<Vec<bool>>>()
        .map_or_else(bad, Ok)
}

fn finding(q: &Question, yes: bool) -> Value {
    let mut item = Map::new();
    item.insert("question".into(), json!(q.0));
    item.insert("provision".into(), json!(q.1));
    item.insert("answer".into(), json!(yes));
    if !yes {
        item.insert("outcome".into(), json!(q.2));
        item.insert("action".into(), json!(q.3));
    }
    Value::Object(item)
}

pub fn endorsement_review(
    worksheet_arg: &Arg,
    answers_arg: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let (form, questions, not_asked) = worksheet(worksheet_arg)?;
    let given = answers(answers_arg, questions)?;
    let failed: Vec<&Question> = questions
        .iter()
        .zip(&given)
        .filter(|(_, yes)| !**yes)
        .map(|(q, _)| q)
        .collect();
    let status = if failed.iter().any(|q| q.2 == NOT_ENTITLED) {
        NOT_ENTITLED
    } else if failed.is_empty() {
        "continues"
    } else {
        ACTION_NEEDED
    };
    let mut out = result(KIND, &year, Vec::new(), Vec::new())?;
    out["worksheet"] = json!(form);
    out["status"] = json!(status);
    out["findings"] = json!(
        questions
            .iter()
            .zip(&given)
            .map(|(q, yes)| finding(q, *yes))
            .collect::<Vec<_>>()
    );
    out["not_asked"] = json!(not_asked);
    out["record"] = json!(
        "Have the Board, Committee or Trustee approve the completed worksheet and keep it with the entity's records; do not send it to the ATO."
    );
    Ok(out)
}
