//! Compose review artefacts from assessed rows without recalculating the engine.

use crate::assess::Assessed;
use crate::practitioner_pack::{
    EXPOSURE_FIELDS, ReportRow, parse_report_snapshot, render_practitioner_pack,
};
use crate::py::{Date, OsStyle, PyPath};
use crate::rates::TableSource;
use crate::report::{NoteFacts, render_csv};
use crate::{Failure, LAW_CONTENT_DATE, OsError, Result};

/// A JSON value written with `json.dumps(..., indent=2, ensure_ascii=True)`.
pub enum Out {
    Null,
    Bool(bool),
    Int(String),
    Str(String),
    List(Vec<Out>),
    Object(Vec<(String, Out)>),
}

fn escape(text: &str, out: &mut String) {
    out.push('"');
    for cp in crate::pytext::code_points(text) {
        match cp {
            0x22 => out.push_str("\\\""),
            0x5C => out.push_str("\\\\"),
            0x0A => out.push_str("\\n"),
            0x0D => out.push_str("\\r"),
            0x09 => out.push_str("\\t"),
            0x08 => out.push_str("\\b"),
            0x0C => out.push_str("\\f"),
            0x20..=0x7E => out.push(char::from_u32(cp).expect("ASCII")),
            0x10000.. => {
                let v = cp - 0x10000;
                out.push_str(&format!(
                    "\\u{:04x}\\u{:04x}",
                    0xD800 + (v >> 10),
                    0xDC00 + (v & 0x3FF)
                ));
            }
            _ => out.push_str(&format!("\\u{cp:04x}")),
        }
    }
    out.push('"');
}

impl Out {
    pub fn dumps(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out
    }

    fn write(&self, out: &mut String, level: usize) {
        let pad = |n: usize| "  ".repeat(n);
        match self {
            Out::Null => out.push_str("null"),
            Out::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Out::Int(digits) => out.push_str(digits),
            Out::Str(s) => escape(s, out),
            Out::List(items) if items.is_empty() => out.push_str("[]"),
            Out::List(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    out.push_str(if i == 0 { "\n" } else { ",\n" });
                    out.push_str(&pad(level + 1));
                    item.write(out, level + 1);
                }
                out.push('\n');
                out.push_str(&pad(level));
                out.push(']');
            }
            Out::Object(items) if items.is_empty() => out.push_str("{}"),
            Out::Object(items) => {
                out.push('{');
                for (i, (key, value)) in items.iter().enumerate() {
                    out.push_str(if i == 0 { "\n" } else { ",\n" });
                    out.push_str(&pad(level + 1));
                    escape(key, out);
                    out.push_str(": ");
                    value.write(out, level + 1);
                }
                out.push('\n');
                out.push_str(&pad(level));
                out.push('}');
            }
        }
    }
}

fn date_out(d: Option<Date>) -> Out {
    d.map_or(Out::Null, |d| Out::Str(d.isoformat()))
}

fn row_out(row: &ReportRow) -> Out {
    let mut fields = vec![
        ("source_row".to_string(), Out::Int(row.source_row.0.clone())),
        ("qe_day".into(), Out::Str(row.qe_day.isoformat())),
        ("pathway".into(), Out::Str(row.pathway.clone())),
        ("due_date".into(), date_out(row.due_date)),
        ("verdict".into(), Out::Str(row.verdict.clone())),
        (
            "days_late".into(),
            row.days_late.clone().map_or(Out::Null, Out::Int),
        ),
        (
            "lateness_measured_to".into(),
            Out::Str(row.lateness_measured_to.clone()),
        ),
        ("sg_amount".into(), Out::Str(row.sg_amount.fixed(2, false))),
    ];
    for (field, value) in EXPOSURE_FIELDS.iter().zip(&row.exposure) {
        fields.push((
            field.to_string(),
            value
                .as_ref()
                .map_or(Out::Null, |v| Out::Str(v.fixed(2, false))),
        ));
    }
    fields.push(("caveats".into(), Out::Str(row.caveats.clone())));
    fields.push(("notes".into(), Out::Str(row.notes.clone())));
    fields.push((
        "unassessable_between".into(),
        Out::Str(row.unassessable_between.clone()),
    ));
    Out::Object(fields)
}

/// The run facts an evidence pack records.
pub struct PackFacts<'a> {
    pub as_at: Date,
    pub assessment_date: Option<Date>,
    pub gic_provenance: &'a str,
    pub remittance_only_confirmed: bool,
    pub rate_tables: &'a [TableSource],
}

