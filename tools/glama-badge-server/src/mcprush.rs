//! Offline restyling of the observed Aus Accounting MCP full-card grammar.

use super::validation::{attribute_is, elements, leaf, metadata, number, parse, path_data, shape};
use super::{
    IVORY, InvalidBadge, PLATINUM, Result, escaped, require, write_node, write_painted_node,
};
use roxmltree::Node;
use std::collections::HashSet;

const PURPLE: &str = "#66023C";
const FRAME: &str =
    "M6.5 0.5h367a6 6 0 0 1 6 6v187a6 6 0 0 1-6 6h-367a6 6 0 0 1-6-6v-187a6 6 0 0 1 6-6z";
const TAGS: [&str; 28] = [
    "title", "desc", "defs", "path", "g", "g", "circle", "g", "g", "circle", "g", "path", "g",
    "path", "g", "g", "path", "g", "g", "path", "g", "g", "g", "path", "g", "rect", "path", "g",
];
const GROUPS: [(usize, f64, f64); 15] = [
    (4, 38.0, 0.018),
    (5, 59.0, 0.0125),
    (7, 94.45, 0.0125),
    (8, 94.4, 0.0125),
    (10, 94.4, 0.0125),
    (12, 94.4, 0.0125),
    (14, 130.0, 0.0125),
    (15, 130.0, 0.0125),
    (17, 130.0, 0.0125),
    (18, 130.0, 0.0125),
    (20, 130.0, 0.0125),
    (21, 130.0, 0.0125),
    (22, 130.0, 0.0125),
    (24, 180.0, 0.0125),
    (27, 180.0, 0.0115),
];

fn paint(node: Node<'_, '_>, key: &str) -> Result<()> {
    let value = node.attribute(key).unwrap_or_default();
    require(
        value.len() == 7
            && value.starts_with('#')
            && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit),
        "Only solid hex colours are accepted",
    )
}

fn transform(node: Node<'_, '_>) -> Result<(f64, f64, f64)> {
    let value = node.attribute("transform").unwrap_or_default();
    let (position, scale) = value
        .strip_prefix("translate(")
        .and_then(|s| s.strip_suffix(')'))
        .and_then(|s| s.split_once(") scale("))
        .ok_or(InvalidBadge("Unsupported card transform"))?;
    let position: Vec<_> = position.split_whitespace().collect();
    require(position.len() == 2, "Unsupported card translation")?;
    let parse = |s: &str| {
        s.parse::<f64>()
            .map_err(|_| InvalidBadge("Invalid card transform"))
    };
    let (x, y, scale) = (parse(position[0])?, parse(position[1])?, parse(scale)?);
    require(
        x.is_finite()
            && (0.0..=380.0).contains(&x)
            && y.is_finite()
            && (0.0..=200.0).contains(&y)
            && scale.is_finite()
            && (0.001..=2.0).contains(&scale),
        "Card transform outside bounds",
    )?;
    Ok((x, y, scale))
}

fn definitions<'a, 'i>(node: Node<'a, 'i>) -> Result<HashSet<&'a str>> {
    shape(node, "defs", &[])?;
    let children = elements(node)?;
    require(
        !children.is_empty() && children.len() <= 96,
        "Unexpected card glyph count",
    )?;
    let mut ids = HashSet::new();
    for glyph in children {
        leaf(glyph, "path", &["id", "d"])?;
        let id = glyph.attribute("id").unwrap_or_default();
        require(
            id.strip_prefix('g').is_some_and(|s| {
                !s.is_empty() && s.len() <= 3 && s.bytes().all(|b| b.is_ascii_digit())
            }),
            "Unsupported card glyph ID",
        )?;
        require(ids.insert(id), "Duplicate card glyph ID")?;
        path_data(glyph)?;
    }
    Ok(ids)
}

fn group(node: Node<'_, '_>, ids: &HashSet<&str>) -> Result<(f64, f64, f64)> {
    shape(node, "g", &["transform", "fill"])?;
    paint(node, "fill")?;
    let children = elements(node)?;
    require(!children.is_empty(), "Missing card glyphs")?;
    for glyph in children {
        leaf(
            glyph,
            "use",
            if glyph.has_attribute("x") {
                &["href", "x"]
            } else {
                &["href"]
            },
        )?;
        require(
            glyph
                .attribute("href")
                .and_then(|v| v.strip_prefix('#'))
                .is_some_and(|id| ids.contains(id)),
            "Unresolved card glyph",
        )?;
        if glyph.has_attribute("x") {
            number(glyph, "x", 0.0, 30_000.0)?;
        }
    }
    transform(node)
}

