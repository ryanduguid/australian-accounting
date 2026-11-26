// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `disability_support_pension` and `carer_payment`: one fortnight's rate of
//! each under Pension Rate Calculator A, which s 1064(1) of the Social
//! Security Act 1991 (compilation 234 of 20 September 2026, read on 27
//! November 2026) applies to age pension, to disability support pension of a
//! person who has turned 21 or has a dependent child, and to carer payment.
//! The calculator, its income and assets tests and its dated amounts are
//! `age_pension`'s: the Social Security Guide's common pension rates
//! (5.1.8.10, version 1.345) are one table for all 3 payments, and Services
//! Australia's carer payment pages (read on 27 November 2026) give the same
//! amounts. These commands add no amount of their own; they state the facts
//! that differ and refuse what the Act treats differently: the under-21
//! rates of Pension Rate Calculator D (s 1066A), the blind (s 1065), a leave
//! or termination payment that Module F spreads for disability support
//! pension, and carer allowance and carer supplement, which are other
//! payments. Both also refuse the rate caps of s 1064(4) to (7), quarterly
//! supplements, and the deductions and remote area allowance of step 12.

use serde_json::Value;

use super::age_pension::{self, AgePensionArgs, FACTS, Payment};
use super::{Error, Result, refuse};

const QUARTERLY: (&str, &str) = (
    "no_quarterly_election",
    "no election under s 1061VA(1) is in force and quarterly energy supplement is not payable to the person, so the pension supplement and energy supplement are in the fortnightly rate in full (points 1064-BA2 and 1064-C2)",
);

const NO_CAP: (&str, &str) = (
    "no_armed_services_cap",
    "the person is not an armed services widow or widower receiving a Veterans' Entitlements Act pension at a rate set by reference to its s 30(1), or the MRCA weekly amount or lump sum, whose rate s 1064(5) to (7) caps",
);

const NO_UNDERAGE: (&str, &str) = (
    "no_underage_relationship",
    "the person is not in a de facto relationship in which either is under the age of consent where they live, whose rate s 1064(4) caps at the partnered rate",
);

const NO_ADJUSTMENTS: (&str, &str) = (
    "no_adjustments",
    "no special employment advance deduction (Part 3.16B), advance payment deduction (Part 3.16A) or remote area allowance (Module H) applies (step 12 of point 1064-A1)",
);

const PARTNER: [(&str, &str); 1] = [(
    "partner_pensioner",
    "the partner also receives a pension whose rate Pension Rate Calculator A works out (age pension, disability support pension at 21 or over or with a dependent child, or carer payment) and the couple are not illness separated, in respite care or apart because the partner is imprisoned",
)];

const DSP_FACTS: [(&str, &str); 10] = [
    (
        "aged_21_or_with_child",
        "the person has turned 21, or has not turned 21 and has a dependent child, so s 1064(1)(b) applies Pension Rate Calculator A; the under-21 rates of Pension Rate Calculator D (s 1066A) are not worked here",
    ),
    (
        "not_blind",
        "the person is not permanently blind (s 1064(2); the blind rate is s 1065, not worked here)",
    ),
    (
        "no_module_f_payment",
        "neither the person nor the partner has a leave payment, termination payment or lump sum from terminated employment that Module F (points 1064-F1 to F14) treats as ordinary income over an income maintenance period",
    ),
    FACTS[0],
    FACTS[2],
    FACTS[3],
    QUARTERLY,
    NO_CAP,
    NO_UNDERAGE,
    NO_ADJUSTMENTS,
];

const CARER_FACTS: [(&str, &str); 8] = [
    (
        "carer_payment_only",
        "the amount asked for is carer payment (Part 2.5), whose rate s 1064(1)(d) works out under Pension Rate Calculator A; carer allowance (Part 2.19) and carer supplement are other payments, not worked here",
    ),
    FACTS[0],
    FACTS[2],
    FACTS[3],
    QUARTERLY,
    NO_CAP,
    NO_UNDERAGE,
    NO_ADJUSTMENTS,
];

const DSP: Payment = Payment {
    kind: "disability_support_pension",
    facts: &DSP_FACTS,
    couple_facts: &PARTNER,
};

const CARER: Payment = Payment {
    kind: "carer_payment",
    facts: &CARER_FACTS,
    couple_facts: &PARTNER,
};

pub fn disability_support_pension(args: &AgePensionArgs) -> Result<Value> {
    age_pension::pension(&DSP, args).or_else(dsp_refusal)
}

pub fn carer_payment(args: &AgePensionArgs) -> Result<Value> {
    age_pension::pension(&CARER, args).or_else(carer_refusal)
}

/// The calculator's refusal under this command's code, its message kept.
fn dsp_refusal(e: Error) -> Result<Value> {
    let message = e.to_string();
    match e.code() {
        "age_pension.assets_inconsistent" => {
            refuse("disability_support_pension.assets_inconsistent", message)
        }
        "age_pension.date_invalid" => refuse("disability_support_pension.date_invalid", message),
        "age_pension.date_uncovered" => {
            refuse("disability_support_pension.date_uncovered", message)
        }
        "age_pension.facts_invalid" => refuse("disability_support_pension.facts_invalid", message),
        "age_pension.facts_not_stated" => {
            refuse("disability_support_pension.facts_not_stated", message)
        }
        "age_pension.relationship_invalid" => {
            refuse("disability_support_pension.relationship_invalid", message)
        }
        _ => Err(e),
    }
}

fn carer_refusal(e: Error) -> Result<Value> {
    let message = e.to_string();
    match e.code() {
        "age_pension.assets_inconsistent" => refuse("carer_payment.assets_inconsistent", message),
        "age_pension.date_invalid" => refuse("carer_payment.date_invalid", message),
        "age_pension.date_uncovered" => refuse("carer_payment.date_uncovered", message),
        "age_pension.facts_invalid" => refuse("carer_payment.facts_invalid", message),
        "age_pension.facts_not_stated" => refuse("carer_payment.facts_not_stated", message),
        "age_pension.relationship_invalid" => refuse("carer_payment.relationship_invalid", message),
        _ => Err(e),
    }
}