pub const FILES: [&str; 4] = [
    "report.csv",
    "practitioner-review.md",
    "exceptions.json",
    "decision-log.md",
];

/// The four files, in order, with row references and no identifiers.
pub fn build_evidence_pack(
    results: &[Assessed],
    facts: &PackFacts,
) -> Result<Vec<(String, String)>> {
    let report = format!(
        "\u{feff}{}",
        render_csv(
            results,
            &NoteFacts {
                as_at: facts.as_at,
                law_date: LAW_CONTENT_DATE,
                assessment_date: facts.assessment_date,
                source: None,
                gic_provenance: facts.gic_provenance,
            },
            false,
        )?
    );
    let snapshot = parse_report_snapshot(report.as_bytes(), "report.csv", true)?;
    let tables: Vec<Out> = facts
        .rate_tables
        .iter()
        .map(|t| {
            Out::Object(vec![
                ("uri".into(), Out::Str(t.uri.clone())),
                ("sha256".into(), Out::Str(t.sha256.clone())),
            ])
        })
        .collect();
    let queue = Out::Object(vec![
        ("schema_version".into(), Out::Int("1".into())),
        (
            "report_sha256".into(),
            Out::Str(snapshot.source_sha256.clone()),
        ),
        ("as_at".into(), Out::Str(facts.as_at.isoformat())),
        ("assessment_date".into(), date_out(facts.assessment_date)),
        (
            "remittance_only_confirmed".into(),
            Out::Bool(facts.remittance_only_confirmed),
        ),
        (
            "manifest".into(),
            Out::Object(vec![("rate_table_uris".into(), Out::List(tables))]),
        ),
        (
            "exceptions".into(),
            Out::List(
                snapshot
                    .rows
                    .iter()
                    .filter(|r| r.verdict != "ON_TIME")
                    .map(row_out)
                    .collect(),
            ),
        ),
    ]);
    let decision = [
        "# Payday Super decision log".to_string(),
        String::new(),
        "Template only. No decisions or sign-off have been recorded.".into(),
        String::new(),
        format!("Report SHA-256: `{}`", snapshot.source_sha256),
        format!("As at: {}", facts.as_at.isoformat()),
        format!(
            "Remittance-only confirmation supplied: {}",
            facts.remittance_only_confirmed
        ),
        String::new(),
        "## Evidence reviewed".into(),
        String::new(),
        "- [ ] Match the report SHA-256 to the file reviewed.".into(),
        "- [ ] Reconcile source row references to the private contribution input.".into(),
        "- [ ] Record evidence references for receipts, allocation and assessment facts.".into(),
        "- [ ] Obtain missing fund receipt evidence; never infer receipt from remittance.".into(),
        String::new(),
        "Evidence references: ______________________________".into(),
        String::new(),
        "## Decisions".into(),
        String::new(),
        "Record a human decision or escalation for every row in exceptions.json.".into(),
        String::new(),
        "| Source row | Evidence reference | Decision and reasons | Reviewer | Date | Follow-up |"
            .into(),
        "| --- | --- | --- | --- | --- | --- |".into(),
        "| | | | | | |".into(),
        String::new(),
        "## Practitioner sign-off".into(),
        String::new(),
        "- [ ] Every exception is resolved with evidence or remains explicitly escalated.".into(),
        "- [ ] Review the work under the applicable APES 110 and TPB obligations.".into(),
        "- [ ] An appropriately authorised human decides any advice, correction or lodgement."
            .into(),
        String::new(),
        "Reviewer: ______________________________".into(),
        String::new(),
        "Review date (Australia): ______________________________".into(),
        String::new(),
        "Conclusion and workpaper reference: ______________________________".into(),
        String::new(),
        "Private review aid, not advice, an ATO assessment or a compliance determination. \
         This workflow does not lodge, pay, post or approve. Missing evidence and \
         review flags remain unresolved until a human records the decision."
            .into(),
        String::new(),
    ]
    .join("\n");
    Ok(vec![
        ("report.csv".into(), report),
        (
            "practitioner-review.md".into(),
            render_practitioner_pack(&snapshot, true)?,
        ),
        ("exceptions.json".into(), format!("{}\n", queue.dumps())),
        ("decision-log.md".into(), decision),
    ])
}

/// `os.mkdir(path, 0o700)`: owner-only on Unix and, as Python 3.13 and later
/// do on Windows, a protected ACL granting full control to SYSTEM,
/// Administrators and the owner alone.
#[cfg(unix)]
fn create_private_dir(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().mode(0o700).create(path)
}

