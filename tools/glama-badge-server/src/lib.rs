//! Bounded, offline transformation of the observed Glama score SVG grammar.

use roxmltree::Node;
use std::fmt;

mod validation;
use validation::{Badge, grade_colour, number, parse, validate};
mod mcprush;
pub use mcprush::restyle as restyle_mcprush_card;

const IVORY: &str = "#FFFFF0";
const PLATINUM: &str = "#E5E4E2";
const GOLD: &str = "#FFD32A";
const DARK_GOLD: &str = "#9B7735";
const GREEN: &str = "#0F4B32";
const RED: &str = "#E7162F";
const HEALTH: &str =
    "M34.5 7c1.93 0 3.5 1.57 3.5 3.5s-1.57 3.5-3.5 3.5-3.5-1.57-3.5-3.5 1.57-3.5 3.5-3.5Z";

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

fn write_body_node(output: &mut String, node: &Node<'_, '_>, badge: &Badge<'_, '_>) -> Result<()> {
    if badge.grades.contains(node) {
        write_node(output, *node, Some(grade_colour(*node)?));
    } else if badge.health.contains(node) {
        write_node(
            output,
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
            output,
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
            write_node(output, *node, Some(PLATINUM));
        }
        // The original translucent brand background is replaced by flat black.
    } else {
        write_node(output, *node, None);
    }
    Ok(())
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
    write_painted_node(output, node, fill, None);
}

fn write_painted_node(
    output: &mut String,
    node: Node<'_, '_>,
    fill: Option<&str>,
    stroke: Option<&str>,
) {
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
            } else if attribute.name() == "stroke" {
                stroke.unwrap_or(attribute.value())
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
        write_body_node(&mut output, node, &badge)?;
    }
    output.push_str("</svg>");
    Ok(output)
}
