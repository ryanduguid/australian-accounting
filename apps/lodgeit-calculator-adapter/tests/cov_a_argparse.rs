// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The argparse port's parsing, refusals and help layout, case by case, for the
//! parts of argparse the adapter's own command line does not use. Every test
//! fixes an 80-column terminal and plain output, so the expected text is the
//! text Python 3.15's argparse writes under the same settings.
use std::sync::Once;

use lodgeit_calculator_adapter::argparse::{
    Action, Exit, Get, Namespace, Nargs, Parser, Value, format_help,
};
use lodgeit_calculator_adapter::argtheme::PLAIN;

fn setup() {
    static ONCE: Once = Once::new();
    // SAFETY: every test in this file sets the same values before reading any,
    // and std's environment functions hold a lock.
    ONCE.call_once(|| unsafe {
        std::env::set_var("PYTHON_COLORS", "0");
        std::env::set_var("COLUMNS", "80");
        std::env::set_var("LINES", "24");
    });
}

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn parse(p: &Parser, items: &[&str]) -> Namespace {
    setup();
    match p.parse(&args(items)) {
        Ok(ns) => ns,
        Err(exit) => panic!("{items:?} was refused: {exit:?}"),
    }
}

/// The message after "prog: error: " for a refused command line.
fn refusal(p: &Parser, items: &[&str]) -> String {
    setup();
    match p.parse(&args(items)) {
        Err(Exit::Error(text)) => {
            let marker = format!("{}: error: ", p.prog);
            let at = text.rfind(&marker).expect("error line");
            text[at + marker.len()..].trim_end().to_string()
        }
        other => panic!("{items:?} was not refused: {other:?}"),
    }
}

fn usage(p: &Parser) -> String {
    setup();
    let Exit::Error(text) = p.error("x") else {
        panic!("error() is always an error")
    };
    text.strip_suffix(&format!("{}: error: x\n", p.prog))
        .expect("usage then the error line")
        .trim_end()
        .to_string()
}

fn str_value(text: &str) -> Value {
    Value::Str(text.into())
}

#[test]
fn positionals_take_their_values_and_an_optional_one_falls_back_to_its_default() {
    let p = Parser::new("prog")
        .arg(Action::positional("src"))
        .arg(Action::optional_positional("dst", str_value("out")))
        .arg(Action::optional_positional("count", str_value("7")).int())
        .arg(Action::optional_positional("level", Value::Int(3)).int());
    let ns = parse(&p, &["in"]);
    assert_eq!(ns.str("src").as_deref(), Some("in"));
    assert_eq!(ns.str("dst").as_deref(), Some("out"));
    // A string default of an int positional is converted, a non-string kept.
    assert_eq!(ns.int("count"), Some(7));
    assert_eq!(ns.int("level"), Some(3));
    let ns = parse(&p, &["in", "there", "9"]);
    assert_eq!(ns.str("dst").as_deref(), Some("there"));
    assert_eq!(ns.int("count"), Some(9));
}

#[test]
fn dashes_negative_numbers_and_spaced_words_read_as_positionals() {
    let p = Parser::new("prog").arg(Action::positional("value"));
    assert_eq!(parse(&p, &["-"]).str("value").as_deref(), Some("-"));
    assert_eq!(parse(&p, &["-5"]).str("value").as_deref(), Some("-5"));
    assert_eq!(parse(&p, &["-1.5"]).str("value").as_deref(), Some("-1.5"));
    assert_eq!(parse(&p, &["-a b"]).str("value").as_deref(), Some("-a b"));
    // After `--` an option-like word is a value, and the `--` itself is dropped.
    assert_eq!(parse(&p, &["--", "-x"]).str("value").as_deref(), Some("-x"));
}

#[test]
fn an_int_positional_reads_a_negative_number() {
    let p = Parser::new("prog").arg(Action::positional("n").int());
    assert_eq!(parse(&p, &["-5"]).int("n"), Some(-5));
}

#[test]
fn an_optional_positional_waits_for_values_after_an_option() {
    let p = Parser::new("prog")
        .arg(Action::positional("a"))
        .arg(Action::optional_positional("b", str_value("none")))
        .arg(Action::flag("--fast"));
    let ns = parse(&p, &["1", "--fast", "2"]);
    assert_eq!(ns.str("a").as_deref(), Some("1"));
    assert_eq!(ns.str("b").as_deref(), Some("2"));
    assert!(ns.flag("fast"));
}

