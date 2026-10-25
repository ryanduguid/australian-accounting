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
fn help() -> String {
    "gst-review: GST coding and BAS reconciliation review aids\n\nUsage: gst-review <review-coding|reconcile-bas> < input.json\nAmounts are decimal strings; dates are YYYY-MM-DD. See README.md for schemas.\nReview aid only, not advice, an assessment or a lodgment. Obtain human review.\n".into()
}
