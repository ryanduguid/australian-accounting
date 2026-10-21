//! Bounded, offline transformation of the observed Glama score SVG grammar.

use base64::{Engine, engine::general_purpose::STANDARD};
use roxmltree::{Document, Node, NodeType, ParsingOptions};
use std::{collections::HashSet, fmt};
use svgtypes::{PathParser, PathSegment};

const SVG: &str = "http://www.w3.org/2000/svg";
const MAX_BYTES: usize = 32_768;
const IVORY: &str = "#FFFFF0";
const PLATINUM: &str = "#E5E4E2";
const GOLD: &str = "#FFD32A";
const DARK_GOLD: &str = "#9B7735";
const GREEN: &str = "#0F4B32";
const RED: &str = "#E7162F";
const BACKGROUND: &str =
    "M2 0h106c1.1 0 2 .9 2 2v16c0 1.1-.9 2-2 2h-106c-1.1 0-2-.9-2-2v-16c0-1.1.9-2 2-2Z";
const FRAME: &str = "M2 0h106c1.1 0 2 .9 2 2v16c0 1.1-.9 2-2 2h-106c-1.1 0-2-.9-2-2v-16c0-1.1.9-2 2-2Zm0 1h106c.55 0 1 .45 1 1v16c0 .55-.45 1-1 1h-106c-.55 0-1-.45-1-1v-16c0-.55.45-1 1-1Z";
const HEALTH: &str =
    "M34.5 7c1.93 0 3.5 1.57 3.5 3.5s-1.57 3.5-3.5 3.5-3.5-1.57-3.5-3.5 1.57-3.5 3.5-3.5Z";
const HEALTH_FRAME: &str = "M34.5 7c1.93 0 3.5 1.57 3.5 3.5s-1.57 3.5-3.5 3.5-3.5-1.57-3.5-3.5 1.57-3.5 3.5-3.5Zm0 1c1.38 0 2.5 1.12 2.5 2.5s-1.12 2.5-2.5 2.5-2.5-1.12-2.5-2.5 1.12-2.5 2.5-2.5Z";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidBadge(&'static str);

impl fmt::Display for InvalidBadge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for InvalidBadge {}

type Result<T> = std::result::Result<T, InvalidBadge>;

fn require(condition: bool, message: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(InvalidBadge(message))
    }
}

fn parse(input: &str) -> Result<Document<'_>> {
    require(input.len() <= MAX_BYTES, "SVG exceeds byte limit")?;
    let document = Document::parse_with_options(
        input,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: 256,
            ..ParsingOptions::default()
        },
    )
    .map_err(|_| InvalidBadge("Invalid or forbidden XML"))?;
    for node in document.descendants() {
        require(
            node.ancestors().take(8).count() < 8,
            "SVG exceeds depth limit",
        )?;
        match node.node_type() {
            NodeType::Root => {}
            NodeType::Element => {
                require(
                    node.tag_name().namespace() == Some(SVG),
                    "Foreign namespace",
                )?;
                require(
                    node.namespaces()
                        .all(|ns| ns.name().is_none() && ns.uri() == SVG),
                    "Unexpected namespace declaration",
                )?;
                require(
                    node.attributes().all(|a| a.namespace().is_none()),
                    "Namespaced attribute",
                )?;
            }
            NodeType::Text => {}
            _ => {
                return Err(InvalidBadge(
                    "Comments and processing instructions are forbidden",
                ));
            }
        }
    }
    Ok(document)
}

fn elements<'a, 'i>(node: Node<'a, 'i>) -> Result<Vec<Node<'a, 'i>>> {
    require(
        node.children()
            .all(|child| child.is_element() || child.text().is_some_and(|t| t.trim().is_empty())),
        "Unexpected text",
    )?;
    Ok(node.children().filter(Node::is_element).collect())
}

