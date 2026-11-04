// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! No tracked file may carry a checksum-valid Australian identifier unless
//! `synthetic-identifiers.txt` lists it for that file. The screen runs evatt's
//! own detectors and check digits over every file `git ls-files` lists.
//! Findings name the file, line and kind, never the value, so a CI log never
//! repeats one. Adapted from accounting-review-pipeline's
//! `packages/evatt/tests/fixture_screen.rs` at 40aac54, which screens fixture
//! directories only.
use evatt::{patterns, pytext, redact::Lines};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The allowlist, from the repository root. It names the values it allows,
/// so it is not screened itself.
const ALLOWLIST: &str = "tools/fixture-screen/tests/synthetic-identifiers.txt";

/// Kinds with a check digit. BSB, email and phone have none, so a match
/// cannot separate a real value from a fabricated one.
const CHECKED: [&str; 4] = ["tfn", "abn", "acn", "medicare"];

/// Each finding's place, kind and digits. A place is a line, or a byte offset
/// in a binary file, after the name of any zip part holding it.
type Findings = BTreeSet<(String, String, String)>;

/// An allowlist entry's path, kind and digits.
type Entry = (String, String, String);

/// A text the screen reads from a file, with its findings' place prefix and
/// whether a place is a byte offset rather than a line.
struct Piece {
    prefix: String,
    binary: bool,
    text: String,
}

/// The texts a tracked file holds, each screened on its own because evatt
/// joins digit groups across line breaks: UTF-8 text whole, each part of a zip
/// archive such as an xlsx workbook, and otherwise the printable ASCII with
/// every other byte read as NUL, which evatt never joins across and which
/// keeps offsets in bytes. A DER certificate keeps its subject strings there.
/// shortcut: UTF-16 text and compressed streams outside a zip archive are not
/// decoded, so their identifiers are not screened; decode one if it is ever
/// committed.
fn pieces(prefix: &str, bytes: &[u8]) -> Result<Vec<Piece>, String> {
    if !bytes.starts_with(b"PK\x03\x04") {
        let piece = match std::str::from_utf8(bytes) {
            Ok(text) => Piece {
                prefix: prefix.into(),
                binary: false,
                text: text.into(),
            },
            Err(_) => Piece {
                prefix: format!("{prefix}byte "),
                binary: true,
                text: bytes
                    .iter()
                    .map(|&b| {
                        if b == b' ' || b.is_ascii_graphic() {
                            b as char
                        } else {
                            '\0'
                        }
                    })
                    .collect(),
            },
        };
        return Ok(vec![piece]);
    }
    let unreadable = |error| format!("{prefix}unreadable zip archive: {error}");
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(unreadable)?;
    let mut found = Vec::new();
    for index in 0..archive.len() {
        let mut part = archive.by_index(index).map_err(unreadable)?;
        let name = format!("{prefix}{}:", part.name().map_err(unreadable)?);
        let mut content = Vec::new();
        part.read_to_end(&mut content)
            .map_err(|error| format!("{name} unreadable: {error}"))?;
        found.extend(pieces(&name, &content)?);
    }
    Ok(found)
}

/// The shortest hexadecimal token read as a digest, float bits or encoded
/// bytes: a 64-bit hash or float is 16 characters, a SHA-256 64, and the
/// recorded corpora hex-encode their CLI input.
const HEX_TOKEN: usize = 16;

/// Whether `start..end` lies in a run of at least [`HEX_TOKEN`] hexadecimal
/// characters or a `0x` literal such as FNV's `0x100000001b3`, where a
/// check-digit match is chance. evatt never matches inside an all-digit run,
/// so such a token always holds a letter. A separator such as a space, colon
/// or quote ends the token, so a value beside a digest is still screened.
/// shortcut: an identifier hex-encoded in a recording is not decoded, so it is
/// not screened; decode the recorded streams if one ever needs to be.
fn in_hex_token(text: &str, start: usize, end: usize) -> bool {
    let bytes = text.as_bytes();
    if !bytes[start..end].iter().all(u8::is_ascii_hexdigit) {
        return false;
    }
    let from = bytes[..start]
        .iter()
        .rposition(|b| !b.is_ascii_hexdigit())
        .map_or(0, |i| i + 1);
    let to = bytes[end..]
        .iter()
        .position(|b| !b.is_ascii_hexdigit())
        .map_or(bytes.len(), |i| end + i);
    to - from >= HEX_TOKEN || bytes[..from].ends_with(b"0x")
}

