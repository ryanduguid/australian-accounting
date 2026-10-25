use base64::{Engine, engine::general_purpose::STANDARD};
use glama_badge_server::restyle;
use roxmltree::{Document, Node};

const CASES: [(&str, &str); 11] = [
    (
        "unhealthy connector",
        include_str!("fixtures/connector-unhealthy.svg"),
    ),
    ("A/B/C", include_str!("fixtures/grade-c.svg")),
    ("F/-/B", include_str!("fixtures/licence-f-correct.svg")),
    ("unlisted", include_str!("fixtures/licence-f.svg")),
    ("A/-/D", include_str!("fixtures/quality-c.svg")),
    (
        "verified connector",
        include_str!("fixtures/connector-verified-healthy.svg"),
    ),
    (
        "unverified connector",
        include_str!("fixtures/connector-unverified-healthy.svg"),
    ),
    ("A/-/F", include_str!("fixtures/server-a-missing-f.svg")),
    ("A/B/D", include_str!("fixtures/server-a-b-d.svg")),
    ("A/-/A", include_str!("fixtures/server-a-missing-a.svg")),
    ("A/A/D", include_str!("fixtures/server-a-a-d.svg")),
];

fn elements_named<'a, 'i>(doc: &'a Document<'i>, tag: &str) -> Vec<Node<'a, 'i>> {
    doc.descendants()
        .filter(|node| node.has_tag_name(tag))
        .collect()
}

#[test]
fn all_captured_glyphs_metadata_order_and_positions_survive() {
    for (name, input) in CASES {
        let output = restyle(input).unwrap_or_else(|e| panic!("{name}: {e}"));
        let before = Document::parse(input).unwrap();
        let after = Document::parse(&output).unwrap();
        for tag in ["title", "desc"] {
            assert_eq!(
                elements_named(&before, tag)
                    .iter()
                    .map(Node::text)
                    .collect::<Vec<_>>(),
                elements_named(&after, tag)
                    .iter()
                    .map(Node::text)
                    .collect::<Vec<_>>(),
                "{name} {tag}"
            );
        }
        let original_uses = elements_named(&before, "use");
        let styled_uses = elements_named(&after, "use");
        assert_eq!(original_uses.len(), styled_uses.len(), "{name}");
        for (old, new) in original_uses.iter().zip(&styled_uses) {
            for attr in ["href", "x", "y"] {
                assert_eq!(old.attribute(attr), new.attribute(attr), "{name}: {attr}");
            }
            if let Some(fill) = old.attribute("fill") {
                assert_eq!(
                    new.attribute("fill"),
                    Some(match fill {
                        "#37a169" => "#FFFFF0",
                        "#f5a623" => "#FFD32A",
                        "#fe4a4b" => "#E7162F",
                        "#555" => "#E5E4E2",
                        _ => panic!("unexpected fixture fill"),
                    })
                );
            }
        }
        for old in elements_named(&before, "path")
            .into_iter()
            .filter(|node| node.has_attribute("id"))
        {
            let new = after
                .descendants()
                .find(|node| node.attribute("id") == old.attribute("id"))
                .unwrap();
            assert_eq!(
                old.attribute("d"),
                new.attribute("d"),
                "{name} glyph geometry"
            );
        }
        let positive_count = original_uses
            .iter()
            .filter(|node| node.attribute("fill") == Some("#37a169"))
            .count();
        assert_eq!(
            elements_named(&after, "rect")
                .iter()
                .filter(|node| node.attribute("fill") == Some("#0F4B32"))
                .count(),
            positive_count,
            "{name}: green panels"
        );
        assert!(!output.contains("data:") && !output.contains("<image"));
        assert!(elements_named(&after, "linearGradient").is_empty());
        assert!(elements_named(&after, "filter").is_empty());
        assert!(
            styled_uses
                .iter()
                .all(|node| node.attribute("href").unwrap().starts_with("#g"))
        );
        assert_eq!(after.root_element().attribute("role"), Some("img"));
    }
}

#[test]
fn mixed_grades_do_not_use_the_summary_as_the_licence_grade() {
    let output = restyle(CASES[2].1).unwrap();
    let document = Document::parse(&output).unwrap();
    let grades: Vec<_> = document
        .root_element()
        .children()
        .filter(|n| n.has_tag_name("use"))
        .collect();
    assert_eq!(
        grades
            .iter()
            .map(|n| n.attribute("fill").unwrap())
            .collect::<Vec<_>>(),
        ["#E7162F", "#E5E4E2", "#FFFFF0"]
    );
    assert!(output.contains("MCP server rated B on Glama"));
    // The F vector survives even though the summary rating is B.
    let before = Document::parse(CASES[2].1).unwrap();
    assert_eq!(
        grades[0].attribute("href"),
        elements_named(&before, "use")[0].attribute("href")
    );
}

