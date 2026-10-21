//! Distribution checks use a copied server, registered tools and an empty PATH.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "native-mcp-installation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let cleanup = self.0.canonicalize().and_then(|resolved| {
            if resolved == self.0 {
                fs::remove_dir_all(resolved)
            } else {
                Err(std::io::Error::other("owned temporary path changed"))
            }
        });
        if !thread::panicking() {
            cleanup.unwrap();
        }
    }
}

struct Server {
    child: Child,
    lines: mpsc::Receiver<String>,
    reader: Option<thread::JoinHandle<()>>,
    next_id: u32,
}
impl Server {
    fn start(binary: &Path, cwd: &Path, rulings: Option<&Path>) -> Self {
        let mut command = Command::new(binary);
        command.current_dir(cwd).env("PATH", "");
        for name in aus_accounting_mcp::resources::RETRIEVAL_FOLDERS {
            command.env_remove(name);
        }
        if let Some(root) = rulings {
            command.env("AUS_ACCOUNTING_RULINGS_ROOT", root);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, lines) = mpsc::channel();
        let reader = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let mut server = Self {
            child,
            lines,
            reader: Some(reader),
            next_id: 0,
        };
        let hello = server.request(
            "initialize",
            json!({
                "protocolVersion":"2025-11-25", "capabilities":{},
                "clientInfo":{"name":"native-installation-check","version":"1"}
            }),
        );
        assert_eq!(hello["serverInfo"]["version"], aus_accounting_mcp::VERSION);
        server.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        server
    }
    fn send(&mut self, request: Value) {
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{request}").unwrap();
        stdin.flush().unwrap();
    }
    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let line = self
            .lines
            .recv_timeout(Duration::from_secs(10))
            .expect("bounded MCP reply");
        let reply: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(reply["id"], id, "{reply}");
        assert!(reply.get("error").is_none(), "{reply}");
        reply.get("result").expect("JSON-RPC result").clone()
    }
    fn resource(&mut self, uri: &str) -> Value {
        let result = self.request("resources/read", json!({"uri":uri}));
        assert_eq!(result["contents"].as_array().unwrap().len(), 1);
        assert_eq!(result["contents"][0]["uri"], uri);
        serde_json::from_str(result["contents"][0]["text"].as_str().unwrap()).unwrap()
    }
    fn tool(&mut self, name: &str, arguments: Value) -> Value {
        let result = self.request("tools/call", json!({"name":name,"arguments":arguments}));
        assert_eq!(result["isError"], false, "{name}: {result}");
        let data = result["structuredContent"]
            .as_object()
            .expect("structured evidence");
        let text: Value =
            serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(
            &text,
            &Value::Object(data.clone()),
            "text and structured evidence"
        );
        Value::Object(data.clone())
    }
    fn finish(&mut self) {
        self.child.stdin.take();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < deadline, "server did not exit after EOF");
            thread::sleep(Duration::from_millis(10));
        }
        self.reader.take().unwrap().join().unwrap();
        assert!(self.lines.try_recv().is_err(), "unsolicited reply");
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.child.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
fn installed_server(root: &Path) -> (PathBuf, PathBuf) {
    let bin = root.join("bin");
    let empty = root.join("empty");
    fs::create_dir(&bin).unwrap();
    fs::create_dir(&empty).unwrap();
    let server = bin.join(format!(
        "aus-accounting-mcp{}",
        std::env::consts::EXE_SUFFIX
    ));
    fs::copy(env!("CARGO_BIN_EXE_aus-accounting-mcp"), &server).unwrap();
    (server, empty)
}