/// Whether the digit group `start..end` belongs to a hexadecimal token, is a
/// float's exponent such as the `123` of `8.51e+123`, or is a percentage.
/// evatt itself reads a run beside a decimal point as part of an amount.
fn in_number_group(text: &str, start: usize, end: usize) -> bool {
    let bytes = text.as_bytes();
    let exponent = start >= 3
        && matches!(bytes[start - 1], b'+' | b'-')
        && matches!(bytes[start - 2], b'e' | b'E')
        && bytes[start - 3].is_ascii_digit();
    in_hex_token(text, start, end) || exponent || bytes.get(end) == Some(&b'%')
}

/// Whether the span's first and last digit groups both belong to numbers.
/// evatt joins groups across spaces and line breaks, so a float's exponent can
/// run into the next line's hex bits.
fn in_number(text: &str, start: usize, end: usize) -> bool {
    let span = &text.as_bytes()[start..end];
    let first_end = span
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map_or(end, |i| start + i);
    let last_start = span
        .iter()
        .rposition(|b| !b.is_ascii_digit())
        .map_or(start, |i| start + i + 1);
    in_number_group(text, start, first_end) && in_number_group(text, last_start, end)
}

/// Each checksum-valid identifier evatt detects outside numbers, other than an
/// all-zero placeholder, at its line, or at its offset in a binary piece.
fn identifiers(text: &str, binary: bool) -> patterns::Result<BTreeSet<(usize, String, String)>> {
    // evatt skips a bare number that a comma joins to another digit, as part of
    // a longer number. That hides a CSV or JSON value beside a numeric field,
    // so the screen reads each comma as a field break.
    let text = pytext::nfc(&text.replace("\r\n", "\n").replace(',', ";"));
    let lines = Lines::new(&text);
    let place = |offset| {
        if binary {
            offset
        } else {
            lines.locate(offset).0
        }
    };
    let mut found = BTreeSet::new();
    for span in patterns::structured_spans(&text)? {
        if !CHECKED.contains(&span.kind.as_str()) || in_number(&text, span.start, span.end) {
            continue;
        }
        // A Medicare span can carry the individual reference number after its 10 checked digits.
        let width = if span.kind == "medicare" {
            10
        } else {
            usize::MAX
        };
        let digits: String = patterns::digits(&span.value).chars().take(width).collect();
        if !digits.bytes().all(|b| b == b'0') && patterns::valid(&span.kind, &digits)? {
            found.insert((place(span.start), span.kind, digits));
        }
    }
    // structured_spans leaves a number that runs into a list marker for triage.
    for a in patterns::ambiguous(&text)? {
        let digits = patterns::digits(&a.value);
        for kind in CHECKED {
            if patterns::valid(kind, &digits)? {
                found.insert((place(a.digit_start), kind.into(), digits));
                break;
            }
        }
    }
    Ok(found)
}

fn findings(path: &str, bytes: &[u8]) -> Result<Findings, String> {
    let mut found = Findings::new();
    for piece in pieces("", bytes).map_err(|error| format!("{path}: {error}"))? {
        let hits =
            identifiers(&piece.text, piece.binary).map_err(|error| format!("{path}: {error}"))?;
        found.extend(
            hits.into_iter()
                .map(|(place, kind, digits)| (format!("{}{place}", piece.prefix), kind, digits)),
        );
    }
    Ok(found)
}

/// Each entry's path, kind and digits with its line number. A malformed line
/// is reported by number alone, so a CI log never repeats its digits.
fn allowlist(text: &str) -> (HashMap<Entry, usize>, Vec<String>) {
    let mut entries = HashMap::new();
    let mut malformed = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.splitn(4, ' ').collect::<Vec<_>>()[..] {
            [path, kind, digits, reason]
                if CHECKED.contains(&kind)
                    && !digits.is_empty()
                    && digits.chars().all(char::is_numeric)
                    && !reason.trim().is_empty() =>
            {
                entries.insert((path.into(), kind.into(), digits.into()), index + 1);
            }
            _ => malformed.push(format!(
                "{ALLOWLIST}:{}: expected 'path kind digits reason'",
                index + 1
            )),
        }
    }
    (entries, malformed)
}