#[test]
fn health_and_verified_ownership_remain_distinct() {
    for (_, input) in [CASES[0], CASES[5], CASES[6]] {
        let output = restyle(input).unwrap();
        let before = Document::parse(input).unwrap();
        let after = Document::parse(&output).unwrap();
        let original_brand = elements_named(&before, "g")[0];
        let styled_brand = elements_named(&after, "g")[0];
        assert_eq!(
            styled_brand.attribute("fill"),
            Some(if original_brand.attribute("fill") == Some("#4493f8") {
                "#FFD32A"
            } else {
                "#E5E4E2"
            })
        );
        let original_health = elements_named(&before, "path")
            .into_iter()
            .find(|n| n.attribute("d").unwrap().starts_with("M34.5 7"))
            .unwrap();
        let styled_health = elements_named(&after, "path")
            .into_iter()
            .find(|n| n.attribute("d") == original_health.attribute("d"))
            .unwrap();
        assert_eq!(
            styled_health.attribute("fill"),
            Some(if original_health.attribute("fill") == Some("#37a169") {
                "#0F4B32"
            } else {
                "#E7162F"
            })
        );
    }
}

#[test]
fn xml_text_is_escaped_without_becoming_markup() {
    let input = CASES[2].1.replace(
        "DealScore MCP",
        "A &amp; B &lt;script&gt; &quot;quoted&quot;",
    );
    let output = restyle(&input).unwrap();
    let document = Document::parse(&output).unwrap();
    assert!(
        elements_named(&document, "title")[0]
            .text()
            .unwrap()
            .starts_with("A & B <script> \"quoted\"")
    );
    assert!(elements_named(&document, "script").is_empty());
}

#[test]
fn malformed_active_external_and_unknown_svg_is_rejected() {
    let input = CASES[2].1;
    let mutations = [
        input[..input.len() - 5].to_owned(),
        format!("<!DOCTYPE svg [<!ENTITY x 'pass'>]>{input}"),
        format!("<!DOCTYPE svg SYSTEM 'https://example.invalid/entity'>{input}"),
        input.replace("<title>", "<?injected instruction?><title>"),
        input.replace("<title>", "<!-- unexpected --><title>"),
        input.replace("<title>", "<script>alert(1)</script><title>"),
        input.replace("<title>", "<foreignObject/><title>"),
        input.replace("width=\"110\"", "width=\"110\" onload=\"alert(1)\""),
        input.replace("href=\"#g0\"", "href=\"https://example.invalid/x.svg#g0\""),
        input.replace("href=\"#g0\"", "href=\"#absent\""),
        input.replace("id=\"g1\"", "id=\"g0\""),
        input.replace("<defs>", "<defs><use href=\"#g0\"/>"),
        input.replace("#37a169", "#008000"),
        input.replace("viewBox=\"0 0 110 20\"", "viewBox=\"0 0 1 1\""),
        input.replace("y=\"14\"", "y=\"NaN\""),
        input.replace("x=\"14.55\"", "x=\"-999\""),
        input.replace(
            "<defs>",
            "<defs style=\"filter:url(https://example.invalid)\">",
        ),
        input.replace(
            "xmlns=\"http://www.w3.org/2000/svg\"",
            "xmlns=\"https://example.invalid/svg\"",
        ),
        input.replace(
            "width=\"110\"",
            "width=\"110\" xmlns:other=\"https://example.invalid\"",
        ),
        input.replace(
            "width=\"110\"",
            "width=\"110\" xml:base=\"https://example.invalid\"",
        ),
        input.replace("id=\"g0\"", "id=\"g0\" onclick=\"alert(1)\""),
    ];
    for (index, changed) in mutations.iter().enumerate() {
        assert_ne!(input, changed, "mutation {index} must affect the fixture");
        assert!(restyle(changed).is_err(), "accepted mutation {index}");
    }
}

fn with_logo(input: &str, logo: &str) -> String {
    let document = Document::parse(input).unwrap();
    let image = elements_named(&document, "image")[0];
    input.replace(
        image.attribute("href").unwrap(),
        &format!("data:image/svg+xml;base64,{}", STANDARD.encode(logo)),
    )
}

