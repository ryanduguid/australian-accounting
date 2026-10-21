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
fn help() -> String {
    "lodgment-calendar: cited Australian lodgment program dates\n\nUsage: lodgment-calendar due-dates < input.json\nDates use YYYY-MM-DD. Unknown facts and incomplete holiday coverage remain UNKNOWN.\nPreparation aid only. Confirm each client's dates with the ATO and obtain human review.\n".into()
}