#[test]
fn explicit_and_abbreviated_option_values_are_read() {
    let p = Parser::new("prog").arg(Action::option("--count").int());
    assert_eq!(parse(&p, &["--count=5"]).int("count"), Some(5));
    assert_eq!(parse(&p, &["--cou=6"]).int("count"), Some(6));
    assert_eq!(parse(&p, &["--cou", "8"]).int("count"), Some(8));
    assert_eq!(
        refusal(&p, &["--count", "x"]),
        "argument --count: invalid int value: 'x'"
    );
    assert_eq!(
        refusal(&p, &["--count"]),
        "argument --count: expected one argument"
    );
}

#[test]
fn short_flags_cluster_and_the_last_may_carry_a_value() {
    let p = Parser::new("prog")
        .arg(Action::flag("-v"))
        .arg(Action::flag("-q"))
        .arg(Action::option("-n"));
    let ns = parse(&p, &["-vq"]);
    assert!(ns.flag("v") && ns.flag("q"));
    assert_eq!(ns.str("n"), None);
    let ns = parse(&p, &["-vn=3"]);
    assert!(ns.flag("v"));
    assert_eq!(ns.str("n").as_deref(), Some("3"));
    let ns = parse(&p, &["-vqn4"]);
    assert!(ns.flag("q"));
    assert_eq!(ns.str("n").as_deref(), Some("4"));
    assert_eq!(refusal(&p, &["-vz"]), "unrecognized arguments: -z");
    assert_eq!(
        refusal(&p, &["-v=1"]),
        "argument -v: ignored explicit argument '1'"
    );
    assert_eq!(
        refusal(&p, &["-v-q"]),
        "argument -v: ignored explicit argument '-q'"
    );
}

#[test]
fn a_long_flag_refuses_an_explicit_value() {
    let p = Parser::new("prog").arg(Action::flag("--verbose"));
    assert_eq!(
        refusal(&p, &["--verbose=1"]),
        "argument --verbose: ignored explicit argument '1'"
    );
}

#[test]
fn a_single_dash_long_option_may_be_abbreviated() {
    let p = Parser::new("prog").arg(Action::option("-foo"));
    assert_eq!(parse(&p, &["-fo", "x"]).str("foo").as_deref(), Some("x"));
}

#[test]
fn an_optional_positional_is_bracketed_in_the_usage() {
    let p = Parser::new("prog").arg(Action::optional_positional("dst", Value::None));
    assert_eq!(usage(&p), "usage: prog [-h] [dst]");
}

#[test]
fn an_abbreviation_matching_two_options_is_ambiguous() {
    let p = Parser::new("prog")
        .arg(Action::option("--alpha"))
        .arg(Action::option("--alps"));
    assert_eq!(
        refusal(&p, &["--al", "x"]),
        "ambiguous option: --al could match --alpha, --alps"
    );
}

#[test]
fn unknown_options_and_stray_words_are_unrecognised() {
    let p = Parser::new("prog").arg(Action::flag("--fast"));
    assert_eq!(refusal(&p, &["--bogus"]), "unrecognized arguments: --bogus");
    assert_eq!(
        refusal(&p, &["stray", "--fast"]),
        "unrecognized arguments: stray"
    );
}

#[test]
fn missing_required_arguments_are_named() {
    let p = Parser::new("prog")
        .arg(Action::positional("src"))
        .arg(Action::option("--name").required());
    assert_eq!(
        refusal(&p, &[]),
        "the following arguments are required: src, --name"
    );
}

#[test]
fn append_options_collect_every_value_after_their_default() {
    let p = Parser::new("prog")
        .arg(Action::append("--tag", Value::None))
        .arg(Action::append("--keep", Value::List(vec!["base".into()])));
    let ns = parse(&p, &["--tag", "a", "--tag", "b", "--keep", "c"]);
    assert_eq!(ns.list("tag"), ["a", "b"]);
    assert_eq!(ns.list("keep"), ["base", "c"]);
    assert_eq!(parse(&p, &[]).list("tag"), Vec::<String>::new());
}

#[test]
fn one_or_more_values_are_kept_in_order() {
    let p = Parser::new("prog")
        .arg(Action::option("--ids").int().one_or_more())
        .arg(Action::positional("names").one_or_more());
    let ns = parse(&p, &["--ids", "1", "2", "--", "x", "y"]);
    assert_eq!(ns.list("ids"), ["1", "2"]);
    assert_eq!(ns.list("names"), ["x", "y"]);
    assert_eq!(
        refusal(&p, &["x", "--ids", "1", "q"]),
        "argument --ids: invalid int value: 'q'"
    );
    assert_eq!(
        refusal(&p, &["x", "--ids"]),
        "argument --ids: expected at least one argument"
    );
}

