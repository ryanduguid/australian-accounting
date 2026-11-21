// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

mod context;
mod json;
mod score;
mod xml;

use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    time::Duration,
};

const REFERENCE: &[u8] = include_bytes!("../../evaluation/questions.xml");
const HELP: &str = "Offline supplementary tool-selection evaluation. Recordings are never executed.\n\nUsage: aus-accounting-mcp-evaluate [--reference PATH] <context|questions|score RECORDING [--reference-sha256 SHA]>\n\ncontext    Print actual native server instructions, tools and preloaded scope.\nquestions  Print questions without tools or answers.\nscore      Compare recorded calls and answers; an imperfect valid score exits 0.\n\nThe default reference is embedded. Context launches the sibling aus-accounting-mcp executable.\n";

const LICENCE_NOTICE: &str = "\n\
    Copyright 2026 Ryan Duguid. No warranty.\n\
    AGPL-3.0-or-later with additional terms (see NOTICE).\n\
    Licence: https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/LICENSE\n";

async fn blocking_read(path: &Path) -> Result<Vec<u8>, String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || std::fs::read(path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// The sibling server next to this process. The path is the loaded program,
/// so a copy made for a test still finds the server placed beside that copy.
/// The command words are not used, because an operator can set the program
/// name to any text.
fn sibling_server() -> std::io::Result<PathBuf> {
    Ok(executable_path()?.with_file_name(format!(
        "aus-accounting-mcp{}",
        std::env::consts::EXE_SUFFIX
    )))
}

fn executable_path() -> std::io::Result<PathBuf> {
    let read: fn() -> std::io::Result<PathBuf> = std::env::current_exe;
    read()
}

async fn blocking_read_text(path: &Path) -> Result<String, String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || std::fs::read_to_string(path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

fn scoring_args(args: &[OsString]) -> Result<(&Path, Option<&str>), ()> {
    let mut recording = None;
    let mut frozen = None;
    let mut args = args.iter();
    let mut positional = false;
    while let Some(arg) = args.next() {
        if !positional && arg == "--" {
            positional = true;
            continue;
        }
        if !positional && arg == "--reference-sha256" {
            frozen = Some(args.next().and_then(|a| a.to_str()).ok_or(())?);
        } else if !positional
            && let Some(value) = arg
                .to_str()
                .and_then(|s| s.strip_prefix("--reference-sha256="))
        {
            frozen = Some(value);
        } else {
            if !positional && arg.to_str().is_some_and(|s| s.starts_with('-') && s != "-") {
                return Err(());
            }
            if recording.replace(Path::new(arg)).is_some() {
                return Err(());
            }
        }
    }
    Ok((recording.ok_or(())?, frozen))
}

enum Prepared {
    Exit(i32),
    Run {
        command: OsString,
        rest: Vec<OsString>,
        reference: Option<PathBuf>,
    },
}

fn global_flag(command: &OsStr) -> Option<i32> {
    if command == "--help" || command == "-h" {
        print!("{HELP}{LICENCE_NOTICE}");
        Some(0)
    } else if command == "--version" {
        println!("aus-accounting-mcp-evaluate {}", env!("CARGO_PKG_VERSION"));
        Some(0)
    } else {
        None
    }
}

fn subcommand_help(command: &OsStr, rest: &[OsString]) -> bool {
    ["context", "questions", "score"]
        .iter()
        .any(|name| command == *name)
        && rest
            .iter()
            .take_while(|arg| *arg != "--")
            .any(|arg| arg == "--help" || arg == "-h")
}

fn prepare(args: Vec<OsString>) -> Prepared {
    let mut args = args.into_iter();
    let Some(mut command) = args.next() else {
        eprint!("{HELP}");
        return Prepared::Exit(2);
    };
    let reference = if command == "--reference" {
        let Some(path) = args.next() else {
            eprintln!("--reference requires a path");
            return Prepared::Exit(2);
        };
        let path = PathBuf::from(path);
        let Some(next) = args.next() else {
            eprint!("{HELP}");
            return Prepared::Exit(2);
        };
        command = next;
        Some(path)
    } else {
        None
    };
    if let Some(code) = global_flag(&command) {
        return Prepared::Exit(code);
    }
    let rest: Vec<_> = args.collect();
    if subcommand_help(&command, &rest) {
        print!("{HELP}{LICENCE_NOTICE}");
        return Prepared::Exit(0);
    }
    Prepared::Run {
        command,
        rest,
        reference,
    }
}

fn finish(result: Result<(i32, String, String), String>) -> i32 {
    match result {
        Ok((code, out, err)) => {
            print!("{out}");
            eprint!("{err}");
            code
        }
        Err(error) => {
            eprintln!("aus-accounting-mcp-evaluate: {error}");
            1
        }
    }
}

async fn run_context() -> Result<(i32, String, String), String> {
    match sibling_server() {
        Ok(path) => context::describe(&path, Duration::from_secs(30))
            .await
            .map(|s| (0, s, String::new())),
        Err(e) => Err(e.to_string()),
    }
}

async fn run_questions(reference: Option<&Path>) -> Result<(i32, String, String), String> {
    match reference {
        Some(path) => blocking_read(path).await,
        None => Ok(REFERENCE.to_vec()),
    }
    .and_then(|b| score::questions(&b))
    .map(|s| (0, s, String::new()))
}

async fn score_command(rest: &[OsString], reference: Option<&Path>) -> i32 {
    let Ok((recording, frozen)) = scoring_args(rest) else {
        eprint!("{HELP}");
        return 2;
    };
    let text = blocking_read_text(recording).await;
    // A bad recording is reported before a missing custom reference.
    let reference_bytes = match reference {
        Some(path) => Some(blocking_read(path).await),
        None => None,
    };
    finish(score::judge(text, reference_bytes, REFERENCE, frozen))
}

async fn dispatch(command: &OsStr, rest: &[OsString], reference: Option<&Path>) -> i32 {
    if command == "context" && rest.is_empty() {
        finish(run_context().await)
    } else if command == "questions" && rest.is_empty() {
        finish(run_questions(reference).await)
    } else if command == "score" {
        score_command(rest, reference).await
    } else {
        eprint!("{HELP}");
        2
    }
}

pub async fn main(args: Vec<OsString>) -> i32 {
    match prepare(args) {
        Prepared::Exit(code) => code,
        Prepared::Run {
            command,
            rest,
            reference,
        } => dispatch(&command, &rest, reference.as_deref()).await,
    }
}
