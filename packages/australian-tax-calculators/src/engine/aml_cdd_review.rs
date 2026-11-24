// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `aml_cdd_review`: deterministic Australian AML/CTF Tranche 2 customer due
//! diligence (CDD) helper for 2026-27 with zero personal data. Covers 3
//! questions: `country_risk` (AUSTRAC Basel AML Index 0-10 bands with FATF and
//! Australian country-sanctions overrides and conditional s 32(d) notice),
//! `beneficial_ownership` (AML/CTF Act 2006 ss 5, 11 and 28(2)(d) exact 25%
//! indirect chain ownership and control tests with Rules ss 6-7(1)-(1C) and
//! 6-8), and `cdd_review` (business-relationship ongoing CDD under ss 30 to 32
//! and Rules ss 5-2, 6-16 and 6-20 to 6-24, cleanly separated from AUSTRAC
//! Accountant Program Starter Kit 1/2/3-year defaults and custom firm policies).
//!
//! Read on 24 November 2026: Anti-Money Laundering and Counter-Terrorism
//! Financing Act 2006 (C2006A00169, compilation 63, 1 October 2026);
//! Anti-Money Laundering and Counter-Terrorism Financing Rules 2025
//! (F2025L01026, compilation 1, 31 March 2026); AUSTRAC Step 2: Identify and
//! assess your risks; AUSTRAC Accountant program starter kit: Step 2; AUSTRAC
//! Determining ownership and control structures; AUSTRAC Reviewing and
//! updating customers' ML/TF risk and KYC information (29 October 2026).

use std::collections::BTreeSet;

use serde_json::Value;

use super::{Arg, Result, flag, refuse, scope};

mod beneficial_ownership;
mod cdd_review;
mod country_risk;

pub(super) const KIND: &str = "aml_cdd_review";
const QUESTIONS: [&str; 3] = ["country_risk", "beneficial_ownership", "cdd_review"];

pub(super) struct Given<'a> {
    question: &'a str,
    named: &'a [(String, Arg)],
}

impl<'a> Given<'a> {
    fn check_unique(&self) -> Result<()> {
        let mut seen = BTreeSet::new();
        for (k, _) in self.named {
            if !seen.insert(k.as_str()) {
                return refuse(
                    "aml_cdd_review.duplicate_field",
                    format!(
                        "{k} was supplied more than once for question {}.",
                        self.question
                    ),
                );
            }
        }
        Ok(())
    }

    pub(super) fn get(&self, key: &str) -> Option<&'a Arg> {
        self.named.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub(super) fn need(&self, key: &str) -> Result<&'a Arg> {
        match self.get(key) {
            Some(arg) => Ok(arg),
            None => refuse(
                "input.missing_field",
                format!("{key} is required for question {}.", self.question),
            ),
        }
    }

    pub(super) fn need_flag(&self, key: &str) -> Result<bool> {
        flag(self.need(key)?, key)
    }

    pub(super) fn only(&self, takes: &[&str]) -> Result<()> {
        match self
            .named
            .iter()
            .find(|(k, _)| !takes.contains(&k.as_str()))
        {
            Some((key, _)) => refuse(
                "input.field_unknown",
                format!("{key} is not an input of question {}.", self.question),
            ),
            None => Ok(()),
        }
    }
}

pub(super) fn choice(arg: &Arg, allowed: &[&'static str]) -> Option<&'static str> {
    arg.text()
        .and_then(|t| allowed.iter().copied().find(|a| *a == t))
}

pub fn aml_cdd_review(
    question: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    named: &[(String, Arg)],
) -> Result<Value> {
    let period = scope(scope_confirmed, year, KIND)?;
    let Some(question) = choice(question, &QUESTIONS) else {
        return refuse(
            "aml_cdd_review.question_invalid",
            format!("question must be one of {}.", QUESTIONS.join(", ")),
        );
    };
    let given = Given { question, named };
    given.check_unique()?;
    match question {
        "country_risk" => country_risk::country_risk(&given, &period),
        "beneficial_ownership" => beneficial_ownership::beneficial_ownership(&given, &period),
        _ => cdd_review::cdd_review(&given, &period),
    }
}
