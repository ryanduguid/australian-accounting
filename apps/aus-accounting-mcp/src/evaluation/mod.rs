mod json;

use indexmap::IndexMap;
use json::Json;
use payday_super_checker::py::is_space;
use rmcp::{
    ServiceExt,
    model::{ReadResourceRequestParams, ResourceContents, Tool},
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

const REFERENCE: &[u8] = include_bytes!("../../evaluation/questions.xml");
const SCOPE: &str = "aus-accounting://scope";
const HELP: &str = "Offline supplementary tool-selection evaluation. Recordings are never executed.\n\nUsage: aus-accounting-mcp-evaluate [--reference PATH] <context|questions|score RECORDING [--reference-sha256 SHA]>\n\ncontext    Print actual native server instructions, tools and preloaded scope.\nquestions  Print questions without tools or answers.\nscore      Compare recorded calls and answers; an imperfect valid score exits 0.\n\nThe default reference is embedded. Context launches the sibling aus-accounting-mcp executable.\n";

fn strip(value: &str) -> &str {
    value.trim_matches(is_space)
}
fn child<'a, 'i>(node: roxmltree::Node<'a, 'i>, tag: &str) -> Option<roxmltree::Node<'a, 'i>> {
    node.children().find(|n| {
        n.is_element() && n.tag_name().namespace().is_none() && n.tag_name().name() == tag
    })
}
fn tagged(node: roxmltree::Node<'_, '_>, tag: &str) -> bool {
    node.is_element() && node.tag_name().namespace().is_none() && node.tag_name().name() == tag
}
fn element_text(node: Option<roxmltree::Node<'_, '_>>) -> String {
    // ElementTree discards comments/PIs and joins their surrounding text, but
    // findtext excludes nested element content and tails after that element.
    node.into_iter()
        .flat_map(|n| n.children())
        .take_while(|n| !n.is_element())
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect()
}
fn xml(bytes: &[u8]) -> Result<roxmltree::Document<'_>, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    // The embedded reference needs no DTD. Refuse DTDs before any entity
    // expansion; roxmltree's per-entity guards do not cap total amplification.
    roxmltree::Document::parse(text).map_err(|e| e.to_string())
}
fn validate(recorded: &Json) -> Result<&IndexMap<String, Json>, String> {
    let cases = recorded
        .object()
        .ok_or("recording must be an object keyed by question id")?;
    for (case, entry) in cases {
        if entry
            .array()
            .is_some_and(|a| a.iter().all(|n| n.string().is_some()))
        {
            continue;
        }
        let fail =
            || format!("{case}: supply calls and a string answer, or a legacy tool-name list");
        let object = entry.object().ok_or_else(fail)?;
        if object.len() != 2
            || !object.contains_key("calls")
            || !object.contains_key("answer")
            || object["answer"].string().is_none()
        {
            return Err(fail());
        }
        let calls = object["calls"].array().ok_or_else(fail)?;
        for call in calls {
            if !call.object().is_some_and(|c| {
                c.len() == 2
                    && c.get("name").and_then(Json::string).is_some()
                    && c.get("arguments").and_then(Json::object).is_some()
            }) {
                return Err(format!(
                    "{case}: each call requires a string name and an arguments object"
                ));
            }
        }
    }
    Ok(cases)
}

