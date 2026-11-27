//! The `lodgeit-adapter` binary passes its arguments to `cli::main` and exits
//! with its code: 0 for help, 2 for a usage error, as argparse exits.
use std::process::Command;

fn adapter(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lodgeit-adapter"))
        .args(args)
        .env_remove("LODGEIT_ADAPTER_ENABLED")
        .output()
        .expect("run the adapter")
}

#[test]
fn help_exits_zero_with_the_usage_on_stdout() {
    let out = adapter(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("usage: "));
}

#[test]
fn a_missing_command_is_a_usage_error() {
    // argparse reports the missing subcommand before the unknown option.
    let out = adapter(&["--no-such-option"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("usage: lodgeit-adapter "), "{stderr}");
    assert!(
        stderr.ends_with("lodgeit-adapter: error: the following arguments are required: command\n"),
        "{stderr}"
    );
}
