//! `rates`' command line against what CPython 3.15's argparse prints for it: layouts at narrow
//! widths, option and argument errors, clustered help flags and the namespaces parsed.
mod cov_b_common;

use au_tax_rates_data::argparse::{Action, Exit, Parser, Value};
use au_tax_rates_data::pyjson::{Json, loads};
use cov_b_common::{case, same_as_python};

#[test]
fn narrow_terminals_wrap_usage_and_help_as_python_does() {
    // 30 columns keeps the usage after the program name; 20 and 12 move it below.
    for name in [
        "check_help_30",
        "check_help_20",
        "snapshot_help_12",
        "no_command_24",
        "no_command_16",
    ] {
        same_as_python(name);
    }
}

#[test]
fn option_and_argument_errors_match_python() {
    for name in [
        "out_without_value",
        "bare_equals_is_ambiguous",
        "long_help_with_value",
        "short_help_with_value",
        "short_help_with_dash",
        "clustered_help_with_value",
        "unknown_option",
        "stray_after_command",
        "stray_before_option",
    ] {
        same_as_python(name);
    }
}

#[test]
fn dashes_numbers_and_spaces_are_read_as_the_subcommand() {
    // A lone dash, a negative number and a dashed word with a space are positionals, so the
    // subcommand refuses them as choices rather than as unknown options.
    for name in [
        "lone_dash",
        "negative_integer",
        "negative_decimal",
        "dash_with_space",
    ] {
        same_as_python(name);
    }
}

#[test]
fn only_a_close_word_is_offered_as_a_suggestion() {
    // A 258-character word takes difflib's autojunk path in Python; the port skips it and
    // still offers nothing.
    for name in ["far_word", "near_word", "very_long_word"] {
        same_as_python(name);
    }
    assert!(
        case("near_word")
            .stderr
            .contains("maybe you meant 'validate'?")
    );
    assert!(!case("very_long_word").stderr.contains("maybe you meant"));
}

#[test]
fn clustered_and_separated_help_flags_print_help() {
    for name in [
        "clustered_help",
        "help_then_unknown",
        "clustered_help_then_unknown",
        "help_after_dashes",
    ] {
        same_as_python(name);
    }
}

/// The parser `rates` builds, with "reports" as the `--out` default.
fn rates_parser() -> Parser {
    Parser::new("rates")
        .raw_description("Snapshot, validate and re-check the AU tax rates dataset.\n")
        .subcommands("command")
        .sub("snapshot", None, |p| p)
        .sub("validate", None, |p| p)
        .sub("check", None, |p| {
            p.arg(
                Action::option("--out")
                    .default(Value::Str("reports".into()))
                    .help("folder for the Markdown report"),
            )
        })
        .sub("ato-tables", None, |p| p)
}

#[test]
fn equals_signs_abbreviations_dashes_and_repeats_parse_as_python_does() {
    for name in [
        "out_after_equals",
        "abbreviated_out",
        "abbreviated_out_equals",
        "command_after_dashes",
        "last_out_wins",
    ] {
        let case = case(name);
        let Ok(namespace) = rates_parser().parse(&case.args) else {
            panic!("{name} parses");
        };
        let parsed: Vec<(String, String)> = namespace
            .iter()
            .map(|(key, value)| match value {
                Value::Str(s) => (key.clone(), s.clone()),
                other => panic!("{name}: {key} is {other:?}"),
            })
            .collect();
        let Ok(Json::Object(python)) = loads(&case.stdout) else {
            panic!("{name}: Python printed a namespace");
        };
        let python: Vec<(String, String)> = python
            .iter()
            .map(|(k, v)| (k.as_str().unwrap().into(), v.as_str().unwrap().into()))
            .collect();
        assert_eq!(parsed, python, "{name}");
    }
}

#[test]
fn a_subcommand_with_help_is_listed_under_the_subcommand_argument() {
    // `rates` gives its subcommands no help, so its own help lists none.
    let parser = Parser::new("tool")
        .subcommands("command")
        .sub("snap", Some("take a snapshot"), |p| p)
        .sub("quiet", None, |p| p);
    let Err(Exit::Help(text)) = parser.parse(&["-h".to_string()]) else {
        panic!("-h prints help");
    };
    let plain = strip_escapes(&text);
    assert!(plain.contains("\n    snap"), "{plain}");
    assert!(plain.contains("take a snapshot"), "{plain}");
    assert!(!plain.contains("\n    quiet"), "{plain}");
}

/// The text without the colour codes a terminal on stdout would add.
fn strip_escapes(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
