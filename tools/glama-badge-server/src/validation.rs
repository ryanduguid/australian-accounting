use super::{GOLD, HEALTH, IVORY, InvalidBadge, PLATINUM, RED, Result, require, write_node};
use base64::{Engine, engine::general_purpose::STANDARD};
use roxmltree::{Document, Node, NodeType, ParsingOptions};
use std::collections::HashSet;
use svgtypes::{PathParser, PathSegment};

const SVG: &str = "http://www.w3.org/2000/svg";
const MAX_BYTES: usize = 32_768;
const BACKGROUND: &str =
    "M2 0h106c1.1 0 2 .9 2 2v16c0 1.1-.9 2-2 2h-106c-1.1 0-2-.9-2-2v-16c0-1.1.9-2 2-2Z";
const FRAME: &str = "M2 0h106c1.1 0 2 .9 2 2v16c0 1.1-.9 2-2 2h-106c-1.1 0-2-.9-2-2v-16c0-1.1.9-2 2-2Zm0 1h106c.55 0 1 .45 1 1v16c0 .55-.45 1-1 1h-106c-.55 0-1-.45-1-1v-16c0-.55.45-1 1-1Z";
const HEALTH_FRAME: &str = "M34.5 7c1.93 0 3.5 1.57 3.5 3.5s-1.57 3.5-3.5 3.5-3.5-1.57-3.5-3.5 1.57-3.5 3.5-3.5Zm0 1c1.38 0 2.5 1.12 2.5 2.5s-1.12 2.5-2.5 2.5-2.5-1.12-2.5-2.5 1.12-2.5 2.5-2.5Z";

pub(super) fn parse(input: &str) -> Result<Document<'_>> {
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

pub(super) fn elements<'a, 'i>(node: Node<'a, 'i>) -> Result<Vec<Node<'a, 'i>>> {
    require(
        node.children()
            .all(|child| child.is_element() || child.text().is_some_and(|t| t.trim().is_empty())),
        "Unexpected text",
    )?;
    Ok(node.children().filter(Node::is_element).collect())
}

pub(super) fn shape(node: Node<'_, '_>, name: &str, attributes: &[&str]) -> Result<()> {
    require(node.tag_name().name() == name, "Unexpected element")?;
    require(
        node.attributes().len() == attributes.len()
            && attributes.iter().all(|key| node.has_attribute(*key)),
        "Unexpected attributes",
    )
}

pub(super) fn leaf(node: Node<'_, '_>, name: &str, attributes: &[&str]) -> Result<()> {
    shape(node, name, attributes)?;
    require(!node.has_children(), "Unexpected element children")
}

pub(super) fn attribute_is(node: Node<'_, '_>, key: &str, value: &str) -> Result<()> {
    require(
        node.attribute(key) == Some(value),
        "Unexpected attribute value",
    )
}

pub(super) fn metadata(node: Node<'_, '_>, name: &str) -> Result<()> {
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

fn segment_bounded(segment: PathSegment) -> bool {
    let bounded = |values: &[f64]| values.iter().all(|n| n.is_finite() && n.abs() <= 1024.0);
    match segment {
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
    }
}

pub(super) fn path_data(node: Node<'_, '_>) -> Result<()> {
    let data = node.attribute("d").ok_or(InvalidBadge("Missing path"))?;
    require(
        !data.is_empty()
            && data.len() <= 16_384
            && data
                .bytes()
                .all(|b| b.is_ascii_digit() || b"MmZzLlHhVvCcSsQqTtAaEe.,-+ \t\r\n".contains(&b)),
        "Unsupported path data",
    )?;
    let mut drawing = false;
    let mut count = 0;
    for result in PathParser::from(data) {
        let segment = result.map_err(|_| InvalidBadge("Malformed SVG path"))?;
        count += 1;
        require(count <= 2048, "Path exceeds segment limit")?;
        let valid = segment_bounded(segment);
        require(valid, "Path coordinates exceed bounds")?;
        drawing |= !matches!(
            segment,
            PathSegment::MoveTo { .. } | PathSegment::ClosePath { .. }
        );
    }
    require(drawing, "Empty glyph geometry")
}

pub(super) fn number(node: Node<'_, '_>, attribute: &str, min: f64, max: f64) -> Result<f64> {
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

pub(super) fn grade_colour(node: Node<'_, '_>) -> Result<&'static str> {
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

pub(super) struct Badge<'a, 'i> {
    pub(super) children: Vec<Node<'a, 'i>>,
    pub(super) metadata_count: usize,
    pub(super) grades: Vec<Node<'a, 'i>>,
    pub(super) health: Vec<Node<'a, 'i>>,
    pub(super) brand: Node<'a, 'i>,
    pub(super) inline_logo: Option<String>,
    pub(super) verified: bool,
}

fn badge_header<'a, 'i>(root: Node<'a, 'i>) -> Result<(Vec<Node<'a, 'i>>, usize)> {
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
    Ok((children, metadata_count))
}

fn ownership(body: &[Node<'_, '_>]) -> Result<bool> {
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
    Ok(verified)
}

fn unlisted_brand<'a, 'i>(
    body: &[Node<'a, 'i>],
    title: Node<'a, 'i>,
    verified: bool,
) -> Result<Node<'a, 'i>> {
    require(body.len() == 4 && !verified, "Unknown unlisted layout")?;
    require(
        title.text() == Some("This MCP server is not listed on Glama"),
        "Unknown unlisted message",
    )?;
    let brand = body[2];
    shape(brand, "g", &["fill"])?;
    attribute_is(brand, "fill", "#7d8284")?;
    let uses = elements(brand)?;
    require(uses.len() == 8, "Unknown unlisted glyph layout")?;
    for node in uses {
        glyph_use(node, false)?;
    }
    Ok(brand)
}

fn score_slots<'a, 'i>(body: &[Node<'a, 'i>]) -> Result<(Vec<Node<'a, 'i>>, Vec<Node<'a, 'i>>)> {
    let mut grades = Vec::new();
    let mut health = Vec::new();
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
    Ok((grades, health))
}

fn listed_brand<'a, 'i>(body: &[Node<'a, 'i>], verified: bool) -> Result<(Node<'a, 'i>, String)> {
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
    let inline_logo = logo(body[7])?;
    let brand = body[8];
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
    Ok((brand, inline_logo))
}

pub(super) fn validate<'a, 'i>(root: Node<'a, 'i>) -> Result<Badge<'a, 'i>> {
    let (children, metadata_count) = badge_header(root)?;
    let body = &children[metadata_count..];
    require(body.len() >= 4, "Incomplete badge body")?;
    let verified = ownership(body)?;
    let defs = *body
        .last()
        .ok_or(InvalidBadge("Missing glyph definitions"))?;
    definitions(defs, root)?;
    let (grades, health, brand, inline_logo) = if metadata_count == 1 {
        let brand = unlisted_brand(body, children[0], verified)?;
        (Vec::new(), Vec::new(), brand, None)
    } else {
        require(body.len() == 10, "Unknown listed layout")?;
        let (grades, health) = score_slots(body)?;
        let (brand, logo) = listed_brand(body, verified)?;
        (grades, health, brand, Some(logo))
    };
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
