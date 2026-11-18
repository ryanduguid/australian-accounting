// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! ABN and ACN check digits. Arithmetic only: no lookup, and nothing about registration.
//!
//! These live in the facade, as in the Python server, because no linked engine
//! holds these rules. They are the published format tests of 2 registers, with no
//! rate, date or dataset to go stale.
use serde_json::{Value, json};

const ABN_WEIGHTS: [u32; 11] = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
const ACN_WEIGHTS: [u32; 8] = [8, 7, 6, 5, 4, 3, 2, 1];

const ABN_SOURCE: &str = "https://abr.business.gov.au/Help/AbnFormat";
const ABN_PROVISION: &str = "ABN Lookup, Format of the ABN: 'To verify an ABN' (version 9.9.7)";
const ACN_SOURCE: &str =
    "https://download.asic.gov.au/media/vy0ensam/els-document-messages-specification-v5-10.pdf";
const ACN_PROVISION: &str = "ASIC EDGE Document Messages Specification version 5.10 (1 December 2024), Appendix D, Australian Company Number (ACN) Check Digit";
const SOURCE_CHECKED: &str = "2026-10-31";

const ABN_NOT_ESTABLISHED: &str = "A valid check digit shows only that the digits pass the arithmetic ABN Lookup publishes. It proves neither that the number was issued to anyone or is active, nor that an entity is registered in the Australian Business Register or registered for GST. ABN Lookup (https://abr.business.gov.au) shows an ABN's status and if and when an entity is registered for GST.";
const ACN_NOT_ESTABLISHED: &str = "A valid check digit shows only that the digits pass the arithmetic ASIC publishes. It proves neither that ASIC issued the number or that a company holding it exists, nor that an entity is registered in the Australian Business Register or registered for GST. Search ASIC's registers for the company and ABN Lookup (https://abr.business.gov.au) for ABN and GST status.";

/// The digits, or None. Only ASCII spaces are removed: stripping anything else
/// could turn a malformed identifier into a different, valid one.
fn digits(value: &str, length: usize) -> Option<Vec<u32>> {
    if !value.chars().all(|c| c == ' ' || c.is_ascii_digit()) {
        return None;
    }
    let digits: Vec<u32> = value.chars().filter_map(|c| c.to_digit(10)).collect();
    (digits.len() == length).then_some(digits)
}

fn abn_passes(digits: &[u32]) -> bool {
    // ABR's first step subtracts 1 from the first digit, which a leading zero
    // cannot do, though 00000090000 would still divide by 89.
    if digits[0] == 0 {
        return false;
    }
    let total: u32 = digits
        .iter()
        .zip(ABN_WEIGHTS)
        .enumerate()
        .map(|(i, (d, w))| if i == 0 { d - 1 } else { *d } * w)
        .sum();
    total.is_multiple_of(89)
}

fn acn_passes(digits: &[u32]) -> bool {
    let total: u32 = digits.iter().zip(ACN_WEIGHTS).map(|(d, w)| d * w).sum();
    (10 - total % 10) % 10 == digits[8]
}

struct Register {
    identifier: &'static str,
    groups: &'static [usize],
    not_established: &'static str,
    source: &'static str,
    provision: &'static str,
}

fn result(register: &Register, digits: Option<Vec<u32>>, passes: bool) -> Value {
    let length: usize = register.groups.iter().sum();
    let formatted = digits.as_ref().map(|digits| {
        let mut rest = digits.as_slice();
        register
            .groups
            .iter()
            .map(|&size| {
                let (part, tail) = rest.split_at(size);
                rest = tail;
                part.iter().map(u32::to_string).collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(" ")
    });
    let identifier = register.identifier;
    let (outcome, reason) = match (&digits, passes) {
        (None, _) => (
            "MALFORMED",
            format!("An {identifier} is {length} ASCII digits, optionally separated by spaces."),
        ),
        (Some(_), true) => (
            "CHECK_DIGIT_VALID",
            format!("The digits pass the published {identifier} check digit test."),
        ),
        (Some(_), false) => (
            "CHECK_DIGIT_INVALID",
            format!(
                "The digits fail the published {identifier} check digit test, so this is not a valid {identifier} as written. Check it against its source."
            ),
        ),
    };
    json!({
        "ok": true,
        "identifier": identifier,
        "result": outcome,
        "check_digit_valid": passes,
        "formatted": formatted,
        "reason": reason,
        "abr_registration_established": false,
        "gst_registration_established": false,
        "not_established": register.not_established,
        "source_url": register.source,
        "source_provision": register.provision,
        "source_checked": SOURCE_CHECKED,
    })
}

pub fn check_abn(value: &str) -> Value {
    let digits = digits(value, 11);
    let passes = digits.as_deref().is_some_and(abn_passes);
    let register = Register {
        identifier: "ABN",
        groups: &[2, 3, 3, 3],
        not_established: ABN_NOT_ESTABLISHED,
        source: ABN_SOURCE,
        provision: ABN_PROVISION,
    };
    result(&register, digits, passes)
}

pub fn check_acn(value: &str) -> Value {
    let digits = digits(value, 9);
    let passes = digits.as_deref().is_some_and(acn_passes);
    let register = Register {
        identifier: "ACN",
        groups: &[3, 3, 3],
        not_established: ACN_NOT_ESTABLISHED,
        source: ACN_SOURCE,
        provision: ACN_PROVISION,
    };
    result(&register, digits, passes)
}