/// Every failure: findings no entry names for their file, files that could
/// not be screened, and malformed or stale entries.
fn verdict(results: Vec<(String, Result<Findings, String>)>, allowlist_text: &str) -> Vec<String> {
    let (entries, mut failures) = allowlist(allowlist_text);
    let mut used = HashSet::new();
    for (path, result) in results {
        match result {
            Err(error) => failures.push(error),
            Ok(found) => {
                for (place, kind, digits) in found {
                    let key = (path.clone(), kind, digits);
                    if entries.contains_key(&key) {
                        used.insert(key);
                    } else {
                        failures.push(format!("{path}:{place}: checksum-valid {}", key.1));
                    }
                }
            }
        }
    }
    let mut stale: Vec<_> = entries
        .iter()
        .filter(|(key, _)| !used.contains(*key))
        .map(|(_, &n)| n)
        .collect();
    stale.sort_unstable();
    failures.extend(
        stale
            .iter()
            .map(|n| format!("{ALLOWLIST}:{n}: entry matches no finding")),
    );
    failures
}

fn git(dir: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git runs");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "git {args:?} failed: {stderr}");
    output.stdout
}

fn tracked_files() -> (PathBuf, Vec<String>) {
    let top = git(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &["rev-parse", "--show-toplevel"],
    );
    let root = PathBuf::from(
        String::from_utf8(top)
            .expect("UTF-8 checkout path")
            .trim_end(),
    );
    let paths = git(&root, &["ls-files", "-z"])
        .split(|&b| b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8(p.to_vec()).expect("tracked paths are UTF-8"))
        .filter(|p| p != ALLOWLIST)
        .collect();
    (root, paths)
}