fn shape(node: Node<'_, '_>, name: &str, attributes: &[&str]) -> Result<()> {
    require(node.tag_name().name() == name, "Unexpected element")?;
    require(
        node.attributes().len() == attributes.len()
            && attributes.iter().all(|key| node.has_attribute(*key)),
        "Unexpected attributes",
    )
}

fn leaf(node: Node<'_, '_>, name: &str, attributes: &[&str]) -> Result<()> {
    shape(node, name, attributes)?;
    require(!node.has_children(), "Unexpected element children")
}

fn attribute_is(node: Node<'_, '_>, key: &str, value: &str) -> Result<()> {
    require(
        node.attribute(key) == Some(value),
        "Unexpected attribute value",
    )
}

fn metadata(node: Node<'_, '_>, name: &str) -> Result<()> {
    shape(node, name, &[])?;
    require(
        node.children().all(|n| n.is_text()),
        "Metadata must be plain text",
    )?;
    require(
        node.text().is_some_and(|s| !s.trim().is_empty()),
        "Missing metadata",
    )
}

fn path_data(node: Node<'_, '_>) -> Result<()> {
    let data = node.attribute("d").ok_or(InvalidBadge("Missing path"))?;
    require(
        !data.is_empty()
            && data.len() <= 16_384
            && data
                .bytes()
                .all(|b| b.is_ascii_digit() || b"MmZzLlHhVvCcSsQqTtAaEe.,-+ \t\r\n".contains(&b)),
        "Unsupported path data",
    )?;
    let bounded = |values: &[f64]| values.iter().all(|n| n.is_finite() && n.abs() <= 1024.0);
    let mut drawing = false;
    let mut count = 0;
    for result in PathParser::from(data) {
        let segment = result.map_err(|_| InvalidBadge("Malformed SVG path"))?;
        count += 1;
        require(count <= 2048, "Path exceeds segment limit")?;
        let valid = match segment {
            PathSegment::MoveTo { x, y, .. }
            | PathSegment::LineTo { x, y, .. }
            | PathSegment::SmoothQuadratic { x, y, .. } => bounded(&[x, y]),
            PathSegment::HorizontalLineTo { x, .. } => bounded(&[x]),
            PathSegment::VerticalLineTo { y, .. } => bounded(&[y]),
            PathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
                ..
            } => bounded(&[x1, y1, x2, y2, x, y]),
            PathSegment::SmoothCurveTo { x2, y2, x, y, .. } => bounded(&[x2, y2, x, y]),
            PathSegment::Quadratic { x1, y1, x, y, .. } => bounded(&[x1, y1, x, y]),
            PathSegment::EllipticalArc {
                rx,
                ry,
                x_axis_rotation,
                x,
                y,
                ..
            } => bounded(&[rx, ry, x_axis_rotation, x, y]),
            PathSegment::ClosePath { .. } => true,
        };
        require(valid, "Path coordinates exceed bounds")?;
        drawing |= !matches!(
            segment,
            PathSegment::MoveTo { .. } | PathSegment::ClosePath { .. }
        );
    }
    require(drawing, "Empty glyph geometry")
}

fn number(node: Node<'_, '_>, attribute: &str, min: f64, max: f64) -> Result<f64> {
    let value = node
        .attribute(attribute)
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or(InvalidBadge("Invalid position"))?;
    require(
        value.is_finite() && value >= min && value <= max,
        "Position outside badge",
    )?;
    Ok(value)
}

fn grade_colour(node: Node<'_, '_>) -> Result<&'static str> {
    match node.attribute("fill") {
        Some("#37a169") => Ok(IVORY),
        Some("#f5a623") => Ok(GOLD),
        Some("#fe4a4b") => Ok(RED),
        Some("#555") => Ok(PLATINUM),
        _ => Err(InvalidBadge("Unknown grade colour")),
    }
}

