// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `re.compile` and `re.search` against CPython 3.15.0: error texts and
//! positions, escapes, group names and references, conditional groups,
//! inline flags, ASCII and case-insensitive matching, sets, anchors and
//! greedy, lazy and possessive repeats. Each case is `(pattern, flags,
//! subject)` with CPython's answer: `str(re.error)` or `Class: message` when
//! compiling raises; then `groups N` with no subject, or `re.search`'s span
//! of every group (`-` where a group took no part) or `no match`.
//! `tests/data/cov_a4_sre.py` printed the table.

use super::*;

fn text(points: &[u32]) -> String {
    points
        .iter()
        .map(|&c| char::from_u32(c).unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

/// The answer as the oracle printed it.
fn answer(pattern: &str, flags: u32, subject: Option<&str>) -> String {
    let compiled = match compile(&pts(pattern), flags) {
        Ok(compiled) => compiled,
        Err(CompileError::Pattern(message)) => return text(&message),
        Err(CompileError::Other(class, message)) => return format!("{class}: {message}"),
    };
    let Some(subject) = subject else {
        return format!("groups {}", compiled.groups);
    };
    let Some(found) = compiled.search(&pts(subject)) else {
        return String::from("no match");
    };
    let span = |i| match found.span(i) {
        Some((a, b)) => format!("{a}-{b}"),
        None => String::from("-"),
    };
    let spans: Vec<String> = (0..=compiled.groups).map(span).collect();
    spans.join(" ")
}

#[test]
fn compile_and_search_answer_as_cpython_does() {
    let (got, want): (Vec<String>, Vec<String>) = CASES
        .iter()
        .map(|&(pattern, flags, subject, want)| {
            let case = format!("{pattern:?} {flags} {subject:?}");
            (
                format!("{case} {}", answer(pattern, flags, subject)),
                format!("{case} {want}"),
            )
        })
        .unzip();
    assert_eq!(got, want);
}

/// The patterns too long for the table.
#[test]
fn long_patterns_answer_as_cpython_does() {
    let digits = format!("a{{{}}}", "1".repeat(5000));
    let limit = "ValueError: Exceeds the limit (4300 digits) for integer string conversion: \
                 value has 5000 digits; use sys.set_int_max_str_digits() to increase the limit";
    assert_eq!(answer(&digits, 0, None), limit);
    let deep = "RecursionError: maximum recursion depth exceeded";
    let groups = format!("{}{}", "(".repeat(500), ")".repeat(500));
    assert_eq!(answer(&groups, 0, None), deep);
    let plain = format!("{}{}", "(?:".repeat(500), ")".repeat(500));
    assert_eq!(answer(&plain, 0, None), deep);
    let deepest = format!("[a]{}a{}", "(?:".repeat(479), ")".repeat(479));
    assert_eq!(answer(&deepest, 0, Some("aa")), "0-2");
}

const CASES: &[(&str, u32, Option<&str>, &str)] = &[
    (
        "a(\nb",
        0,
        None,
        "missing ), unterminated subpattern at position 1 (line 1, column 2)",
    ),
    (
        "(\nb",
        0,
        None,
        "missing ), unterminated subpattern at position 0 (line 1, column 1)",
    ),
    ("[^\\D]", 0, Some("a1"), "1-2"),
    ("[^\\s]", 0, Some(" a"), "1-2"),
    ("[^\\S]", 0, Some("a b"), "1-2"),
    ("[^\\w]", 0, Some("a-"), "1-2"),
    ("[^\\W]", 0, Some("-a"), "1-2"),
    (".a|.b", 0, Some("xb"), "0-2"),
    ("[xy]a|[xy]b", 0, Some("yb"), "0-2"),
    ("\\ba|\\bb", 0, Some(" b"), "1-2"),
    ("(a)(?:\\1b|\\1c)", 0, Some("aac"), "0-3 0-1"),
    ("(?!)a|(?!)b", 0, Some("b"), "no match"),
    ("(?P<", 0, None, "missing group name at position 4"),
    (
        "(?P<ab",
        0,
        None,
        "missing >, unterminated name at position 4",
    ),
    ("(?P=", 0, None, "missing group name at position 4"),
    (
        "(?P=ab",
        0,
        None,
        "missing ), unterminated name at position 4",
    ),
    (
        "(?P<a-b>x)",
        0,
        None,
        "bad character in group name 'a-b' at position 4",
    ),
    (
        "(?P<x>(?P=x))",
        0,
        None,
        "cannot refer to an open group at position 10",
    ),
    (
        "(?<=(?P<x>a)(?P=x))b",
        0,
        None,
        "cannot refer to group defined in the same lookbehind subpattern at position 18",
    ),
    ("(?Px)", 0, None, "unknown extension ?Px at position 1"),
    (
        "(a\\1)",
        0,
        None,
        "cannot refer to an open group at position 2",
    ),
    ("(a)?(?<=(?(1)b|c))d", 0, Some("cd"), "1-2 -"),
    (
        "(a)?(?<=(?(1)b))d",
        0,
        Some("bd"),
        "look-behind requires fixed-width pattern",
    ),
    ("(?P<n>a)?(?(n)b|c)", 0, Some("ab"), "0-2 0-1"),
    ("(?P<n>a)?(?(n)b|c)", 0, Some("c"), "0-1 -"),
    (
        "(?(5)a)",
        0,
        None,
        "invalid group reference 5 at position 3",
    ),
    (
        "(?(01)a)",
        0,
        None,
        "invalid group reference 1 at position 3",
    ),
    (
        "(a)(?(1)b",
        0,
        None,
        "missing ), unterminated subpattern at position 3",
    ),
    (
        "\\a\\f\\n\\r\\t\\v\\\\",
        0,
        Some("\u{7}\u{c}\n\r\t\u{b}\\"),
        "0-7",
    ),
    (
        "[\\a\\b\\f\\n\\r\\t\\v\\\\]+",
        0,
        Some("\u{8}\u{7}\\"),
        "0-3",
    ),
    ("\\D+", 0, Some("12ab"), "2-4"),
    ("\\U0001F600", 0, Some("x\u{1f600}"), "1-2"),
    (
        "\\U0011FFFF",
        0,
        None,
        "bad escape \\U0011FFFF at position 0",
    ),
    (
        "\\U1234",
        0,
        None,
        "incomplete escape \\U1234 at position 0",
    ),
    ("[\\x41]", 0, Some("A"), "0-1"),
    ("[\\101]", 0, Some("A"), "0-1"),
    ("[\\0]", 0, Some("\0"), "0-1"),
    ("[\\8]", 0, None, "bad escape \\8 at position 1"),
    ("[\\q]", 0, None, "bad escape \\q at position 1"),
    ("[\\U0001F600]", 0, Some("\u{1f600}"), "0-1"),
    ("[\\u0041]", 0, Some("A"), "0-1"),
    ("[\\N{LATIN SMALL LETTER A}]", 0, Some("a"), "0-1"),
    (
        "[\\N{NO SUCH NAME}]",
        0,
        None,
        "undefined character name 'NO SUCH NAME' at position 1",
    ),
    (
        "\\N{NO SUCH NAME}",
        0,
        None,
        "undefined character name 'NO SUCH NAME' at position 0",
    ),
    ("\\q", 0, None, "bad escape \\q at position 0"),
    ("\\0", 0, Some("\0"), "0-1"),
    ("\\01", 0, Some("\u{1}"), "0-1"),
    ("\\x41", 0, Some("A"), "0-1"),
    (
        "(a)\\10",
        0,
        None,
        "invalid group reference 10 at position 4",
    ),
    ("\\u0041", 0, Some("A"), "0-1"),
    ("a|b|[cd]", 0, Some("d"), "0-1"),
    ("[^a]|b", 0, Some("a"), "no match"),
    ("x|[^a]", 0, Some("a"), "no match"),
    ("[a-]", 0, Some("-"), "0-1"),
    ("[\\w-]", 0, Some("-"), "0-1"),
    ("[a-\\x7a]", 0, Some("q"), "0-1"),
    ("a]", 0, Some("a]"), "0-2"),
    ("(?x)a\\ b", 0, Some("a b"), "0-3"),
    ("a{", 0, Some("a{"), "0-2"),
    ("a{,", 0, Some("a{,"), "0-3"),
    ("a{x}", 0, Some("a{x}"), "0-4"),
    (
        "a{1,99999999999}",
        0,
        None,
        "OverflowError: the repetition number is too large",
    ),
    ("^*", 0, None, "nothing to repeat at position 1"),
    ("(?:)*", 0, Some("a"), "0-0"),
    ("(?!)", 0, Some("a"), "no match"),
    ("a(?!)", 0, Some("a"), "no match"),
    ("(?i", 0, None, "missing -, : or ) at position 3"),
    ("(?z)", 0, None, "unknown extension ?z at position 1"),
    ("(?i!", 0, None, "missing -, : or ) at position 3"),
    ("(?-)", 0, None, "missing flag at position 3"),
    ("(?-:a)", 0, None, "missing flag at position 3"),
    (
        "(?-a:x)",
        0,
        None,
        "bad inline flags: cannot turn off flags 'a', 'u' and 'L' at position 4",
    ),
    (
        "(?-u:x)",
        0,
        None,
        "bad inline flags: cannot turn off flags 'a', 'u' and 'L' at position 4",
    ),
    ("(?-i", 0, None, "missing : at position 4"),
    ("(?-i!)", 0, None, "missing : at position 4"),
    ("(?-iz:a)", 0, None, "unknown flag at position 4"),
    ("(?i-:a)", 0, None, "missing flag at position 4"),
    ("(?a-i:x)", 0, None, "groups 0"),
    ("(?-x:a)", 0, Some("a"), "0-1"),
    ("(?i-x)", 0, None, "missing : at position 5"),
    ("(?x-i)", 0, None, "missing : at position 5"),
    ("a(?a:b)", 0, None, "groups 0"),
    ("\\d+", re.ASCII, Some("\u{661}\u{662}1"), "2-3"),
    ("\\s+", re.ASCII, Some("\u{2003} \t"), "1-3"),
    ("\\D+", re.ASCII, Some("\u{661}a1"), "0-2"),
    ("\\S+", re.ASCII, Some("\u{2003}a "), "0-2"),
    ("[^\\d]+", re.ASCII, Some("a1"), "0-1"),
    ("k", re.ASCII | re.IGNORECASE, Some("K\u{212a}"), "0-1"),
    (
        "\u{212a}",
        re.ASCII | re.IGNORECASE,
        Some("k\u{212a}"),
        "1-2",
    ),
    ("[^k]+", re.ASCII | re.IGNORECASE, Some("Kx"), "1-2"),
    ("[k-m]+", re.ASCII | re.IGNORECASE, Some("KLM"), "0-3"),
    ("\\w+", re.ASCII | re.IGNORECASE, Some("\u{e9}a"), "1-2"),
    ("K", re.IGNORECASE, Some("\u{212a}"), "0-1"),
    ("(?u)K", re.IGNORECASE, Some("\u{212a}"), "0-1"),
    (
        "a",
        re.ASCII | re.UNICODE,
        None,
        "ValueError: ASCII and UNICODE flags are incompatible",
    ),
    ("[k]", re.IGNORECASE, Some("\u{212a}"), "0-1"),
    ("[a-c]x", re.IGNORECASE, Some("BX"), "0-2"),
    ("[\u{17f}-\u{17f}]", re.IGNORECASE, Some("S"), "0-1"),
    ("[k-k]", re.IGNORECASE, Some("\u{212a}"), "0-1"),
    ("[\u{130}]", re.IGNORECASE, Some("i"), "0-1"),
    ("[\u{130}-\u{131}]", re.IGNORECASE, Some("I"), "0-1"),
    ("[^\u{17f}]", re.IGNORECASE, Some("s"), "no match"),
    ("[a\u{212a}-\u{212b}]", re.IGNORECASE, Some("\u{e5}"), "0-1"),
    ("[0-9k]", re.IGNORECASE, Some("K"), "0-1"),
    (
        "(?<=(?:a{100000}){100000})b",
        0,
        None,
        "looks too much behind",
    ),
    ("a$", re.MULTILINE, Some("a\nb"), "0-1"),
    ("a$", re.MULTILINE, Some("ab"), "no match"),
    ("^b", re.MULTILINE, Some("a\nb"), "2-3"),
    ("\\Aa\\Z", 0, Some("a"), "0-1"),
    ("a{2,3}+a", 0, Some("aaaa"), "0-4"),
    ("(?:ab)*+", 0, Some("ababa"), "0-4"),
    ("a{2}+", 0, Some("aaa"), "0-2"),
    ("a{3}+", 0, Some("aa"), "no match"),
    ("(a|ab){1,2}+c", 0, Some("abc"), "no match"),
    ("(?:a|b)*+b", 0, Some("ab"), "no match"),
    ("(a)*+", 0, Some("aa"), "0-2 1-2"),
    ("(?:)*+x", 0, Some("x"), "0-1"),
    ("a{2,3}?", 0, Some("aaa"), "0-2"),
    ("(?:a|b){2,}?c", 0, Some("abac"), "0-4"),
    ("(?:)*?x", 0, Some("x"), "0-1"),
    ("(a?){2,3}?b", 0, Some("ab"), "0-2 1-1"),
    ("(?:a{0,2}?){3}b", 0, Some("aab"), "0-3"),
    ("a{1,2}?b", 0, Some("ab"), "0-2"),
    ("a{2,}?", 0, Some("a"), "no match"),
    ("(?:a*)*?b", 0, Some("aab"), "0-3"),
];
