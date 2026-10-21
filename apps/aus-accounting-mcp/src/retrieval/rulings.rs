//! Validated, immutable operator runs. Metadata signatures avoid repeat validation.
use super::Lines;
use super::{
    Context, POLICY,
    corpus::{self, Ranked},
    linked, page, terms,
};
use num_bigint::BigInt;
use payday_super_checker::{py, pyre::Regex};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Value, json};
use std::{
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    fmt, fs,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::SystemTime,
};

fn policy(name: &str) -> &'static str {
    POLICY["rulings"][name].as_str().unwrap()
}
static RUN_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(policy("RUN_NAME")));
static DOCID: LazyLock<Regex> = LazyLock::new(|| Regex::new(policy("DOCID")));
static ROW_REF: LazyLock<Regex> = LazyLock::new(|| Regex::new(policy("ROW_REF")));
static ISO_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(policy("ISO_DATE")));
static SHA256: LazyLock<Regex> = LazyLock::new(|| Regex::new(policy("SHA256")));

#[derive(Clone)]
struct Run {
    name: String,
    path: PathBuf,
    date: String,
    documents: HashSet<String>,
    excluded: HashSet<String>,
    notice: String,
    url: String,
}
struct Catalogue {
    runs: Vec<Run>,
    owner: HashMap<String, String>,
    withheld: HashMap<String, String>,
}
type Signature = (u64, SystemTime, u64, SystemTime);
static VALIDATED: LazyLock<Mutex<HashMap<PathBuf, (Signature, Run)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
fn canonical(id: &str) -> String {
    id.trim_matches(py::is_space)
        .trim_end_matches('/')
        .to_uppercase()
}
fn family_priority(row: &Value) -> usize {
    let families = POLICY["rulings"]["FAMILIES"].as_array().unwrap();
    families
        .iter()
        .position(|f| *f == row["family"])
        .unwrap_or(families.len())
}
fn count(value: &Value) -> Option<BigInt> {
    let n = value.as_number()?.to_string();
    if n.strip_prefix('-')
        .unwrap_or(&n)
        .bytes()
        .all(|b| b.is_ascii_digit())
    {
        n.parse().ok()
    } else {
        None
    }
}
fn withdrawal(row: &Value) -> Result<(String, String), String> {
    let status = row.get("withdrawal_status").unwrap_or(&Value::Null);
    let status = if row.get("withdrawal_status").is_none() {
        Some("UNKNOWN")
    } else {
        status.as_str()
    };
    let notice = if row.get("withdrawal_notice").is_none() {
        Some("")
    } else {
        row["withdrawal_notice"].as_str()
    };
    match (status, notice) {
        (Some(status @ ("UNKNOWN" | "NOTICE_PRESENT")), Some(notice))
            if (status == "NOTICE_PRESENT") == !notice.trim_matches(py::is_space).is_empty() =>
        {
            Ok((status.into(), notice.into()))
        }
        _ => Err("Rulings record has inconsistent withdrawal metadata.".into()),
    }
}