#[test]
fn embedded_svg_is_validated_as_strictly_as_the_outer_document() {
    let input = CASES[2].1;
    let document = Document::parse(input).unwrap();
    let href = elements_named(&document, "image")[0]
        .attribute("href")
        .unwrap();
    let decoded = STANDARD
        .decode(href.strip_prefix("data:image/svg+xml;base64,").unwrap())
        .unwrap();
    let original = String::from_utf8(decoded).unwrap();
    for changed in [
        original.replace("<path", "<script/> <path"),
        original.replace(
            "<path",
            "<image href=\"https://example.invalid/x.svg\"/><path",
        ),
        original.replace("fill=\"white\"", "fill=\"url(https://example.invalid)\""),
        original.replace("<path", "<path onload=\"alert(1)\""),
        format!("<!DOCTYPE svg [<!ENTITY x 'pass'>]>{original}"),
        original.replace(
            "xmlns=\"http://www.w3.org/2000/svg\"",
            "xmlns=\"https://example.invalid\"",
        ),
        original.replace("viewBox=\"0 0 220 300\"", "viewBox=\"0 0 1 1\""),
    ] {
        assert!(restyle(&with_logo(input, &changed)).is_err());
    }
    assert!(restyle(&input.replace(href, "data:image/svg+xml;base64,!!")).is_err());
    assert!(restyle(&input.replace(href, "https://example.invalid/logo.svg")).is_err());
    assert!(restyle(&input.replace(href, "data:image/png;base64,aGVsbG8=")).is_err());
}

#[test]
fn byte_node_depth_and_logo_limits_fail_closed() {
    assert!(restyle(&" ".repeat(32_769)).is_err());
    let excessive_nodes = CASES[2]
        .1
        .replace("<title>", &format!("{}<title>", "<g/>".repeat(257)));
    assert!(restyle(&excessive_nodes).is_err());
    let deep = CASES[2]
        .1
        .replace("<title>", &format!("{}<title>", "<g>".repeat(10)))
        .replace("</title>", &format!("</title>{}", "</g>".repeat(10)));
    assert!(restyle(&deep).is_err());
    assert!(restyle(&with_logo(CASES[2].1, &" ".repeat(8_193))).is_err());
}

#[test]
fn output_is_not_accepted_as_provider_input() {
    assert!(restyle(&restyle(CASES[2].1).unwrap()).is_err());
}

#[test]
fn malformed_invisible_and_overflowing_paths_are_rejected() {
    let input = CASES[2].1;
    let document = Document::parse(input).unwrap();
    let glyph = document
        .descendants()
        .find(|n| n.has_tag_name("path") && n.attribute("id") == Some("g0"))
        .unwrap();
    let original = format!("d=\"{}\"", glyph.attribute("d").unwrap());
    for invalid in [
        "M0 0L",
        "M1e999 0L1 1",
        "M0 0L1e308 1",
        "M0 0X1 1",
        "M0 0",
        "",
        "Z",
    ] {
        assert!(
            restyle(&input.replace(&original, &format!("d=\"{invalid}\""))).is_err(),
            "accepted {invalid}"
        );
    }
    let oversized_path = format!("M0 0{}", "L1 1".repeat(2049));
    assert!(restyle(&input.replace(&original, &format!("d=\"{oversized_path}\""))).is_err());
    let logo = "<svg xmlns=\"http://www.w3.org/2000/svg\" fill=\"none\" height=\"14\" width=\"10.266666666666667\" viewBox=\"0 0 220 300\"><path fill=\"white\" d=\"M0 0L\"/></svg>";
    assert!(restyle(&with_logo(input, logo)).is_err());
}

#[test]
fn embedded_logo_geometry_and_viewport_survive_inlining() {
    for (name, input) in CASES {
        let before = Document::parse(input).unwrap();
        let Some(image) = elements_named(&before, "image").first().copied() else {
            continue;
        };
        let encoded = image
            .attribute("href")
            .unwrap()
            .strip_prefix("data:image/svg+xml;base64,")
            .unwrap();
        let decoded = String::from_utf8(STANDARD.decode(encoded).unwrap()).unwrap();
        let original_logo = Document::parse(&decoded).unwrap();
        let output = restyle(input).unwrap();
        let after = Document::parse(&output).unwrap();
        let nested = elements_named(&after, "svg")
            .into_iter()
            .find(|n| n.parent().is_some_and(|p| p.has_tag_name("svg")))
            .unwrap();
        for key in ["x", "y", "width", "height", "preserveAspectRatio"] {
            assert_eq!(nested.attribute(key), image.attribute(key), "{name}: {key}");
        }
        assert_eq!(
            nested.attribute("viewBox"),
            original_logo.root_element().attribute("viewBox"),
            "{name}"
        );
        let original_path = elements_named(&original_logo, "path")[0];
        let inline_path = nested.children().find(|n| n.has_tag_name("path")).unwrap();
        assert_eq!(
            original_path.attribute("d"),
            inline_path.attribute("d"),
            "{name}: logo path"
        );
        assert_eq!(inline_path.attribute("fill"), Some("#FFFFF0"));
    }
}
