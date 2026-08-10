//! One triggering plus one non-triggering case per M7 policy row.

use crate::adapter::{htmlcss, AdapterFlags};
use crate::classify::classify_edit;
use crate::surface::SurfaceEvent;

fn events(before: &str, after: &str) -> Vec<SurfaceEvent> {
    classify_edit(&htmlcss::ADAPTER, &AdapterFlags::default(), before, &[], after, &[])
}

fn surface_rules(evts: &[SurfaceEvent]) -> Vec<&'static str> {
    evts.iter().filter(|e| e.surface).filter_map(|e| e.rule).collect()
}

const SHEET: &str = ":root {\n  --color-ok: #1a7f37;\n  --gap: 1.5rem;\n}\n.btn { color: var(--color-ok); }\n.card { margin: var(--gap); }\n";

#[test]
fn token_added_or_removed_is_surface_but_value_change_is_no_event() {
    let grown = format!("{}\n:root {{ --radius: 3px; }}\n", SHEET);
    assert_eq!(surface_rules(&events(SHEET, &grown)), vec!["web-token-membership"]);
    let shrunk = SHEET.replace("  --gap: 1.5rem;\n", "");
    assert_eq!(surface_rules(&events(SHEET, &shrunk)), vec!["web-token-membership"]);
    // The non-trigger is the tokens' whole point: a same-type value change
    // produces no event at all.
    let recolored = SHEET.replace("#1a7f37", "#2ea043");
    assert!(events(SHEET, &recolored).is_empty());
}

#[test]
fn token_retype_is_surface_but_same_type_is_not() {
    let retyped = SHEET.replace("--gap: 1.5rem;", "--gap: #fff;");
    assert_eq!(surface_rules(&events(SHEET, &retyped)), vec!["web-token-retype"]);
    let same_type = SHEET.replace("--gap: 1.5rem;", "--gap: 2rem;");
    assert!(events(SHEET, &same_type).is_empty());
}

#[test]
fn class_membership_is_surface_but_a_styling_tweak_is_not() {
    let grown = format!("{}.badge {{ color: red; }}\n", SHEET);
    assert_eq!(surface_rules(&events(SHEET, &grown)), vec!["web-class-membership"]);
    let dropped = SHEET.replace(".card { margin: var(--gap); }\n", "");
    assert_eq!(surface_rules(&events(SHEET, &dropped)), vec!["web-class-membership"]);
    // Styling inside an existing class is the vocabulary's body, not its
    // membership.
    let tweaked = SHEET.replace(".btn { color: var(--color-ok); }", ".btn { color: var(--color-ok); padding: 2px; }");
    assert!(events(SHEET, &tweaked).is_empty());
}

#[test]
fn a_class_defined_in_a_second_rule_is_no_membership_event() {
    let restyled = format!("{}.btn:focus {{ outline: none; }}\n", SHEET);
    assert!(events(SHEET, &restyled).is_empty());
}

#[test]
fn layer_order_change_is_surface_but_a_rule_moving_between_layers_is_not_an_order_event() {
    let layered = "@layer base, components;\n@layer base { .a { color: red; } }\n";
    let reordered = "@layer components, base;\n@layer base { .a { color: red; } }\n";
    assert_eq!(surface_rules(&events(layered, reordered)), vec!["web-layer-order"]);
    let introduced = events(SHEET, &format!("@layer util;\n{SHEET}"));
    assert_eq!(surface_rules(&introduced), vec!["web-layer-order"]);
    // Moving one rule into an existing layer re-grades that rule's
    // visibility, not the declared order.
    let moved = "@layer base, components;\n@layer components { .a { color: red; } }\n";
    assert!(surface_rules(&events(layered, moved)).is_empty());
}

#[test]
fn stylesheet_link_rewiring_is_surface_but_markup_is_not() {
    let page = "<html><head>\n<link rel=\"stylesheet\" href=\"style.css\">\n</head><body><div class=\"card\">x</div></body></html>\n";
    let rewired = page.replace("style.css", "other.css");
    let rules = surface_rules(&events(page, &rewired));
    assert_eq!(rules, vec!["web-stylesheet-link", "web-stylesheet-link"], "add + remove");
    // Markup edits — structure, class consumption — are not edit-time
    // surface; the pair contract check owns them.
    let restructured = page.replace("<div class=\"card\">x</div>", "<div class=\"card new\"><p>x</p></div>");
    assert!(events(page, &restructured).is_empty());
}
