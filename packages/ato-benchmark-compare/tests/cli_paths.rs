// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! argparse paths of the command line that the parity corpus does not reach:
//! the `--` marker, explicit values, unknown options and long invalid choices.
use ato_benchmark_compare::cli;

const BAKERY: &str = "Bakeries and hot bread shops";

fn run(args: &[&str]) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    cli::run(&args)
}

#[test]
fn a_lone_marker_leaves_the_command_missing() {
    let (code, out, err) = run(&["--"]);
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(err.contains("error: the following arguments are required: command"));
}

#[test]
fn a_marker_before_the_command_is_dropped() {
    assert_eq!(run(&["--", "buckets"]), run(&["buckets"]));
    assert_eq!(run(&["buckets"]).0, 0);
}

#[test]
fn an_unknown_option_before_the_command_is_refused() {
    let (code, _, err) = run(&["--bogus", "buckets"]);
    assert_eq!(code, 2);
    assert!(
        err.ends_with("error: unrecognized arguments: --bogus\n"),
        "{err}"
    );
}

#[test]
fn help_and_version_refuse_an_explicit_value() {
    let (code, _, err) = run(&["--help=x"]);
    assert_eq!(code, 2);
    assert!(
        err.contains("error: argument -h/--help: ignored explicit argument 'x'"),
        "{err}"
    );
    let (code, _, err) = run(&["--version=1"]);
    assert_eq!(code, 2);
    assert!(
        err.contains("error: argument --version: ignored explicit argument '1'"),
        "{err}"
    );
}

#[test]
fn a_flag_refuses_an_explicit_value() {
    let (code, _, err) = run(&[
        "map",
        "--force=yes",
        "--profit-and-loss",
        "p.csv",
        "--out",
        "m.csv",
    ]);
    assert_eq!(code, 2);
    assert!(
        err.contains("error: argument --force: ignored explicit argument 'yes'"),
        "{err}"
    );
}

#[test]
fn a_trailing_marker_after_the_industry_changes_nothing() {
    let plain = run(&["show", BAKERY]);
    assert_eq!(plain.0, 0, "{}", plain.2);
    assert_eq!(run(&["show", BAKERY, "--"]), plain);
}

#[test]
fn an_explicit_option_value_reads_like_a_separate_one() {
    let joined = run(&["show", BAKERY, "--year=2023-24"]);
    assert_eq!(joined.0, 0, "{}", joined.2);
    assert_eq!(joined, run(&["show", BAKERY, "--year", "2023-24"]));
}

#[test]
fn a_stray_word_before_an_option_is_unrecognised() {
    let (code, _, err) = run(&["industries", "stray", "--search", "bak"]);
    assert_eq!(code, 2);
    assert!(
        err.ends_with("error: unrecognized arguments: stray\n"),
        "{err}"
    );
}

#[test]
fn a_long_invalid_command_gets_no_suggestion() {
    // 200 or more characters trigger difflib's autojunk, which drops the
    // popular 'a' from the index and then extends matches over it.
    for word in [
        format!("map{}", "a".repeat(300)),
        format!("xap{}", "a".repeat(300)),
    ] {
        let (code, _, err) = run(&[&word]);
        assert_eq!(code, 2);
        assert!(
            err.contains(&format!("argument command: invalid choice: '{word}'")),
            "{err}"
        );
        assert!(!err.contains("maybe you meant"), "{err}");
    }
}

#[test]
fn a_near_miss_command_gets_a_suggestion() {
    let (code, _, err) = run(&["compar"]);
    assert_eq!(code, 2);
    assert!(err.contains("maybe you meant 'compare'?"), "{err}");
}

#[cfg(unix)]
#[test]
fn an_undecodable_argument_byte_is_written_as_its_surrogate_escape() {
    use std::os::unix::ffi::OsStrExt;
    let text = cli::decode_os(std::ffi::OsStr::from_bytes(b"a\xffb"));
    assert_eq!(cli::output_bytes(&text), b"a\\udcffb".to_vec());
}

#[test]
fn a_transposed_command_still_gets_a_suggestion() {
    // difflib.get_close_matches('campore', [...]) is ['compare']; matching it
    // stops scanning at a 'p' past the block being searched.
    let (code, _, err) = run(&["campore"]);
    assert_eq!(code, 2);
    assert!(err.contains("maybe you meant 'compare'?"), "{err}");
}

/// `compare` on the bakery example without its interest row, as JSON.
fn bakery_without_interest(mapping: &str, confirm: bool) -> String {
    let dir = std::env::temp_dir().join(format!(
        "ato-confirm-{}-{}-{confirm}",
        mapping.len(),
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let (pnl, reviewed) = (dir.join("pnl.csv"), dir.join("mapping.csv"));
    let source =
        include_str!("../examples/bakery-pnl.csv").replace("Interest received,1200.00,900.00", "");
    std::fs::write(&pnl, source).unwrap();
    std::fs::write(&reviewed, mapping).unwrap();
    let (pnl, reviewed) = (pnl.to_str().unwrap(), reviewed.to_str().unwrap());
    let mut args = vec![
        "compare",
        "--profit-and-loss",
        pnl,
        "--mapping",
        reviewed,
        "--industry",
        BAKERY,
        "--amount-column",
        "1",
        "--json",
        "-",
    ];
    if confirm {
        args.push("--confirm-other-income-nil");
    }
    let (code, out, err) = run(&args);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(code, 0, "{err}");
    out
}

#[test]
fn a_mapped_other_income_account_missing_from_the_export_blocks_the_nil_confirmation() {
    let mapping = include_str!("../examples/bakery-mapping.csv");
    assert!(mapping.contains("Interest received,other_income,"));
    // The mapping still routes Interest received to other income, so a nil
    // confirmation cannot stand in for it.
    assert_eq!(
        bakery_without_interest(mapping, true),
        bakery_without_interest(mapping, false)
    );
    // With that row gone from the mapping too, the confirmation counts.
    let without: String = mapping
        .lines()
        .filter(|l| !l.starts_with("Interest received,"))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_ne!(
        bakery_without_interest(&without, true),
        bakery_without_interest(&without, false)
    );
}
