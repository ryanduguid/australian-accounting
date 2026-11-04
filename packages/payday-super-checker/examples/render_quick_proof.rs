// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: FSL-1.1-ALv2

//! Rewrites the README's quick proof from the fabricated sample: the console
//! transcript `assets/quick-proof.md` and its card `assets/quick-proof.svg`.
//!
//!     cargo run --example render_quick_proof [-- --check]
//!
//! `--check` lists stale proof files instead of writing them;
//! `tests/quick_proof.rs` holds the committed transcript to the current run.

use payday_super_checker::cli;
use std::path::Path;
use std::process::ExitCode;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const SAMPLE: &str = "examples/sample_payrun_no_transition.csv";
const REQUIRED: [&str; 4] = [
    "ON_TIME: 3  AT_RISK: 1  LATE: 1  UNPAID: 1  UNKNOWN: 0  SKIPPED: 1",
    "shortfall $780.00",
    "experimental estimated SG charge $788.76 - $1262.02",
    "Educational tool, not advice",
];

fn command() -> String {
    format!("payday-super-check {SAMPLE} --as-at 2026-09-10")
}

/// The sample's console output, with the report path as `report.csv`.
fn run_example() -> Result<String, String> {
    std::env::set_current_dir(ROOT).map_err(|e| e.to_string())?;
    // The report is a throwaway copy of fabricated output, and create_dir refuses
    // a path that already exists.
    // nosemgrep: rust.lang.security.temp-dir.temp-dir
    let directory = std::env::temp_dir().join(format!("payday-super-proof-{}", std::process::id()));
    std::fs::create_dir(&directory).map_err(|e| e.to_string())?;
    let report = directory.join("report.csv").to_string_lossy().into_owned();
    let args = [SAMPLE, "--as-at", "2026-09-10", "-o", &report].map(String::from);
    let (code, out, err) = cli::dispatch(&args);
    let _ = std::fs::remove_dir_all(&directory);
    if code != 2 {
        return Err(format!(
            "expected review-required exit code 2, got {code}: {out}{err}"
        ));
    }
    let output = format!("{out}{err}")
        .replace(&report, "report.csv")
        .replace("\r\n", "\n");
    let missing: Vec<&str> = REQUIRED
        .into_iter()
        .filter(|text| !output.contains(text))
        .collect();
    if !missing.is_empty() {
        return Err(format!("the example output changed: missing {missing:?}"));
    }
    Ok(output)
}

fn render_transcript(output: &str) -> String {
    format!(
        "# Fabricated Payday Super review\n\nCommand:\n\n```bash\n{}\n```\n\n\
         Exit code: 2, review required.\n\n```text\n{}\n```\n\n\
         The input is fabricated. No employer or employee data are used.\n",
        command(),
        output.trim_end()
    )
}

/// The README card; `{command}` stands for the sample command.
const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="450" viewBox="0 0 1000 450" role="img" aria-labelledby="title desc">
  <title id="title">Fabricated Payday Super review</title>
  <desc id="desc">A fabricated seven-line review with three on-time lines and three lines requiring attention.</desc>
  <rect width="1000" height="450" rx="20" fill="#000000"/>
  <rect x="34" y="34" width="932" height="382" rx="14" fill="#0A0A0A" stroke="#E5E4E2"/>
  <text x="68" y="84" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="28" font-weight="700">payday-super-checker</text>
  <text x="68" y="116" fill="#C8C8C8" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="15">FABRICATED REVIEW PROOF  |  AS AT 2026-09-10</text>
  <line x1="68" y1="142" x2="932" y2="142" stroke="#606060"/>
  <text x="68" y="181" fill="#E5E4E2" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="15">{command}</text>
  <text x="68" y="232" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="18">ON_TIME  3     AT_RISK  1     LATE  1</text>
  <text x="68" y="270" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="18">UNPAID  1      UNKNOWN  0     SKIPPED  1</text>
  <text x="68" y="320" fill="#E5E4E2" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="17">shortfall</text>
  <text x="240" y="320" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="17">$780.00</text>
  <text x="420" y="320" fill="#E5E4E2" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="17">experimental range</text>
  <text x="672" y="320" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="17">$788.76 - $1,262.02</text>
  <rect x="704" y="354" width="206" height="40" rx="20" fill="#990024" stroke="#E5E4E2"/>
  <text x="807" y="380" text-anchor="middle" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="15" font-weight="700">REVIEW REQUIRED</text>
  <text x="68" y="380" fill="#C8C8C8" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="15">exit 2  |  full transcript linked</text>
</svg>
"##;

fn render_svg() -> String {
    SVG.replace("{command}", &command())
}

fn main() -> ExitCode {
    // Only the --check flag is read; nothing is trusted from argv.
    // nosemgrep: rust.lang.security.args.args
    let check = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [flag] if flag == "--check" => true,
        _ => {
            eprintln!("usage: render_quick_proof [--check]");
            return ExitCode::from(2);
        }
    };
    let output = match run_example() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let expected = [
        ("assets/quick-proof.svg", render_svg()),
        ("assets/quick-proof.md", render_transcript(&output)),
    ];
    if check {
        let stale: Vec<&str> = expected
            .iter()
            .filter(|(path, text)| {
                std::fs::read_to_string(Path::new(ROOT).join(path))
                    .ok()
                    .as_ref()
                    != Some(text)
            })
            .map(|(path, _)| *path)
            .collect();
        for path in &stale {
            println!("stale: {path}");
        }
        if !stale.is_empty() {
            return ExitCode::FAILURE;
        }
        println!("quick proof is current");
        return ExitCode::SUCCESS;
    }
    for (path, text) in &expected {
        if let Err(e) = std::fs::write(Path::new(ROOT).join(path), text) {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
        println!("wrote {path}");
    }
    ExitCode::SUCCESS
}
