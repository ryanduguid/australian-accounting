//! The answers CPython 3.15's argparse gave for the `rates` command line
//! (`tests/data/cov_b_argparse.py` records them) and a runner for the built binary.
#![allow(dead_code)]

use au_tax_rates_data::pyjson::{Json, loads};
use std::process::Command;

/// The variables argparse's width and colour read; each case sets only its own.
const TERMINAL: [&str; 6] = [
    "COLUMNS",
    "LINES",
    "PYTHON_COLORS",
    "NO_COLOR",
    "FORCE_COLOR",
    "TERM",
];

pub struct Case {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn text(value: &Json) -> String {
    value.as_str().expect("a string").to_string()
}

/// The recorded case called `name`.
pub fn case(name: &str) -> Case {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/cov_b_argparse-python.json"
    );
    let data = std::fs::read_to_string(path).expect("read the recorded answers");
    let all = loads(&data).unwrap_or_else(|_| panic!("the recorded answers are JSON"));
    let case = all.get(name).unwrap_or_else(|| panic!("no case {name}"));
    let Json::Array(args) = case.at("args") else {
        panic!("args is a list")
    };
    let Json::Object(env) = case.at("env") else {
        panic!("env is an object")
    };
    let Json::Int(code) = case.at("code") else {
        panic!("code is an integer")
    };
    Case {
        args: args.iter().map(text).collect(),
        env: env
            .iter()
            .map(|(k, v)| (k.as_str().expect("a key").to_string(), text(v)))
            .collect(),
        code: i32::try_from(code).expect("a small exit code"),
        stdout: text(case.at("stdout")),
        stderr: text(case.at("stderr")),
    }
}

/// What the built `rates` printed for the case: (status, stdout, stderr), with CRLF read as LF.
pub fn run(case: &Case) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rates"));
    command.args(&case.args);
    for name in TERMINAL {
        command.env_remove(name);
    }
    command.envs(case.env.iter().map(|(k, v)| (k, v)));
    let done = command.output().expect("run rates");
    let lf = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .expect("UTF-8")
            .replace("\r\n", "\n")
    };
    (
        done.status.code().expect("an exit status"),
        lf(done.stdout),
        lf(done.stderr),
    )
}

/// `rates` prints argparse's help, then its licence notice.
pub const NOTICE: &str = "\nCopyright 2026 Ryan Duguid. No warranty.\n";

/// Run the case and check the binary answers as Python did: the same status and stderr, and
/// for help the same text followed by the licence notice.
pub fn same_as_python(name: &str) {
    let case = case(name);
    let (code, stdout, stderr) = run(&case);
    assert_eq!(code, case.code, "{name}: status");
    assert_eq!(stderr, case.stderr, "{name}: stderr");
    if case.code == 0 {
        let rest = stdout
            .strip_prefix(&case.stdout)
            .unwrap_or_else(|| panic!("{name}: help\n{stdout}\nexpected\n{}", case.stdout));
        assert!(rest.starts_with(NOTICE), "{name}: notice {rest:?}");
    } else {
        assert_eq!(stdout, "", "{name}: stdout");
    }
}