#[test]
fn installed_binary_exercises_every_advertised_worksheet_period() {
    let directory = Directory::new();
    let (binary, empty) = installed_server(&directory.0);
    let mut server = Server::start(&binary, &empty, None);
    let versions = server.resource("aus-accounting://component-versions");
    assert_eq!(versions["server"]["version"], aus_accounting_mcp::VERSION);
    let engines = versions["engines"].as_array().unwrap();
    let expected_versions = [
        ("ato-benchmark-compare", ato_benchmark_compare::VERSION),
        ("payday-super-checker", payday_super_checker::VERSION),
        ("div7a-loan-review", div7a_loan_review::VERSION),
        (
            "australian-tax-calculators",
            australian_tax_calculators::engine::VERSION,
        ),
        ("gst-review", gst_review::VERSION),
        ("lodgment-calendar", lodgment_calendar::VERSION),
    ];
    assert_eq!(engines.len(), expected_versions.len());
    for (name, version) in expected_versions {
        let matched: Vec<_> = engines
            .iter()
            .filter(|e| e["distribution"] == name)
            .collect();
        assert_eq!(matched.len(), 1, "{name}");
        assert_eq!(matched[0]["version"], version, "{name}");
    }
    let scope = server.resource("aus-accounting://scope");
    let catalogue = scope["calculation_worksheets"].as_object().unwrap();
    let frozen: Value =
        serde_json::from_str(include_str!("fixtures/native/resource-contracts.json")).unwrap();
    let original = frozen["aus-accounting://scope"]["calculation_worksheets"]
        .as_object()
        .unwrap();
    assert_eq!(catalogue.len(), 14);
    assert_eq!(
        catalogue.keys().collect::<Vec<_>>(),
        original.keys().collect::<Vec<_>>()
    );
    let mut calls = 0;
    for (kind, entry) in catalogue {
        assert_eq!(
            entry["supported_periods"], original[kind]["supported_periods"],
            "{kind}"
        );
        assert_eq!(
            entry["example"]["facts"], original[kind]["example"]["facts"],
            "{kind}"
        );
        let periods = entry["supported_periods"].as_array().unwrap();
        assert!(!periods.is_empty(), "{kind}");
        for period in periods {
            let mut facts = entry["example"]["facts"].as_object().unwrap().clone();
            facts.extend(period["arguments"].as_object().unwrap().clone());
            let result = server.tool("calculate_tax_worksheet", json!({"facts":facts}));
            assert_eq!(result["calculation"], kind.as_str());
            assert_eq!(result["source_checked"], entry["source_checked"], "{kind}");
            assert_eq!(
                result["engine_version"],
                australian_tax_calculators::engine::VERSION
            );
            assert!(!result["warnings"].as_array().unwrap().is_empty(), "{kind}");
            calls += 1;
        }
    }
    assert_eq!(
        calls,
        original
            .values()
            .map(|e| e["supported_periods"].as_array().unwrap().len())
            .sum::<usize>()
    );
    server.finish();
    assert_eq!(fs::read_dir(&empty).unwrap().count(), 0);
    println!(
        "Installed native server: {} worksheet kinds, {calls} supported-period calls, 6 linked engine identities; synthetic examples, not tax validation.",
        catalogue.len()
    );
}

fn builder_run(binary: &Path, root: &Path, response: Value, out: &Path) -> i32 {
    fs::create_dir(root).unwrap();
    let script = root.join("http.json");
    let log = root.join("http.log");
    let targets = root.join("targets.json");
    fs::write(
        &script,
        json!({"today":"2099-01-01","responses":[response]}).to_string(),
    )
    .unwrap();
    fs::write(&targets, json!(["TXR/TR20991/NAT/ATO/00001"]).to_string()).unwrap();
    let mut command = Command::new(binary);
    command
        .args(["rulings"])
        .arg(&targets)
        .arg("--out")
        .arg(out)
        .current_dir(root)
        .env("PATH", "")
        .env("FADDEN_FAKE_HTTP", &script)
        .env("FADDEN_FAKE_HTTP_LOG", &log)
        .env("NO_PROXY", "")
        .env("no_proxy", "");
    // The builder's own differential harness uses this feature probe. A normal
    // build cannot pass it; proxies also keep a mistaken build off the network.
    for name in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ] {
        command.env(name, "http://127.0.0.1:9");
    }
    struct OwnedChild(Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = OwnedChild(
        command
            .stdin(Stdio::null())
            .stdout(fs::File::create(root.join("stdout.log")).unwrap())
            .stderr(fs::File::create(root.join("stderr.log")).unwrap())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "builder timed out");
        thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(
        fs::read_to_string(log).expect("fake-http build must log its scripted request"),
        "GET https://www.ato.gov.au/law/view/document?docid=TXR/TR20991/NAT/ATO/00001\n"
    );
    status.code().expect("builder exit status")
}

