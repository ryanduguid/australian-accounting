// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Small native interface, shared with the umbrella command.
use std::ffi::OsStr;
pub type Outcome = (i32, String, String);
pub fn decode_os(value: &OsStr) -> String {
    value.to_string_lossy().into_owned()
}
pub fn output_bytes(value: &str) -> Vec<u8> {
    value.as_bytes().to_vec()
}
pub fn run(args: &[String]) -> Outcome {
    match args {
        [] => (0, help(), String::new()),
        [flag] if ["--help", "-h"].contains(&flag.as_str()) => (0, help(), String::new()),
        [flag] if flag == "--api-help" => (
            0,
            "JSON stdin command: due-dates\nSee README.md for the schema.\n".into(),
            String::new(),
        ),
        [flag] if flag == "--version" => (
            0,
            format!("lodgment-calendar {}\n", crate::VERSION),
            String::new(),
        ),
        _ => (
            2,
            String::new(),
            "error: expected due-dates with one JSON object on stdin; see --help\n".into(),
        ),
    }
}
const LICENCE_NOTICE: &str = "\n\
    Copyright 2026 Ryan Duguid. No warranty.\n\
    AGPL-3.0-or-later with additional terms (see NOTICE).\n\
    Licence: https://github.com/ryanduguid/australian-accounting/blob/main/packages/lodgment-calendar/LICENSE\n";

fn help() -> String {
    format!(
        "lodgment-calendar: cited Australian lodgment program dates\n\nUsage: lodgment-calendar due-dates < input.json\nDates use YYYY-MM-DD. Unknown facts and incomplete holiday coverage remain UNKNOWN.\nPreparation aid only. Confirm each client's dates with the ATO and obtain human review.\n{LICENCE_NOTICE}"
    )
}
