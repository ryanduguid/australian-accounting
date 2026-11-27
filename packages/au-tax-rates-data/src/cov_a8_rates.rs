// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Snapshot names, record stems, the record listing, `write_text` failures,
//! subprocess edges and ATO-table error paths: each answer is what CPython
//! 3.15.0 printed for the same input (`tests/data/cov_a8_rates.py`).

use super::*;

/// A new, empty folder under the crate's `target/` for one test.
fn folder(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("cov-a8-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test folder");
    path
}

fn exc_pair(e: Exc) -> (&'static str, String) {
    (e.class, pyvalue::points_lossy(&e.text))
}

/// `urlparse(url).path` drops `;params` only after the last slash, and only
/// for a scheme that takes them.
#[test]
fn snapshot_names_drop_params_as_urlparse_does() {
    let ctx = Ctx::new();
    for (url, name) in [
        ("http:a;b", "a.txt"),
        ("https://h/x;p/y;q", "y.txt"),
        ("https://h/x;p/", "x;p.txt"),
    ] {
        let snap = snapshot_path(&ctx, &pts(url)).expect("snapshot path");
        assert_eq!(snap.name, pts(name), "{url}");
    }
}

#[test]
fn record_stems_follow_pathlib() {
    for (name, stem) in [
        ("r0.json", "r0"),
        (".json", ".json"),
        ("a.b.json", "a.b"),
        ("..json", "..json"),
    ] {
        let entry = Entry {
            name: pts(name),
            path: PathBuf::from(name),
            display: name.to_string(),
        };
        assert_eq!(entry.stem(), pts(stem), "{name}");
    }
}

