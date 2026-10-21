//! Index local engine reports without merging their conclusions or scope.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path},
    process::ExitCode,
};

const LIMIT: u64 = 2_000_000;
const SCOPE: &str = "Index only. Entity, period, engine version and scope labels are supplied assertions. Read each preserved source output for its result and caveats. No group verdict, tax consolidation, elimination or approval is calculated.";
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    reports: Vec<Object<Report>>,
}

// Serde's derived structs also accept JSON arrays. The original accepts objects
// only. Deserialize through MapAccess without first losing duplicate keys.
struct Object<T>(T);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct Visitor<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
            type Value = Object<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                map: M,
            ) -> std::result::Result<Self::Value, M::Error> {
                T::deserialize(serde::de::value::MapAccessDeserializer::new(map)).map(Object)
            }
        }
        deserializer.deserialize_map(Visitor(std::marker::PhantomData))
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Report {
    id: String,
    entity: String,
    period: String,
    engine: String,
    engine_version: String,
    scope: String,
    path: String,
    sha256: String,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn blank(text: &str) -> bool {
    // Python str.strip also treats these 4 information separators as whitespace.
    text.chars()
        .all(|c| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
}

#[cfg(windows)]
fn same_drive_relative<'a>(path: &'a Path, root: &Path) -> Result<&'a Path> {
    use std::path::Prefix;
    let mut parts = path.components();
    if let Some(Component::Prefix(prefix)) = parts.next() {
        let root_drive = match root.components().next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => Some(drive),
                _ => None,
            },
            _ => None,
        };
        if let Prefix::Disk(drive) = prefix.kind()
            && root_drive.is_some_and(|r| r.eq_ignore_ascii_case(&drive))
        {
            return Ok(parts.as_path());
        }
        return Err("Report path must remain inside the manifest directory".into());
    }
    Ok(path)
}

fn manifest_text(bytes: &[u8]) -> Result<String> {
    // Match json.loads(bytes): BOMs and the initial JSON ASCII bytes identify
    // UTF-8, UTF-16 and UTF-32, including BOM-less little/big endian inputs.
    let (body, width, little) = if let Some(body) = bytes.strip_prefix(&[255, 254, 0, 0]) {
        (body, 4, true)
    } else if let Some(body) = bytes.strip_prefix(&[0, 0, 254, 255]) {
        (body, 4, false)
    } else if let Some(body) = bytes.strip_prefix(&[255, 254]) {
        (body, 2, true)
    } else if let Some(body) = bytes.strip_prefix(&[254, 255]) {
        (body, 2, false)
    } else if let Some(body) = bytes.strip_prefix(&[239, 187, 191]) {
        (body, 1, false)
    } else if bytes.len() >= 4 && bytes[0] == 0 {
        (bytes, if bytes[1] == 0 { 4 } else { 2 }, false)
    } else if bytes.len() >= 4 && bytes[1] == 0 {
        (bytes, if bytes[2..4] == [0, 0] { 4 } else { 2 }, true)
    } else if bytes.len() == 2 && (bytes[0] == 0 || bytes[1] == 0) {
        (bytes, 2, bytes[1] == 0)
    } else {
        (bytes, 1, false)
    };
    if width == 1 {
        return Ok(String::from_utf8(body.to_vec())?);
    }
    if body.len() % width != 0 {
        return Err("Incomplete manifest encoding".into());
    }
    if width == 2 {
        let units = body.as_chunks::<2>().0.iter().map(|pair| {
            if little {
                u16::from_le_bytes(*pair)
            } else {
                u16::from_be_bytes(*pair)
            }
        });
        return Ok(char::decode_utf16(units).collect::<std::result::Result<String, _>>()?);
    }
    body.as_chunks::<4>()
        .0
        .iter()
        .map(|b| {
            let value = if little {
                u32::from_le_bytes(*b)
            } else {
                u32::from_be_bytes(*b)
            };
            char::from_u32(value).ok_or_else(|| "Invalid UTF-32 manifest character".into())
        })
        .collect()
}

