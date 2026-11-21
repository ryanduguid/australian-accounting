// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::{Context, linked};
use super::manifest::{manifest, validate};
use super::{Catalogue, RUN_NAME, Run, VALIDATED};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

fn is_run(path: &Path) -> bool {
    path.join("rulings.jsonl").exists() || path.join("manifest.json").exists()
}
fn directories(context: &Context) -> Result<Vec<PathBuf>, String> {
    let root = context.root("AUS_ACCOUNTING_RULINGS_ROOT", "folder of rulings runs", "")?;
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
pub(super) fn catalogue(context: &Context) -> Result<Catalogue, String> {
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