/// `sorted(DATA.glob("*.json"))` skips other names and keeps hidden ones.
#[test]
fn the_record_listing_keeps_only_json_names_in_order() {
    let dir = folder("glob");
    for name in ["b.json", "a.json", "notes.txt", ".h.json", "x.json.bak"] {
        std::fs::write(dir.join(name), "{}").unwrap();
    }
    let entries = glob_json(&PyPath::new(&dir.to_string_lossy())).expect("listing");
    let names: Vec<S> = entries.into_iter().map(|e| e.name).collect();
    assert_eq!(names, [pts(".h.json"), pts("a.json"), pts("b.json")]);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn write_text_fails_as_path_write_text_does() {
    let dir = folder("write");
    let missing = dir.join("no-folder").join("x.txt");
    let display = pts("T/no-folder/x.txt");
    let e = write_text(Some(&missing), &display, &pts("x")).unwrap_err();
    assert_eq!(
        exc_pair(e),
        (
            "FileNotFoundError",
            "[Errno 2] No such file or directory: 'T/no-folder/x.txt'".to_string()
        )
    );
    // A name a POSIX path cannot hold has no file to write.
    let lone = p!("/r/a", vec![0xD800], ".txt");
    let e = write_text(None, &lone, &pts("x")).unwrap_err();
    assert_eq!(
        exc_pair(e),
        (
            "UnicodeEncodeError",
            "'utf-8' codec can't encode character '\\ud800' in position 4: surrogates not allowed"
                .to_string()
        )
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn sorted_set_decimal_and_json_edges() {
    let items = sorted_set(vec![Json::Float(f64::NAN), Json::Int(1.into())]).unwrap();
    assert_eq!(items.len(), 2);
    let huge = Decimal::parse_exact("1e200000000").ok().unwrap();
    assert_eq!(
        exc_pair(grouped(&huge, 0).unwrap_err()),
        ("MemoryError", String::new())
    );
    let dir = folder("missing-glob");
    let missing = PyPath::new(&dir.join("absent").to_string_lossy());
    assert_eq!(
        glob_json(&missing).err().unwrap().class,
        "FileNotFoundError"
    );
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(
        exc_pair(json_error(pyjson::DecodeError::Duplicate)),
        (
            "json.decoder.JSONDecodeError",
            "duplicate member".to_string()
        )
    );
    assert!(json_scale_rows(&Json::Null).unwrap().is_empty());
    assert_eq!(
        exc_pair(parse_text(&Json::str("unknown"), &pts("10")).err().unwrap()),
        ("KeyError", "'unknown'".to_string())
    );
}

#[test]
fn write_text_and_digest_refuse_lone_surrogates() {
    let dir = folder("surr");
    let file = dir.join("f.txt");
    let body = p!("a", vec![0xD800], "b");
    let want = (
        "UnicodeEncodeError",
        "'utf-8' codec can't encode character '\\ud800' in position 1: surrogates not allowed"
            .to_string(),
    );
    assert_eq!(
        exc_pair(write_text(Some(&file), &pts("f.txt"), &body).unwrap_err()),
        want
    );
    assert_eq!(std::fs::read(&file).unwrap(), b"");
    assert_eq!(exc_pair(digest(&body).unwrap_err()), want);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn browser_timeout_signal_and_none_streams() {
    let sleep_cmd: Vec<Json> = if cfg!(windows) {
        ["cmd", "/c", "ping", "-n", "7", "127.0.0.1"]
            .map(Json::str)
            .to_vec()
    } else {
        ["sh", "-c", "sleep 6"].map(Json::str).to_vec()
    };
    let err = run_with_limit(&sleep_cmd, Duration::from_millis(100)).unwrap_err();
    assert_eq!(err.class, "subprocess.TimeoutExpired");
    assert!(pyvalue::points_lossy(&err.text).ends_with("' timed out after 0 seconds"));
    #[cfg(unix)]
    {
        let cmd = ["/bin/sh", "-c", "kill -9 $$"].map(Json::str);
        let (code, _, _) = run_with_limit(&cmd, Duration::from_secs(5)).unwrap();
        assert_eq!(code, -9);
    }
    #[cfg(windows)]
    {
        let cmd = [
            "powershell",
            "-NoProfile",
            "-Command",
            "[Console]::OpenStandardOutput().WriteByte(255)",
        ]
        .map(Json::str);
        let (code, stdout, stderr) = run_with_limit(&cmd, Duration::from_secs(10)).unwrap();
        assert_eq!(code, 0);
        assert!(stdout.is_none());
        assert!(stderr.is_some());
    }
    assert_eq!(
        exc_pair(read_output(2, None, None).unwrap_err()),
        (
            "AttributeError",
            "'NoneType' object has no attribute 'strip'".to_string()
        )
    );
    assert_eq!(
        exc_pair(read_output(0, None, Some("")).unwrap_err()),
        (
            "TypeError",
            "the JSON object must be str, bytes or bytearray, not NoneType".to_string()
        )
    );
}

#[test]
fn report_missing_url_and_duplicate_column_edges() {
    let ctx = Ctx::new();
    let rec = Json::obj([
        ("id", Json::str("r0")),
        ("source_url", Json::str("https://www.ato.gov.au/x")),
    ]);
    assert_eq!(
        exc_pair(build_report(&ctx, &[rec], &[], "2026-11-28").unwrap_err()),
        ("KeyError", "'https://www.ato.gov.au/x'".to_string())
    );
    let table = loads(
        r#"{"columns":[
            {"name":"DT_EFFECT"},{"name":"DT_END"},{"name":"TX_DESC"},
            {"name":"AM_INC_MIN"},{"name":"AM_INC_MAX"},{"name":"AM_OFFSET"},
            {"name":"PC_TAX_RATE"},{"name":"PC_TAX_RATE"}
        ],"rows":[["2025-07-01","9999-12-31","Resident Full Year",0,18200,0,"0.10","0.30"]]}"#,
    )
    .unwrap();
    let brackets = ato_brackets(
        &table,
        "TC2TAXRTE",
        ("TX_DESC", "Resident Full Year"),
        &Json::str("2026-07-01"),
    )
    .unwrap();
    assert_eq!(brackets[0].3.fixed_format(), "30.00");
    assert_eq!(
        exc_pair(read_figure(Reader::SecondTop, brackets).err().unwrap()),
        ("IndexError", "list index out of range".to_string())
    );
}

#[test]
fn ato_table_report_invalid_period_and_id_edges() {
    assert!(figure_eq(&Figure::None, &Figure::None).unwrap());
    let bad_period =
        loads(r#"{"id":"r1","label":"Medicare levy rate","period":{"income_year":{}}}"#).unwrap();
    let (lines, findings) = ato_table_report(&[bad_period], &[]).unwrap();
    assert_eq!(findings, 1);
    assert!(pyvalue::points_lossy(&lines[0]).contains("slice(None, 4, None)"));
    let int_id = loads(r#"{"id":42,"label":"not-in-ato-tables"}"#).unwrap();
    assert_eq!(
        exc_pair(ato_table_report(&[int_id], &[]).unwrap_err()),
        (
            "TypeError",
            "sequence item 0: expected str instance, int found".to_string()
        )
    );
}

fn empty_records() -> R<Vec<Json>> {
    Ok(Vec::new())
}

#[test]
fn table_decode_and_help_flush_errors() {
    assert_eq!(
        exc_pair(read_table("t", Ok((200, vec![0xFF]))).unwrap_err()),
        (
            "UnicodeDecodeError",
            "'utf-8' codec can't decode byte 0xff in position 0: invalid start byte".to_string()
        )
    );
    assert_eq!(
        bytes_decode_error(b"\xed\xa0\x80ok"),
        "'utf-8' codec can't decode"
    );
    let mut out = Out::new(Vec::new());
    let empty_table = br#"{"columns":[],"rows":[]}"#.to_vec();
    let ok_code = ato_tables_with(
        &mut out,
        |_, _| Ok((200, empty_table.clone())),
        empty_records,
    )
    .unwrap();
    assert_eq!(ok_code, 0);
    let err = ato_tables_with(
        &mut out,
        |_, _| Ok((200, vec![b'['; 70_000])),
        empty_records,
    )
    .unwrap_err();
    assert_eq!(err.class, "RecursionError");
    struct FailFlush;
    impl Write for FailFlush {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::from_raw_os_error(28))
        }
    }
    let ctx = Ctx::new();
    let world = World {
        fetch: &mut fetch_text,
        pause: false,
        get: &mut get_table,
    };
    let argv = [OsString::from("--help")];
    assert_eq!(run(&ctx, &argv, world, FailFlush), 120);
}
