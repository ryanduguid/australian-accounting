//! Guard values written to CSV against spreadsheet formula injection. A
//! leading `=` or `@` is always escaped; a leading `+` or `-` only when what
//! follows is not a plain number, so a ledger code like -00123 still joins.

use std::sync::LazyLock;

use crate::pyre::Regex;
use crate::pytext;

static NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[\d,]*+(?:\.\d++|(?<=\d))$"));

/// `str.lstrip()`.
fn lstrip(text: &str) -> &str {
    text.trim_start_matches(|c: char| pytext::is_space(c as u32))
}

pub fn guard(value: Option<&str>) -> String {
    let text = value.unwrap_or("");
    if text.is_empty() {
        return String::new();
    }
    // Spreadsheets ignore leading whitespace when recognising a formula.
    let stripped = lstrip(text);
    let mut chars = stripped.chars();
    let first = chars.next();
    if matches!(first, Some('=' | '@')) {
        return format!("'{text}");
    }
    if matches!(first, Some('+' | '-')) && !NUMBER.matches_start(chars.as_str()) {
        return format!("'{text}");
    }
    // A leading tab, carriage return or newline can push the rest onto a line
    // a spreadsheet reads as a formula.
    if text.starts_with(['\t', '\r', '\n']) {
        return format!("'{text}");
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::guard;

    #[test]
    fn guards_match_python() {
        for (raw, guarded) in [
            ("=1+1", "'=1+1"),
            ("@SUM", "'@SUM"),
            ("-00123", "-00123"),
            ("+A1", "'+A1"),
            ("-1,000.5", "-1,000.5"),
            ("-.5", "-.5"),
            ("-", "'-"),
            ("  =x", "'  =x"),
            ("\tTAB", "'\tTAB"),
            ("plain", "plain"),
            ("-5\n", "-5\n"),
            ("-\u{663}", "-\u{663}"),
        ] {
            assert_eq!(guard(Some(raw)), guarded, "{raw:?}");
        }
    }
}