fn build_index(manifest_path: &Path) -> Result<Value> {
    let content = fs::read(manifest_path)?;
    // Typed structures reject duplicate decoded field names as well as missing,
    // unknown or mistyped fields, without discarding keys through a Value map.
    let manifest = serde_json::from_str::<Object<Manifest>>(&manifest_text(&content)?)?.0;
    if manifest.schema_version != "group-review-input.v1" {
        return Err("Unknown manifest schema".into());
    }
    if manifest.reports.is_empty() {
        return Err("At least one report is required".into());
    }
    let resolved_manifest = manifest_path.canonicalize()?;
    let root = resolved_manifest.parent().ok_or("Manifest has no parent")?;
    let mut records = Vec::new();
    let mut identities = HashSet::new();
    for Object(report) in manifest.reports {
        if [
            &report.id,
            &report.entity,
            &report.period,
            &report.engine,
            &report.engine_version,
            &report.scope,
            &report.path,
            &report.sha256,
        ]
        .into_iter()
        .any(|text| blank(text))
        {
            return Err("Every report metadata value must be non-empty text".into());
        }
        if !identities.insert(report.id.clone()) {
            return Err("Duplicate report ID".into());
        }
        let relative = Path::new(&report.path);
        if relative.is_absolute() {
            return Err("Report path must remain inside the manifest directory".into());
        }
        #[cfg(windows)]
        let relative = same_drive_relative(relative, root)?;
        if relative.components().any(|part| match part {
            Component::CurDir => false,
            Component::RootDir => !cfg!(windows),
            Component::Normal(name) => name.to_str().is_none_or(|s| s.starts_with('.')),
            _ => true,
        }) {
            return Err("Report path must remain inside the manifest directory".into());
        }
        // Python Path normalises trailing separators and '/.' before IO.
        let relative = relative.components().collect::<std::path::PathBuf>();
        let path = root.join(relative).canonicalize()?;
        if !path.starts_with(root)
            || !matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("txt" | "json")
            )
        {
            return Err("Report must be text or JSON inside the manifest directory".into());
        }
        let file = File::open(path)?;
        if file.metadata()?.len() > LIMIT {
            return Err("Report exceeds the 2 MB example limit".into());
        }
        let mut source = Vec::new();
        file.take(LIMIT + 1).read_to_end(&mut source)?;
        if source.len() as u64 > LIMIT {
            return Err("Report exceeds the 2 MB example limit".into());
        }
        if digest(&source) != report.sha256 {
            return Err(format!("Evidence digest mismatch: {}", report.id).into());
        }
        let text = std::str::from_utf8(source.strip_prefix(&[239, 187, 191]).unwrap_or(&source))?;
        if blank(text) {
            return Err("Engine report is empty".into());
        }
        let mut record = serde_json::to_value(report)?;
        record["source_output"] = json!(text);
        records.push(record);
    }
    Ok(
        json!({"schema_version":"group-review-index.v1", "manifest_sha256":digest(&content), "reports":records, "scope":SCOPE}),
    )
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!(
            "Index local engine reports without merging their conclusions or scope.\nUsage: group_review [--] MANIFEST"
        );
        return ExitCode::SUCCESS;
    }
    let literal = args.first().is_some_and(|arg| arg == "--");
    if literal {
        args.remove(0);
    }
    if args.len() != 1 || (!literal && args[0].to_string_lossy().starts_with('-')) {
        eprintln!("Group review index: Usage: group_review [--] MANIFEST");
        return ExitCode::from(2);
    }
    let result = build_index(Path::new(&args[0])).and_then(|index| {
        let mut output = serde_json::to_vec_pretty(&index)?;
        output.push(b'\n');
        std::io::stdout().lock().write_all(&output)?;
        Ok(())
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Group review index: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "native-group-review-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn unhex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect()
    }
    #[test]
    fn complete_original_successes_and_refusals() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/native/group-review.json"))
                .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let temp = Temp::new();
            for file in case["files"].as_array().unwrap() {
                let path = temp.0.join(file["path"].as_str().unwrap());
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                let bytes = if let Some(hex) = file["hex"].as_str() {
                    unhex(hex)
                } else {
                    vec![
                        file["repeat_byte"].as_u64().unwrap() as u8;
                        file["count"].as_u64().unwrap() as usize
                    ]
                };
                fs::write(path, bytes).unwrap();
            }
            let manifest = temp.0.join("manifest.json");
            fs::write(&manifest, unhex(case["manifest_hex"].as_str().unwrap())).unwrap();
            let result = build_index(&manifest);
            if case["refused"] == true {
                assert!(result.is_err(), "{}", case["label"]);
                continue;
            }
            let mut result = result.unwrap_or_else(|e| panic!("{}: {e}", case["label"]));
            if case["summary"] == true {
                for report in result["reports"].as_array_mut().unwrap() {
                    let text = report["source_output"].as_str().unwrap();
                    report["source_output"] =
                        json!({"sha256":digest(text.as_bytes()), "utf8_bytes":text.len()});
                }
            }
            assert_eq!(result, case["expected"], "{}", case["label"]);
        }
    }

    #[test]
    fn malformed_manifest_encodings_and_lone_surrogates_are_refused() {
        for bytes in [
            vec![255, 254, 123],
            vec![0, 0, 254, 255, 0],
            vec![255, 254, 0, 216],
            vec![255, 254, 0, 0, 0, 0, 17, 0],
            vec![255],
        ] {
            assert!(manifest_text(&bytes).is_err());
        }
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/native/group-review.json"))
                .unwrap();
        let text = String::from_utf8(unhex(fixture["cases"][0]["manifest_hex"].as_str().unwrap()))
            .unwrap();
        let text = text.replace("Synthetic Entity 0", "\\ud800");
        assert!(serde_json::from_str::<Object<Manifest>>(&text).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_use_resolved_manifest_root_and_resolved_report_extension() {
        use std::os::unix::fs::symlink;
        let temp = Temp::new();
        let root = temp.0.join("root");
        fs::create_dir(&root).unwrap();
        fs::create_dir(temp.0.join("alias")).unwrap();
        let source = b"REVIEW: fabricated. No approval.\n";
        for name in ["root/report.txt", "root/report.bin", "outside.txt"] {
            fs::write(temp.0.join(name), source).unwrap();
        }
        let manifest = root.join("manifest.json");
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/native/group-review.json"))
                .unwrap();
        let mut input: Value = serde_json::from_slice(&unhex(
            fixture["cases"][0]["manifest_hex"].as_str().unwrap(),
        ))
        .unwrap();
        input["reports"].as_array_mut().unwrap().truncate(1);
        input["reports"][0]["sha256"] = json!(digest(source));
        for (alias, target, accepted) in [
            ("inside.bin", "report.txt", true),
            ("inside.txt", "report.bin", false),
            ("escape.txt", "../outside.txt", false),
        ] {
            symlink(target, root.join(alias)).unwrap();
            input["reports"][0]["path"] = json!(alias);
            fs::write(&manifest, serde_json::to_vec(&input).unwrap()).unwrap();
            assert_eq!(build_index(&manifest).is_ok(), accepted, "{alias}");
        }
        symlink(&temp.0, root.join("outside-directory")).unwrap();
        input["reports"][0]["path"] = json!("outside-directory/outside.txt");
        fs::write(&manifest, serde_json::to_vec(&input).unwrap()).unwrap();
        assert!(build_index(&manifest).is_err());
        input["reports"][0]["path"] = json!("report.txt");
        fs::write(&manifest, serde_json::to_vec(&input).unwrap()).unwrap();
        let alias = temp.0.join("alias/manifest.json");
        symlink(&manifest, &alias).unwrap();
        assert_eq!(
            build_index(&manifest).unwrap(),
            build_index(&alias).unwrap()
        );
    }
}