#[test]
fn tracked_files_carry_no_undocumented_identifier() {
    let (root, paths) = tracked_files();
    assert!(
        !paths.is_empty(),
        "no tracked files found under {}",
        root.display()
    );
    // One thread per file: in a debug build the largest recordings take most of a minute each.
    let results = std::thread::scope(|scope| {
        let workers: Vec<_> = paths
            .iter()
            .map(|path| {
                let root = &root;
                scope.spawn(move || {
                    let bytes = std::fs::read(root.join(path))
                        .map_err(|error| format!("{path}: unreadable: {error}"));
                    (path.clone(), bytes.and_then(|bytes| findings(path, &bytes)))
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|w| w.join().expect("screen thread"))
            .collect()
    });
    let failures = verdict(results, include_str!("synthetic-identifiers.txt"));
    assert!(
        failures.is_empty(),
        "{} failures in {} screened files. Replace a real value, or list a synthetic one for its file in \
         {ALLOWLIST}:\n{}",
        failures.len(),
        paths.len(),
        failures.join("\n")
    );
}

fn check(path: &str, bytes: &[u8], allowlist_text: &str) -> Vec<String> {
    verdict(vec![(path.into(), findings(path, bytes))], allowlist_text)
}

#[test]
fn a_checksum_valid_run_inside_a_digest_hex_bytes_or_an_amount_is_not_a_finding() {
    let digest =
        "\"sha256\": \"7f6401c3b7abba89d0f40cd076101fe41620c59ba123456782fc4e09871d4f63\"\n";
    assert!(check("p.json", digest.as_bytes(), "").is_empty());
    // Hex-encoded CSV bytes, as the recorded CLI corpora hold them.
    let stdin = "\"stdin_hex\": \"6d617267696e5f61745f636f6d706c6574696f6e0a123456782c\"\n";
    assert!(check("p.json", stdin.as_bytes(), "").is_empty());
    assert!(check("p.json", b"\"inclusive\": \"123456782.70\"\n", "").is_empty());
    assert!(check("p.txt", b"12210.123456782\n", "").is_empty());
    assert!(check("p.txt", b"40c7d9a123456782 12210.89\n", "").is_empty());
    // A float's exponent joined to the next line's hex bits, a percentage, zeros
    // and a hex literal.
    assert!(check("p.txt", b"8.51e+123\n456782f100000000 1\n", "").is_empty());
    assert!(check("p.json", b"\"ok\": \"-123456782%\"\n", "").is_empty());
    assert!(check("p.json", b"\"abn_or_acn\": \"000000000\"\n", "").is_empty());
    assert!(check("p.rs", b"h.wrapping_mul(0x100000001b3);\n", "").is_empty());
    // A short token beside letters, a value beside a digest or with only its
    // first group in one, and a value ending a sentence stay findings.
    for text in [
        "\"ref\": \"ab123456782\"\n",
        "7f6401c3b7abba89d0f40cd076101fe4 123456782\n",
        "7f6401c3b7abba89d0f40cd076101fe123 456 782\n",
        "Their TFN is 123456782.\n",
    ] {
        assert_eq!(
            check("p.txt", text.as_bytes(), ""),
            ["p.txt:1: checksum-valid tfn"],
            "{text}"
        );
    }
}

#[test]
fn a_planted_checksum_valid_tfn_fails_and_an_invalid_one_passes() {
    let planted = b"Client: Sample Holdings Pty Ltd\nTFN: 123 456 782\n";
    assert_eq!(check("p.md", planted, ""), ["p.md:2: checksum-valid tfn"]);
    assert_eq!(
        check("p.csv", b"id,tfn,code\n1,123456782,7\n", ""),
        ["p.csv:2: checksum-valid tfn"]
    );
    // A value that wraps onto a list-like line is triage for redact, and still a finding here.
    assert_eq!(
        check("p.md", b"TFN 123 456\n782. Next\n", ""),
        ["p.md:1: checksum-valid tfn"]
    );
    // So is a wrapped Medicare number with its individual reference number.
    assert_eq!(
        check("p.md", b"Medicare: 2123 4567\n0. 1/1\n", ""),
        ["p.md:1: checksum-valid medicare"]
    );
    assert!(check("p.md", b"TFN: 123 456 789\n", "").is_empty());
    assert!(check("p.csv", b"id,tfn,code\n1,123456789,7\n", "").is_empty());
    // A binary file is screened by its printable runs at byte offsets and a zip
    // archive part by part, and digits never join across a run or part.
    assert_eq!(
        check("p.der", b"\x00\xffTFN 123456782\x00", ""),
        ["p.der:byte 6: checksum-valid tfn"]
    );
    assert!(check("p.der", b"\xffTFN 123\x05456 782\x00", "").is_empty());
    let mut workbook = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (part, text) in [
        (
            "xl/sharedStrings.xml",
            "<sst><si><t>123456782</t></si></sst>",
        ),
        ("xl/a.xml", "<t>TFN 123"),
        ("xl/b.xml", "456 782</t>"),
    ] {
        workbook
            .start_file(part, zip::write::SimpleFileOptions::default())
            .expect("part starts");
        workbook.write_all(text.as_bytes()).expect("part writes");
    }
    let workbook = workbook.finish().expect("archive closes").into_inner();
    assert_eq!(
        check("p.xlsx", &workbook, ""),
        ["p.xlsx:xl/sharedStrings.xml:1: checksum-valid tfn"]
    );
    assert_eq!(check("p.xlsx", b"PK\x03\x04 truncated", "").len(), 1);
}

#[test]
fn an_entry_allows_its_value_in_its_own_file_only_and_must_match() {
    let planted = b"TFN: 123 456 782\n";
    let entry = "p.md tfn 123456782 documented test vector\n";
    assert!(check("p.md", planted, entry).is_empty());
    let stale = format!("{ALLOWLIST}:1: entry matches no finding");
    // The same value in another file is a finding, and the unused entry is stale.
    assert_eq!(
        check("q.md", planted, entry),
        ["q.md:1: checksum-valid tfn".to_string(), stale.clone()]
    );
    assert_eq!(check("p.md", b"no identifiers\n", entry), [stale]);
    let malformed = check("p.md", planted, "# header\np.md tfn 123456782\n");
    assert_eq!(
        malformed,
        [
            format!("{ALLOWLIST}:2: expected 'path kind digits reason'"),
            "p.md:1: checksum-valid tfn".to_string(),
        ]
    );
    assert!(!malformed[0].contains("123456782"));
}
