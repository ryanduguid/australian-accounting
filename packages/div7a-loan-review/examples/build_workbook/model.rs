//! A semantic model of an .xlsx package, read straight from its OOXML parts,
//! for comparing the Rust builder's workbook with the Python builder's. It
//! shares no code with either writer.
//!
//! Each line states one observable fact. Representation is normalised away;
//! meaning is kept:
//! - style ids resolve to their effective number format, font, fill, border,
//!   alignment and protection; `apply*` flags and the cell style link are
//!   ignored, as Excel applies the xf's records regardless;
//! - a font with no name or size takes the workbook default font's, and font
//!   family, scheme and charset are ignored (Excel's own resave of the shipped
//!   workbook fills them the same way);
//! - colour alpha is ignored (openpyxl writes 00, Excel and rust_xlsxwriter FF),
//!   and so is the background colour of a solid cell fill, which never shows;
//! - shared and inline strings compare as text, and an empty string as an
//!   empty cell (Excel saves openpyxl's empty strings that way);
//! - a formula cell's cached value and its type are ignored (pre-Excel);
//! - column records expand to one line per column; validation and conditional
//!   format ranges compare as cell sets, and identical validation rules are
//!   merged, as Excel does on save;
//! - schema defaults are applied before comparing attributes;
//! - the package inventory counts content types, except shared strings.
//!
//! Differential fills (dxf) are reported as written, not as an effective
//! colour: openpyxl and rust_xlsxwriter write solid dxf fills differently,
//! and whether Excel renders them alike is unverified.
//!
//! Not read, so outside this comparison: document properties, theme content,
//! the indexed colour palette, cell selections and tab selection, rich-text
//! runs, and comments, drawings or hyperlink targets (neither builder writes
//! any). The rules above hold for these two builders' outputs, not for any
//! workbook.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read};

use roxmltree::{Document, Node};

type Archive<'a> = zip::ZipArchive<Cursor<&'a [u8]>>;

const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

pub(crate) fn read_part(archive: &mut Archive, name: &str) -> Result<String, String> {
    let mut text = String::new();
    archive
        .by_name(name)
        .map_err(|e| format!("{name}: {e}"))?
        .read_to_string(&mut text)
        .map_err(|e| format!("{name}: {e}"))?;
    Ok(text)
}

fn has_part(archive: &Archive, name: &str) -> bool {
    archive.index_for_name(name).is_some()
}

/// Resolve a relationship target against the directory of its source part.
pub(crate) fn resolve(source: &str, target: &str) -> String {
    if let Some(absolute) = target.strip_prefix('/') {
        return absolute.to_string();
    }
    let mut parts: Vec<&str> = source.split('/').collect();
    parts.pop();
    for segment in target.split('/') {
        match segment {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            _ => parts.push(segment),
        }
    }
    parts.join("/")
}

fn rels_of(part: &str) -> String {
    let (dir, file) = part.rsplit_once('/').unwrap_or(("", part));
    format!("{dir}/_rels/{file}.rels")
}

/// Relationships of `part` as (id, type URI, resolved target); none if the
/// part has no relationships file.
pub(crate) fn relationships(
    archive: &mut Archive,
    part: &str,
) -> Result<Vec<(String, String, String)>, String> {
    let rels = rels_of(part);
    if !has_part(archive, &rels) {
        return Ok(Vec::new());
    }
    let xml = read_part(archive, &rels)?;
    let doc = Document::parse(&xml).map_err(|e| format!("{rels}: {e}"))?;
    Ok(doc
        .descendants()
        .filter(|n| n.has_tag_name("Relationship"))
        .map(|n| {
            let attr = |name| n.attribute(name).unwrap_or_default().to_string();
            let target = if n.attribute("TargetMode") == Some("External") {
                format!("external:{}", attr("Target"))
            } else {
                resolve(part, &attr("Target"))
            };
            (attr("Id"), attr("Type"), target)
        })
        .collect())
}

/// The sheets of `xl/workbook.xml` in tab order as (name, state, part).
pub(crate) fn sheets(archive: &mut Archive) -> Result<Vec<(String, String, String)>, String> {
    let xml = read_part(archive, "xl/workbook.xml")?;
    let doc = Document::parse(&xml).map_err(|e| e.to_string())?;
    let rels = relationships(archive, "xl/workbook.xml")?;
    doc.descendants()
        .filter(|n| n.has_tag_name("sheet"))
        .map(|n| {
            let name = n.attribute("name").unwrap_or_default().to_string();
            let id = n.attribute((R, "id")).unwrap_or_default();
            let part = rels
                .iter()
                .find(|(rid, _, _)| rid == id)
                .map(|(_, _, target)| target.clone())
                .ok_or_else(|| format!("no relationship {id} for sheet {name}"))?;
            let state = n.attribute("state").unwrap_or("visible").to_string();
            Ok((name, state, part))
        })
        .collect()
}