// Validate duplicate members recursively before decoding to Value. Reuse serde's
// JSON parser and arbitrary-precision support; no second JSON grammar lives here.
struct Unique;
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Check;
        impl<'de> Visitor<'de> for Check {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON without duplicate object keys")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Unique, M::Error> {
                let mut keys = HashSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key) {
                        return Err(de::Error::custom("duplicate key"));
                    }
                    map.next_value::<Unique>()?;
                }
                Ok(Unique)
            }
            fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Unique, S::Error> {
                while seq.next_element::<Unique>()?.is_some() {}
                Ok(Unique)
            }
            fn visit_bool<E: de::Error>(self, _: bool) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_i64<E: de::Error>(self, _: i64) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_u64<E: de::Error>(self, _: u64) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_str<E: de::Error>(self, _: &str) -> Result<Unique, E> {
                Ok(Unique)
            }
            fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
                Ok(Unique)
            }
        }
        deserializer.deserialize_any(Check)
    }
}
struct Manifest {
    date: String,
    listed: serde_json::Map<String, Value>,
    excluded: HashSet<String>,
    notice: String,
    url: String,
}
fn manifest(context: &Context, path: &Path, run: &str) -> Result<Manifest, String> {
    let invalid =
        |reason: String| format!("Rulings run {run} has an invalid manifest.json: {reason}.");
    let readable = || invalid("it is not readable UTF-8 JSON without duplicate keys".into());
    let raw = context
        .read(path, context.limit("MAX_MANIFEST_BYTES", 4_000_000))
        .map_err(|_| readable())?;
    let text = std::str::from_utf8(&raw).map_err(|_| readable())?;
    let data = super::parse_json(text).map_err(|_| readable())?;
    match serde_json::from_str::<Unique>(text) {
        Ok(_) => {}
        Err(error) if error.to_string().contains("recursion limit exceeded") => {
            stacker::grow(16 * 1024 * 1024, || {
                let mut decoder = serde_json::Deserializer::from_str(text);
                decoder.disable_recursion_limit();
                Unique::deserialize(&mut decoder)
            })
            .map_err(|_| readable())?;
        }
        Err(_) => return Err(readable()),
    }
    if !data.is_object() || data["stage"] != "rulings" {
        return Err(invalid("stage must be 'rulings'".into()));
    }
    let date = data["fetched_on"]
        .as_str()
        .filter(|s| {
            ISO_DATE.fullmatch(s)
                && !s.starts_with("0000-")
                && chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
        })
        .ok_or_else(|| invalid("fetched_on must be a YYYY-MM-DD date".into()))?;
    let notice = data["reuse_notice"]
        .as_str()
        .filter(|s| !s.trim_matches(py::is_space).is_empty());
    let url = data["reuse_notice_url"]
        .as_str()
        .filter(|s| s.starts_with("https://"));
    let (Some(notice), Some(url)) = (notice, url) else {
        return Err(invalid(
            "the reuse notice and its https address are required".into(),
        ));
    };
    let maximum = context.limit("MAX_DOCUMENTS", 1000);
    let documents = data["documents"]
        .as_array()
        .filter(|a| a.len() <= maximum)
        .ok_or_else(|| invalid(format!("documents must be a list of at most {maximum}")))?;
    let excluded = data["excluded"]
        .as_array()
        .ok_or_else(|| invalid("excluded must be a list".into()))?;
    let rows = count(&data["rows"])
        .filter(|n| *n >= BigInt::from(0))
        .ok_or_else(|| invalid("rows must be a non-negative integer".into()))?;
    let mut listed = serde_json::Map::new();
    for (index, document) in documents.iter().enumerate() {
        let index = index + 1;
        let id = document["docid"]
            .as_str()
            .map(canonical)
            .filter(|id| DOCID.fullmatch(id))
            .ok_or_else(|| invalid(format!("document {index} has no supported docid")))?;
        if listed.contains_key(&id) {
            return Err(invalid(format!(
                "document {index} repeats an earlier docid"
            )));
        }
        if count(&document["paragraphs"]).is_none_or(|n| n < BigInt::from(1)) {
            return Err(invalid(format!(
                "document {index} must count at least one paragraph"
            )));
        }
        if family_priority(document) == POLICY["rulings"]["FAMILIES"].as_array().unwrap().len() {
            return Err(invalid(format!(
                "document {index} names an unsupported family"
            )));
        }
        if document["fetched_on"] != date {
            return Err(invalid(format!(
                "document {index} carries a different date from its run"
            )));
        }
        if !document["sha256"]
            .as_str()
            .is_some_and(|s| SHA256.fullmatch(s))
        {
            return Err(invalid(format!(
                "document {index} needs the SHA-256 of its page"
            )));
        }
        if !matches!(
            document["licence_basis"].as_str(),
            Some("document-notice" | "site-notice-via-dc-rights")
        ) {
            return Err(invalid(format!(
                "document {index} records no supported reuse basis"
            )));
        }
        if !["title", "source_url"]
            .iter()
            .all(|f| document[*f].is_string())
        {
            return Err(invalid(format!(
                "document {index} needs a title and a source_url"
            )));
        }
        withdrawal(document)?;
        listed.insert(id, document.clone());
    }
    let mut withheld = HashSet::new();
    for (index, entry) in excluded.iter().enumerate() {
        let id = entry["docid"]
            .as_str()
            .filter(|s| !s.trim_matches(py::is_space).is_empty())
            .ok_or_else(|| invalid(format!("excluded entry {} needs a docid", index + 1)))?;
        let id = canonical(id);
        if listed.contains_key(&id) {
            return Err(invalid(format!(
                "excluded entry {} names a document the run also accepted",
                index + 1
            )));
        }
        withheld.insert(id);
    }
    if rows
        != listed
            .values()
            .map(|d| count(&d["paragraphs"]).unwrap())
            .sum::<BigInt>()
    {
        return Err(invalid(
            "rows does not equal the sum of the documents' paragraph counts".into(),
        ));
    }
    Ok(Manifest {
        date: date.into(),
        listed,
        excluded: withheld,
        notice: notice.into(),
        url: url.into(),
    })
}