#[test]
fn one_or_more_values_are_each_checked_against_the_choices() {
    let p = Parser::new("prog").arg(
        Action::option("--day")
            .one_or_more()
            .choices(&["mon", "tue"]),
    );
    assert_eq!(
        parse(&p, &["--day", "tue", "mon"]).list("day"),
        ["tue", "mon"]
    );
    assert_eq!(
        refusal(&p, &["--day", "mon", "wed"]),
        "argument --day: invalid choice: 'wed' (choose from 'mon', 'tue')"
    );
}

#[test]
fn an_option_with_an_optional_value_reads_one_word() {
    let p = Parser::new("prog").arg(Action {
        nargs: Nargs::Optional,
        ..Action::option("--level")
    });
    assert_eq!(
        parse(&p, &["--level", "3"]).str("level").as_deref(),
        Some("3")
    );
}

#[test]
fn int_choices_compare_the_converted_value_without_a_suggestion() {
    let p = Parser::new("prog").arg(Action::option("--n").int().choices(&["1", "2"]));
    assert_eq!(parse(&p, &["--n", "02"]).int("n"), Some(2));
    assert_eq!(
        refusal(&p, &["--n", "3"]),
        "argument --n: invalid choice: '3' (choose from '1', '2')"
    );
}

#[test]
fn a_mistyped_choice_suggests_the_closest_one() {
    let p = Parser::new("prog").arg(Action::option("--mode").choices(&["fast", "slow"]));
    assert_eq!(
        refusal(&p, &["--mode", "fsat"]),
        "argument --mode: invalid choice: 'fsat', maybe you meant 'fast'? (choose from 'fast', 'slow')"
    );
    assert_eq!(
        refusal(&p, &["--mode", "lsow"]),
        "argument --mode: invalid choice: 'lsow', maybe you meant 'slow'? (choose from 'fast', 'slow')"
    );
    assert_eq!(
        refusal(&p, &["--mode", "zzz"]),
        "argument --mode: invalid choice: 'zzz' (choose from 'fast', 'slow')"
    );
}

#[test]
fn a_suggestion_needs_a_ratio_of_at_least_six_tenths() {
    // 'abc' matches in the middle of both, with one unmatched letter at each
    // end: 2 * 3 / 10 is exactly 0.6.
    let p = Parser::new("prog").arg(Action::option("--m").choices(&["xabcy"]));
    assert_eq!(
        refusal(&p, &["--m", "yabcx"]),
        "argument --m: invalid choice: 'yabcx', maybe you meant 'xabcy'? (choose from 'xabcy')"
    );
}

#[test]
fn a_long_mistyped_choice_still_matches_through_its_repeated_letters() {
    // In a word of 200 or more letters a letter used more than 3 times is not
    // indexed, so the match grows from the one indexed letter through the
    // repeated ones on either side, as difflib's does.
    let run = "x".repeat(198);
    for (choice, typed) in [
        (format!("y{run}q"), format!("x{run}q")),
        (format!("q{run}y"), format!("q{run}x")),
    ] {
        let p = Parser::new("prog").arg(Action::option("--m").choices(&[choice.as_str()]));
        assert_eq!(
            refusal(&p, &["--m", &typed]),
            format!(
                "argument --m: invalid choice: '{typed}', maybe you meant '{choice}'? (choose from '{choice}')"
            )
        );
    }
    let p = Parser::new("prog").arg(Action::option("--m").choices(&["fast"]));
    let typed = format!("fast{}", "x".repeat(196));
    assert_eq!(
        refusal(&p, &["--m", &typed]),
        format!("argument --m: invalid choice: '{typed}' (choose from 'fast')")
    );
}

#[test]
fn a_sub_command_reports_its_own_errors_and_takes_values_after_dashes() {
    let p = Parser::new("prog")
        .subcommands("command")
        .sub("run", Some("run it"), |s| {
            s.arg(Action::option("--name").required())
        });
    let ns = parse(&p, &["--", "run", "--name", "x"]);
    assert_eq!(ns.str("command").as_deref(), Some("run"));
    assert_eq!(ns.str("name").as_deref(), Some("x"));
    setup();
    let Err(Exit::Error(text)) = p.parse(&args(&["run"])) else {
        panic!("a missing required sub-command option is refused")
    };
    assert_eq!(
        text,
        "usage: prog run [-h] --name NAME\nprog run: error: the following arguments are required: --name\n"
    );
    assert_eq!(
        refusal(&p, &["rn"]),
        "argument command: invalid choice: 'rn', maybe you meant 'run'? (choose from 'run')"
    );
}