fn flag(node: Node, name: &str, default: bool) -> bool {
    match node.attribute(name) {
        Some(v) => matches!(v, "1" | "true"),
        None => default,
    }
}

fn number(text: &str) -> String {
    text.parse::<f64>()
        .map_or_else(|_| format!("?{text}"), |v| format!("{v}"))
}

fn quote(text: &str) -> String {
    format!("{text:?}")
}

fn colour(node: Option<Node>) -> String {
    let Some(node) = node else {
        return "none".into();
    };
    let tint = node
        .attribute("tint")
        .map(|t| format!("~{}", number(t)))
        .unwrap_or_default();
    if let Some(rgb) = node.attribute("rgb") {
        let rgb = rgb.to_ascii_uppercase();
        format!("#{}{tint}", &rgb[rgb.len().saturating_sub(6)..])
    } else if let Some(theme) = node.attribute("theme") {
        format!("theme{theme}{tint}")
    } else if let Some(indexed) = node.attribute("indexed") {
        format!("indexed{indexed}{tint}")
    } else if flag(node, "auto", false) {
        "auto".into()
    } else {
        "none".into()
    }
}

fn child<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.children().find(|n| n.has_tag_name(name))
}

/// Built-in number formats (ECMA-376 Part 1, 18.8.30) that can appear without
/// a numFmt record.
fn builtin_format(id: u32) -> Option<&'static str> {
    Some(match id {
        0 => "General",
        1 => "0",
        2 => "0.00",
        3 => "#,##0",
        4 => "#,##0.00",
        9 => "0%",
        10 => "0.00%",
        11 => "0.00E+00",
        12 => "# ?/?",
        13 => "# ??/??",
        14 => "mm-dd-yy",
        15 => "d-mmm-yy",
        16 => "d-mmm",
        17 => "mmm-yy",
        18 => "h:mm AM/PM",
        19 => "h:mm:ss AM/PM",
        20 => "h:mm",
        21 => "h:mm:ss",
        22 => "m/d/yy h:mm",
        37 => "#,##0 ;(#,##0)",
        38 => "#,##0 ;[Red](#,##0)",
        39 => "#,##0.00;(#,##0.00)",
        40 => "#,##0.00;[Red](#,##0.00)",
        45 => "mm:ss",
        46 => "[h]:mm:ss",
        47 => "mmss.0",
        48 => "##0.0E+0",
        49 => "@",
        _ => return None,
    })
}

struct Styles {
    default_font: String,
    /// Effective style text per cellXfs index.
    cells: Vec<String>,
    /// Differential format text per dxfs index.
    dxfs: Vec<String>,
}

fn font_text(font: Option<Node>, default: Option<(&str, &str)>) -> String {
    let Some(font) = font else {
        return "font{}".into();
    };
    let val = |name: &str| child(font, name).and_then(|n| n.attribute("val").or(Some("")));
    let mut parts = Vec::new();
    match (val("name"), default) {
        (Some(name), _) => parts.push(format!("name={name}")),
        (None, Some((name, _))) => parts.push(format!("name={name}")),
        (None, None) => {}
    }
    match (val("sz"), default) {
        (Some(size), _) => parts.push(format!("sz={}", number(size))),
        (None, Some((_, size))) => parts.push(format!("sz={}", number(size))),
        (None, None) => {}
    }
    for name in ["b", "i", "strike", "outline", "shadow"] {
        if let Some(node) = child(font, name)
            && flag(node, "val", true)
        {
            parts.push(name.to_string());
        }
    }
    if let Some(u) = child(font, "u") {
        let kind = u.attribute("val").unwrap_or("single");
        if kind != "none" {
            parts.push(format!("u={kind}"));
        }
    }
    if let Some(v) = val("vertAlign")
        && v != "baseline"
    {
        parts.push(format!("vertAlign={v}"));
    }
    if let Some(c) = child(font, "color") {
        parts.push(format!("color={}", colour(Some(c))));
    }
    format!("font{{{}}}", parts.join(" "))
}

/// `dxf` keeps a missing patternType visible: in a cell fill it means none,
/// in a differential fill Excel applies its own solid-fill reading.
fn fill_text(fill: Option<Node>, dxf: bool) -> String {
    let Some(pattern) = fill.and_then(|f| child(f, "patternFill")) else {
        return match fill.and_then(|f| child(f, "gradientFill")) {
            Some(g) => format!("fill{{gradient {}}}", g.range().len()),
            None => "fill{}".into(),
        };
    };
    let kind = pattern
        .attribute("patternType")
        .unwrap_or(if dxf { "(absent)" } else { "none" });
    let mut text = format!("fill{{{kind}");
    // A solid cell fill shows only its foreground colour (ECMA-376 18.8.32);
    // rust_xlsxwriter and Excel add bgColor indexed 64, openpyxl omits it.
    let colours: &[&str] = if !dxf && kind == "solid" {
        &["fgColor"]
    } else {
        &["fgColor", "bgColor"]
    };
    for name in colours {
        if let Some(c) = child(pattern, name) {
            text.push_str(&format!(" {name}={}", colour(Some(c))));
        }
    }
    text.push('}');
    text
}

