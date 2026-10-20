//! Guards binding the documentation to the behaviour it describes: the
//! primary-source review and the claims it settled, the limitations register,
//! the evaluation packs' guides, the OpenAccountants handoff and the
//! remittance-only example. A document that no longer matches the engine
//! fails here.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const LOCKED_CALENDAR_COMMAND: &str = "uv run --locked python packages/payday-super-checker/tools/generate_calendar.py > packages/payday-super-checker/data/business_days.json";

fn read(relative: &str) -> String {
    std::fs::read_to_string(format!("{ROOT}/{relative}")).unwrap()
}

fn squeeze(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Run the checker from the package directory: (exit code, stdout).
fn check(args: &[&str]) -> (i32, String) {
    static RUNS: AtomicUsize = AtomicUsize::new(0);
    let run = RUNS.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("payday-docs-{}-{run}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_payday-super-check"))
        .current_dir(ROOT)
        .args(args)
        .arg("-o")
        .arg(dir.join("report.csv"))
        .env("COLUMNS", "80")
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap().replace("\r\n", "\n"),
    )
}

#[test]
fn the_rounding_authority_and_experimental_boundary_stay_visible() {
    let (design, review, readme) = (
        read("docs/design.md"),
        read("docs/primary-source-review-2026-08-15.md"),
        read("README.md"),
    );
    for document in [&design, &review, &readme] {
        for phrase in ["ROUND_HALF_UP", "TAA 1953 s 16B", "final assessed"] {
            assert!(document.contains(phrase), "{phrase}");
        }
        assert!(document.to_lowercase().contains("experimental"));
    }
    assert!(design.contains("nearest multiple of 5 cents"));
    assert!(review.contains("implementation choice"));
}

#[test]
fn the_final_out_of_cycle_determination_is_pinned_in_user_guidance() {
    let readme = read("README.md");
    let review = read("docs/primary-source-review-2026-08-15.md");
    assert!(readme.contains("F2026L00784"));
    for kind in [
        "allowances",
        "bonuses",
        "commissions",
        "loadings",
        "payments in advance",
        "back payments",
    ] {
        assert!(readme.contains(kind), "{kind}");
    }
    assert!(readme.contains("established timing, pattern or schedule"));
    assert!(readme.contains("subsequent"));
    for phrase in [
        "LI 2026/20",
        "does not display that shorthand",
        "registered identifier and text control",
    ] {
        assert!(review.contains(phrase), "{phrase}");
    }
}

#[test]
fn the_final_ruling_status_and_transition_gate_are_current() {
    let review = read("docs/primary-source-review-2026-08-15.md");
    let readme = read("README.md");
    let design = read("docs/design.md");
    for document in [&review, &readme, &design] {
        for phrase in [
            "LCR 2026/1",
            "LCR 2026/2",
            "LCR 2026/3",
            "LCR 2026/D1",
            "draft",
            "--confirm-transition-allocation",
        ] {
            assert!(document.contains(phrase), "{phrase}");
        }
    }
    assert!(readme.contains("--confirm-remittance-only"));
    assert!(design.contains("--confirm-remittance-only"));
    assert!(review.contains("5 August 2026"));
    assert!(review.contains("Department of Education v Commissioner of Taxation"));
}

#[test]
fn the_item_4_and_import_allocation_boundaries_stay_visible() {
    for document in [
        read("docs/primary-source-review-2026-08-15.md"),
        read("README.md"),
        read("docs/design.md"),
    ] {
        for phrase in [
            "LCR 2026/2",
            "fund-receipt order",
            "earliest",
            "shortfall",
            "--confirm-statutory-allocation",
            "on-time fund receipt",
            "UNKNOWN",
        ] {
            assert!(document.contains(phrase), "{phrase}");
        }
        assert!(document.to_lowercase().contains("item 4"));
    }
}

#[test]
fn regulations_11_and_12_are_an_operator_input_boundary() {
    for document in [
        read("docs/primary-source-review-2026-08-15.md"),
        read("README.md"),
        read("docs/design.md"),
    ] {
        let lower = document.to_lowercase();
        assert!(lower.contains("regulations 11 and 12"));
        assert!(lower.contains("operator"));
        assert!(document.contains("sg_amount"));
    }
}

#[test]
fn the_primary_review_pins_regulations_and_direct_mcb_provenance() {
    let review = read("docs/primary-source-review-2026-08-15.md");
    let rates = read("data/rates.json");
    for regulation in ["11", "12", "13", "13A", "13B", "13C", "13D"] {
        assert!(
            review.contains(&format!("Regulation {regulation}")),
            "{regulation}"
        );
    }
    assert!(review.contains("F2026C00535"));
    assert!(review.contains("$270,830"));
    assert!(rates.contains("\"seen\": \"2026-08-15\""));
    assert!(rates.contains("\"cross_check\""));
    assert!(!rates.contains("search-result snippets"));
}