#[test]
#[ignore = "requires FADDEN_SETUP_BINARY built with the corpus builder's fake-http feature"]
fn native_builder_fabricated_page_reaches_registered_mcp_citation_tools() {
    let retained = std::env::var_os("FADDEN_SETUP_OUT").map(PathBuf::from);
    if let Some(path) = &retained {
        assert!(path.is_absolute(), "FADDEN_SETUP_OUT must be absolute");
        assert!(
            matches!(fs::symlink_metadata(path), Err(error) if error.kind() == std::io::ErrorKind::NotFound),
            "FADDEN_SETUP_OUT must name an absent directory with an existing parent"
        );
    }
    let builder = std::env::var_os("FADDEN_SETUP_BINARY")
        .expect("set the exact fake-http fadden binary path");
    let builder = PathBuf::from(builder).canonicalize().unwrap();
    assert!(builder.is_file());
    let directory = Directory::new();
    let (binary, empty) = installed_server(&directory.0);
    let copied_builder = binary.with_file_name(format!("fadden{}", std::env::consts::EXE_SUFFIX));
    fs::copy(builder, &copied_builder).unwrap();
    // Require an observed scripted 404 before supplying the fabricated HTML.
    assert_eq!(
        builder_run(
            &copied_builder,
            &directory.0.join("probe"),
            json!({"status":404}),
            &directory.0.join("probe-run")
        ),
        1
    );
    let corpus = retained.unwrap_or_else(|| directory.0.join("corpus"));
    fs::create_dir(&corpus).unwrap();
    let corpus = corpus.canonicalize().unwrap();
    let run = corpus.join("run-synthetic");
    let html = include_str!("fixtures/native/setup-ruling.html");
    assert_eq!(
        builder_run(
            &copied_builder,
            &directory.0.join("build"),
            json!({"status":200,"type":"text/html; charset=utf-8","body":html}),
            &run
        ),
        0,
        "{}",
        fs::read_to_string(directory.0.join("build/stderr.log")).unwrap()
    );
    let mut server = Server::start(&binary, &empty, Some(&corpus));
    let found = server.tool(
        "search_ato_rulings",
        json!({"query":"Synthetic setup evidence"}),
    );
    let matches = found["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "{found}");
    let matched = &matches[0];
    assert_eq!(matched["docid"], "TXR/TR20991/NAT/ATO/00001");
    assert_eq!(
        matched["text"],
        concat!(
            "1. Synthetic setup evidence only. ",
            "You are free to copy, adapt, modify, transmit and distribute this material as you wish ",
            "(but not in any way that suggests the ATO or the Commonwealth endorses you or any of your services or products)."
        )
    );
    assert_eq!(
        matched["source_sha256"],
        payday_super_checker::rates::hex(&Sha256::digest(html.as_bytes()))
    );
    assert_eq!(matched["fetched_on"], "2099-01-01");
    assert_eq!(
        matched["source_url"],
        "https://www.ato.gov.au/law/view/document?docid=TXR/TR20991/NAT/ATO/00001"
    );
    assert_eq!(matched["withdrawal_status"], "UNKNOWN");
    let read = server.tool("read_ato_ruling", json!({"row_ref":matched["row_ref"]}));
    assert_eq!(
        read["paragraph"], *matched,
        "search citation must resolve to the same complete paragraph"
    );
    assert!(!found["notice"].as_str().unwrap().is_empty());
    server.finish();
    assert_eq!(fs::read_dir(&empty).unwrap().count(), 0);
    // The explicitly selected evidence directory survives success or failure.
    // Default runs remain temporary, including this inspection report.
    let report = json!({
        "synthetic": true,
        "network_requests": 0,
        "scope": "Fabricated HTML through the actual native builder and registered MCP retrieval tools.",
        "limits": "No live capture, current-law verification, model behaviour or publication is established.",
        "search": found,
        "read": read
    });
    {
        let mut file = fs::File::options()
            .write(true)
            .create_new(true)
            .open(corpus.join("setup-report.json"))
            .unwrap();
        serde_json::to_writer_pretty(&mut file, &report).unwrap();
        file.write_all(b"\n").unwrap();
    }
    println!(
        "Fabricated HTML passed through the actual native builder and registered MCP search/read citation; scripted HTTP only, no live capture or current-law validation."
    );
}