fn glyph_use(node: Node<'_, '_>, grade: bool) -> Result<()> {
    leaf(
        node,
        "use",
        if grade {
            &["href", "x", "y", "fill"]
        } else {
            &["href", "x", "y"]
        },
    )?;
    let reference = node.attribute("href").unwrap_or_default();
    require(
        reference
            .strip_prefix("#g")
            .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())),
        "Only local glyph references are accepted",
    )?;
    number(node, "x", 0.0, 109.0)?;
    attribute_is(node, "y", "14")?;
    if grade {
        grade_colour(node)?;
    }
    Ok(())
}

fn definitions(node: Node<'_, '_>, root: Node<'_, '_>) -> Result<()> {
    shape(node, "defs", &[])?;
    let paths = elements(node)?;
    require(
        !paths.is_empty() && paths.len() <= 16,
        "Unexpected glyph count",
    )?;
    let mut ids = HashSet::new();
    for path in paths {
        leaf(path, "path", &["id", "d"])?;
        let id = path.attribute("id").unwrap_or_default();
        require(
            id.strip_prefix('g')
                .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())),
            "Unexpected glyph ID",
        )?;
        require(ids.insert(id), "Duplicate glyph ID")?;
        path_data(path)?;
    }
    for used in root.descendants().filter(|n| n.has_tag_name("use")) {
        require(
            used.attribute("href")
                .and_then(|s| s.strip_prefix('#'))
                .is_some_and(|id| ids.contains(id)),
            "Unresolved glyph reference",
        )?;
    }
    Ok(())
}

fn logo(node: Node<'_, '_>) -> Result<String> {
    leaf(
        node,
        "image",
        &["x", "y", "width", "height", "href", "preserveAspectRatio"],
    )?;
    for (key, value) in [
        ("x", "58"),
        ("y", "3"),
        ("width", "10"),
        ("height", "14"),
        ("preserveAspectRatio", "none"),
    ] {
        attribute_is(node, key, value)?;
    }
    let encoded = node
        .attribute("href")
        .and_then(|s| s.strip_prefix("data:image/svg+xml;base64,"))
        .ok_or(InvalidBadge("Only the embedded SVG logo is accepted"))?;
    require(encoded.len() <= 8_192, "Embedded logo exceeds byte limit")?;
    let decoded = STANDARD
        .decode(encoded)
        .map_err(|_| InvalidBadge("Invalid base64 logo"))?;
    let text = std::str::from_utf8(&decoded).map_err(|_| InvalidBadge("Invalid logo encoding"))?;
    let document = parse(text)?;
    let root = document.root_element();
    shape(root, "svg", &["fill", "height", "width", "viewBox"])?;
    attribute_is(root, "fill", "none")?;
    attribute_is(root, "height", "14")?;
    attribute_is(root, "viewBox", "0 0 220 300")?;
    number(root, "width", 10.26, 10.27)?;
    let paths = elements(root)?;
    require(paths.len() == 1, "Unknown logo geometry")?;
    leaf(paths[0], "path", &["d", "fill"])?;
    attribute_is(paths[0], "fill", "white")?;
    path_data(paths[0])?;
    let mut output = String::from(
        "<svg x=\"58\" y=\"3\" width=\"10\" height=\"14\" viewBox=\"0 0 220 300\" preserveAspectRatio=\"none\">",
    );
    write_node(&mut output, paths[0], Some(IVORY));
    output.push_str("</svg>");
    Ok(output)
}

struct Badge<'a, 'i> {
    children: Vec<Node<'a, 'i>>,
    metadata_count: usize,
    grades: Vec<Node<'a, 'i>>,
    health: Vec<Node<'a, 'i>>,
    brand: Node<'a, 'i>,
    inline_logo: Option<String>,
    verified: bool,
}

