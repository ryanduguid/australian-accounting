// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `snapshot` against `rates.py` (24fcf24, the differential reference) under
//! CPython 3.15.0, with the browser replaced: `tests/data/cov_a6_snapshot.py`
//! printed the expected output, exit status and files.

use super::*;

/// A new folder under the crate's `target/` for one test.
fn folder(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("cov-a6-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(path.join("data")).expect("test folder");
    path
}

/// The package with one record for each URL, its snapshots in `folder`.
fn package(folder: &Path, urls: &[&str]) -> Ctx {
    for (i, url) in urls.iter().enumerate() {
        let record = format!(r#"{{"id": "r{i}", "source_url": "{url}"}}"#);
        std::fs::write(folder.join("data").join(format!("record-{i}.json")), record)
            .expect("write record");
    }
    let real = Ctx::new();
    Ctx {
        data: PyPath::new(&folder.join("data").to_string_lossy()),
        snapshots: PyPath::new(&folder.join("snapshots").to_string_lossy()),
        root: real.root,
    }
}

fn newlines_of(text: &str) -> String {
    if cfg!(windows) {
        text.replace('\n', "\r\n")
    } else {
        text.to_string()
    }
}

fn snapshot(ctx: &Ctx, fetch: &mut dyn FnMut(&Json) -> R<S>, pause: bool) -> (R<i32>, String) {
    let mut out = Out::new(Vec::new());
    let mut world = World {
        fetch,
        pause,
        get: &mut |_, _| panic!("a table read"),
    };
    let code = cmd_snapshot(ctx, &mut out, &mut world);
    (code, String::from_utf8(out.w).expect("UTF-8 output"))
}

#[test]
fn snapshot_saves_each_page_and_reports_each_failure_as_rates_py_does() {
    let root = folder("saves");
    let ctx = package(
        &root,
        &[
            "https://www.ato.gov.au/b-page",
            "https://www.ato.gov.au/a-page/",
        ],
    );
    let mut fetched = Vec::new();
    let (code, printed) = snapshot(
        &ctx,
        &mut |url| {
            fetched.push(url.as_str().expect("a str URL").to_string());
            match url.as_str() {
                Some("https://www.ato.gov.au/a-page/") => {
                    Err(Exc::msg("RuntimeError", "browser closed"))
                }
                _ => Ok(pts("Effective date: 1 July 2026\nend\n")),
            }
        },
        false,
    );
    assert_eq!(code.ok(), Some(1));
    assert_eq!(
        fetched,
        [
            "https://www.ato.gov.au/a-page/",
            "https://www.ato.gov.au/b-page"
        ]
    );
    assert_eq!(
        printed,
        newlines_of("FAILED https://www.ato.gov.au/a-page/: browser closed\nsaved b-page.txt\n")
    );
    let names: Vec<String> = std::fs::read_dir(root.join("snapshots"))
        .expect("the snapshots folder")
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names, ["b-page.txt"]);
    let saved = std::fs::read_to_string(root.join("snapshots").join("b-page.txt")).expect("saved");
    assert_eq!(saved, newlines_of("Effective date: 1 July 2026\nend\n"));
    let _ = std::fs::remove_dir_all(&root);
}

fn text_of(points: &[u32]) -> String {
    points.iter().filter_map(|&c| char::from_u32(c)).collect()
}

// The expected values below are what CPython 3.15.0 printed for
// `"a\r\nb\rc\n".splitlines()`, `Decimal(1) / Decimal(0)` and
// `int(Decimal("NaN"))` (also `int(Decimal("-sNaN"))`).

#[test]
fn splitlines_takes_a_crlf_pair_as_one_break() {
    let text = pts("a\r\nb\rc\n");
    let lines: Vec<String> = splitlines(&text).into_iter().map(text_of).collect();
    assert_eq!(lines, ["a", "b", "c"]);
}

#[test]
fn division_by_zero_is_worded_as_decimal_words_it() {
    let e = decimal_signal(Signal::DivisionByZero);
    assert_eq!(e.class, "decimal.DivisionByZero");
    assert_eq!(text_of(&e.text), "[<class 'decimal.DivisionByZero'>]");
}

#[test]
fn int_of_a_nan_decimal_is_a_value_error() {
    for nan in ["NaN", "-sNaN"] {
        let value = Decimal::parse_exact(nan).ok().expect("a NaN");
        let Err(e) = decimal_int(&value) else {
            panic!("int({nan}) succeeded")
        };
        assert_eq!(e.class, "ValueError");
        assert_eq!(text_of(&e.text), "cannot convert NaN to integer");
    }
}

#[test]
fn snapshot_pauses_two_seconds_after_each_page() {
    let root = folder("pause");
    let ctx = package(&root, &["https://www.ato.gov.au/c-page"]);
    let started = Instant::now();
    let (code, printed) = snapshot(&ctx, &mut |_| Ok(pts("page")), true);
    assert!(started.elapsed() >= Duration::from_secs(2));
    assert_eq!(code.ok(), Some(0));
    assert_eq!(printed, newlines_of("saved c-page.txt\n"));
    let _ = std::fs::remove_dir_all(&root);
}
