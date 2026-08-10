//! Facts-extraction coverage for the pair adapter's source slicing.

use super::*;
use crate::adapter::AdapterFlags;

fn css_facts_of(text: &str) -> Vec<SymbolFacts> {
    facts(text, &[], &AdapterFlags::default())
}

fn named<'a>(list: &'a [SymbolFacts], name: &str) -> &'a SymbolFacts {
    list.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("no fact `{name}`: {list:?}"))
}

#[test]
fn tokens_are_lifted_with_their_value_type_as_signature() {
    let list = css_facts_of(
        ":root {\n  --color-ok: #1a7f37;\n  --gap: 1.5rem;\n  --weight: bold;\n  --alias: var(--gap);\n}\n",
    );
    assert_eq!(named(&list, "--color-ok").kind, "token");
    assert_eq!(named(&list, "--color-ok").signature, "color");
    assert_eq!(named(&list, "--gap").signature, "dimension");
    assert_eq!(named(&list, "--weight").signature, "keyword");
    assert_eq!(named(&list, "--alias").signature, "reference");
}

#[test]
fn token_usage_in_var_is_not_a_declaration() {
    let list = css_facts_of(".x { color: var(--brand); }\n");
    assert!(!list.iter().any(|f| f.kind == "token"), "{list:?}");
}

#[test]
fn classes_are_deduped_across_rules_and_bodies_are_not_selectors() {
    let list = css_facts_of(
        ".btn { color: red; }\n.btn:hover { color: blue; }\n.card .btn { margin: 0; }\np { content: \".fake\"; }\n",
    );
    let classes: Vec<&str> =
        list.iter().filter(|f| f.kind == "class").map(|f| f.name.as_str()).collect();
    assert_eq!(classes, vec!["btn", "card"], "{list:?}");
}

#[test]
fn layer_order_is_one_symbol_and_layered_rules_grade_visibility() {
    let list = css_facts_of(
        "@layer base, components;\n@layer components {\n  .btn { color: red; }\n}\n.loud { color: blue; }\n",
    );
    assert_eq!(named(&list, "@layer").kind, "layer-order");
    assert_eq!(named(&list, "@layer").signature, "base, components");
    assert_eq!(named(&list, "btn").visibility, "layer:components");
    assert_eq!(named(&list, "loud").visibility, "unlayered");
}

#[test]
fn css_comments_are_stripped_before_slicing() {
    let list = css_facts_of("/* .ghost { } --spooky: 1; */\n.real { color: red; }\n");
    assert!(!list.iter().any(|f| f.name == "ghost" || f.name == "--spooky"), "{list:?}");
    assert_eq!(named(&list, "real").kind, "class");
}

#[test]
fn html_lifts_stylesheet_links_only() {
    let list = facts(
        "<!doctype html>\n<html><head>\n<link rel=\"stylesheet\" href=\"style.css\">\n<link rel=\"icon\" href=\"i.png\">\n</head>\n<body class=\"page\"><div class=\"card\">x</div></body></html>\n",
        &[],
        &AdapterFlags::default(),
    );
    assert_eq!(list.len(), 1, "{list:?}");
    assert_eq!(list[0].kind, "stylesheet-link");
    assert_eq!(list[0].name, "style.css");
}

#[test]
fn script_contents_are_opaque_bytes() {
    let list = facts(
        "<html><head>\n<script>\nconst s = '<link rel=\"stylesheet\" href=\"evil.css\">';\n</script>\n<link rel=\"stylesheet\" href=\"real.css\">\n</head></html>\n",
        &[],
        &AdapterFlags::default(),
    );
    let links: Vec<&str> =
        list.iter().filter(|f| f.kind == "stylesheet-link").map(|f| f.name.as_str()).collect();
    assert_eq!(links, vec!["real.css"], "{list:?}");
}

#[test]
fn tag_parsing_reads_class_and_id_attributes() {
    let parsed = tags("<div class=\"a b\" id=\"main\" data-x>\n<span class='c'></span>");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].1.attr("class").as_deref(), Some("a b"));
    assert_eq!(parsed[0].1.attr("id").as_deref(), Some("main"));
    assert_eq!(parsed[1].1.attr("class").as_deref(), Some("c"));
}
