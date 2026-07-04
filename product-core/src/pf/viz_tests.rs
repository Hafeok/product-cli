//! Unit tests for [`super`] — the What-graph view projection.

use super::*;
use crate::pf::model::*;
fn sample() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "ctx".into(), label: "Ctx".into(), glossary: vec!["Order".into()], ..Default::default() });
    g.entities.push(Entity {
        id: "Order".into(), label: "Order".into(), context: "ctx".into(), definition: "d".into(),
        is_aggregate_root: true, attributes: vec![Attribute { name: "total".into(), ty: Some("Money".into()) }], ..Default::default()
    });
    g.commands.push(Command { fields: vec![], id: "Place".into(), label: "Place".into(), context: "ctx".into(), targets: "Order".into(), emits: vec!["Placed".into()] });
    g.events.push(Event { fields: vec![], id: "Placed".into(), label: "Placed".into(), context: "ctx".into(), changes: "Order".into() });
    g.read_models.push(ReadModel { id: "Cart".into(), label: "Cart".into(), projects: vec!["Order".into()], ..Default::default() });
    g.systems.push(System {
        id: "sys".into(), label: "Shop".into(), kind: "application".into(), purpose: "shop".into(),
        target_classes: vec!["gui".into()], target_platforms: vec!["web".into()], references_domain: vec!["ctx".into()], ..Default::default()
    });
    g.triggers.push(Trigger { id: "t".into(), label: "User places".into(), source: "user".into(), issues: "Place".into(), ..Default::default() });
    g.products.push(Product { id: "prod".into(), label: "Acme".into(), purpose: "sell".into(), owns_domain: vec!["ctx".into()], owns_system: vec!["sys".into()], version: Some("1.0".into()) });
    g.relations.push(Relation { id: "r".into(), label: Some("has".into()), from: "Order".into(), to: "Order".into(), cardinality: "1 - *".into(), rationale: "x".into() });
    g
}

#[test]
fn surfaces_triggers_and_systems() {
    let v = to_view_graph(&sample());
    assert!(v.nodes.iter().any(|n| n.id == "t" && n.kind == "trigger" && n.model == EVENT), "trigger node in event lane");
    assert!(v.nodes.iter().any(|n| n.id == "sys" && n.kind == "system" && n.model == DOMAIN), "system node in domain lane");
    assert!(v.edges.iter().any(|e| e.from == "t" && e.to == "Place" && e.kind == "issues"), "trigger issues command");
}

#[test]
fn surfaces_product_and_ownership() {
    let v = to_view_graph(&sample());
    let p = v.nodes.iter().find(|n| n.id == "prod").expect("product node");
    assert_eq!(p.kind, "product");
    assert_eq!(p.purpose, "sell");
    assert!(p.tags.iter().any(|t| t.contains("1.0")), "version tag");
    assert!(v.edges.iter().any(|e| e.from == "prod" && e.to == "ctx" && e.kind == "owns-domain"));
    assert!(v.edges.iter().any(|e| e.from == "prod" && e.to == "sys" && e.kind == "owns-system"));
    assert!(v.edges.iter().any(|e| e.from == "sys" && e.to == "ctx" && e.kind == "references"));
}

#[test]
fn enriches_entity_and_system_detail() {
    let v = to_view_graph(&sample());
    let order = v.nodes.iter().find(|n| n.id == "Order").expect("entity");
    assert!(order.aggregate, "aggregate root flagged");
    assert!(order.fields.iter().any(|f| f == "total: Money"), "attribute line");
    let sys = v.nodes.iter().find(|n| n.id == "sys").expect("system");
    assert_eq!(sys.purpose, "shop");
    assert_eq!(sys.context, "application", "system kind on context field");
    assert!(sys.tags.contains(&"gui".to_string()) && sys.tags.contains(&"web".to_string()), "class + platform tags");
    assert!(v.contexts.iter().any(|c| c.id == "ctx" && c.glossary == vec!["Order".to_string()]), "context glossary");
    assert!(v.edges.iter().any(|e| e.kind == "relation" && e.card == "1 - *" && e.label == "has"), "relation cardinality");
}

#[test]
fn tags_each_node_with_a_lane() {
    let v = to_view_graph(&sample());
    assert!(!v.nodes.is_empty());
    for n in &v.nodes {
        assert!(n.model == DOMAIN || n.model == EVENT, "{} has no lane", n.id);
    }
    let lane = |id: &str| v.nodes.iter().find(|n| n.id == id).map(|n| n.model.as_str());
    assert_eq!(lane("Order"), Some(DOMAIN));
    assert_eq!(lane("Place"), Some(EVENT));
    assert_eq!(lane("Placed"), Some(EVENT));
    assert_eq!(lane("Cart"), Some(EVENT));
}

#[test]
fn bridges_run_event_to_domain_only() {
    let v = to_view_graph(&sample());
    let bridge = |from: &str, to: &str| v.edges.iter().find(|e| e.from == from && e.to == to).map(|e| e.bridge);
    // command -> entity, event -> entity, read-model -> entity all bridge.
    assert_eq!(bridge("Place", "Order"), Some(true));
    assert_eq!(bridge("Placed", "Order"), Some(true));
    assert_eq!(bridge("Cart", "Order"), Some(true));
    // command -> event stays inside the event lane.
    assert_eq!(bridge("Place", "Placed"), Some(false));
    // every bridge runs event -> domain.
    for e in &v.edges {
        if e.bridge {
            let lane = |id: &str| v.nodes.iter().find(|n| n.id == id).map(|n| n.model.as_str());
            assert_eq!(lane(&e.from), Some(EVENT));
            assert_eq!(lane(&e.to), Some(DOMAIN));
        }
    }
}

#[test]
fn how_lane_projects_blueprints_and_deployable_units() {
    use crate::pf::deployable_unit::{DeployableUnit, DeploymentIdentity};
    let units = vec![DeployableUnit {
        id: "shop-ios".into(),
        built_from: "rn-app".into(),
        deploys_system: vec!["sys".into()],
        environment: Some("production".into()),
        identity: DeploymentIdentity { bundle_id: Some("com.acme.shop".into()), ..Default::default() },
    }];
    let v = to_view_graph_with_how(&sample(), &["rn-app".to_string()], &units);
    // Blueprint + deployable-unit nodes land in the how lane.
    assert!(v.nodes.iter().any(|n| n.id == "bp:rn-app" && n.kind == "blueprint" && n.model == HOW));
    assert!(v.nodes.iter().any(|n| n.id == "shop-ios" && n.kind == "deployable-unit" && n.model == HOW));
    // built-from (unit→blueprint) and deploys (unit→system §3.2.5) edges.
    assert!(v.edges.iter().any(|e| e.from == "shop-ios" && e.to == "bp:rn-app" && e.kind == "built-from"));
    assert!(v.edges.iter().any(|e| e.from == "shop-ios" && e.to == "sys" && e.kind == "deploys"));
    // The plain What projection carries no how-lane nodes (additive, opt-in).
    assert!(to_view_graph(&sample()).nodes.iter().all(|n| n.model != HOW));
}
