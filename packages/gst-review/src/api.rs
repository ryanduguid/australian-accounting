//! JSON mapping of the two review entrypoints. Amounts are decimal strings;
//! dates are canonical YYYY-MM-DD strings. No files or services are accessed.
use crate::{Result, bas, coding, object};
use serde_json::Value;

pub const COMMANDS: [&str; 2] = ["review-coding", "reconcile-bas"];
pub fn evaluate(command: &str, input: &Value) -> Result<Value> {
    object(input)?;
    match command {
        "review-coding" => coding::review(input),
        "reconcile-bas" => bas::reconcile(input),
        _ => Err(format!("unknown command: {command}")),
    }
}
