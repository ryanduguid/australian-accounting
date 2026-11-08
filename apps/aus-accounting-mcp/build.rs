// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

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