fn validate<'a, 'i>(root: Node<'a, 'i>) -> Result<Badge<'a, 'i>> {
    shape(root, "svg", &["width", "height", "viewBox"])?;
    for (key, value) in [
        ("width", "110"),
        ("height", "20"),
        ("viewBox", "0 0 110 20"),
    ] {
        attribute_is(root, key, value)?;
    }
    let children = elements(root)?;
    require(children.len() >= 5, "Incomplete badge")?;
    metadata(children[0], "title")?;
    let listed = children[1].has_tag_name("desc");
    let metadata_count = if listed {
        metadata(children[1], "desc")?;
        2
    } else {
        1
    };
    let body = &children[metadata_count..];
    require(body.len() >= 4, "Incomplete badge body")?;
    leaf(body[0], "path", &["d", "fill"])?;
    attribute_is(body[0], "d", BACKGROUND)?;
    leaf(body[1], "path", &["d", "fill", "fill-rule"])?;
    attribute_is(body[1], "d", FRAME)?;
    attribute_is(body[1], "fill-rule", "evenodd")?;
    let verified = match (body[0].attribute("fill"), body[1].attribute("fill")) {
        (Some("#0e1618"), Some("#1b2326")) => false,
        (Some("#131d2e"), Some("#21365a")) => true,
        _ => return Err(InvalidBadge("Unknown ownership palette")),
    };
    let defs = *body
        .last()
        .ok_or(InvalidBadge("Missing glyph definitions"))?;
    definitions(defs, root)?;
    let mut grades = Vec::new();
    let mut health = Vec::new();
    let brand;
    let inline_logo;
    if !listed {
        require(body.len() == 4 && !verified, "Unknown unlisted layout")?;
        require(
            children[0].text() == Some("This MCP server is not listed on Glama"),
            "Unknown unlisted message",
        )?;
        brand = body[2];
        shape(brand, "g", &["fill"])?;
        attribute_is(brand, "fill", "#7d8284")?;
        let uses = elements(brand)?;
        require(uses.len() == 8, "Unknown unlisted glyph layout")?;
        for node in uses {
            glyph_use(node, false)?;
        }
        inline_logo = None;
    } else {
        require(body.len() == 10, "Unknown listed layout")?;
        if body[3].has_tag_name("use") && body[4].has_tag_name("use") {
            for node in &body[2..5] {
                glyph_use(*node, true)?;
                grades.push(*node);
            }
            let positions: Vec<_> = grades
                .iter()
                .map(|n| number(*n, "x", 10.0, 38.0))
                .collect::<Result<_>>()?;
            require(
                positions.windows(2).all(|p| p[1] - p[0] >= 9.0),
                "Overlapping grade slots",
            )?;
        } else {
            glyph_use(body[2], true)?;
            attribute_is(body[2], "x", "17.89")?;
            grades.push(body[2]);
            for (node, data, attrs) in [
                (body[3], HEALTH, &["d", "fill"][..]),
                (body[4], HEALTH_FRAME, &["d", "fill", "fill-rule"][..]),
            ] {
                leaf(node, "path", attrs)?;
                attribute_is(node, "d", data)?;
                require(
                    matches!(node.attribute("fill"), Some("#37a169" | "#fe4a4b")),
                    "Unknown health colour",
                )?;
                health.push(node);
            }
            attribute_is(body[4], "fill-rule", "evenodd")?;
            require(
                body[3].attribute("fill") == body[4].attribute("fill"),
                "Inconsistent health state",
            )?;
        }
        leaf(
            body[5],
            "rect",
            &["x", "y", "width", "height", "fill", "fill-opacity"],
        )?;
        for (key, value) in [
            ("x", "55"),
            ("y", "1"),
            ("width", "54"),
            ("height", "18"),
            ("fill", "#fff"),
            ("fill-opacity", ".051"),
        ] {
            attribute_is(body[5], key, value)?;
        }
        leaf(body[6], "rect", &["x", "y", "width", "height", "fill"])?;
        for (key, value) in [("x", "55"), ("y", "1"), ("width", "1"), ("height", "18")] {
            attribute_is(body[6], key, value)?;
        }
        require(
            body[6].attribute("fill") == body[1].attribute("fill"),
            "Inconsistent ownership divider",
        )?;
        inline_logo = Some(logo(body[7])?);
        brand = body[8];
        shape(brand, "g", &["fill"])?;
        attribute_is(brand, "fill", if verified { "#4493f8" } else { "#fff" })?;
        let uses = elements(brand)?;
        require(uses.len() == 5, "Unknown wordmark layout")?;
        for (node, position) in uses
            .into_iter()
            .zip(["74.2", "82.4", "85.07", "91.24", "100.88"])
        {
            glyph_use(node, false)?;
            attribute_is(node, "x", position)?;
        }
    }
    Ok(Badge {
        children,
        metadata_count,
        grades,
        health,
        brand,
        inline_logo,
        verified,
    })
}

