use glama_badge_server::restyle_mcprush_card;
use roxmltree::{Document, Node};

const DARK: &str = include_str!("fixtures/mcprush/card-dark.svg");
const LIGHT: &str = include_str!("fixtures/mcprush/card-light.svg");

fn attributes(node: Node<'_, '_>) -> Vec<(String, String)> {
    node.attributes()
        .map(|a| (a.name().to_owned(), a.value().to_owned()))
        .collect()
}

fn preservation(source: &str, output: &str) {
    let before = Document::parse(source).unwrap();
    let after = Document::parse(output).unwrap();
    for tag in ["title", "desc"] {
        let value = |doc: &Document<'_>| {
            doc.descendants()
                .find(|n| n.has_tag_name(tag))
                .unwrap()
                .text()
                .unwrap()
                .to_owned()
        };
        assert_eq!(value(&before), value(&after));
    }
    assert_eq!(
        before.root_element().attribute("aria-label"),
        after.root_element().attribute("aria-label")
    );
    // All text remains the provider's glyph geometry, placement and order.
    for tag in ["path", "use", "g"] {
        let geometry = |doc: &Document<'_>| {
            doc.descendants()
                .filter(|n| n.has_tag_name(tag))
                .map(|n| {
                    attributes(n)
                        .into_iter()
                        .filter(|(k, _)| !["fill", "stroke"].contains(&k.as_str()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            geometry(&before),
            geometry(&after),
            "{tag} geometry changed"
        );
    }
    let source_children: Vec<_> = before
        .root_element()
        .children()
        .filter(Node::is_element)
        .collect();
    let target_children: Vec<_> = after
        .root_element()
        .children()
        .filter(Node::is_element)
        .collect();
    // Four new purple backgrounds precede the source body, so locate live signals by geometry.
    for index in [6, 7, 9] {
        let node = source_children[index];
        assert!(
            target_children
                .iter()
                .any(|out| out.tag_name().name() == node.tag_name().name()
                    && attributes(*out) == attributes(node))
        );
    }
}

#[test]
fn both_provider_themes_preserve_all_fields_and_use_the_palette() {
    let output = restyle_mcprush_card(DARK).unwrap();
    preservation(DARK, &output);
    preservation(LIGHT, &restyle_mcprush_card(LIGHT).unwrap());
    // Theme-specific grade backgrounds remain live and may differ.
    assert!(output.contains("#000000"));
    assert!(output.contains("#66023C"));
    assert!(output.contains("#E5E4E2"));
    assert!(output.contains("#FFFFF0"));
    assert!(!output.contains("linearGradient"));
    assert!(output.contains("319/mo PyPI"));
    assert!(output.contains("works with 14 clients"));
}

#[test]
fn falling_grade_and_changed_downloads_are_not_frozen() {
    let changed = DARK
        .replace("security A", "security F")
        .replace("#BEF264", "#EF4444")
        .replace("#1E2312", "#450A0A")
        .replace(
            "scale(0.0125)\" fill=\"#EF4444\"><use href=\"#g0\"",
            "scale(0.0125)\" fill=\"#EF4444\"><use href=\"#g46\"",
        )
        .replace("319/mo PyPI", "919/mo PyPI")
        .replace(
            "translate(36 130) scale(0.0125)\" fill=\"#FAFAFA\"><use href=\"#g29\"",
            "translate(36 130) scale(0.0125)\" fill=\"#FAFAFA\"><use href=\"#g31\"",
        );
    assert!(changed.contains("#g46\"/></g><g transform=\"translate(47"));
    let output = restyle_mcprush_card(&changed).unwrap();
    preservation(&changed, &output);
    assert!(output.contains("security F"));
    assert!(output.contains("919/mo PyPI"));
    assert!(output.contains("#EF4444"));
    assert!(!output.contains("#BEF264"));
}

#[test]
fn changed_metric_positions_move_the_value_backgrounds() {
    let changed = DARK
        .replace("translate(36 130)", "translate(37 130)")
        .replace("translate(80.24 130)", "translate(90.24 130)");
    let output = restyle_mcprush_card(&changed).unwrap();
    preservation(&changed, &output);
    let doc = Document::parse(&output).unwrap();
    let rect = doc
        .descendants()
        .find(|n| {
            n.has_tag_name("rect")
                && n.attribute("x") == Some("35")
                && n.attribute("y") == Some("117")
        })
        .unwrap();
    let width: f64 = rect.attribute("width").unwrap().parse().unwrap();
    assert!((width - 55.24).abs() < 0.0001);
}

#[test]
fn rejects_active_external_or_unknown_svg_content() {
    let samples = [
        DARK.replace("<defs>", "<script>alert(1)</script><defs>"),
        DARK.replace("<defs>", "<?instruction unsafe?><defs>"),
        DARK.replace("<defs>", "<!-- hidden --><defs>"),
        DARK.replace(
            "<defs>",
            "<image href=\"https://example.invalid/image\"/><defs>",
        ),
        DARK.replace("href=\"#g0\"", "href=\"https://example.invalid/g0\""),
        DARK.replace("href=\"#g0\"", "href=\"#missing\""),
        DARK.replace(
            "fill=\"#FAFAFA\"",
            "fill=\"url(https://example.invalid/paint)\"",
        ),
        DARK.replace("fill=\"#FAFAFA\"", "fill=\"#FAFAFA\" onload=\"alert(1)\""),
        DARK.replace("xmlns=", "xmlns:foreign=\"https://example.invalid\" xmlns="),
        format!("<!DOCTYPE svg [<!ENTITY x 'unsafe'>]>{DARK}"),
    ];
    for sample in samples {
        assert!(restyle_mcprush_card(&sample).is_err());
    }
}

#[test]
fn fixed_decoration_layout_changes_are_rejected() {
    for source in [DARK, LIGHT] {
        let changes = [
            (
                "frame",
                source.replace(
                    "M6.5 0.5h367a6 6 0 0 1 6 6v187a6 6 0 0 1-6 6h-367a6 6 0 0 1-6-6v-187a6 6 0 0 1 6-6z",
                    "M0 80H380V200H0Z",
                ),
            ),
            ("separator", source.replace("M18 153.5H362", "M18 125H362")),
            (
                "logo tile",
                source.replace("<rect x=\"290.81\"", "<rect x=\"18\""),
            ),
        ];
        let accepted: Vec<_> = changes
            .iter()
            .filter_map(|(name, changed)| {
                assert_ne!(changed, source, "{name} mutation did not apply");
                restyle_mcprush_card(changed).is_ok().then_some(*name)
            })
            .collect();
        assert!(
            accepted.is_empty(),
            "Accepted changed layouts: {accepted:?}"
        );
    }
}

#[test]
fn rejects_identity_layout_and_geometry_drift() {
    for sample in [
        DARK.replace("Aus Accounting MCP", "Another project"),
        DARK.replace("viewBox=\"0 0 380 200\"", "viewBox=\"0 0 3800 2000\""),
        DARK.replace("translate(18 38)", "translate(NaN 38)"),
        DARK.replace("translate(18 38)", "translate(18 39)"),
        DARK.replace("scale(0.018)", "scale(10)"),
        DARK.replace("translate(80.24 130)", "translate(35 130)"),
        DARK.replace("<path id=\"g0\"", "<path id=\"g1\""),
        DARK.replace("d=\"M", "d=\"M999999 999999M"),
    ] {
        assert!(restyle_mcprush_card(&sample).is_err());
    }
    let mut missing = DARK.to_owned();
    let start = missing.find("<desc>").unwrap();
    let end = missing.find("</desc>").unwrap() + 7;
    missing.replace_range(start..end, "");
    assert!(restyle_mcprush_card(&missing).is_err());
    assert!(restyle_mcprush_card(&" ".repeat(32_769)).is_err());
    assert!(restyle_mcprush_card(&format!("<svg>{}</svg>", "<g/>".repeat(300))).is_err());
}