#[test]
fn a_suppressed_default_leaves_the_name_out_until_given() {
    let p = Parser::new("prog").arg(Action::option("--x").default(Value::Suppress));
    assert!(!parse(&p, &[]).contains_key("x"));
    assert_eq!(parse(&p, &["--x", "1"]).str("x").as_deref(), Some("1"));
}

#[test]
fn namespace_getters_answer_none_or_empty_for_a_missing_or_other_value() {
    let mut ns = Namespace::new();
    ns.insert("n".into(), Value::Int(4));
    ns.insert("s".into(), str_value("t"));
    assert_eq!(ns.str("n"), None);
    assert_eq!(ns.int("s"), None);
    assert_eq!(ns.int("n"), Some(4));
    assert_eq!(ns.list("s"), Vec::<String>::new());
    assert!(!ns.flag("s"));
}

#[test]
fn exit_report_returns_zero_for_help_and_two_for_an_error() {
    assert_eq!(Exit::Help(String::new()).report(), 0);
    assert_eq!(Exit::Error(String::new()).report(), 2);
}

#[test]
fn help_lays_out_metavars_required_options_and_blank_help() {
    setup();
    let p = Parser::new("prog")
        .description("Adds   numbers\n together.")
        .arg(
            Action::positional("values")
                .one_or_more()
                .metavar("N")
                .help("numbers to add"),
        )
        .arg(Action::option("--scale").required().int().help("   "))
        .arg(Action::option("--mode").choices(&["fast", "slow"]));
    let expected = [
        "usage: prog [-h] --scale SCALE [--mode {fast,slow}] N [N ...]",
        "",
        "Adds numbers together.",
        "",
        "positional arguments:",
        &format!("  {:<18}  numbers to add", "N"),
        "",
        "options:",
        &format!("  {:<18}  show this help message and exit", "-h, --help"),
        &format!("  {:<18}  ", "--scale SCALE"),
        "  --mode {fast,slow}",
        "",
    ]
    .join("\n");
    assert_eq!(format_help(&p, &PLAIN), expected);
}

#[test]
fn a_raw_description_keeps_its_own_lines() {
    let p = Parser::new("prog").raw_description("Line one\n  indented   here");
    assert_eq!(
        format_help(&p, &PLAIN),
        "usage: prog [-h]\n\nLine one\n  indented   here\n\noptions:\n  -h, --help  show this help message and exit\n"
    );
}

#[test]
fn a_parser_without_arguments_has_a_bare_usage() {
    let mut p = Parser::new("prog");
    p.actions.clear();
    assert_eq!(usage(&p), "usage: prog");
}

#[test]
fn wrapped_usage_of_positionals_only_lines_up_under_the_first() {
    let mut p = Parser::new("prog");
    p.actions.clear();
    let names = ["a", "b", "c"].map(|c| c.repeat(30));
    for name in &names {
        p = p.arg(Action::positional(name));
    }
    let [a, b, c] = &names;
    assert_eq!(
        usage(&p),
        format!("usage: prog {a} {b}\n{}{c}", " ".repeat(12))
    );
    let help = format_help(&p, &PLAIN);
    assert!(help.contains(&format!("positional arguments:\n  {a}\n  {b}\n  {c}\n")));
    assert!(!help.contains("options:"));
}

#[test]
fn a_long_program_name_puts_the_usage_on_the_next_lines() {
    let prog = "p".repeat(60);
    let indent = " ".repeat(7);
    let short = Parser::new(&prog).arg(Action::option("--alpha"));
    assert_eq!(
        usage(&short),
        format!("usage: {prog}\n{indent}[-h] [--alpha ALPHA]")
    );
    let p = ["aaaaaaaaaa", "bbbbbbbbbb", "cccccccccc"]
        .iter()
        .fold(Parser::new(&prog), |p, name| {
            p.arg(Action::option(&format!("--{name}")))
        })
        .arg(Action::positional("target"));
    assert_eq!(
        usage(&p),
        format!(
            "usage: {prog}\n{indent}[-h] [--aaaaaaaaaa AAAAAAAAAA] [--bbbbbbbbbb BBBBBBBBBB]\n{indent}[--cccccccccc CCCCCCCCCC]\n{indent}target"
        )
    );
}