fn escaped(output: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\r' => output.push_str("&#13;"),
            '\n' => output.push_str("&#10;"),
            '\t' => output.push_str("&#9;"),
            _ => output.push(ch),
        }
    }
}

fn write_node(output: &mut String, node: Node<'_, '_>, fill: Option<&str>) {
    let name = node.tag_name().name();
    output.push('<');
    output.push_str(name);
    for attribute in node.attributes() {
        output.push(' ');
        output.push_str(attribute.name());
        output.push_str("=\"");
        escaped(
            output,
            if attribute.name() == "fill" {
                fill.unwrap_or(attribute.value())
            } else {
                attribute.value()
            },
        );
        output.push('"');
    }
    if node.has_children() {
        output.push('>');
        for child in node.children() {
            if child.is_element() {
                write_node(output, child, None);
            } else if let Some(text) = child.text() {
                escaped(output, text);
            }
        }
        output.push_str("</");
        output.push_str(name);
        output.push('>');
    } else {
        output.push_str("/>");
    }
}

/// Restyle a captured provider badge, or reject it without producing an SVG.
pub fn restyle(input: &str) -> Result<String> {
    let document = parse(input)?;
    let badge = validate(document.root_element())?;
    let mut output = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"110\" height=\"20\" viewBox=\"0 0 110 20\" role=\"img\">",
    );
    for node in &badge.children[..badge.metadata_count] {
        write_node(&mut output, *node, None);
    }
    let body = &badge.children[badge.metadata_count..];
    write_node(&mut output, body[0], Some("#000000"));
    for node in &badge.grades {
        if node.attribute("fill") == Some("#37a169") {
            let x = number(*node, "x", 0.0, 109.0)? - 1.5;
            output.push_str(&format!(
                "<rect x=\"{x}\" y=\"3\" width=\"11\" height=\"14\" rx=\"2\" fill=\"{GREEN}\"/>"
            ));
        }
    }
    write_node(&mut output, body[1], Some(DARK_GOLD));
    output.push_str(&format!(
        "<path d=\"M3 1.5h104\" fill=\"none\" stroke=\"{GOLD}\" stroke-width=\".5\"/>"
    ));
    for node in &body[2..] {
        if badge.grades.contains(node) {
            write_node(&mut output, *node, Some(grade_colour(*node)?));
        } else if badge.health.contains(node) {
            write_node(
                &mut output,
                *node,
                Some(if node.attribute("fill") == Some("#37a169") {
                    GREEN
                } else {
                    RED
                }),
            );
            if *node == badge.health[0] {
                output.push_str(&format!(
                    "<path d=\"{HEALTH}\" fill=\"none\" stroke=\"{IVORY}\" stroke-width=\".5\"/>"
                ));
            }
        } else if *node == badge.brand {
            write_node(
                &mut output,
                *node,
                Some(if badge.verified { GOLD } else { PLATINUM }),
            );
        } else if node.has_tag_name("image") {
            output.push_str(
                badge
                    .inline_logo
                    .as_deref()
                    .ok_or(InvalidBadge("Missing validated logo"))?,
            );
        } else if node.has_tag_name("rect") {
            if node.attribute("width") == Some("1") {
                write_node(&mut output, *node, Some(PLATINUM));
            }
            // The original translucent brand background is replaced by flat black.
        } else {
            write_node(&mut output, *node, None);
        }
    }
    output.push_str("</svg>");
    Ok(output)
}
