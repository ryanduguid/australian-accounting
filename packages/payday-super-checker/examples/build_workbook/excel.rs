//! The desktop Excel pass: one COM session recalculates the built workbook so
//! the shipped file carries real cached values, then the build-machine path
//! Excel stamps into xl/workbook.xml is stripped.

use std::path::Path;
use std::process::Command;

use crate::replace_part;

const RECALC: &str = include_str!("recalc.ps1");

pub const UNAVAILABLE: &str = "the Excel pass needs desktop Excel on Windows; pass --no-excel";

/// Recalculate and save `path` in desktop Excel, strip the build path and
/// return what the script printed (`overall=<status>`). `path` is a staging
/// file: on failure it may be left half-written.
pub fn recalc(path: &Path) -> Result<String, String> {
    if !cfg!(windows) {
        return Err(UNAVAILABLE.into());
    }
    let path = std::path::absolute(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-EncodedCommand"])
        .arg(encoded_command(RECALC))
        .env("PAYDAY_WORKBOOK", &path)
        .output()
        .map_err(|e| format!("powershell: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() { stdout } else { stderr });
    }
    if !matches!(
        stdout.as_str(),
        "overall=PASS" | "overall=REVIEW" | "overall=BLOCKED"
    ) {
        return Err(format!("unexpected output from the Excel pass: {stdout:?}"));
    }
    let io = |e: std::io::Error| format!("{}: {e}", path.display());
    let stripped = strip_abs_path(&std::fs::read(&path).map_err(io)?)?;
    std::fs::write(&path, stripped).map_err(io)?;
    Ok(stdout)
}

/// Remove every `<x15ac:absPath …/>` from xl/workbook.xml, copying every other
/// part as Excel wrote it.
pub fn strip_abs_path(xlsx: &[u8]) -> Result<Vec<u8>, String> {
    const PART: &str = "xl/workbook.xml";
    const TAG: &str = "<x15ac:absPath";
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(xlsx)).map_err(|e| e.to_string())?;
    let mut xml = crate::model::read_part(&mut archive, PART)?;
    while let Some(start) = xml.find(TAG) {
        let end = xml[start..]
            .find('>')
            .map(|i| start + i + 1)
            .filter(|&end| xml[..end].ends_with("/>"))
            .ok_or("xl/workbook.xml: absPath is not an empty element")?;
        xml.replace_range(start..end, "");
    }
    replace_part(xlsx, PART, xml.as_bytes())
}

/// PowerShell's -EncodedCommand form: base64 of the UTF-16LE script, which
/// passes the script through the command line without any quoting.
fn encoded_command(script: &str) -> String {
    const DIGITS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let mut acc = [0u8; 4];
        acc[1..=chunk.len()].copy_from_slice(chunk);
        let n = u32::from_be_bytes(acc);
        for i in 0..4 {
            out.push(if i <= chunk.len() {
                char::from(DIGITS[((n >> (18 - 6 * i)) & 63) as usize])
            } else {
                '='
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::read_part;
    use std::io::{Cursor, Write};

    fn workbook(xml: &str) -> Vec<u8> {
        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for (name, body) in [("xl/workbook.xml", xml), ("xl/styles.xml", "<styleSheet/>")] {
            out.start_file(name, options).unwrap();
            out.write_all(body.as_bytes()).unwrap();
        }
        out.finish().unwrap().into_inner()
    }

    fn part(xlsx: &[u8], name: &str) -> String {
        read_part(&mut zip::ZipArchive::new(Cursor::new(xlsx)).unwrap(), name).unwrap()
    }

    #[test]
    fn strips_only_the_build_path() {
        let before = workbook(
            r#"<workbook><mc:Choice Requires="x15"><x15ac:absPath url="C:\Users\someone\" xmlns:x15ac="urn:x"/></mc:Choice><sheets/></workbook>"#,
        );
        let after = strip_abs_path(&before).unwrap();
        assert_eq!(
            part(&after, "xl/workbook.xml"),
            r#"<workbook><mc:Choice Requires="x15"></mc:Choice><sheets/></workbook>"#
        );
        assert_eq!(part(&after, "xl/styles.xml"), "<styleSheet/>");
        let clean = workbook("<workbook><sheets/></workbook>");
        assert_eq!(
            part(&strip_abs_path(&clean).unwrap(), "xl/workbook.xml"),
            "<workbook><sheets/></workbook>"
        );
    }

    #[test]
    fn refuses_an_abs_path_with_content() {
        let odd = workbook(r#"<workbook><x15ac:absPath url="C:\">x</x15ac:absPath></workbook>"#);
        assert!(strip_abs_path(&odd).is_err());
    }

    #[test]
    fn encodes_the_script_as_powershell_reads_it() {
        // [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes(...))
        assert_eq!(encoded_command("a"), "YQA=");
        assert_eq!(encoded_command("ab"), "YQBiAA==");
        assert_eq!(encoded_command("abc"), "YQBiAGMA");
        assert_eq!(encoded_command("'é'"), "JwDpACcA");
    }
}