struct Reference {
    id: String,
    calls: Json,
    answer: String,
}
fn references(bytes: &[u8]) -> Result<Vec<Reference>, String> {
    let document = xml(bytes)?;
    let mut result = Vec::new();
    let mut ids = BTreeSet::new();
    for pair in document
        .root_element()
        .children()
        .filter(|n| tagged(*n, "qa_pair"))
    {
        let id = pair.attribute("id").unwrap_or("");
        if id.is_empty() || !ids.insert(id) {
            return Err("reference question IDs must be present and unique".into());
        }
        if ["question", "calls", "answer"]
            .iter()
            .any(|tag| pair.children().filter(|n| tagged(*n, tag)).count() != 1)
        {
            return Err(format!(
                "{id}: reference question, calls and answer must appear once"
            ));
        }
        if strip(&element_text(child(pair, "question"))).is_empty() {
            return Err(format!("{id}: reference question must not be blank"));
        }
        let calls = json::parse(&element_text(child(pair, "calls")))?;
        let answer = element_text(child(pair, "answer"));
        let entry = Json::Object(IndexMap::from([
            ("calls".into(), calls.clone()),
            ("answer".into(), Json::String(answer.clone())),
        ]));
        validate(&Json::Object(IndexMap::from([(id.into(), entry)])))?;
        let declared: Vec<_> = pair
            .children()
            .filter(|n| tagged(*n, "tools"))
            .flat_map(|n| n.children().filter(|n| tagged(*n, "tool")))
            .map(|n| element_text(Some(n)))
            .collect();
        let declared_set: BTreeSet<_> = declared.iter().map(String::as_str).collect();
        if declared.len() != declared_set.len() || declared_set != names(&calls) {
            return Err(format!("{id}: reference tool set differs from calls"));
        }
        result.push(Reference {
            id: id.into(),
            calls,
            answer,
        });
    }
    if result.is_empty() {
        return Err("reference must contain questions".into());
    }
    Ok(result)
}
fn names(calls: &Json) -> BTreeSet<&str> {
    calls
        .array()
        .unwrap()
        .iter()
        .map(|c| c.object().unwrap()["name"].string().unwrap())
        .collect()
}
fn questions(bytes: &[u8]) -> Result<String, String> {
    let document = xml(bytes)?;
    // The original questions command is deliberately separate from strict scoring.
    let mut questions = IndexMap::new();
    for pair in document
        .root_element()
        .children()
        .filter(|n| tagged(*n, "qa_pair"))
    {
        let id = pair.attribute("id").ok_or("reference question has no id")?;
        let question = element_text(child(pair, "question"))
            .split(is_space)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        questions.insert(id, question);
    }
    Ok(questions
        .into_iter()
        .map(|(id, q)| format!("{id}: {q}\n"))
        .collect())
}
fn score(
    recorded: &IndexMap<String, Json>,
    bytes: &[u8],
    frozen: Option<&str>,
) -> Result<(i32, String, String), String> {
    let references = references(bytes)?;
    let digest: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if frozen.is_some_and(|sha| sha != digest) {
        return Err("reference digest differs from the frozen trial".into());
    }
    let ids: BTreeSet<_> = references.iter().map(|r| r.id.as_str()).collect();
    let mut unknown = recorded
        .keys()
        .filter(|id| !ids.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    unknown.sort();
    if !unknown.is_empty() {
        return Ok((
            2,
            String::new(),
            format!("not questions in this evaluation: {}\n", unknown.join(", ")),
        ));
    }
    let width = references
        .iter()
        .map(|r| r.id.chars().count())
        .max()
        .unwrap();
    let mut output = String::new();
    let mut exact = 0;
    let mut verified = 0;
    for reference in &references {
        let id = format!(
            "{}{}",
            reference.id,
            " ".repeat(width - reference.id.chars().count())
        );
        let Some(entry) = recorded.get(&reference.id) else {
            output.push_str(&format!("{id}  NOT ANSWERED\n"));
            continue;
        };
        let detailed = entry.object();
        let selected = if let Some(object) = detailed {
            names(&object["calls"])
        } else {
            entry
                .array()
                .unwrap()
                .iter()
                .map(|v| v.string().unwrap())
                .collect()
        };
        let expected = names(&reference.calls);
        let missed: Vec<_> = expected.difference(&selected).copied().collect();
        let extra: Vec<_> = selected.difference(&expected).copied().collect();
        if missed.is_empty() && extra.is_empty() {
            exact += 1;
        }
        let mut details = Vec::new();
        if !missed.is_empty() {
            details.push(format!("missed {}", missed.join(", ")));
        }
        if !extra.is_empty() {
            details.push(format!("also called {}", extra.join(", ")));
        }
        if let Some(object) = detailed {
            if object["calls"] != reference.calls {
                details.push("calls differ (arguments, order or count)".into());
            }
            if strip(object["answer"].string().unwrap()) != strip(&reference.answer) {
                details.push("answer differs".into());
            }
            if details.is_empty() {
                verified += 1;
            }
        } else {
            details.push("arguments and answer NOT EVALUATED".into());
        }
        output.push_str(&format!(
            "{id}  {}\n",
            if details.is_empty() {
                "calls and answer match".into()
            } else {
                details.join("; ")
            }
        ));
    }
    output.push_str(&format!("\nreference_sha256 {digest}\n{exact} of {} questions selected exactly the published tools.\n{verified} of {} recorded calls and answers match the reference.\n", references.len(), references.len()));
    Ok((0, output, String::new()))
}

fn schema(value: serde_json::Value) -> Result<String, String> {
    let mut value = value;
    value.sort_all_objects();
    let pretty = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    // Python's json.dumps uses ensure_ascii=True, including UTF-16 surrogate pairs.
    let mut ascii = String::new();
    for c in pretty.chars() {
        if c.is_ascii() {
            ascii.push(c);
        } else {
            for unit in c.encode_utf16(&mut [0; 2]) {
                ascii.push_str(&format!("\\u{unit:04x}"));
            }
        }
    }
    Ok(ascii)
}
fn format_context(instructions: &str, tools: Vec<Tool>, scope: &str) -> Result<String, String> {
    let mut output = format!("# Server instructions\n\n{instructions}\n\n# Tools\n\n");
    for tool in tools {
        output.push_str(&format!(
            "## {}{}\n\n{}\n\nInput schema:\n\n{}\n\nOutput schema:\n\n{}\n\n",
            tool.name,
            tool.title
                .filter(|s| !s.is_empty())
                .map(|s| format!(" ({s})"))
                .unwrap_or_default(),
            strip(tool.description.as_deref().unwrap_or("")),
            schema(serde_json::to_value(tool.input_schema).map_err(|e| e.to_string())?)?,
            schema(serde_json::to_value(tool.output_schema).map_err(|e| e.to_string())?)?
        ));
    }
    output.push_str(&format!(
        "# Preloaded scope resource\n\n{SCOPE}\n\n{scope}\n\n"
    ));
    Ok(output)
}
async fn describe(server: &Path, deadline: Duration) -> Result<String, String> {
    let mut child = tokio::process::Command::new(server)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("{}: {e}", server.display()))?;
    let transport = (child.stdout.take().unwrap(), child.stdin.take().unwrap());
    let result = tokio::time::timeout(deadline, async {
        let mut service = ().serve(transport).await.map_err(|e| e.to_string())?;
        let result = async {
            let instructions = service
                .peer_info()
                .and_then(|p| p.instructions.clone())
                .unwrap_or_default();
            let tools = service
                .list_tools(None)
                .await
                .map_err(|e| e.to_string())?
                .tools;
            let scope = service
                .read_resource(ReadResourceRequestParams::new(SCOPE))
                .await
                .map_err(|e| e.to_string())?;
            let Some(ResourceContents::TextResourceContents { text, .. }) = scope.contents.first()
            else {
                return Err("scope resource has no text".into());
            };
            format_context(&instructions, tools, text)
        }
        .await;
        service
            .close_with_timeout(Duration::from_secs(2))
            .await
            .map_err(|e| e.to_string())?;
        result
    })
    .await
    .map_err(|_| "native server context timed out".to_owned())
    .and_then(|r| r);
    // Always reap the owned child, including failed handshakes and request timeouts.
    match tokio::time::timeout(Duration::from_secs(2), child.wait()).await {
        Ok(status) => {
            status.map_err(|e| e.to_string())?;
        }
        Err(_) => {
            child.kill().await.map_err(|e| e.to_string())?;
            child.wait().await.map_err(|e| e.to_string())?;
        }
    }
    result
}