fn border_text(border: Option<Node>) -> String {
    let Some(border) = border else {
        return String::new();
    };
    let mut sides = Vec::new();
    for side in ["left", "right", "top", "bottom", "diagonal"] {
        if let Some(node) = child(border, side)
            && let Some(style) = node.attribute("style")
            && style != "none"
        {
            sides.push(format!("{side}={style}:{}", colour(child(node, "color"))));
        }
    }
    if sides.is_empty() {
        String::new()
    } else {
        format!(" border{{{}}}", sides.join(" "))
    }
}

fn alignment_text(xf: Node) -> String {
    let Some(a) = child(xf, "alignment") else {
        return String::new();
    };
    let mut parts = Vec::new();
    for (name, default) in [("horizontal", "general"), ("vertical", "bottom")] {
        if let Some(v) = a.attribute(name)
            && v != default
        {
            parts.push(format!("{name}={v}"));
        }
    }
    for name in ["wrapText", "shrinkToFit", "justifyLastLine"] {
        if flag(a, name, false) {
            parts.push(name.to_string());
        }
    }
    for name in ["indent", "textRotation", "relativeIndent", "readingOrder"] {
        if let Some(v) = a.attribute(name)
            && v != "0"
        {
            parts.push(format!("{name}={v}"));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" align{{{}}}", parts.join(" "))
    }
}

fn protection_text(xf: Node) -> String {
    let p = child(xf, "protection");
    let locked = p.is_none_or(|p| flag(p, "locked", true));
    let hidden = p.is_some_and(|p| flag(p, "hidden", false));
    match (locked, hidden) {
        (true, false) => String::new(),
        _ => format!(
            " protect{{locked={} hidden={}}}",
            u8::from(locked),
            u8::from(hidden)
        ),
    }
}

fn styles(archive: &mut Archive) -> Result<Styles, String> {
    let xml = read_part(archive, "xl/styles.xml")?;
    let doc = Document::parse(&xml).map_err(|e| format!("styles: {e}"))?;
    let root = doc.root_element();
    let list = |name: &str, item: &str| -> Vec<Node> {
        child(root, name)
            .map(|n| n.children().filter(|c| c.has_tag_name(item)).collect())
            .unwrap_or_default()
    };
    let formats: BTreeMap<u32, String> = list("numFmts", "numFmt")
        .into_iter()
        .filter_map(|n| {
            Some((
                n.attribute("numFmtId")?.parse().ok()?,
                n.attribute("formatCode")?.to_string(),
            ))
        })
        .collect();
    let format = |id: u32| {
        formats
            .get(&id)
            .cloned()
            .or_else(|| builtin_format(id).map(String::from))
            .unwrap_or_else(|| format!("builtin{id}"))
    };
    let fonts = list("fonts", "font");
    let fills = list("fills", "fill");
    let borders = list("borders", "border");
    let base = fonts.first().copied();
    let default_name = base
        .and_then(|f| child(f, "name"))
        .and_then(|n| n.attribute("val"))
        .unwrap_or("Calibri");
    let default_size = base
        .and_then(|f| child(f, "sz"))
        .and_then(|n| n.attribute("val"))
        .unwrap_or("11");
    let default = Some((default_name, default_size));
    let default_font = font_text(base, default);
    let index = |xf: Node, name: &str| -> usize {
        xf.attribute(name).and_then(|v| v.parse().ok()).unwrap_or(0)
    };
    let cells = list("cellXfs", "xf")
        .into_iter()
        .map(|xf| {
            let font = font_text(fonts.get(index(xf, "fontId")).copied(), default);
            let font = if font == default_font {
                String::new()
            } else {
                format!(" {font}")
            };
            let fill = fill_text(fills.get(index(xf, "fillId")).copied(), false);
            let fill = if fill == "fill{}" || fill == "fill{none}" {
                String::new()
            } else {
                format!(" {fill}")
            };
            format!(
                "fmt={}{font}{fill}{}{}{}",
                quote(&format(index(xf, "numFmtId") as u32)),
                border_text(borders.get(index(xf, "borderId")).copied()),
                alignment_text(xf),
                protection_text(xf)
            )
        })
        .collect();
    let dxfs = list("dxfs", "dxf")
        .into_iter()
        .map(|dxf| {
            let mut parts = Vec::new();
            if let Some(f) = child(dxf, "font") {
                parts.push(font_text(Some(f), None));
            }
            if let Some(f) = child(dxf, "fill") {
                parts.push(fill_text(Some(f), true));
            }
            if let Some(n) = child(dxf, "numFmt") {
                parts.push(format!(
                    "fmt={}",
                    quote(n.attribute("formatCode").unwrap_or(""))
                ));
            }
            let border = border_text(child(dxf, "border"));
            if !border.is_empty() {
                parts.push(border.trim_start().to_string());
            }
            format!("dxf{{{}}}", parts.join(" "))
        })
        .collect();
    Ok(Styles {
        default_font,
        cells,
        dxfs,
    })
}