fn validate<'a, 'i>(root: Node<'a, 'i>) -> Result<Vec<Node<'a, 'i>>> {
    shape(
        root,
        "svg",
        &["width", "height", "viewBox", "role", "aria-label"],
    )?;
    for (key, value) in [
        ("width", "380"),
        ("height", "200"),
        ("viewBox", "0 0 380 200"),
        ("role", "img"),
    ] {
        attribute_is(root, key, value)?;
    }
    let children = elements(root)?;
    require(children.len() == TAGS.len(), "Unsupported card layout")?;
    require(
        children
            .iter()
            .zip(TAGS)
            .all(|(n, tag)| n.has_tag_name(tag)),
        "Unsupported card field order",
    )?;
    metadata(children[0], "title")?;
    metadata(children[1], "desc")?;
    require(
        children[0].text() == root.attribute("aria-label"),
        "Card accessibility labels disagree",
    )?;
    require(
        children[0]
            .text()
            .is_some_and(|s| s.starts_with("Aus Accounting MCP on mcprush: security "))
            && children[1].text().is_some_and(|s| {
                s.starts_with("Aus Accounting MCP by ryanduguid on mcprush · security ")
            }),
        "Unsupported card identity",
    )?;
    let ids = definitions(children[2])?;
    for (index, y, scale) in GROUPS {
        let (_, actual_y, actual_scale) = group(children[index], &ids)?;
        require(
            actual_y == y && actual_scale == scale,
            "Unsupported card text layout",
        )?;
    }
    leaf(children[3], "path", &["d", "fill", "stroke"])?;
    path_data(children[3])?;
    attribute_is(children[3], "d", FRAME)?;
    paint(children[3], "fill")?;
    paint(children[3], "stroke")?;
    for index in [6, 9] {
        leaf(
            children[index],
            "circle",
            if index == 6 {
                &["cx", "cy", "r", "fill", "stroke", "stroke-width"]
            } else {
                &["cx", "cy", "r", "fill"]
            },
        )?;
        number(children[index], "cx", 10.0, 370.0)?;
        attribute_is(children[index], "cy", "90")?;
        attribute_is(
            children[index],
            "r",
            if index == 6 { "10.25" } else { "4.5" },
        )?;
        paint(children[index], "fill")?;
    }
    paint(children[6], "stroke")?;
    attribute_is(children[6], "stroke-width", "1.5")?;
    for index in [11, 13, 16, 19] {
        leaf(
            children[index],
            "path",
            &[
                "transform",
                "d",
                "fill",
                "stroke",
                "stroke-width",
                "stroke-linecap",
                "stroke-linejoin",
            ],
        )?;
        transform(children[index])?;
        path_data(children[index])?;
        attribute_is(children[index], "fill", "none")?;
        paint(children[index], "stroke")?;
        attribute_is(children[index], "stroke-width", "1.2")?;
        attribute_is(children[index], "stroke-linecap", "round")?;
        attribute_is(children[index], "stroke-linejoin", "round")?;
    }
    leaf(children[23], "path", &["d", "stroke", "stroke-dasharray"])?;
    path_data(children[23])?;
    attribute_is(children[23], "d", "M18 153.5H362")?;
    paint(children[23], "stroke")?;
    attribute_is(children[23], "stroke-dasharray", "3 3")?;
    leaf(
        children[25],
        "rect",
        &["x", "y", "width", "height", "rx", "fill"],
    )?;
    attribute_is(children[25], "x", "290.81")?;
    for (key, value) in [("y", "167"), ("width", "16"), ("height", "16"), ("rx", "4")] {
        attribute_is(children[25], key, value)?;
    }
    paint(children[25], "fill")?;
    leaf(children[26], "path", &["transform", "d", "fill"])?;
    transform(children[26])?;
    path_data(children[26])?;
    paint(children[26], "fill")?;
    for (value, label) in [(14, 15), (17, 18), (21, 22)] {
        let x = transform(children[value])?.0;
        let label_x = transform(children[label])?.0;
        require(
            x >= 2.0 && label_x > x && label_x <= 362.0,
            "Unsupported card value layout",
        )?;
    }
    Ok(children)
}

/// Restyle the observed Aus Accounting MCP card without fetching or inventing data.
/// Unsupported XML or provider layout changes return an error, never a partial card.
pub fn restyle(input: &str) -> Result<String> {
    let document = parse(input)?;
    let root = document.root_element();
    let children = validate(root)?;
    let mut output = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"380\" height=\"200\" viewBox=\"0 0 380 200\" role=\"img\" aria-label=\"",
    );
    escaped(
        &mut output,
        root.attribute("aria-label").unwrap_or_default(),
    );
    output.push_str("\">");
    for node in &children[..3] {
        write_node(&mut output, *node, None);
    }
    write_painted_node(&mut output, children[3], Some("#000000"), Some(PLATINUM));
    output.push_str(&format!(
        "<rect x=\"12\" y=\"16\" width=\"356\" height=\"30\" rx=\"3\" fill=\"{PURPLE}\"/>"
    ));
    for (value, label) in [(14, 15), (17, 18), (21, 22)] {
        let x = transform(children[value])?.0;
        let width = transform(children[label])?.0 - x + 2.0;
        output.push_str(&format!(
            "<rect x=\"{}\" y=\"117\" width=\"{width}\" height=\"17\" rx=\"2\" fill=\"{PURPLE}\"/>",
            x - 2.0
        ));
    }
    for (index, node) in children.iter().enumerate().skip(4) {
        if [6, 7, 9].contains(&index) {
            // Provider grade and access colours are live signals, not palette tokens.
            write_node(&mut output, *node, None);
        } else {
            let fill = if node.attribute("fill") == Some("none") {
                None
            } else if index == 26 {
                Some("#000000")
            } else if [4, 14, 17, 21].contains(&index) {
                Some(IVORY)
            } else {
                Some(PLATINUM)
            };
            write_painted_node(&mut output, *node, fill, Some(PLATINUM));
        }
    }
    output.push_str("</svg>");
    Ok(output)
}