pub async fn main(args: Vec<OsString>) -> i32 {
    let mut args = args.into_iter();
    let mut reference = None;
    let Some(mut command) = args.next() else {
        eprint!("{HELP}");
        return 2;
    };
    if command == "--reference" {
        let Some(path) = args.next() else {
            eprintln!("--reference requires a path");
            return 2;
        };
        reference = Some(PathBuf::from(path));
        let Some(next) = args.next() else {
            eprint!("{HELP}");
            return 2;
        };
        command = next;
    }
    if command == "--help" || command == "-h" {
        print!("{HELP}");
        return 0;
    }
    if command == "--version" {
        println!("aus-accounting-mcp-evaluate {}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    let rest: Vec<_> = args.collect();
    if ["context", "questions", "score"]
        .iter()
        .any(|c| command == *c)
        && rest
            .iter()
            .take_while(|a| *a != "--")
            .any(|a| a == "--help" || a == "-h")
    {
        print!("{HELP}");
        return 0;
    }
    let result = if command == "context" && rest.is_empty() {
        let server = std::env::current_exe().map(|p| {
            p.with_file_name(format!(
                "aus-accounting-mcp{}",
                std::env::consts::EXE_SUFFIX
            ))
        });
        match server {
            Ok(path) => describe(&path, Duration::from_secs(30))
                .await
                .map(|s| (0, s, String::new())),
            Err(e) => Err(e.to_string()),
        }
    } else if command == "questions" && rest.is_empty() {
        reference
            .as_ref()
            .map(std::fs::read)
            .unwrap_or_else(|| Ok(REFERENCE.to_vec()))
            .map_err(|e| e.to_string())
            .and_then(|b| questions(&b))
            .map(|s| (0, s, String::new()))
    } else if command == "score" {
        let Ok((recording, frozen)) = scoring_args(&rest) else {
            eprint!("{HELP}");
            return 2;
        };
        // Parsing, equality and destruction all recurse. Reserve the larger
        // stack for the complete operation, including all error paths and drops.
        stacker::grow(16 * 1024 * 1024, || {
            let result = std::fs::read_to_string(recording)
                .map_err(|e| e.to_string())
                .and_then(|text| {
                    let recorded = json::parse(&text)?;
                    let recorded = validate(&recorded)?;
                    let bytes = reference
                        .as_ref()
                        .map(std::fs::read)
                        .unwrap_or_else(|| Ok(REFERENCE.to_vec()))
                        .map_err(|e| e.to_string())?;
                    score(recorded, &bytes, frozen)
                });
            match result {
                Ok(output) => Ok(output),
                Err(e) => Ok((2, String::new(), format!("invalid recording: {e}\n"))),
            }
        })
    } else {
        eprint!("{HELP}");
        return 2;
    };
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