fn shared_strings(archive: &mut Archive) -> Result<Vec<String>, String> {
    let Some(part) = relationships(archive, "xl/workbook.xml")?
        .into_iter()
        .find(|(_, kind, _)| kind.ends_with("/sharedStrings"))
        .map(|(_, _, target)| target)
    else {
        return Ok(Vec::new());
    };
    let xml = read_part(archive, &part)?;
    let doc = Document::parse(&xml).map_err(|e| format!("{part}: {e}"))?;
    Ok(doc
        .root_element()
        .children()
        .filter(|n| n.has_tag_name("si"))
        .map(string_item)
        .collect())
}

/// The text of a shared string item or inline string, ignoring phonetic runs.
fn string_item(item: Node) -> String {
    item.descendants()
        .filter(|n| n.has_tag_name("t") && !n.ancestors().any(|a| a.has_tag_name("rPh")))
        .filter_map(|n| n.text())
        .collect()
}

fn column_number(letters: &str) -> u32 {
    letters.bytes().fold(0, |n, b| {
        n * 26 + u32::from(b.to_ascii_uppercase() - b'A' + 1)
    })
}

fn column_letters(mut n: u32) -> String {
    let mut out = Vec::new();
    while n > 0 {
        let r = (n - 1) % 26;
        out.push(b'A' + r as u8);
        n = (n - 1) / 26;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

fn split_ref(cell: &str) -> (u32, u32) {
    let cell = cell.replace('$', "");
    let at = cell
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or(cell.len());
    (column_number(&cell[..at]), cell[at..].parse().unwrap_or(0))
}

/// A space-separated range list as a set of (column, row) cells.
fn cells_of(sqref: &str) -> BTreeSet<(u32, u32)> {
    let mut set = BTreeSet::new();
    for range in sqref.split_whitespace() {
        let (a, b) = range.split_once(':').unwrap_or((range, range));
        let ((c1, r1), (c2, r2)) = (split_ref(a), split_ref(b));
        for c in c1.min(c2)..=c1.max(c2) {
            for r in r1.min(r2)..=r1.max(r2) {
                set.insert((c, r));
            }
        }
    }
    set
}

/// A cell set as column-wise row runs, the canonical spelling of a range list.
fn ranges_text(cells: &BTreeSet<(u32, u32)>) -> String {
    let mut runs: Vec<String> = Vec::new();
    let mut current: Option<(u32, u32, u32)> = None;
    for &(c, r) in cells {
        current = match current {
            Some((cc, start, end)) if cc == c && end + 1 == r => Some((cc, start, r)),
            Some(run) => {
                runs.push(run_text(run));
                Some((c, r, r))
            }
            None => Some((c, r, r)),
        };
    }
    if let Some(run) = current {
        runs.push(run_text(run));
    }
    runs.join(" ")
}

fn run_text((c, start, end): (u32, u32, u32)) -> String {
    let col = column_letters(c);
    if start == end {
        format!("{col}{start}")
    } else {
        format!("{col}{start}:{col}{end}")
    }
}

const PROTECTION_DEFAULTS: [(&str, bool); 16] = [
    ("sheet", false),
    ("objects", false),
    ("scenarios", false),
    ("formatCells", true),
    ("formatColumns", true),
    ("formatRows", true),
    ("insertColumns", true),
    ("insertRows", true),
    ("insertHyperlinks", true),
    ("deleteColumns", true),
    ("deleteRows", true),
    ("selectLockedCells", false),
    ("sort", true),
    ("autoFilter", true),
    ("pivotTables", true),
    ("selectUnlockedCells", false),
];

const VALIDATION_DEFAULTS: [(&str, &str); 9] = [
    ("type", "none"),
    ("operator", "between"),
    ("errorStyle", "stop"),
    ("imeMode", "noControl"),
    ("allowBlank", "0"),
    ("showDropDown", "0"),
    ("showInputMessage", "0"),
    ("showErrorMessage", "0"),
    ("errorTitle", ""),
];

fn sheet_lines(
    archive: &mut Archive,
    name: &str,
    part: &str,
    styles: &Styles,
    strings: &[String],
    out: &mut Vec<String>,
) -> Result<(), String> {
    let xml = read_part(archive, part)?;
    let doc = Document::parse(&xml).map_err(|e| format!("{part}: {e}"))?;
    let root = doc.root_element();
    let at = |what: String| format!("sheet {name} | {what}");

    for view in root.descendants().filter(|n| n.has_tag_name("sheetView")) {
        let grid = flag(view, "showGridLines", true);
        let zoom = view.attribute("zoomScale").unwrap_or("100");
        out.push(at(format!("view gridlines={} zoom={zoom}", u8::from(grid))));
        if let Some(pane) = child(view, "pane") {
            out.push(at(format!(
                "pane xSplit={} ySplit={} topLeft={} state={}",
                pane.attribute("xSplit").unwrap_or("0"),
                pane.attribute("ySplit").unwrap_or("0"),
                pane.attribute("topLeftCell").unwrap_or(""),
                pane.attribute("state").unwrap_or("split")
            )));
        }
    }
    // Print margins; Excel's own defaults apply when the element is absent.
    let margins = child(root, "pageMargins");
    let mut text = String::from("margins");
    for (attr, default) in [
        ("left", "0.7"),
        ("right", "0.7"),
        ("top", "0.75"),
        ("bottom", "0.75"),
        ("header", "0.3"),
        ("footer", "0.3"),
    ] {
        let v = margins.and_then(|m| m.attribute(attr)).unwrap_or(default);
        text.push_str(&format!(" {attr}={}", number(v)));
    }
    out.push(at(text));
    let setup = child(root, "pageSetup");
    let mut text = String::from("page");
    for (attr, default) in [
        ("orientation", "default"),
        ("paperSize", "1"),
        ("scale", "100"),
        ("fitToWidth", "1"),
        ("fitToHeight", "1"),
        ("horizontalDpi", "600"),
        ("verticalDpi", "600"),
        ("blackAndWhite", "0"),
        ("draft", "0"),
    ] {
        let v = setup.and_then(|m| m.attribute(attr)).unwrap_or(default);
        text.push_str(&format!(" {attr}={v}"));
    }
    out.push(at(text));
    if let Some(format) = child(root, "sheetFormatPr") {
        for attr in [
            "defaultColWidth",
            "defaultRowHeight",
            "customHeight",
            "zeroHeight",
        ] {
            if let Some(v) = format.attribute(attr)
                && !(attr == "defaultRowHeight" && number(v) == "15")
                && v != "0"
            {
                out.push(at(format!("format {attr}={}", number(v))));
            }
        }
    }
    for col in root.descendants().filter(|n| n.has_tag_name("col")) {
        let min: u32 = col
            .attribute("min")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let max: u32 = col
            .attribute("max")
            .and_then(|v| v.parse().ok())
            .unwrap_or(min);
        let width = col.attribute("width").map(number).unwrap_or_default();
        let style: usize = col
            .attribute("style")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        for c in min..=max {
            out.push(at(format!(
                "col {} width={width} custom={} hidden={} style=[{}]",
                column_letters(c),
                u8::from(flag(col, "customWidth", false)),
                u8::from(flag(col, "hidden", false)),
                styles.cells.get(style).map_or("?", String::as_str)
            )));
        }
    }
    for row in root.descendants().filter(|n| n.has_tag_name("row")) {
        let r = row.attribute("r").unwrap_or("?");
        if flag(row, "customHeight", false) || flag(row, "hidden", false) {
            out.push(at(format!(
                "row {r} ht={} hidden={}",
                row.attribute("ht").map(number).unwrap_or_default(),
                u8::from(flag(row, "hidden", false))
            )));
        }
        if flag(row, "customFormat", false) {
            let s: usize = row.attribute("s").and_then(|v| v.parse().ok()).unwrap_or(0);
            out.push(at(format!(
                "row {r} style=[{}]",
                styles.cells.get(s).map_or("?", String::as_str)
            )));
        }
    }
    for cell in root.descendants().filter(|n| n.has_tag_name("c")) {
        let r = cell.attribute("r").unwrap_or("?");
        let s: usize = cell
            .attribute("s")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let style = styles.cells.get(s).map_or("?", String::as_str);
        let content = if let Some(f) = child(cell, "f") {
            let mut text = format!("formula {}", quote(f.text().unwrap_or("")));
            for attr in ["t", "ref", "aca", "dt2D", "dtr", "si"] {
                if let Some(v) = f.attribute(attr)
                    && !(attr == "t" && v == "normal")
                {
                    text.push_str(&format!(" {attr}={v}"));
                }
            }
            text
        } else {
            let v = child(cell, "v").and_then(|v| v.text());
            match (cell.attribute("t").unwrap_or("n"), v) {
                ("s", Some(v)) => format!(
                    "text {}",
                    quote(
                        v.parse::<usize>()
                            .ok()
                            .and_then(|i| strings.get(i))
                            .map_or("?", |s| s)
                    )
                ),
                ("inlineStr", _) => format!(
                    "text {}",
                    quote(&child(cell, "is").map(string_item).unwrap_or_default())
                ),
                ("str", Some(v)) => format!("text {}", quote(v)),
                ("b", Some(v)) => format!("bool {v}"),
                ("e", Some(v)) => format!("error {v}"),
                ("n", Some(v)) => format!("number {}", number(v)),
                (kind, Some(v)) => format!("{kind} {v}"),
                (_, None) => "empty".into(),
            }
        };
        // openpyxl writes "" as an empty inline string; Excel saves it as an
        // empty cell, as rust_xlsxwriter writes it.
        let content = if content == "text \"\"" {
            "empty".into()
        } else {
            content
        };
        if content == "empty" && s == 0 {
            continue;
        }
        out.push(at(format!("cell {r} {content} | {style}")));
    }
    for merge in root.descendants().filter(|n| n.has_tag_name("mergeCell")) {
        out.push(at(format!(
            "merge {}",
            merge.attribute("ref").unwrap_or("")
        )));
    }
    if let Some(p) = child(root, "sheetProtection") {
        let mut text = String::from("protection");
        for (attr, default) in PROTECTION_DEFAULTS {
            text.push_str(&format!(" {attr}={}", u8::from(flag(p, attr, default))));
        }
        let password = p.attribute("password").is_some() || p.attribute("hashValue").is_some();
        text.push_str(&format!(" password={}", u8::from(password)));
        out.push(at(text));
    }

    let mut rules: BTreeMap<String, BTreeSet<(u32, u32)>> = BTreeMap::new();
    for dv in root
        .descendants()
        .filter(|n| n.has_tag_name("dataValidation"))
    {
        let mut key = String::new();
        for (attr, default) in VALIDATION_DEFAULTS {
            let v = dv.attribute(attr).unwrap_or(default);
            let v = match v {
                "true" => "1",
                "false" => "0",
                other => other,
            };
            key.push_str(&format!(" {attr}={}", quote(v)));
        }
        for attr in ["error", "promptTitle", "prompt"] {
            key.push_str(&format!(
                " {attr}={}",
                quote(dv.attribute(attr).unwrap_or(""))
            ));
        }
        for formula in ["formula1", "formula2"] {
            let text = child(dv, formula).and_then(|n| n.text()).unwrap_or("");
            key.push_str(&format!(" {formula}={}", quote(text)));
        }
        rules
            .entry(key)
            .or_default()
            .extend(cells_of(dv.attribute("sqref").unwrap_or("")));
    }
    for (key, cells) in rules {
        out.push(at(format!("validation {} |{key}", ranges_text(&cells))));
    }

    let mut cf = Vec::new();
    for block in root
        .descendants()
        .filter(|n| n.has_tag_name("conditionalFormatting"))
    {
        let range = ranges_text(&cells_of(block.attribute("sqref").unwrap_or("")));
        for rule in block.children().filter(|n| n.has_tag_name("cfRule")) {
            let priority: i64 = rule
                .attribute("priority")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let dxf = rule
                .attribute("dxfId")
                .and_then(|v| v.parse::<usize>().ok())
                .and_then(|i| styles.dxfs.get(i))
                .map_or("dxf{}", String::as_str);
            let formulas: Vec<String> = rule
                .children()
                .filter(|n| n.has_tag_name("formula"))
                .map(|n| quote(n.text().unwrap_or("")))
                .collect();
            cf.push((
                priority,
                format!(
                    "conditional {range} | type={} operator={} stop={} formulas=[{}] {dxf}",
                    rule.attribute("type").unwrap_or(""),
                    rule.attribute("operator").unwrap_or(""),
                    u8::from(flag(rule, "stopIfTrue", false)),
                    formulas.join(",")
                ),
            ));
        }
    }
    cf.sort();
    for (rank, (_, text)) in cf.into_iter().enumerate() {
        out.push(at(format!("{text} | rank={rank}")));
    }

    let hyperlinks = root
        .descendants()
        .filter(|n| n.has_tag_name("hyperlink"))
        .count();
    if hyperlinks > 0 {
        out.push(at(format!("hyperlinks {hyperlinks}")));
    }
    for (_, kind, target) in relationships(archive, part)? {
        if kind.ends_with("/table") {
            table_lines(archive, name, &target, out)?;
        } else {
            let kind = kind.rsplit('/').next().unwrap_or("");
            out.push(at(format!("relationship {kind}")));
        }
    }
    Ok(())
}

fn table_lines(
    archive: &mut Archive,
    sheet: &str,
    part: &str,
    out: &mut Vec<String>,
) -> Result<(), String> {
    let xml = read_part(archive, part)?;
    let doc = Document::parse(&xml).map_err(|e| format!("{part}: {e}"))?;
    let t = doc.root_element();
    let name = t
        .attribute("displayName")
        .or(t.attribute("name"))
        .unwrap_or("?");
    let at = |what: String| format!("table {name} | {what}");
    let style = child(t, "tableStyleInfo");
    let style_flag = |attr| style.map_or(0, |s| u8::from(flag(s, attr, false)));
    out.push(at(format!(
        "sheet={sheet} name={} ref={} header={} totals={} autofilter={} style={} firstCol={} lastCol={} rowStripes={} colStripes={}",
        t.attribute("name").unwrap_or(""),
        t.attribute("ref").unwrap_or(""),
        t.attribute("headerRowCount").unwrap_or("1"),
        t.attribute("totalsRowCount").unwrap_or("0"),
        u8::from(child(t, "autoFilter").is_some()),
        style.and_then(|s| s.attribute("name")).unwrap_or("(none)"),
        style_flag("showFirstColumn"),
        style_flag("showLastColumn"),
        style_flag("showRowStripes"),
        style_flag("showColumnStripes"),
    )));
    for col in t.descendants().filter(|n| n.has_tag_name("tableColumn")) {
        let formula = child(col, "calculatedColumnFormula");
        let mut text = format!(
            "column {} {}",
            col.attribute("id").unwrap_or("?"),
            quote(col.attribute("name").unwrap_or(""))
        );
        if let Some(f) = formula {
            text.push_str(&format!(
                " formula={} array={}",
                quote(f.text().unwrap_or("")),
                u8::from(flag(f, "array", false))
            ));
        }
        for attr in ["totalsRowFunction", "totalsRowLabel", "dataDxfId"] {
            if let Some(v) = col.attribute(attr) {
                text.push_str(&format!(" {attr}={v}"));
            }
        }
        out.push(at(text));
    }
    Ok(())
}

fn workbook_lines(archive: &mut Archive, out: &mut Vec<String>) -> Result<(), String> {
    let xml = read_part(archive, "xl/workbook.xml")?;
    let doc = Document::parse(&xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    let pr = child(root, "workbookPr");
    out.push(format!(
        "workbook | date1904={}",
        u8::from(pr.is_some_and(|p| flag(p, "date1904", false)))
    ));
    // An empty workbookProtection element (openpyxl writes one) locks nothing.
    if let Some(p) = child(root, "workbookProtection") {
        let locks: Vec<&str> = ["lockStructure", "lockWindows", "lockRevision"]
            .into_iter()
            .filter(|a| flag(p, a, false))
            .collect();
        if !locks.is_empty() {
            out.push(format!("workbook | protection {}", locks.join(" ")));
        }
    }
    if let Some(view) = root.descendants().find(|n| n.has_tag_name("workbookView")) {
        out.push(format!(
            "workbook | activeTab={} firstSheet={}",
            view.attribute("activeTab").unwrap_or("0"),
            view.attribute("firstSheet").unwrap_or("0")
        ));
    }
    if let Some(calc) = child(root, "calcPr") {
        out.push(format!(
            "workbook | calc mode={} fullCalcOnLoad={}",
            calc.attribute("calcMode").unwrap_or("auto"),
            u8::from(flag(calc, "fullCalcOnLoad", false))
        ));
    }
    for name in root.descendants().filter(|n| n.has_tag_name("definedName")) {
        out.push(format!(
            "workbook | name {} scope={} hidden={} = {}",
            name.attribute("name").unwrap_or(""),
            name.attribute("localSheetId").unwrap_or("workbook"),
            u8::from(flag(name, "hidden", false)),
            quote(name.text().unwrap_or(""))
        ));
    }
    Ok(())
}

fn inventory_lines(archive: &mut Archive, out: &mut Vec<String>) -> Result<(), String> {
    let xml = read_part(archive, "[Content_Types].xml")?;
    let doc = Document::parse(&xml).map_err(|e| e.to_string())?;
    let defaults: BTreeMap<String, String> = doc
        .descendants()
        .filter(|n| n.has_tag_name("Default"))
        .filter_map(|n| {
            Some((
                n.attribute("Extension")?.to_ascii_lowercase(),
                n.attribute("ContentType")?.to_string(),
            ))
        })
        .collect();
    let overrides: BTreeMap<String, String> = doc
        .descendants()
        .filter(|n| n.has_tag_name("Override"))
        .filter_map(|n| {
            Some((
                n.attribute("PartName")?.trim_start_matches('/').to_string(),
                n.attribute("ContentType")?.to_string(),
            ))
        })
        .collect();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let names: Vec<String> = archive
        .file_names()
        .map(|n| n.map(|n| n.into_owned()))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    for name in names {
        if name == "[Content_Types].xml" || name.ends_with('/') {
            continue;
        }
        let kind = overrides.get(&name).cloned().or_else(|| {
            let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())?;
            defaults.get(&ext).cloned()
        });
        let kind = kind.unwrap_or_else(|| format!("(no content type: {name})"));
        if kind.ends_with("sharedStrings+xml") {
            continue;
        }
        let kind = kind
            .rsplit('/')
            .next()
            .unwrap_or(&kind)
            .replace("vnd.openxmlformats-", "");
        *counts.entry(kind).or_default() += 1;
    }
    for (kind, n) in counts {
        out.push(format!("package | {kind} x{n}"));
    }
    Ok(())
}

/// The model of an .xlsx package as sorted, deduplicated lines.
pub fn model(xlsx: &[u8]) -> Result<Vec<String>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(xlsx)).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    inventory_lines(&mut archive, &mut out)?;
    workbook_lines(&mut archive, &mut out)?;
    let styles = styles(&mut archive)?;
    out.push(format!("workbook | default {}", styles.default_font));
    let strings = shared_strings(&mut archive)?;
    for (i, (name, state, part)) in sheets(&mut archive)?.into_iter().enumerate() {
        out.push(format!(
            "workbook | sheet {i} {} state={state}",
            quote(&name)
        ));
        sheet_lines(&mut archive, &name, &part, &styles, &strings, &mut out)?;
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// Lines only in `reference` (prefixed `-`) or only in `candidate` (`+`).
pub fn differences(reference: &[String], candidate: &[String]) -> Vec<String> {
    let a: BTreeSet<&String> = reference.iter().collect();
    let b: BTreeSet<&String> = candidate.iter().collect();
    let mut out: Vec<String> = a.difference(&b).map(|l| format!("- {l}")).collect();
    out.extend(b.difference(&a).map(|l| format!("+ {l}")));
    out.sort_by(|x, y| x[2..].cmp(&y[2..]));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::{
        ConditionalFormatCell, ConditionalFormatCellRule, DataValidation, DataValidationRule,
        Format, Formula, Table, TableColumn, Workbook,
    };

    const CALC: &str = "tblT[[#This Row],[amount]]*2";

    /// One change the model must notice, applied to an otherwise fixed sample.
    #[derive(Clone, Copy, PartialEq)]
    enum Change {
        None,
        CellFormula,
        NumberFormat,
        Unlocked,
        ValidationErrorOff,
        ConditionalValue,
        Unprotected,
        TableFormula,
        DefinedName,
        HiddenColumn,
        RowHeight,
    }

    fn sample(change: Change) -> Vec<u8> {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet().set_name("Register").unwrap();
        ws.write(0, 0, "amount").unwrap();
        ws.write(0, 1, "double").unwrap();
        let money = Format::new().set_num_format(if change == Change::NumberFormat {
            "0.0"
        } else {
            "0.00"
        });
        let input = if change == Change::Unlocked {
            money.clone().set_unlocked()
        } else {
            money.clone()
        };
        ws.write_with_format(1, 0, 5, &input).unwrap();
        let formula = if change == Change::TableFormula {
            "tblT[[#This Row],[amount]]*3"
        } else {
            CALC
        };
        let columns = [
            TableColumn::new().set_header("amount"),
            TableColumn::new()
                .set_header("double")
                .set_formula(Formula::new(formula)),
        ];
        ws.add_table(
            0,
            0,
            1,
            1,
            &Table::new().set_name("tblT").set_columns(&columns),
        )
        .unwrap();
        let total = if change == Change::CellFormula {
            "SUM(A2:A2)+1"
        } else {
            "SUM(A2:A2)"
        };
        ws.write_formula_with_format(3, 0, Formula::new(total), &money)
            .unwrap();
        let validation = DataValidation::new()
            .allow_whole_number(DataValidationRule::Between(0, 100))
            .show_error_message(change != Change::ValidationErrorOff);
        ws.add_data_validation(1, 0, 1, 0, &validation).unwrap();
        let limit = if change == Change::ConditionalValue {
            60
        } else {
            50
        };
        let rule = ConditionalFormatCell::new()
            .set_rule(ConditionalFormatCellRule::GreaterThan(limit))
            .set_format(Format::new().set_bold());
        ws.add_conditional_format(1, 0, 1, 0, &rule).unwrap();
        if change != Change::Unprotected {
            ws.protect();
        }
        ws.set_column_hidden(if change == Change::HiddenColumn { 2 } else { 3 })
            .unwrap();
        ws.set_row_height(5, if change == Change::RowHeight { 30 } else { 45 })
            .unwrap();
        let target = if change == Change::DefinedName {
            "=tblT[double]"
        } else {
            "=tblT[amount]"
        };
        wb.define_name("Amounts", target).unwrap();
        wb.save_to_buffer().unwrap()
    }

    #[test]
    fn the_model_is_stable_for_the_same_workbook() {
        let base = model(&sample(Change::None)).unwrap();
        assert!(base.iter().any(|l| l.starts_with("table tblT | column 2")));
        assert_eq!(
            differences(&base, &model(&sample(Change::None)).unwrap()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn every_semantic_change_shows_in_the_lines_it_touches() {
        let base = model(&sample(Change::None)).unwrap();
        for (change, marker) in [
            (Change::CellFormula, "cell A4 formula"),
            (Change::NumberFormat, "fmt=\"0.0"),
            (Change::Unlocked, "locked=0"),
            (Change::ValidationErrorOff, "showErrorMessage=\"0\""),
            (Change::ConditionalValue, "conditional A2"),
            (Change::Unprotected, "protection"),
            (Change::TableFormula, "table tblT | column 2"),
            (Change::DefinedName, "name Amounts"),
            (Change::HiddenColumn, "col C width="),
            (Change::RowHeight, "row 6 ht="),
        ] {
            let found = differences(&base, &model(&sample(change)).unwrap());
            assert!(
                found.iter().any(|l| l.contains(marker)),
                "{marker}: {found:#?}"
            );
        }
    }
}
