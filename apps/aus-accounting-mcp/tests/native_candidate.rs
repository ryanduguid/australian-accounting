//! Opt-in qualification of the finished CI archive, after extraction.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

const ENGINES: [&str; 6] = [
    "ato-benchmark-compare",
    "australian-tax-calculators",
    "div7a-loan-review",
    "gst-review",
    "lodgment-calendar",
    "payday-super-checker",
];

fn files(root: &Path) -> BTreeSet<String> {
    fn visit(root: &Path, directory: &Path, found: &mut BTreeSet<String>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            assert!(!kind.is_symlink(), "candidate contains a link");
            if kind.is_dir() {
                visit(root, &entry.path(), found);
            } else {
                assert!(kind.is_file());
                found.insert(
                    entry
                        .path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut found = BTreeSet::new();
    visit(root, root, &mut found);
    found
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("native-candidate-check-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        // The directory was exclusively created above; refuse a replaced root.
        let result = self.0.canonicalize().and_then(|resolved| {
            if resolved == self.0 {
                fs::remove_dir_all(resolved)
            } else {
                Err(std::io::Error::other("owned scratch root changed"))
            }
        });
        if !thread::panicking() {
            result.unwrap();
        }
    }
}

fn run(binary: &Path, arguments: &[&str], cwd: &Path) -> Output {
    let mut command = Command::new(binary);
    command.args(arguments).current_dir(cwd).env("PATH", "");
    for name in aus_accounting_mcp::resources::RETRIEVAL_FOLDERS {
        command.env_remove(name);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let stdout = cwd.parent().unwrap().join("command.stdout");
    let stderr = cwd.parent().unwrap().join("command.stderr");
    let mut child = command
        .stdin(Stdio::null())
        .stdout(fs::File::create(&stdout).unwrap())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    // Context has its own 30-second deadline plus child cleanup. Allow that
    // cleanup first, then terminate the owned tree without pipe-reader joins.
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            #[cfg(windows)]
            let mut terminate = {
                let mut command = Command::new(
                    PathBuf::from(std::env::var_os("SystemRoot").unwrap())
                        .join("System32/taskkill.exe"),
                );
                command.args(["/T", "/F", "/PID", &child.id().to_string()]);
                command
            };
            #[cfg(unix)]
            let mut terminate = {
                let mut command = Command::new("/bin/kill");
                command.args(["-KILL", "--", &format!("-{}", child.id())]);
                command
            };
            let killed = terminate
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = child.kill();
            let status = child.wait().unwrap();
            assert!(
                killed.unwrap().success(),
                "could not terminate owned process tree"
            );
            break status;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let output = Output {
        status,
        stdout: fs::read(stdout).unwrap(),
        stderr: fs::read(stderr).unwrap(),
    };
    assert!(
        !timed_out,
        "candidate command exceeded 45 seconds: {binary:?}"
    );
    assert_eq!(
        fs::read_dir(cwd).unwrap().count(),
        0,
        "command wrote to cwd"
    );
    output
}

fn success(binary: &Path, arguments: &[&str], cwd: &Path) -> String {
    let output = run(binary, arguments, cwd);
    assert!(output.status.success(), "{binary:?}: {:?}", output.stderr);
    assert!(output.stderr.is_empty(), "{binary:?}: {:?}", output.stderr);
    String::from_utf8(output.stdout).unwrap()
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
#[ignore = "requires NATIVE_CANDIDATE_ROOT and NATIVE_CANDIDATE_COMMIT after archive extraction"]
fn extracted_archive_is_complete_and_runs_without_python_or_checkout_assets() {
    let root =
        PathBuf::from(std::env::var_os("NATIVE_CANDIDATE_ROOT").expect("extracted candidate root"))
            .canonicalize()
            .unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    assert!(!root.starts_with(&source), "extract outside the checkout");
    let suffix = std::env::consts::EXE_SUFFIX;
    let binary = |name: &str| root.join(format!("{name}{suffix}"));
    let mut expected = BTreeSet::from([
        format!("aus-accounting-mcp{suffix}"),
        format!("aus-accounting-mcp-demo{suffix}"),
        format!("aus-accounting-mcp-evaluate{suffix}"),
        format!("examples/group_review{suffix}"),
    ]);
    let mut support = vec![
        "apps/aus-accounting-mcp/LICENSE".to_owned(),
        "apps/aus-accounting-mcp/NOTICE".to_owned(),
        "apps/aus-accounting-mcp/DISCLAIMER.md".to_owned(),
        "apps/aus-accounting-mcp/SECURITY.md".to_owned(),
        "apps/aus-accounting-mcp/NATIVE.md".to_owned(),
        "apps/aus-accounting-mcp/examples/GROUP-REVIEW.md".to_owned(),
    ];
    for engine in ENGINES {
        for name in ["LICENSE", "NOTICE", "DISCLAIMER.md"] {
            support.push(format!("packages/{engine}/{name}"));
        }
    }
    for file in support {
        assert_eq!(
            fs::read(root.join(&file)).unwrap(),
            fs::read(source.join(&file)).unwrap(),
            "{file}"
        );
        expected.insert(file);
    }
    let info: Value =
        serde_json::from_slice(&fs::read(root.join("BUILD-INFO.json")).unwrap()).unwrap();
    assert_eq!(info["schema"], "native-ci-candidate.v1");
    assert_eq!(info["component"], "aus-accounting-mcp");
    assert_eq!(info["version"], aus_accounting_mcp::VERSION);
    assert_eq!(
        info["source"]["repository"],
        "ryanduguid/australian-accounting"
    );
    assert_eq!(
        info["source"]["commit"],
        std::env::var("NATIVE_CANDIDATE_COMMIT").unwrap()
    );
    let rustc = Command::new("rustc").arg("-vV").output().unwrap();
    assert!(rustc.status.success());
    let rustc = String::from_utf8(rustc.stdout).unwrap();
    assert_eq!(
        info["target"],
        rustc
            .lines()
            .find_map(|l| l.strip_prefix("host: "))
            .unwrap()
    );
    assert_eq!(info["rustc"], rustc.lines().next().unwrap());
    let mut recorded = BTreeMap::new();
    for entry in info["files"].as_array().unwrap() {
        let path = entry["path"].as_str().unwrap();
        assert!(expected.contains(path), "unexpected recorded path: {path}");
        assert!(
            recorded
                .insert(
                    path.to_owned(),
                    entry["sha256"].as_str().unwrap().to_owned()
                )
                .is_none()
        );
    }
    assert_eq!(recorded.keys().cloned().collect::<BTreeSet<_>>(), expected);
    for (path, digest) in recorded {
        assert_eq!(
            payday_super_checker::rates::hex(&Sha256::digest(fs::read(root.join(&path)).unwrap())),
            digest,
            "{path}"
        );
    }
    expected.insert("BUILD-INFO.json".to_owned());
    assert_eq!(files(&root), expected);

    let scratch = Scratch::new();
    let cwd = scratch.0.join("empty");
    fs::create_dir(&cwd).unwrap();
    assert_eq!(
        success(&binary("aus-accounting-mcp"), &["--version"], &cwd),
        format!("aus-accounting-mcp {}\n", aus_accounting_mcp::VERSION)
    );
    let demo = success(&binary("aus-accounting-mcp-demo"), &[], &cwd);
    let demo: Value = serde_json::from_str(demo.split_once('\n').unwrap().1).unwrap();
    let mut proof: Value = serde_json::from_str(
        include_str!("../docs/quick-proof.txt")
            .split_once('\n')
            .unwrap()
            .1,
    )
    .unwrap();
    proof["calls"][1]["result"]["engine_version"] = json!(div7a_loan_review::VERSION);
    proof["calls"][1]["result"]["source"]["rate_table_uris"][0]["uri"] =
        json!("data/benchmark_rates.csv");
    assert_eq!(demo, proof, "complete packaged proof");

    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/native/evaluation.json")).unwrap();
    let evaluator = binary("aus-accounting-mcp-evaluate");
    assert_eq!(
        success(&evaluator, &["questions"], &cwd),
        fixture["questions"].as_str().unwrap()
    );
    let context = success(&evaluator, &["context"], &cwd);
    let marker = "# Preloaded scope resource\n\naus-accounting://scope\n\n";
    let (definitions, scope) = context.split_once(marker).unwrap();
    assert_eq!(
        definitions,
        fixture["context"]
            .as_str()
            .unwrap()
            .split_once(marker)
            .unwrap()
            .0
    );
    let scope: Value = serde_json::from_str(scope).unwrap();
    let mut expected_scope = aus_accounting_mcp::resources::read("aus-accounting://scope").unwrap();
    for name in aus_accounting_mcp::resources::RETRIEVAL_FOLDERS {
        expected_scope["retrieval_folders_configured"][name] = json!(false);
    }
    assert_eq!(scope, expected_scope, "real sibling MCP scope");
    let case = &fixture["cases"][0];
    assert_eq!(case["label"], "all-36-complete");
    let recording = scratch.0.join("recording.json");
    fs::write(&recording, case["recording"].as_str().unwrap()).unwrap();
    assert_eq!(
        success(
            &evaluator,
            &[
                "score",
                recording.to_str().unwrap(),
                "--reference-sha256",
                fixture["reference_sha256"].as_str().unwrap()
            ],
            &cwd
        ),
        case["stdout"].as_str().unwrap()
    );

    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/native/group-review.json")).unwrap();
    let case = &fixture["cases"][0];
    assert_eq!(case["label"], "complete-independent-reports");
    let reports = scratch.0.join("reports");
    fs::create_dir(&reports).unwrap();
    for file in case["files"].as_array().unwrap() {
        fs::write(
            reports.join(file["path"].as_str().unwrap()),
            unhex(file["hex"].as_str().unwrap()),
        )
        .unwrap();
    }
    let manifest = reports.join("manifest.json");
    fs::write(&manifest, unhex(case["manifest_hex"].as_str().unwrap())).unwrap();
    let example = binary("examples/group_review");
    let index = success(&example, &[manifest.to_str().unwrap()], &cwd);
    assert_eq!(
        serde_json::from_str::<Value>(&index).unwrap(),
        case["expected"]
    );
    fs::write(reports.join("report-0.txt"), "Changed synthetic report").unwrap();
    let refused = run(&example, &[manifest.to_str().unwrap()], &cwd);
    assert_eq!(refused.status.code(), Some(1));
    assert!(refused.stdout.is_empty());
    assert!(!refused.stderr.is_empty());
    assert_eq!(
        files(&root),
        expected,
        "commands changed the installed file list"
    );
}
