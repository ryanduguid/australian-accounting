// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Small native command interface for the JSON review API.
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
        [] => (0,help(),String::new()),
        [flag] if ["--help","-h"].contains(&flag.as_str()) => (0,help(),String::new()),
        [flag] if flag == "--api-help" => (0,format!("JSON stdin commands: {}\nSee README.md for their schemas.\n",crate::api::COMMANDS.join(", ")),String::new()),
        [flag] if flag == "--version" => (0,format!("gst-review {}\n",crate::VERSION),String::new()),
        _ => (2,String::new(),"error: expected review-coding or reconcile-bas with one JSON object on stdin; see --help\n".into()),
    }
}
const LICENCE_NOTICE: &str = "\n\
    Copyright 2026 Ryan Duguid. No warranty.\n\
    AGPL-3.0-or-later with additional terms (see NOTICE).\n\
    Licence: https://github.com/ryanduguid/australian-accounting/blob/main/packages/gst-review/LICENSE\n";

fn help() -> String {
    format!(
        "gst-review: GST coding and BAS reconciliation review aids\n\nUsage: gst-review <review-coding|reconcile-bas> < input.json\nAmounts are decimal strings; dates are YYYY-MM-DD. See README.md for schemas.\nReview aid only, not advice, an assessment or a lodgment. Obtain human review.\n{LICENCE_NOTICE}"
    )
}
