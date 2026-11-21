// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use payday_super_checker::py::is_space;

pub(super) fn strip(value: &str) -> &str {
    value.trim_matches(is_space)
}
pub(super) fn child<'a, 'i>(
    node: roxmltree::Node<'a, 'i>,
    tag: &str,
) -> Option<roxmltree::Node<'a, 'i>> {
    node.children().find(|n| {
        n.is_element() && n.tag_name().namespace().is_none() && n.tag_name().name() == tag
    })
}
pub(super) fn tagged(node: roxmltree::Node<'_, '_>, tag: &str) -> bool {
    node.is_element() && node.tag_name().namespace().is_none() && node.tag_name().name() == tag
}
pub(super) fn element_text(node: Option<roxmltree::Node<'_, '_>>) -> String {
    // ElementTree discards comments/PIs and joins their surrounding text, but
    // findtext excludes nested element content and tails after that element.
    node.into_iter()
        .flat_map(|n| n.children())
        .take_while(|n| !n.is_element())
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect()
}
fn tag_end(after: &str) -> Option<usize> {
    // Attribute values may contain '>' and "/>" but never '<', so the
    // start tag ends at the first '>' outside quotes. An unclosed quote
    // runs to the end, which roxmltree refuses before recursing further.
    let mut quote = None;
    after.bytes().position(|b| match quote {
        Some(q) => {
            if b == q {
                quote = None;
            }
            false
        }
        None if b == b'"' || b == b'\'' => {
            quote = Some(b);
            false
        }
        None => b == b'>',
    })
}
fn nesting(text: &str) -> Result<(), String> {
    // roxmltree's parser recurses per nested element with no depth limit, so
    // bound the nesting before it sees the bytes. Comments, CDATA and processing
    // instructions end exactly where roxmltree's tokenizer ends them (the first
    // terminator after the opener), because ending one early would expose an
    // inner "</" and undercount. Any other "<!" makes roxmltree fail at once, so
    // what the scan does after it cannot matter. The scan may overcount (a safe
    // refusal near the limit) but never undercounts nesting roxmltree reaches.
    const LIMIT: i64 = 1_000;
    let mut depth = 0i64;
    let mut rest = text;
    fn skip<'a>(after: &'a str, opener: usize, end: &str) -> &'a str {
        let body = after.get(opener..).unwrap_or("");
        body.find(end).map_or("", |i| &body[i + end.len()..])
    }
    while let Some(start) = rest.find('<') {
        let after = &rest[start + 1..];
        let tag = after.as_bytes();
        if tag.starts_with(b"!--") {
            rest = skip(after, 3, "-->");
        } else if tag.starts_with(b"![CDATA[") {
            rest = skip(after, 8, "]]>");
        } else if tag.first() == Some(&b'?') {
            rest = skip(after, 1, "?>");
        } else if tag.first() == Some(&b'!') {
            rest = skip(after, 1, ">");
        } else if tag.first() == Some(&b'/') {
            depth = (depth - 1).max(0);
            rest = after;
        } else {
            let end = tag_end(after);
            // roxmltree enters an empty element too, so it counts while open.
            depth += 1;
            if depth > LIMIT {
                return Err("XML exceeds 1,000 nested elements".into());
            }
            if end.map_or(after, |i| &after[..i]).trim_end().ends_with('/') {
                depth -= 1;
            }
            rest = end.map_or("", |i| &after[i + 1..]);
        }
    }
    Ok(())
}
pub(super) fn xml(bytes: &[u8]) -> Result<roxmltree::Document<'_>, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    // The embedded reference needs no DTD. Refuse DTDs before any entity
    // expansion; roxmltree's per-entity guards do not cap total amplification.
    nesting(text)?;
    // The bounded nesting above is still ~1,000 parser frames deep, beyond the
    // default main stack. An unoptimised build needs about 15 MiB for them on
    // x86_64 Linux and more on macOS arm64, where 16 MiB overflowed, so allow 4
    // times the Linux need; only the pages the parse reaches are touched.
    stacker::grow(64 * 1024 * 1024, || {
        roxmltree::Document::parse(text).map_err(|e| e.to_string())
    })
}