#[test]
fn the_calendar_authority_horizon_and_exclusions_are_pinned() {
    let review = squeeze(&read("docs/primary-source-review-2026-08-15.md"));
    let calendar = read("data/business_days.json");
    let data: Value = serde_json::from_str(&calendar).unwrap();
    assert!(calendar.contains("\"verified_until\": \"2027-08-31\""));
    assert!(calendar.contains("\"checked\": \"2026-08-15\""));
    for jurisdiction in ["ACT", "NSW", "NT", "QLD", "SA", "TAS", "VIC", "WA"] {
        assert!(calendar.contains(&format!("\"{jurisdiction}\": \"http")));
    }
    let entries = data["non_business_days"].as_array().unwrap();
    assert!(!entries.iter().any(|e| e["name"] == "Melbourne Cup Day"));
    assert!(
        entries
            .iter()
            .filter(|e| e["date"].as_str().unwrap().starts_with("2028-"))
            .all(|e| e["provisional"] == true)
    );
    for phrase in [
        "WA's official page",
        "Legislative Council second reading",
        "subject to the AFL schedule",
    ] {
        assert!(review.contains(phrase), "{phrase}");
    }
}

#[test]
fn the_calendar_regeneration_guidance_uses_the_locked_environment() {
    let readme = read("README.md");
    let generator = read("tools/generate_calendar.py");
    for guidance in [&readme, &generator] {
        assert!(guidance.contains(LOCKED_CALENDAR_COMMAND));
    }
    assert!(!readme.contains("holidays=="));
    assert!(!generator.contains("pip install holidays=="));
}

#[test]
fn the_limitations_register_names_its_entries_and_conditions() {
    let register = read("LIMITATIONS.md");
    for entry in ["PSC-1", "PSC-2"] {
        assert!(register.contains(entry));
    }
    assert_eq!(register.matches("**Unaffected by ").count(), 2);
    assert!(!register.contains("all operate correctly"));
    assert!(register.contains("receipt-amount evidence"));
    assert!(register.contains("calendar\nand GIC coverage"));
    for verdict in ["ON_TIME", "AT_RISK", "LATE", "UNPAID", "UNKNOWN", "SKIPPED"] {
        assert!(register.contains(&format!("`{verdict}`")), "{verdict}");
    }
    assert!(register.contains("0%") && register.contains("60%"));
    assert!(register.contains("Both exposure totals move"));
    for condition in [
        "employee_id",
        "pay-period-end column",
        "only one of pay-period start or end",
        "no pay-period columns at all",
    ] {
        assert!(register.contains(condition), "{condition}");
    }
}

#[test]
fn psc_1_only_the_exposure_estimate_reads_the_gic_rate() {
    let mut callers = Vec::new();
    for entry in std::fs::read_dir(format!("{ROOT}/src")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&path).unwrap();
        if name != "rates.rs" && name != "api.rs" && text.contains(".daily_rate(") {
            callers.push(name);
        }
    }
    assert_eq!(callers, ["sgc.rs"]);
}

#[test]
fn psc_2_join_declares_each_degraded_match() {
    let source = read("src/join.rs");
    for declared in [
        "matched on employee name because one of the files has no id column",
        "the payroll file has no pay period end column",
        "the super file has only one of the pay period start/end columns",
        "the super file has no pay period columns at all",
    ] {
        assert!(squeeze(&source).contains(declared), "{declared}");
    }
    assert!(source.contains("\"id\"\n    } else {\n        \"name\""));
}

#[test]
fn the_evidence_pack_guide_keeps_its_boundaries_and_is_discoverable() {
    let contract: Value = serde_json::from_str(&read(
        "evaluation/payday_super_evidence/expected_results.json",
    ))
    .unwrap();
    assert_eq!(contract["schema_version"], 1);
    assert_eq!(contract["source_reviewed"], "2026-08-15");
    let decision = contract["human_decision"].as_str().unwrap().to_lowercase();
    for phrase in [
        "eligible fund receipt",
        "allocation",
        "a human must establish",
    ] {
        assert!(decision.contains(phrase), "{phrase}");
    }
    let guide = squeeze(&read("evaluation/payday_super_evidence/README.md")).to_lowercase();
    for phrase in [
        "cannot prove on-time",
        "do not establish fund eligibility",
        "not tax, legal or financial advice",
    ] {
        assert!(guide.contains(phrase), "{phrase}");
    }
    for discovery in ["README.md", "llms.txt"] {
        assert!(
            read(discovery).contains("evaluation/payday_super_evidence"),
            "{discovery}"
        );
    }
}