#[cfg(windows)]
fn create_private_dir(path: &std::path::Path) -> std::io::Result<()> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    #[repr(C)]
    struct SecurityAttributes {
        length: u32,
        descriptor: *mut c_void,
        inherit_handle: i32,
    }
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl: *const u16,
            revision: u32,
            descriptor: *mut *mut c_void,
            size: *mut u32,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateDirectoryW(path: *const u16, attributes: *const SecurityAttributes) -> i32;
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }
    let wide = |text: &std::ffi::OsStr| -> Vec<u16> { text.encode_wide().chain([0]).collect() };
    let sddl = wide(std::ffi::OsStr::new(
        "D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FA;;;OW)",
    ));
    let mut descriptor: *mut c_void = std::ptr::null_mut();
    let mut size = 0u32;
    // SAFETY: the SDDL is a NUL-terminated UTF-16 string, and both out
    // pointers refer to live locals.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            &mut size,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error());
    }
    let attributes = SecurityAttributes {
        length: std::mem::size_of::<SecurityAttributes>() as u32,
        descriptor,
        inherit_handle: 0,
    };
    let target = wide(path.as_os_str());
    // SAFETY: the path is NUL-terminated UTF-16 and the attributes hold the
    // descriptor allocated above, which is freed once the call returns.
    let created = unsafe { CreateDirectoryW(target.as_ptr(), &attributes) };
    let result = if created == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    };
    // SAFETY: the descriptor came from LocalAlloc inside the conversion call.
    unsafe { LocalFree(descriptor) };
    result
}

/// Refuse an existing file, directory or symlink, preserving previous decisions.
pub fn evidence_destination(path: &str) -> Result<PyPath> {
    let destination = PyPath::new(path);
    let fs = destination.fs();
    if std::fs::metadata(&fs).is_ok() || std::fs::symlink_metadata(&fs).is_ok() {
        return Err(Failure::Value(
            "evidence-pack output already exists; choose a new directory".into(),
        ));
    }
    Ok(destination)
}

/// Create a new directory exclusively and write the four files into it;
/// partial files are kept on failure.
pub fn write_evidence_pack(files: &[(String, String)], path: &str) -> Result<()> {
    let mut names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    names.sort();
    let mut wanted = FILES.to_vec();
    wanted.sort();
    if names != wanted {
        return Err(Failure::Value(
            "evidence pack must contain exactly the four fixed filenames".into(),
        ));
    }
    let destination = evidence_destination(path)?;
    let shown = destination.display();
    create_private_dir(&destination.fs())
        .map_err(|e| Failure::Os(OsError::from_io(&e, &shown, OsStyle::Native)))?;
    for (name, text) in files {
        let target = destination.join(name);
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = options
            .open(target.fs())
            .map_err(|e| Failure::Os(OsError::from_io(&e, &target.display(), OsStyle::Errno)))?;
        std::io::Write::write_all(&mut file, text.as_bytes())
            .map_err(|e| Failure::Os(OsError::from_io(&e, &target.display(), OsStyle::Errno)))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_matches_pythons_indented_ascii_form() {
        let value = Out::Object(vec![
            ("a".into(), Out::List(vec![])),
            (
                "b".into(),
                Out::Object(vec![("c".into(), Out::Str("é\u{1F600}\"".into()))]),
            ),
            ("d".into(), Out::List(vec![Out::Null, Out::Bool(true)])),
        ]);
        assert_eq!(
            value.dumps(),
            "{\n  \"a\": [],\n  \"b\": {\n    \"c\": \"\\u00e9\\ud83d\\ude00\\\"\"\n  },\n  \"d\": [\n    null,\n    true\n  ]\n}"
        );
    }

    #[test]
    fn the_writer_refuses_other_names_and_an_existing_directory() {
        let dir = std::env::temp_dir().join(format!("payday-pack-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let output = dir.join("pack");
        let shown = output.to_string_lossy().to_string();
        let err =
            write_evidence_pack(&[("../outside.txt".into(), "refuse".into())], &shown).unwrap_err();
        assert!(err.message().contains("fixed filenames"));
        assert!(!output.exists());

        // The directory is claimed exclusively: one that appears first is
        // neither replaced nor written into.
        std::fs::create_dir(&output).unwrap();
        let err = create_private_dir(&output).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        let files: Vec<(String, String)> = FILES
            .iter()
            .map(|n| ((*n).to_string(), "fabricated".to_string()))
            .collect();
        let err = write_evidence_pack(&files, &shown).unwrap_err();
        assert!(err.message().contains("already exists"));
        assert_eq!(std::fs::read_dir(&output).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
