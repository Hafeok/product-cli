//! Unit tests for the §12.2 content-store manifest profile (canonical YAML).

use super::*;
use crate::pf::model::{ContentRef, DomainGraph, WireframeStep};

const WHOLE: &str = r#"
content_store:
  id: "copy"
  version: "1.0"
  locales_supported: [en, de]
  entries:
    - key: cart.empty.message
      role: empty-message
      values: { en: "Your cart is empty", de: "Ihr Warenkorb ist leer" }
    - key: checkout.title
      role: heading
      values: { en: "Checkout", de: "Kasse" }
"#;

#[test]
fn whole_manifest_validates() {
    let m = parse_content(WHOLE).expect("parses");
    assert!(validate_content(&m).is_empty(), "{:?}", validate_content(&m));
}

#[test]
fn missing_locale_value_fails() {
    let m = parse_content(&WHOLE.replace(", de: \"Kasse\"", "")).expect("parses");
    let f = validate_content(&m);
    assert!(f.iter().any(|s| s.contains("checkout.title") && s.contains("de")), "{f:?}");
}

#[test]
fn empty_error_role_fails() {
    let m = parse_content(&WHOLE.replace("Your cart is empty", "")).expect("parses");
    let f = validate_content(&m);
    assert!(f.iter().any(|s| s.contains("cart.empty.message") && s.contains("empty")), "{f:?}");
}

#[test]
fn coupling_resolves_referenced_keys_per_locale() {
    let m = parse_content(WHOLE).expect("parses");
    let mut g = DomainGraph::default();
    g.wireframe_steps.push(WireframeStep {
        id: "Cart".into(), label: "Cart".into(),
        content_refs: vec![ContentRef { key: "cart.empty.message".into(), role: "empty-message".into() }],
        ..Default::default()
    });
    assert!(couple_content(&m, &g).is_empty(), "all referenced keys resolve");
    g.wireframe_steps[0].content_refs.push(ContentRef { key: "missing.key".into(), role: "body".into() });
    let f = couple_content(&m, &g);
    assert!(f.iter().any(|s| s.contains("missing.key") && s.contains("de")), "{f:?}");
}
