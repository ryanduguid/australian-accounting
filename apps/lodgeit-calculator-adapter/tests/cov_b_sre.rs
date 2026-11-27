// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The `re` port against CPython 3.15: each row's expected value is what
//! `re.compile(pattern, flags)` and then `search`, `match` or `fullmatch`
//! gave in CPython 3.15 on the same input (`str(re.error)` for a refused
//! pattern, the exception class and message for any other refusal).
use lodgeit_calculator_adapter::sre::{self, CompileError};

#[derive(Debug, PartialEq)]
enum Want {
    Err(&'static str),
    Other(&'static str, &'static str),
    Groups(usize),
    NoMatch,
    Spans(&'static [Option<(usize, usize)>]),
}
use Want::*;

fn pts(s: &str) -> Vec<u32> {
    s.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&c| char::from_u32(c)).collect()
}

fn run(pattern: &str, flags: u32, mode: char, input: &str) -> Want {
    let p = match sre::compile(&pts(pattern), flags) {
        Ok(p) => p,
        Result::Err(e) => {
            let shown = text(&e.text());
            return match e {
                CompileError::Pattern(_) => Err(Box::leak(shown.into_boxed_str())),
                CompileError::Other(class, message) => {
                    assert_eq!(shown, message, "text() of {class} is its message");
                    Other(class, Box::leak(message.into_boxed_str()))
                }
            };
        }
    };
    let found = match mode {
        'c' => return Groups(p.groups),
        's' => p.search(&pts(input)),
        'm' => p.match_start(&pts(input)),
        'f' => p.fullmatch(&pts(input)),
        _ => unreachable!("mode {mode}"),
    };
    match found {
        None => NoMatch,
        Some(m) => {
            assert_eq!(m.span(0), Some((m.start(), m.end())));
            let spans: Vec<_> = (0..=p.groups).map(|i| m.span(i)).collect();
            Spans(Box::leak(spans.into_boxed_slice()))
        }
    }
}

#[test]
fn patterns_compile_and_match_as_cpython_does() {
    let rows: &[(&str, u32, char, &str, Want)] = &[
        // BEGIN CPYTHON ROWS
        (
            "(\u{a}",
            0,
            'c',
            "",
            Err("missing ), unterminated subpattern at position 0 (line 1, column 1)"),
        ),
        ("(?P<", 0, 'c', "", Err("missing group name at position 4")),
        (
            "(?P<a",
            0,
            'c',
            "",
            Err("missing >, unterminated name at position 4"),
        ),
        (
            "(?P<>a)",
            0,
            'c',
            "",
            Err("missing group name at position 4"),
        ),
        ("(?P=", 0, 'c', "", Err("missing group name at position 4")),
        ("(?P=)", 0, 'c', "", Err("missing group name at position 4")),
        (
            "(?P",
            0,
            'c',
            "",
            Err("unexpected end of pattern at position 3"),
        ),
        (
            "(?Px)",
            0,
            'c',
            "",
            Err("unknown extension ?Px at position 1"),
        ),
        (
            "(?",
            0,
            'c',
            "",
            Err("unexpected end of pattern at position 2"),
        ),
        (
            "(?<",
            0,
            'c',
            "",
            Err("unexpected end of pattern at position 3"),
        ),
        (
            "(?=a",
            0,
            'c',
            "",
            Err("missing ), unterminated subpattern at position 0"),
        ),
        (
            "(?<=a",
            0,
            'c',
            "",
            Err("missing ), unterminated subpattern at position 0"),
        ),
        (
            "(a)(?(1)b",
            0,
            'c',
            "",
            Err("missing ), unterminated subpattern at position 3"),
        ),
        (
            "(?(4294967296)a)",
            0,
            'c',
            "",
            Err("invalid group reference 4294967296 at position 3"),
        ),
        (
            "(?(00000000004294967296)a)",
            0,
            'c',
            "",
            Err("invalid group reference 4294967296 at position 3"),
        ),
        (
            "(?()a)",
            0,
            'c',
            "",
            Err("missing group name at position 3"),
        ),
        (
            "(?(1a)b)",
            0,
            'c',
            "",
            Err("bad character in group name '1a' at position 3"),
        ),
        ("(?i", 0, 'c', "", Err("missing -, : or ) at position 3")),
        ("(?i!)", 0, 'c', "", Err("missing -, : or ) at position 3")),
        ("(?iz)", 0, 'c', "", Err("unknown flag at position 3")),
        (
            "(?i\\s)",
            0,
            'c',
            "",
            Err("missing -, : or ) at position 3"),
        ),
        ("(?-", 0, 'c', "", Err("missing flag at position 3")),
        ("(?-)", 0, 'c', "", Err("missing flag at position 3")),
        ("(?-z)", 0, 'c', "", Err("unknown flag at position 3")),
        (
            "(?-a:x)",
            0,
            'c',
            "",
            Err("bad inline flags: cannot turn off flags 'a', 'u' and 'L' at position 4"),
        ),
        (
            "(?-u:x)",
            0,
            'c',
            "",
            Err("bad inline flags: cannot turn off flags 'a', 'u' and 'L' at position 4"),
        ),
        ("(?-i", 0, 'c', "", Err("missing : at position 4")),
        ("(?-iz:x)", 0, 'c', "", Err("unknown flag at position 4")),
        ("(?-i!x)", 0, 'c', "", Err("missing : at position 4")),
        ("(?-i)x", 0, 'c', "", Err("missing : at position 4")),
        ("(?i-:x)", 0, 'c', "", Err("missing flag at position 4")),
        ("(?s-i:a)", 0, 'c', "", Groups(0)),
        ("^*", 0, 'c', "", Err("nothing to repeat at position 1")),
        ("$+", 0, 'c', "", Err("nothing to repeat at position 1")),
        ("\\b?", 0, 'c', "", Err("nothing to repeat at position 2")),
        (
            "[a-",
            0,
            'c',
            "",
            Err("unterminated character set at position 0"),
        ),
        (
            "[a-\\",
            0,
            'c',
            "",
            Err("bad escape (end of pattern) at position 3"),
        ),
        (
            "[\\",
            0,
            'c',
            "",
            Err("bad escape (end of pattern) at position 1"),
        ),
        ("[\\8]", 0, 'c', "", Err("bad escape \\8 at position 1")),
        ("[\\q]", 0, 'c', "", Err("bad escape \\q at position 1")),
        (
            "[\\U00110000]",
            0,
            'c',
            "",
            Err("bad escape \\U00110000 at position 1"),
        ),
        (
            "\\U00110000",
            0,
            'c',
            "",
            Err("bad escape \\U00110000 at position 0"),
        ),
        (
            "\\U0011",
            0,
            'c',
            "",
            Err("incomplete escape \\U0011 at position 0"),
        ),
        (
            "\\N{",
            0,
            'c',
            "",
            Err("missing character name at position 3"),
        ),
        (
            "\\N{abc",
            0,
            'c',
            "",
            Err("missing }, unterminated name at position 3"),
        ),
        (
            "\\N{}",
            0,
            'c',
            "",
            Err("missing character name at position 3"),
        ),
        (
            "\\N{NOT A NAME}",
            0,
            'c',
            "",
            Err("undefined character name 'NOT A NAME' at position 0"),
        ),
        (
            "[\\N{NOT A NAME}]",
            0,
            'c',
            "",
            Err("undefined character name 'NOT A NAME' at position 1"),
        ),
        ("\\q", 0, 'c', "", Err("bad escape \\q at position 0")),
        (
            "\\8",
            0,
            'c',
            "",
            Err("invalid group reference 8 at position 1"),
        ),
        (
            "(a)\\18",
            0,
            'c',
            "",
            Err("invalid group reference 18 at position 4"),
        ),
        (
            "(a\\1)",
            0,
            'c',
            "",
            Err("cannot refer to an open group at position 2"),
        ),
        (
            "(?P<a>(?P=a))",
            0,
            'c',
            "",
            Err("cannot refer to an open group at position 10"),
        ),
        (
            "(?<=(a)\\1)",
            0,
            'c',
            "",
            Err("cannot refer to group defined in the same lookbehind subpattern at position 9"),
        ),
        (
            "(?<=(?P<a>x)(?P=a))",
            0,
            'c',
            "",
            Err("cannot refer to group defined in the same lookbehind subpattern at position 18"),
        ),
        ("(?<=(a))(?<=\\1)b", 0, 'c', "", Groups(1)),
        (
            "(?<=(a)(?(1)b|cc))d",
            0,
            'c',
            "",
            Err("cannot refer to group defined in the same lookbehind subpattern at position 12"),
        ),
        (
            "(?<=(a)(?(1)b))d",
            0,
            'c',
            "",
            Err("cannot refer to group defined in the same lookbehind subpattern at position 12"),
        ),
        (
            "(?<=(a)(?(1)|b))d",
            0,
            'c',
            "",
            Err("cannot refer to group defined in the same lookbehind subpattern at position 12"),
        ),
        (
            "a{1,4294967295}",
            0,
            'c',
            "",
            Other("OverflowError", "the repetition number is too large"),
        ),
        (
            "a{4294967295}",
            0,
            'c',
            "",
            Other("OverflowError", "the repetition number is too large"),
        ),
        (
            "a",
            288,
            'c',
            "",
            Other("ValueError", "ASCII and UNICODE flags are incompatible"),
        ),
        (
            "(?a)a",
            32,
            'c',
            "",
            Other("ValueError", "ASCII and UNICODE flags are incompatible"),
        ),
        (
            "(?u)a",
            256,
            'c',
            "",
            Other("ValueError", "ASCII and UNICODE flags are incompatible"),
        ),
        (
            "(a)|\\2",
            0,
            'c',
            "",
            Err("invalid group reference 2 at position 5"),
        ),
        (
            "(?(2)a|b)(c)",
            0,
            'c',
            "",
            Err("invalid group reference 2 at position 3"),
        ),
        (
            "(?<=a{0,70000})b",
            0,
            'c',
            "",
            Err("look-behind requires fixed-width pattern"),
        ),
        (
            "(?<=\\x4)b",
            0,
            'c',
            "",
            Err("incomplete escape \\x4 at position 4"),
        ),
        (
            "\\a\\f\\n\\r\\t\\v\\\\",
            0,
            's',
            "\u{7}\u{c}\u{a}\u{d}\u{9}\u{b}\\",
            Spans(&[Some((0, 7))]),
        ),
        (
            "[\\a][\\b][\\f][\\n][\\r][\\t][\\v][\\\\]",
            0,
            's',
            "\u{7}\u{8}\u{c}\u{a}\u{d}\u{9}\u{b}\\",
            Spans(&[Some((0, 8))]),
        ),
        ("\\U0001F600", 0, 's', "x\u{1f600}", Spans(&[Some((1, 2))])),
        (
            "[\\U0001F600-\\U0001F64F]",
            0,
            's',
            "x\u{1f610}",
            Spans(&[Some((1, 2))]),
        ),
        ("[\\x41-\\x5a]+", 0, 's', "abCDe", Spans(&[Some((2, 4))])),
        ("[\\u0041]", 0, 's', "A", Spans(&[Some((0, 1))])),
        ("[\\101\\7]", 0, 's', "\u{7}A", Spans(&[Some((0, 1))])),
        (
            "[\\.\\-\\\u{e9}]+",
            0,
            's',
            "a.-\u{e9}",
            Spans(&[Some((1, 4))]),
        ),
        ("\\101", 0, 's', "A", Spans(&[Some((0, 1))])),
        ("\\0", 0, 's', "a\u{0}", Spans(&[Some((1, 2))])),
        ("\\x41\\u0042", 0, 's', "AB", Spans(&[Some((0, 2))])),
        ("\\.\\\u{e9}", 0, 's', ".\u{e9}", Spans(&[Some((0, 2))])),
        ("(a)\\1", 0, 's', "aa", Spans(&[Some((0, 2)), Some((0, 1))])),
        (
            "(?P<a>x)(?P=a)",
            0,
            's',
            "xx",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        (
            "(?P<a>x)(?P=a)",
            2,
            's',
            "xX",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        (
            "(a)\\1",
            258,
            's',
            "aA",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        ("(a)?b\\1", 0, 's', "b", NoMatch),
        (
            "(a+)\\1",
            0,
            's',
            "aaa",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        ("(ab)\\1", 0, 's', "aba", NoMatch),
        ("\\Aab", 0, 's', "ab", Spans(&[Some((0, 2))])),
        ("\\Aab", 0, 's', "xab", NoMatch),
        ("ab\\Z", 0, 's', "ab\u{a}", NoMatch),
        ("ab\\z", 0, 's', "xab", Spans(&[Some((1, 3))])),
        ("\\D+", 0, 's', "12ab3", Spans(&[Some((2, 4))])),
        ("\\S+", 0, 's', "  ab ", Spans(&[Some((2, 4))])),
        ("\\W+", 0, 's', "ab-+c", Spans(&[Some((2, 4))])),
        ("\\w+", 256, 's', "\u{e9}_ab1", Spans(&[Some((1, 5))])),
        ("\\W+", 256, 's', "ab\u{e9}_", Spans(&[Some((2, 3))])),
        ("\\s+", 256, 's', "a\u{b}\u{1c} b", Spans(&[Some((1, 2))])),
        ("\\s+", 0, 's', "a\u{1c}\u{1d} b", Spans(&[Some((1, 4))])),
        ("\\d+", 256, 's', "\u{661}\u{662}12", Spans(&[Some((2, 4))])),
        ("\\D+", 256, 's', "12\u{661}\u{662}", Spans(&[Some((2, 4))])),
        ("\\S+", 256, 's', " \u{1c}ab", Spans(&[Some((1, 4))])),
        ("[\\D]+", 0, 's', "12ab3", Spans(&[Some((2, 4))])),
        ("[\\S]+", 0, 's', " ab ", Spans(&[Some((1, 3))])),
        ("[\\W]+", 0, 's', "ab-+c", Spans(&[Some((2, 4))])),
        ("[^\\d]+", 0, 's', "12ab3", Spans(&[Some((2, 4))])),
        ("[^\\D]+", 0, 's', "ab12", Spans(&[Some((2, 4))])),
        ("[^\\w]+", 256, 's', "ab\u{e9}!", Spans(&[Some((2, 4))])),
        ("[\\s\\S]+", 0, 's', "a\u{a}b", Spans(&[Some((0, 3))])),
        ("[^\\s\\S]", 0, 's', "a\u{a}b", NoMatch),
        ("[\\d\\D]", 0, 's', "\u{a}", Spans(&[Some((0, 1))])),
        ("[\\w\\W]", 0, 's', "\u{a}", Spans(&[Some((0, 1))])),
        ("a{}", 0, 's', "a{}", Spans(&[Some((0, 3))])),
        ("a{1,2", 0, 's', "a{1,2", Spans(&[Some((0, 5))])),
        ("a{,2}", 0, 's', "aaa", Spans(&[Some((0, 2))])),
        ("a{2,}", 0, 's', "aaaa", Spans(&[Some((0, 4))])),
        ("a{2}", 0, 's', "aaa", Spans(&[Some((0, 2))])),
        ("a{,}", 0, 's', "aaa", Spans(&[Some((0, 3))])),
        ("a{x}", 0, 's', "a{x}", Spans(&[Some((0, 4))])),
        ("a.c", 0, 's', "a\u{a}c abc", Spans(&[Some((4, 7))])),
        ("a.c", 16, 's', "a\u{a}c", Spans(&[Some((0, 3))])),
        ("(?s:a.c)", 0, 's', "a\u{a}c", Spans(&[Some((0, 3))])),
        ("(?#comment)ab", 0, 's', "ab", Spans(&[Some((0, 2))])),
        ("(?#a)", 0, 's', "", Spans(&[Some((0, 0))])),
        ("(?x)a#comment", 0, 's', "a", Spans(&[Some((0, 1))])),
        ("(?x)a # c\u{a} b", 0, 's', "ab", Spans(&[Some((0, 2))])),
        ("(?=)a", 0, 's', "a", Spans(&[Some((0, 1))])),
        ("(?!)a", 0, 's', "a", NoMatch),
        ("(?<=)a", 0, 's', "a", Spans(&[Some((0, 1))])),
        ("(?<!)a", 0, 's', "a", NoMatch),
        ("(?!a)\\w", 0, 's', "ab", Spans(&[Some((1, 2))])),
        ("(?<!a)b", 0, 's', "abcb", Spans(&[Some((3, 4))])),
        (
            "(?<=(a))b",
            0,
            's',
            "ab",
            Spans(&[Some((1, 2)), Some((0, 1))]),
        ),
        ("(?<=(?>a))b", 0, 's', "ab", Spans(&[Some((1, 2))])),
        ("(?<=(?:ab))c", 0, 's', "abc", Spans(&[Some((2, 3))])),
        (
            "(?<=(a)(?(1)b|c))d",
            0,
            's',
            "abd",
            Err("cannot refer to group defined in the same lookbehind subpattern at position 12"),
        ),
        (
            "(?<=(a)\\w(?(1)b|c))d",
            0,
            's',
            "axbd",
            Err("cannot refer to group defined in the same lookbehind subpattern at position 14"),
        ),
        (
            "(?<=a|bc)d",
            0,
            'c',
            "",
            Err("look-behind requires fixed-width pattern"),
        ),
        (
            "(?<=a*)b",
            0,
            'c',
            "",
            Err("look-behind requires fixed-width pattern"),
        ),
        ("(?<=a{2})b", 0, 's', "aab", Spans(&[Some((2, 3))])),
        (
            "(?<=a{1,2})b",
            0,
            'c',
            "",
            Err("look-behind requires fixed-width pattern"),
        ),
        (
            "(?<=a+?)b",
            0,
            'c',
            "",
            Err("look-behind requires fixed-width pattern"),
        ),
        (
            "(?P<n>a)(?(n)b|c)",
            0,
            's',
            "ab",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        (
            "(?P<n>a)?(?(n)b|c)",
            0,
            's',
            "c",
            Spans(&[Some((0, 1)), None]),
        ),
        ("(a)?(?(1)b)", 0, 's', "x", Spans(&[Some((0, 0)), None])),
        ("^ab", 0, 's', "ab", Spans(&[Some((0, 2))])),
        ("^b", 8, 's', "a\u{a}b", Spans(&[Some((2, 3))])),
        ("^b", 0, 's', "a\u{a}b", NoMatch),
        ("a$", 8, 's', "a\u{a}b", Spans(&[Some((0, 1))])),
        ("(?m)a$", 0, 's', "ab", NoMatch),
        ("(?:ab)c", 0, 's', "abc", Spans(&[Some((0, 3))])),
        ("(?:ab)+", 0, 's', "ababx", Spans(&[Some((0, 4))])),
        ("ab|ac", 0, 's', "ac", Spans(&[Some((0, 2))])),
        ("abc|abd|ab", 0, 's', "abd", Spans(&[Some((0, 3))])),
        (".a|.b", 0, 's', "xb", Spans(&[Some((0, 2))])),
        ("[ab]x|[ab]y", 0, 's', "by", Spans(&[Some((0, 2))])),
        ("^a|^b", 0, 's', "b", Spans(&[Some((0, 1))])),
        ("(?!)a|(?!)b", 0, 's', "b", NoMatch),
        (
            "(x)(?:\\1a|\\1b)",
            0,
            's',
            "xxb",
            Spans(&[Some((0, 3)), Some((0, 1))]),
        ),
        ("a|[bc]", 0, 's', "c", Spans(&[Some((0, 1))])),
        ("a|[^bc]", 0, 's', "d", Spans(&[Some((0, 1))])),
        ("a|b|c", 0, 's', "c", Spans(&[Some((0, 1))])),
        ("a|\\d", 0, 's', "5", Spans(&[Some((0, 1))])),
        ("[^a]", 0, 's', "ab", Spans(&[Some((1, 2))])),
        ("[^ab]", 0, 's', "abc", Spans(&[Some((2, 3))])),
        ("[^a]", 2, 's', "Ab", Spans(&[Some((1, 2))])),
        ("[^a]", 258, 's', "Ab", Spans(&[Some((1, 2))])),
        ("(?ai)a", 0, 's', "A", Spans(&[Some((0, 1))])),
        ("(?ai)1", 0, 's', "1", Spans(&[Some((0, 1))])),
        ("(?ai)\u{e9}", 0, 's', "\u{c9}", NoMatch),
        ("(?ai)[a-c]+", 0, 's', "xABC", Spans(&[Some((1, 4))])),
        ("(?ai)[\u{e9}]", 0, 's', "\u{c9}", NoMatch),
        (
            "(?i)[\u{e9}-\u{eb}]",
            0,
            's',
            "\u{ca}",
            Spans(&[Some((0, 1))]),
        ),
        ("(?ai)[0-9a]+", 0, 's', "A09", Spans(&[Some((0, 3))])),
        (
            "(?i)[\\u0100-\\u0101]",
            0,
            's',
            "\u{101}",
            Spans(&[Some((0, 1))]),
        ),
        (
            "(?i)[\\uff00-\\U00010420]",
            0,
            's',
            "\u{10400}",
            Spans(&[Some((0, 1))]),
        ),
        (
            "(?i)[\\U00010400-\\U00010427]",
            0,
            's',
            "\u{10428}",
            Spans(&[Some((0, 1))]),
        ),
        (
            "(?i)[a-\\U00010428]",
            0,
            's',
            "\u{10400}",
            Spans(&[Some((0, 1))]),
        ),
        (
            "[\\uffff-\\U00010001]",
            0,
            's',
            "\u{10000}",
            Spans(&[Some((0, 1))]),
        ),
        (
            "[\\U00010000-\\U00010001]",
            0,
            's',
            "\u{10001}",
            Spans(&[Some((0, 1))]),
        ),
        (
            "[a-z\\U00010000-\\U00010001]",
            2,
            's',
            "\u{10001}",
            Spans(&[Some((0, 1))]),
        ),
        ("(?i)\\u1e9e", 0, 's', "\u{df}", Spans(&[Some((0, 1))])),
        ("(?i)[\\u1e9e]", 0, 's', "\u{df}", Spans(&[Some((0, 1))])),
        ("(?i)[k]", 0, 's', "\u{212a}", Spans(&[Some((0, 1))])),
        ("(?i)k", 256, 's', "\u{212a}", NoMatch),
        ("(?i)\\w", 0, 's', "-\u{c9}", Spans(&[Some((1, 2))])),
        (
            "(?i)(a)\\1",
            0,
            's',
            "aA",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        (
            "(?i)(\u{212a})\\1",
            0,
            's',
            "\u{212a}k",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        ("(?i)(\u{212a})\\1", 256, 's', "\u{212a}k", NoMatch),
        ("(?i)[^\u{212a}]", 0, 's', "kx", Spans(&[Some((1, 2))])),
        ("\\bab\\b", 0, 's', "x ab y", Spans(&[Some((2, 4))])),
        ("\\b\u{e9}", 0, 's', "\u{e9}", Spans(&[Some((0, 1))])),
        ("\\b\u{e9}", 256, 's', "\u{e9}", NoMatch),
        ("\\B\u{e9}", 256, 's', "a\u{e9}", NoMatch),
        ("a\\b", 0, 's', "ab a", Spans(&[Some((3, 4))])),
        ("\\b", 0, 's', "  ", NoMatch),
        ("^", 8, 's', "a\u{a}", Spans(&[Some((0, 0))])),
        ("$", 8, 's', "a\u{a}b", Spans(&[Some((1, 1))])),
        ("^$", 8, 's', "a\u{a}\u{a}b", Spans(&[Some((2, 2))])),
        ("\\Z", 0, 's', "a\u{a}", Spans(&[Some((2, 2))])),
        ("a*?b", 0, 's', "aab", Spans(&[Some((0, 3))])),
        ("a+?", 0, 's', "aaa", Spans(&[Some((0, 1))])),
        ("a{2,3}?", 0, 's', "aaaa", Spans(&[Some((0, 2))])),
        ("a{2,3}?b", 0, 's', "aab", Spans(&[Some((0, 3))])),
        ("a{3,}?", 0, 's', "aa", NoMatch),
        ("[ab]{2}?c", 0, 's', "xbac", Spans(&[Some((1, 4))])),
        ("(?:ab){2,}?c", 0, 's', "ababc", Spans(&[Some((0, 5))])),
        ("(?:ab){2}?", 0, 's', "abx", NoMatch),
        ("(?:a|b)*?c", 0, 's', "ab", NoMatch),
        ("(a?)*?b", 0, 's', "b", Spans(&[Some((0, 1)), None])),
        ("(?:a|b){2,3}?", 0, 's', "abab", Spans(&[Some((0, 2))])),
        ("a*+", 0, 's', "aaa", Spans(&[Some((0, 3))])),
        ("a++b", 0, 's', "aab", Spans(&[Some((0, 3))])),
        ("a{2}+", 0, 's', "aaa", Spans(&[Some((0, 2))])),
        ("(?:ab)*+", 0, 's', "abab", Spans(&[Some((0, 4))])),
        ("(?:ab)++a", 0, 's', "ababa", Spans(&[Some((0, 5))])),
        ("(?:ab){2,3}+c", 0, 's', "abababc", Spans(&[Some((0, 7))])),
        ("(?:ab){2,}+", 0, 's', "ab", NoMatch),
        ("(?:ab){3}+", 0, 's', "ababx", NoMatch),
        ("(?:a|ab)++c", 0, 's', "abc", NoMatch),
        (
            "(a|)*+b",
            0,
            's',
            "aab",
            Spans(&[Some((0, 3)), Some((2, 2))]),
        ),
        ("(?:a*)*+b", 0, 's', "aab", Spans(&[Some((0, 3))])),
        (
            "(?:(a)|b)++",
            0,
            's',
            "ab",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        ("(a)*+(?:\\1)", 0, 's', "aa", NoMatch),
        (
            "(?:x(a)|x)++",
            0,
            's',
            "xax",
            Spans(&[Some((0, 3)), Some((1, 2))]),
        ),
        ("(?>(a)|b)", 0, 's', "b", Spans(&[Some((0, 1)), None])),
        ("(?>a|ab)c", 0, 's', "abc", NoMatch),
        (
            "(?>(a))\\1",
            0,
            's',
            "aa",
            Spans(&[Some((0, 2)), Some((0, 1))]),
        ),
        ("(?:a{0})+", 0, 's', "a", Spans(&[Some((0, 0))])),
        (
            "(?=(a))a",
            0,
            's',
            "a",
            Spans(&[Some((0, 1)), Some((0, 1))]),
        ),
        ("(?!(a))b", 0, 's', "b", Spans(&[Some((0, 1)), None])),
        ("(?!(a)b)a", 0, 's', "ac", Spans(&[Some((0, 1)), None])),
        ("(?<!(a))b", 0, 's', "cb", Spans(&[Some((1, 2)), None])),
        ("[\\D\\d]", 0, 's', "x", Spans(&[Some((0, 1))])),
        ("[\\S\\s]", 0, 's', "x", Spans(&[Some((0, 1))])),
        ("[\\W\\w]", 0, 's', "x", Spans(&[Some((0, 1))])),
        (
            "(?<=(a(?(1)b|c)))",
            0,
            'c',
            "",
            Err("cannot refer to an open group at position 11"),
        ),
        ("(a)(?<=(?(1)b|c))d", 0, 's', "abd", NoMatch),
        (
            "(a)b(?<=(?(1)b|c))d",
            0,
            's',
            "abd",
            Spans(&[Some((0, 3)), Some((0, 1))]),
        ),
        (
            "(a)(?<=(?(1)b))d",
            0,
            'c',
            "",
            Err("look-behind requires fixed-width pattern"),
        ),
        (
            "(a)(?<=(?(1)bb|c))d",
            0,
            'c',
            "",
            Err("look-behind requires fixed-width pattern"),
        ),
        ("(?-im:a)", 0, 's', "A", NoMatch),
        (
            "(?<=(?:a{4000000000}){2})b",
            0,
            'c',
            "",
            Err("looks too much behind"),
        ),
        (
            "(?i)[\u{10428}]",
            0,
            's',
            "\u{10400}",
            Spans(&[Some((0, 1))]),
        ),
        ("[\u{1f600}]", 0, 's', "x\u{1f600}", Spans(&[Some((1, 2))])),
        (
            "(?ai)[\u{10000}-\u{10001}]",
            0,
            's',
            "\u{10001}",
            Spans(&[Some((0, 1))]),
        ),
        ("(?:a?)*?c", 0, 's', "xb", NoMatch),
        ("ab", 0, 'm', "abc", Spans(&[Some((0, 2))])),
        ("ab", 0, 'm', "xab", NoMatch),
        ("ab", 0, 'f', "abc", NoMatch),
        ("ab|abc", 0, 'f', "abc", Spans(&[Some((0, 3))])),
        ("a*", 0, 'f', "aaa", Spans(&[Some((0, 3))])),
        (
            "(a)(b)?",
            0,
            'f',
            "a",
            Spans(&[Some((0, 1)), Some((0, 1)), None]),
        ),
        ("", 0, 'f', "", Spans(&[Some((0, 0))])),
        ("[\u{1f600}a]", 0, 'f', "\u{1f600}", Spans(&[Some((0, 1))])),
        ("[\u{1f600}a]", 0, 'f', "\u{1f601}", NoMatch),
        ("[\u{1f600}a]", 2, 'f', "\u{1f600}", Spans(&[Some((0, 1))])),
        ("[\u{10400}]", 2, 'f', "\u{10428}", Spans(&[Some((0, 1))])),
        ("[\u{10400}]", 2, 'f', "\u{10401}", NoMatch),
        // END CPYTHON ROWS
    ];
    let mut wrong = Vec::new();
    for (pattern, flags, mode, input, want) in rows {
        let got = run(pattern, *flags, *mode, input);
        if &got != want {
            wrong.push(format!(
                "{pattern:?} {flags} {mode} {input:?}: got {got:?}, CPython {want:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

const DIGITS: &str = "Exceeds the limit (4300 digits) for integer string conversion: value has 5000 digits; use sys.set_int_max_str_digits() to increase the limit";

#[test]
fn long_patterns_are_refused_or_accepted_as_cpython_does() {
    let ones = "1".repeat(5000);
    assert_eq!(
        run(&format!("a{{{ones}}}"), 0, 'c', ""),
        Other("ValueError", DIGITS)
    );
    assert_eq!(
        run(&format!("a{{1,{ones}}}"), 0, 'c', ""),
        Other("ValueError", DIGITS)
    );
    // The nesting guard sits deeper than a test thread's default stack.
    let nested = std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            let recursion = Other("RecursionError", "maximum recursion depth exceeded");
            let deep = format!("{}{}", "(".repeat(1000), ")".repeat(1000));
            assert_eq!(run(&deep, 0, 'c', ""), recursion);
            let deep = format!("{}{}", "(?:".repeat(1000), ")".repeat(1000));
            assert_eq!(run(&deep, 0, 'c', ""), recursion);
            // Conditional branches nest one parse frame each.
            let deep = format!("(a){}{}", "(?(1)".repeat(1000), ")".repeat(1000));
            assert_eq!(run(&deep, 0, 'c', ""), recursion);
        })
        .unwrap();
    nested.join().unwrap();
    let behind = format!("(?<={})b", "a".repeat(70000));
    assert_eq!(run(&behind, 0, 'c', ""), Groups(0));
}

#[test]
#[should_panic(expected = "pattern \"(\"")]
fn a_built_in_pattern_that_does_not_compile_is_a_programming_error() {
    sre::compile_str("(", 0);
}
