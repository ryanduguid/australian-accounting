// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The linked engine's worksheet catalogue, worked out once at build time, so no
//! request builds all of it: each kind's entry as JSON text, and the fields of it
//! the data freshness report reads.
use serde_json::{Value, json};
use std::{env, fs, path::Path};

const FRESHNESS: [&str; 4] = ["source", "sources", "source_checked", "supported_periods"];

fn main() {
    println!("cargo::rerun-if-changed=../../../packages/australian-tax-calculators/src");
    println!("cargo::rerun-if-changed=../../../packages/australian-tax-calculators/data");
    let catalogue = australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({}))
        .expect("the engine's worksheet catalogue");
    let mut out = String::from("pub(crate) static WORKSHEETS: &[(&str, &str, &str)] = &[\n");
    for (kind, entry) in catalogue.as_object().expect("a catalogue object") {
        let fields = FRESHNESS
            .iter()
            .filter_map(|&field| Some((field.to_owned(), entry.get(field)?.clone())));
        let freshness = Value::Object(fields.collect());
        out += &format!("    ({kind:?}, {:?}, {:?}),\n", entry.to_string(), freshness.to_string());
    }
    out += "];\n";
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("worksheets.rs");
    fs::write(path, out).unwrap();
}