#[test]
fn the_receipt_amount_guide_matches_its_contract() {
    let contract: Value =
        serde_json::from_str(&read("evaluation/receipt_amount/expected_results.json")).unwrap();
    let readme = read("evaluation/receipt_amount/README.md");
    assert_eq!(contract["product_release"], "0.1.7");
    assert!(readme.contains("Product release `0.1.7`"));
    assert!(readme.contains("uvx --from payday-super-checker==0.1.7 "));
    assert_eq!(contract["practitioner_review"], "pending");
    assert!(readme.contains(contract["human_decision"].as_str().unwrap()));
    for scenario in contract["scenarios"].as_array().unwrap() {
        let id = scenario["id"].as_str().unwrap();
        let row = readme
            .lines()
            .find(|l| l.starts_with(&format!("| `{id}` |")))
            .unwrap_or_else(|| panic!("{id}"));
        let cells: Vec<&str> = row.trim_matches('|').split('|').map(str::trim).collect();
        assert_eq!(
            cells[3],
            format!("`{}`", scenario["expected_verdict"].as_str().unwrap())
        );
        assert_eq!(cells[4], scenario["expected_exit"].to_string());
        let shortfall = scenario["expected_final_shortfall"].as_str().unwrap();
        if !shortfall.is_empty() {
            assert!(cells[5].starts_with(shortfall), "{id}");
        }
    }
}

#[test]
fn the_openaccountants_handoff_matches_the_engine_run() {
    let case = "examples/quarterly_remittance_2026_27.csv";
    let (code, stdout) = check(&[case, "--as-at", "2026-10-28"]);
    assert_eq!(code, 2);
    let document = read("docs/openaccountants-handoff.md");
    for claim in [
        "ON_TIME: 0  AT_RISK: 0  LATE: 7  UNPAID: 0  UNKNOWN: 0  SKIPPED: 0",
        "experimental estimated SG charge $62.49 - $99.98",
    ] {
        assert!(stdout.contains(claim), "{claim}");
        assert!(squeeze(&document).contains(&squeeze(claim)), "{claim}");
    }
    assert!(stdout.contains("LATE, 101 days late"));
    assert!(stdout.contains("LATE, 15 days late"));
    assert!(document.contains(case));
}

#[test]
fn the_review_pack_confinement_is_documented_and_its_default_output_ignored() {
    let readme = read("README.md");
    let section = readme
        .split_once("### Build a practitioner review pack")
        .unwrap()
        .1
        .split("\n### ")
        .next()
        .unwrap();
    assert!(section.contains("bare `.csv` filename in the current directory"));
    assert!(section.contains("`sub/report.csv`"));
    let security = read("SECURITY.md");
    for (document, heading) in [
        (&readme, "## Local file boundary"),
        (&security, "## Local path trust boundary"),
    ] {
        let boundary = squeeze(document.split_once(heading).unwrap().1);
        assert!(boundary.contains("review-pack") && boundary.contains("bare `.csv` filename"));
    }
    assert!(
        read(".gitignore")
            .lines()
            .any(|l| l == "/practitioner-review.md")
    );
}

#[test]
fn the_remittance_only_example_isolates_its_boundary() {
    let example = "examples/sample_remittance_only.csv";
    let text = read(example);
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let column = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let rows: Vec<Vec<&str>> = lines
        .filter(|l| !l.is_empty())
        .map(|l| l.split(',').collect())
        .collect();
    assert!(!rows.is_empty());
    for row in &rows {
        assert_eq!(row[column("fund_received_date")], "");
        assert_ne!(row[column("remitted_date")], "");
        assert!(row[column("payment_date")] > "2026-07-28");
    }
    let asks = "This file cannot produce ON_TIME: no in-scope positive row has a \
                fund-receipt date on or before the as-at date.";
    let records = "Operator confirmed remittance-only review: no in-scope positive row has \
                   a fund-receipt date on or before the as-at date, so this file cannot \
                   produce ON_TIME.";
    let remitted = "4 line(s) remitted by the deadline but with no fund-receipt date. The \
                    statutory timing test turns on receipt by the fund, not the day you \
                    paid, and clearing-house transit time is the employer's risk.";
    let counts = "ON_TIME: 0  AT_RISK: 4  LATE: 0  UNPAID: 0  UNKNOWN: 0  SKIPPED: 0";
    let (code, plain) = check(&[example, "--as-at", "2026-09-10"]);
    assert_eq!(code, 2);
    assert!(plain.contains(asks) && plain.contains("pass --confirm-remittance-only"));
    let (code, confirmed) = check(&[
        example,
        "--as-at",
        "2026-09-10",
        "--confirm-remittance-only",
    ]);
    assert_eq!(code, 0);
    assert!(confirmed.contains(records));
    for output in [&plain, &confirmed] {
        assert!(output.contains(counts) && output.contains(remitted));
    }
    let readme = read("README.md");
    let both = "payday-super-check examples/sample_remittance_only.csv --as-at 2026-09-10";
    assert!(readme.contains(both));
    assert!(readme.contains(&format!("{both} --confirm-remittance-only")));
    for text in [asks, records, remitted] {
        assert!(readme.contains(text), "{text}");
    }
}
