// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The repeated-key scan of `australian_tax_calculators::api`, copied because
//! one engine never depends on another. The repository's
//! tests/test_shared_blocks.py holds the copy to the original.

use std::collections::HashSet;

/// The first key an object in `bytes` repeats, which decoding into `Value`
/// resolves silently by keeping the last value. `bytes` must already have
/// decoded as JSON; each key is compared decoded, so escapes cannot hide a
/// repeat.
pub fn repeated_key(bytes: &[u8]) -> Option<String> {
    // One entry per open container: an object's keys so far, or None for an array.
    let mut open: Vec<Option<HashSet<String>>> = Vec::new();
    let mut key_next = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                let start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                if key_next && let Some(Some(keys)) = open.last_mut() {
                    let literal = bytes.get(start..=i).unwrap_or_default();
                    if let Ok(key) = serde_json::from_slice::<String>(literal)
                        && !keys.insert(key.clone())
                    {
                        return Some(key);
                    }
                }
                key_next = false;
            }
            b'{' => {
                open.push(Some(HashSet::new()));
                key_next = true;
            }
            b'[' => {
                open.push(None);
                key_next = false;
            }
            b'}' | b']' => {
                open.pop();
                key_next = false;
            }
            b',' => key_next = matches!(open.last(), Some(Some(_))),
            _ => {}
        }
        i += 1;
    }
    None
}