fn validate(path: &Path, name: &str, manifest: &Manifest) -> Result<(), String> {
    let error = format!("Cannot read rulings run {name} as UTF-8 JSONL.");
    let mut counts: HashMap<_, usize> = manifest.listed.keys().map(|k| (k.clone(), 0)).collect();
    for (line, raw) in Lines::open(path).map_err(|_| &error)?.enumerate() {
        let line = line + 1;
        let raw = raw.map_err(|_| &error)?;
        let row = super::parse_json(&raw)
            .map_err(|_| format!("Rulings run {name} has a malformed row at line {line}."))?;
        if !row.is_object()
            || !POLICY["rulings"]["ROW_FIELDS"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| row[f.as_str().unwrap()].is_string())
        {
            return Err(format!(
                "Rulings run {name} row {line} lacks a required field."
            ));
        }
        let key = canonical(row["docid"].as_str().unwrap());
        let document = manifest.listed.get(&key).ok_or_else(|| {
            format!(
                "Rulings run {name} row {line} belongs to a document its manifest does not list."
            )
        })?;
        let disagree = [
            ("family", "family"),
            ("title", "title"),
            ("source_url", "source_url"),
            ("source_sha256", "sha256"),
            ("licence_basis", "licence_basis"),
        ]
        .iter()
        .any(|(r, d)| row[*r] != document[*d]);
        if disagree
            || row["fetched_on"] != manifest.date
            || !matches!(row["numbering"].as_str(), Some("document" | "sequential"))
            || withdrawal(&row)? != withdrawal(document)?
        {
            return Err(format!(
                "Rulings run {name} row {line} disagrees with its manifest record."
            ));
        }
        *counts.get_mut(&key).unwrap() += 1;
    }
    for (key, document) in &manifest.listed {
        if BigInt::from(counts[key]) != count(&document["paragraphs"]).unwrap() {
            return Err(format!(
                "Rulings run {name} holds {} rows for a document its manifest counts as {}.",
                counts[key], document["paragraphs"]
            ));
        }
    }
    Ok(())
}
fn is_run(path: &Path) -> bool {
    path.join("rulings.jsonl").exists() || path.join("manifest.json").exists()
}
fn directories(context: &Context) -> Result<Vec<PathBuf>, String> {
    let root = context.root("AUS_ACCOUNTING_RULINGS_ROOT", "folder of rulings runs")?;
    if is_run(&root) {
        return Ok(vec![root]);
    }
    let mut entries = fs::read_dir(root)
        .map_err(|_| "Cannot read the configured rulings folder.")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Cannot read the configured rulings folder.")?;
    entries.sort_by_key(|e| e.file_name().to_string_lossy().into_owned());
    let mut runs = Vec::new();
    for entry in entries {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.')
            || linked(&path).unwrap_or(true)
            || !path.is_dir()
        {
            continue;
        }
        if is_run(&path) {
            runs.push(path);
        }
        let maximum = context.limit("MAX_RUNS", 500);
        if runs.len() > maximum {
            return Err(format!(
                "The rulings folder holds more than {maximum} runs; configure a smaller one."
            ));
        }
    }
    if runs.is_empty() {
        return Err("No rulings runs found in the configured folder.".into());
    }
    Ok(runs)
}
fn member(directory: &Path, filename: &str, name: &str) -> Result<PathBuf, String> {
    let path = directory.join(filename);
    if !path.is_file() || linked(&path).unwrap_or(true) {
        return Err(format!(
            "Rulings run {name} must hold {filename} as a regular file, as the rulings stage writes it."
        ));
    }
    Ok(path)
}
fn load(
    context: &Context,
    directory: &Path,
    manifest_path: &Path,
    rows: &Path,
) -> Result<Run, String> {
    let name = directory.file_name().unwrap().to_str().unwrap();
    let error = format!("Cannot read rulings run {name} as UTF-8 JSONL.");
    let m = fs::metadata(manifest_path).map_err(|_| &error)?;
    let r = fs::metadata(rows).map_err(|_| &error)?;
    let signature = (
        m.len(),
        m.modified().map_err(|_| &error)?,
        r.len(),
        r.modified().map_err(|_| &error)?,
    );
    let cached = VALIDATED
        .lock()
        .unwrap()
        .get(directory)
        .filter(|(s, _)| *s == signature)
        .map(|(_, run)| run.clone());
    if let Some(cached) = cached {
        return Ok(cached);
    }
    let maximum = context.limit("MAX_MANIFEST_BYTES", 4_000_000);
    if m.len() > maximum as u64 {
        return Err(format!(
            "Rulings run {name} has a manifest over {maximum} bytes."
        ));
    }
    let manifest = manifest(context, manifest_path, name)?;
    validate(rows, name, &manifest)?;
    let run = Run {
        name: name.into(),
        path: rows.to_path_buf(),
        date: manifest.date,
        documents: manifest.listed.into_iter().map(|(k, _)| k).collect(),
        excluded: manifest.excluded,
        notice: manifest.notice,
        url: manifest.url,
    };
    VALIDATED
        .lock()
        .unwrap()
        .insert(directory.to_path_buf(), (signature, run.clone()));
    Ok(run)
}
fn catalogue(context: &Context) -> Result<Catalogue, String> {
    let mut members = Vec::new();
    for directory in directories(context)? {
        let name = directory.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !RUN_NAME.fullmatch(name) {
            return Err(
                "Rulings run folder names may use only letters, digits, '.', '_' and '-'.".into(),
            );
        }
        let manifest = member(&directory, "manifest.json", name)?;
        let rows = member(&directory, "rulings.jsonl", name)?;
        members.push((directory, manifest, rows));
    }
    let mut total = 0;
    for (_, _, rows) in &members {
        total += fs::metadata(rows)
            .map_err(|_| "Cannot read the configured rulings folder.")?
            .len();
    }
    let maximum = context.limit("MAX_CORPUS_BYTES", 320_000_000);
    if total > maximum as u64 {
        return Err(format!(
            "The configured rulings runs hold more than {} MB of rows; configure a smaller folder.",
            maximum / 1_000_000
        ));
    }
    let mut runs = members
        .iter()
        .map(|(d, m, r)| load(context, d, m, r))
        .collect::<Result<Vec<_>, _>>()?;
    runs.sort_by(|a, b| (&a.date, &a.name).cmp(&(&b.date, &b.name)));
    let (mut owner, mut withheld) = (HashMap::new(), HashMap::new());
    for run in &runs {
        for key in &run.documents {
            owner.insert(key.clone(), run.name.clone());
            withheld.remove(key);
        }
        for key in &run.excluded {
            withheld.insert(key.clone(), run.name.clone());
            owner.remove(key);
        }
    }
    Ok(Catalogue {
        runs,
        owner,
        withheld,
    })
}
fn provenance(catalogue: &Catalogue) -> Value {
    let serving: HashSet<&str> = catalogue.owner.values().map(String::as_str).collect();
    let runs = catalogue
        .runs
        .iter()
        .filter(|run| serving.contains(run.name.as_str()))
        .collect::<Vec<_>>();
    let mut notices: Vec<Value> = Vec::new();
    for run in &runs {
        if let Some(notice) = notices
            .iter_mut()
            .find(|n| n["notice"] == run.notice && n["notice_url"] == run.url)
        {
            notice["runs"].as_array_mut().unwrap().push(json!(run.name));
        } else {
            notices.push(json!({"notice":run.notice,"notice_url":run.url,"runs":[run.name]}));
        }
    }
    json!({"runs_configured":catalogue.runs.len(),"runs_serving":runs.len(),"documents_serving":catalogue.owner.len(),"documents_withheld":catalogue.withheld.len(),"fetched_on_earliest":runs.iter().map(|r|&r.date).min(),"fetched_on_latest":runs.iter().map(|r|&r.date).max(),"reuse_notices":notices,"non_endorsement":policy("NON_ENDORSEMENT"),"ranking":policy("RANKING_NOTE")})
}

