// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Embed the bundled reviewed rate records, the same files the Python server reads.
use std::{env, fs, path::Path};

fn main() {
    let folder = Path::new("aus_accounting_mcp/rate_records");
    println!("cargo::rerun-if-changed={}", folder.display());
    let mut names: Vec<String> = fs::read_dir(folder)
        .expect("rate_records folder")
        .map(|entry| {
            entry
                .expect("rate record entry")
                .file_name()
                .into_string()
                .unwrap()
        })
        .filter(|name| name.ends_with(".json"))
        .collect();
    names.sort();
    let root = env::var("CARGO_MANIFEST_DIR").unwrap().replace('\\', "/");
    let items: String = names
        .iter()
        .map(|name| {
            format!(
                "    include_str!(\"{root}/{}/{name}\"),\n",
                folder.display()
            )
        })
        .collect();
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("rate_records.rs");
    fs::write(out, format!("&[\n{items}]\n")).unwrap();
}
