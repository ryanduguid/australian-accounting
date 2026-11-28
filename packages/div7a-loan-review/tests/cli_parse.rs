// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The main parser's refusals, run through the built binary at 80 columns
//! without colour. Each expected text is what CPython 3.15's argparse prints
//! for a parser with this one's prog, `--version` and required subcommands
//! rate, gate, myr and review.

use std::process::Command;

const USAGE: &str = "usage: div7a-loan-review [-h] [--version] {rate,gate,myr,review} ...\n";

fn refusal(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_div7a-loan-review"))
        .args(args)
        .env_remove("FORCE_COLOR")
        .env_remove("NO_COLOR")
        .env("PYTHON_COLORS", "0")
        .env("COLUMNS", "80")
        .output()
        .expect("binary runs");
    let stderr = String::from_utf8(out.stderr).expect("UTF-8 stderr");
    (
        out.status.code().expect("exit code"),
        stderr.replace("\r\n", "\n"),
    )
}

fn error(message: &str) -> (i32, String) {
    (2, format!("{USAGE}div7a-loan-review: error: {message}\n"))
}

const CHOICES: &str = "(choose from 'rate', 'gate', 'myr', 'review')";

#[test]
fn no_command_or_a_bare_marker_asks_for_the_command() {
    let required = error("the following arguments are required: command");
    assert_eq!(refusal(&[]), required);
    assert_eq!(refusal(&["--"]), required);
}

#[test]
fn a_bare_long_prefix_with_a_value_is_ambiguous() {
    assert_eq!(
        refusal(&["--=x"]),
        error("ambiguous option: --=x could match --help, --version")
    );
}

#[test]
fn a_lone_dash_is_a_command_name_and_not_a_flag() {
    assert_eq!(
        refusal(&["-"]),
        error(&format!("argument command: invalid choice: '-' {CHOICES}"))
    );
}

#[test]
fn a_name_matching_under_six_tenths_of_every_command_gets_no_suggestion() {
    // difflib ratios: rate 0.5, gate 0.5, myr 0.0, review 0.2.
    assert_eq!(
        refusal(&["xteg"]),
        error(&format!(
            "argument command: invalid choice: 'xteg' {CHOICES}"
        ))
    );
}

#[test]
fn a_name_of_200_or_more_characters_is_matched_with_difflib_s_autojunk() {
    // 250 'e's are popular in a 254-character word, so difflib drops them from
    // its index and extends the 'vi' match over them to 'eview'; the ratio is
    // still far below 0.6.
    let word = format!("{}view", "e".repeat(250));
    assert_eq!(
        refusal(&[&word]),
        error(&format!(
            "argument command: invalid choice: '{word}' {CHOICES}"
        ))
    );
}