#[test]
fn provenance_matches_original_at_large_catalogue_sizes() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native/calvino-followups.json"
    ))
    .unwrap();
    for case in fixture["provenance_scale"].as_array().unwrap() {
        let count = case["document_count"].as_u64().unwrap() as usize;
        let catalogue = Catalogue {
            runs: (0..500)
                .map(|i| Run {
                    name: format!("run-{i:03}"),
                    path: PathBuf::new(),
                    date: (chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
                        + chrono::Duration::days(i))
                    .to_string(),
                    documents: HashSet::new(),
                    excluded: HashSet::new(),
                    notice: format!("Synthetic notice {}", i % 3),
                    url: format!("https://example.invalid/notice/{}", i % 3),
                })
                .collect(),
            owner: (0..count)
                .map(|i| (format!("DOC{i:06}"), format!("run-{:03}", (i % 5) * 100)))
                .collect(),
            withheld: HashMap::from([("WITHHELD".into(), "run-499".into())]),
        };
        let start = std::time::Instant::now();
        let actual = provenance(&catalogue);
        println!(
            "provenance documents={count} nanoseconds={}",
            start.elapsed().as_nanos()
        );
        assert_eq!(actual, case["expected"]);
    }
}
fn text(row: &Value, field: &str, cap: usize, caveat: &str) -> (String, usize, Vec<String>) {
    let text = row[field].as_str().unwrap_or("");
    let total = text.chars().count();
    (
        text.chars().take(cap).collect(),
        total,
        if total > cap {
            vec![
                caveat
                    .replace("{kept}", &cap.to_string())
                    .replace("{total}", &total.to_string()),
            ]
        } else {
            vec![]
        },
    )
}
fn paragraph(
    row: &Value,
    run: &Run,
    key: &str,
    line: usize,
    catalogue: &Catalogue,
    cap: usize,
) -> Result<Value, String> {
    let (text, total, mut caveats) = text(row, "text", cap, policy("TRUNCATED_CAVEAT"));
    let (status, _) = withdrawal(row)?;
    let (notice, _, extra) = self::text(
        row,
        "withdrawal_notice",
        cap,
        "Withdrawal notice is truncated at {kept} of {total} characters. Read source_url.",
    );
    caveats.extend(extra);
    if status == "NOTICE_PRESENT" {
        caveats.push(format!("A withdrawal notice was present in the page captured on {}. Check its effect and applicable period on the ATO Legal Database before reliance.",row["fetched_on"].as_str().unwrap()));
    }
    let serving = catalogue.owner.get(key) == Some(&run.name);
    if !serving {
        caveats.push(policy("REPLACED_CAVEAT").into());
    }
    if row["family"] == policy("EDITED_FAMILY") {
        caveats.push(policy("EDITED_CAVEAT").into());
    }
    if row["numbering"] == "sequential" {
        caveats.push(policy("SEQUENTIAL_CAVEAT").into());
    }
    let mut value = json!({"row_ref":format!("{}|{key}|{line}",run.name),"text":text,"total_chars":total,"withdrawal_status":status,"withdrawal_notice":notice,"run":run.name,"serving":serving,"caveats":caveats});
    for field in [
        "docid",
        "family",
        "title",
        "paragraph",
        "numbering",
        "heading",
        "source_url",
        "source_sha256",
        "fetched_on",
        "licence_basis",
    ] {
        value[field] = json!(row[field].as_str());
    }
    Ok(value)
}
fn after_catalogue(context: &Context, catalogue: &Catalogue, read: bool) {
    #[cfg(test)]
    if context.fault.as_deref() == Some("rulings-changed-after-validation") {
        let path = &catalogue.runs.last().unwrap().path;
        if read {
            fs::write(path, "{not json\n".repeat(3)).unwrap();
        } else {
            fs::remove_file(path).unwrap();
        }
    }
    let _ = (context, catalogue, read);
}
fn words(query: &str, field: &str) -> Result<Vec<String>, String> {
    let t = terms(query);
    if t.is_empty() {
        Err(format!("Supply at least one word in {field}."))
    } else {
        Ok(t)
    }
}
fn search(context: &Context, input: &Value) -> Result<Value, String> {
    let words = words(input["query"].as_str().unwrap(), "query")?;
    let family = input["family"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| self::words(s, "family"))
        .transpose()?
        .unwrap_or_default();
    let catalogue = catalogue(context)?;
    after_catalogue(context, &catalogue, false);
    let (offset, limit) = (
        super::position(&input["offset"]),
        super::position(&input["limit"]),
    );
    let (mut best, mut order) = (BinaryHeap::new(), 0);
    let error = "Cannot read a UTF-8 rulings run in the configured folder.";
    for run in &catalogue.runs {
        for (line, raw) in Lines::open(&run.path).map_err(|_| error)?.enumerate() {
            let raw = raw.map_err(|_| error)?;
            let Ok(row) = super::parse_json(&raw) else {
                continue;
            };
            if !row.is_object() {
                continue;
            }
            let key = canonical(row["docid"].as_str().unwrap_or(""));
            if catalogue.owner.get(&key) != Some(&run.name)
                || !corpus::holds(&row, &["family"], &family)
                || !corpus::holds(&row, &["title", "heading", "text"], &words)
            {
                continue;
            }
            let value = paragraph(
                &row,
                run,
                &key,
                line + 1,
                &catalogue,
                context.limit("SEARCH_TEXT_CHARS", 1200),
            )?;
            corpus::retain(
                &mut best,
                Ranked {
                    key: (
                        corpus::tier(&words, &["title", "heading"], &["text"], &row),
                        family_priority(&row),
                        order,
                    ),
                    value,
                },
                offset + limit + 1,
            );
            order += 1;
        }
    }
    let best = best.into_sorted_vec();
    let more = best.len() > offset + limit;
    let matches = best
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|v| v.value)
        .collect::<Vec<_>>();
    let empty = matches.is_empty();
    let mut result = page(matches, offset + limit, more, policy("NOTICE"), 10000);
    result["corpus"] = provenance(&catalogue);
    if empty && offset == 0 {
        result["notice"] = json!(format!("{} {}", policy("NOTICE"), policy("NO_MATCH_HINT")));
    }
    Ok(result)
}
fn read(context: &Context, input: &Value) -> Result<Value, String> {
    let reference = input["row_ref"].as_str().unwrap();
    if !ROW_REF.fullmatch(reference) {
        return Err("Use a row_ref returned by search_ato_rulings.".into());
    }
    let captures = ROW_REF.captures(reference).unwrap();
    let (name, key, target) = (
        captures.name("run").unwrap(),
        captures.name("docid").unwrap(),
        captures.name("line").unwrap().parse::<usize>().unwrap(),
    );
    let catalogue = catalogue(context)?;
    after_catalogue(context, &catalogue, true);
    let run = catalogue
        .runs
        .iter()
        .find(|r| r.name == name)
        .ok_or("No rulings run with that name in the configured folder; search again.")?;
    if !run.documents.contains(&key) {
        return Err("That rulings run holds no document with that docid; search again.".into());
    }
    if catalogue.withheld.contains_key(&key) {
        return Err(
            "A later rulings run excluded that document, so no copy of it can be read.".into(),
        );
    }
    let (near, start) = (
        super::position(&input["neighbours"]),
        super::position(&input["start"]),
    );
    let (mut before, mut after, mut found) = (VecDeque::new(), Vec::new(), None);
    let error = "Cannot read that rulings run as UTF-8 JSONL.";
    for (line, raw) in Lines::open(&run.path).map_err(|_| error)?.enumerate() {
        let raw = raw.map_err(|_| error)?;
        let line = line + 1;
        if line < target && near == 0 {
            continue;
        }
        let row = super::parse_json(&raw).map_err(|_| error)?;
        let same = canonical(row["docid"].as_str().ok_or(error)?) == key;
        if line < target {
            if same && near > 0 {
                before.push_back((line, row));
                if before.len() > near {
                    before.pop_front();
                }
            }
            continue;
        }
        if line == target {
            if !same {
                return Err(
                    "The row at that row_ref belongs to another document; search again.".into(),
                );
            }
            found = Some(row);
            if near == 0 {
                break;
            }
            continue;
        }
        if same {
            after.push((line, row));
            if after.len() >= near {
                break;
            }
        }
    }
    let mut found = found
        .ok_or("No paragraph at that row_ref in the configured rulings runs; search again.")?;
    let whole = found["text"].as_str().unwrap_or("");
    let total = whole.chars().count();
    if start > 0 && start >= total {
        return Err(format!(
            "start is past the end of this paragraph, which has {total} characters."
        ));
    }
    let end = start
        .saturating_add(context.limit("READ_TEXT_CHARS", 12000))
        .min(total);
    found["text"] = json!(
        whole
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>()
    );
    let mut selected = paragraph(
        &found,
        run,
        &key,
        target,
        &catalogue,
        context.limit("READ_TEXT_CHARS", 12000),
    )?;
    selected["total_chars"] = json!(total);
    if start > 0 || end < total {
        selected["caveats"].as_array_mut().unwrap().insert(
            0,
            json!(
                policy("PART_CAVEAT")
                    .replace("{start}", &start.to_string())
                    .replace("{end}", &end.to_string())
                    .replace("{total}", &total.to_string())
            ),
        );
    }
    let build = |(line, row): &(usize, Value)| {
        paragraph(
            row,
            run,
            &key,
            *line,
            &catalogue,
            context.limit("SEARCH_TEXT_CHARS", 1200),
        )
    };
    Ok(
        json!({"paragraph":selected,"start":start,"next_start":(end<total).then_some(end),"before":before.iter().map(build).collect::<Result<Vec<_>,_>>()?,"after":after.iter().map(build).collect::<Result<Vec<_>,_>>()?,"corpus":provenance(&catalogue),"notice":policy("NOTICE")}),
    )
}
pub(super) fn evaluate(context: &Context, name: &str, input: &Value) -> Result<Value, String> {
    if name == "search_ato_rulings" {
        search(context, input)
    } else {
        read(context, input)
    }
}
