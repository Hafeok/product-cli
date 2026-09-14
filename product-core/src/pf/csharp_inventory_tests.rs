//! Tests for the inventory loader over the committed fixture inventory.

use crate::pf::csharp_inventory::*;

const FIXTURE: &str =
    include_str!("../../../product-cli/tests/fixtures/csharp-inventory/inventory.json");

#[test]
fn fixture_loads_and_conforms_to_the_vendored_schema() {
    let inv = load_inventory(FIXTURE).expect("loads");
    assert_eq!(inv.inventory_version, "3");
    assert_eq!(inv.registrations.len(), 11);
    assert!(inv.external_types.iter().any(|e| e.id == "T:System.IServiceProvider"));
    assert_eq!(inv.projects.len(), 3);
    assert!(inv.types.len() >= 20 && inv.members.len() >= 50);
    let value: serde_json::Value = serde_json::from_str(FIXTURE).expect("json");
    let findings = schema_findings(&value).expect("schema applies");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn unknown_version_is_refused_before_parsing() {
    let text = r#"{"inventory_version":"2","this_is_not":"parsed"}"#;
    let err = load_inventory(text).expect_err("refused");
    assert!(err.to_string().contains("'2' is not known"), "{err}");
    let err = load_inventory(r#"{"types":[]}"#).expect_err("refused");
    assert!(err.to_string().contains("no `inventory_version`"), "{err}");
}

#[test]
fn schema_rejects_a_judgement_field() {
    let mut value: serde_json::Value = serde_json::from_str(FIXTURE).expect("json");
    value["types"][0]["region"] = serde_json::json!("declared");
    let findings = schema_findings(&value).expect("schema applies");
    assert!(findings.iter().any(|f| f.contains("region")), "{findings:?}");
}

#[test]
fn attributes_arrive_with_typed_arguments() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let slices = inv.types_with_attribute("T:Product.Binding.SliceAttribute");
    let handler = slices.iter().find(|(t, _)| t.name == "PlaceOrderHandler").expect("declared");
    assert_eq!(handler.1.arg(0), Some("PlaceOrder"));
    assert_eq!(handler.1.arg(1), Some("handler"));
    assert_eq!(handler.1.named_str("Profile"), Some("rest-api-v1"));
    let entry: Vec<_> = inv.members.iter().filter(|m| m.is_entry_point).collect();
    assert_eq!(entry.len(), 1);
    assert_eq!(entry[0].name, "Main");
}

#[test]
fn index_resolves_implementors() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let ix = inv.index();
    let impls = ix.implementors.get("T:Shop.Api.Persistence.IOrderRepository").expect("implemented");
    assert_eq!(impls, &vec!["T:Shop.Api.Persistence.OrderRepository"]);
    assert!(ix.members_of.get("T:Shop.Api.Orders.PlaceOrderHandler").map(Vec::len).unwrap_or(0) >= 2);
}
