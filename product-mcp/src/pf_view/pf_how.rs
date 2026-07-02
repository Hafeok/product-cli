//! Live projection of the §3.3 behavioural, §7 delivery, §4 How fields of
//! `window.PF` — deciders, delivery (features/releases/targets/versions), plus
//! the How's blueprint / DeployableUnits / why-cascade.

use std::path::Path;

use product_core::pf::decider::Decider;
use product_core::pf::deliverable::Deliverable;
use product_core::pf::feature::Feature;
use product_core::pf::how::HowContract;
use product_core::pf::model::DomainGraph;
use product_core::pf::release::Release;
use product_core::pf::target::Target;
use serde_json::{json, Value};

use super::load_deployable_units;

/// Read + parse every `*.yaml` in `<repo>/.product/<dir>` through `parse`.
fn load_all<T>(repo_root: &Path, dir: &str, parse: impl Fn(&str) -> product_core::error::Result<T>) -> Vec<T> {
    let d = repo_root.join(".product").join(dir);
    let mut paths: Vec<_> = match std::fs::read_dir(&d) {
        Ok(it) => it.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml")).collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    paths.iter().filter_map(|p| std::fs::read_to_string(p).ok()).filter_map(|t| parse(&t).ok()).collect()
}

fn how_contract(repo_root: &Path) -> Option<HowContract> {
    HowContract::load_opt(&repo_root.join(".product").join("how-contract.yaml")).ok().flatten()
}

/// Blueprint directory names under `.product/blueprints/` (or legacy `archetypes/`).
fn blueprint_names(repo_root: &Path) -> Vec<String> {
    let base = repo_root.join(".product");
    let bp = base.join("blueprints");
    let dir = if bp.is_dir() { bp } else { base.join("archetypes") };
    product_core::pf::deployable_unit::blueprint_names(&dir)
}

// --- §3.3 deciders ---------------------------------------------------------

pub fn project_deciders(repo_root: &Path) -> Value {
    let deciders = load_all(repo_root, "deciders", Decider::from_yaml);
    let v: Vec<Value> = deciders
        .iter()
        .map(|d| {
            let handles: Vec<Value> = d
                .handles
                .iter()
                .map(|c| json!({ "cmd": c, "emits": d.emits, "rejects": d.rejects }))
                .collect();
            let state_read: Vec<Value> = d.reads.iter().map(|f| json!({ "field": f, "readBy": "" })).collect();
            let rejections: Vec<Value> =
                d.rejects.iter().map(|r| json!({ "id": r, "rule": "", "reachable": true })).collect();
            json!({
                "id": d.id,
                "aggregate": d.decides_for,
                "conformance": "realised",
                "handles": handles,
                "evolves": d.evolves_from,
                "stateRead": state_read,
                "rejections": rejections,
                "coverage": { "commands": format!("{}/{}", d.handles.len(), d.handles.len()), "foreign": 0, "outputs": "contained" },
            })
        })
        .collect();
    Value::Array(v)
}

// --- §7 delivery: features / releases / targets / versions -----------------

pub fn project_delivery(g: &DomainGraph, repo_root: &Path) -> Value {
    let features = load_all(repo_root, "features", Feature::from_yaml);
    let deliverables = load_all(repo_root, "deliverables", Deliverable::from_yaml);
    let releases = load_all(repo_root, "releases", Release::from_yaml);
    let targets = load_all(repo_root, "targets", Target::from_yaml);

    let feat_json: Vec<Value> = features.iter().map(|f| feature_json(f, &deliverables)).collect();
    let rel_json: Vec<Value> = releases
        .iter()
        .map(|r| json!({ "id": r.id, "name": r.id, "features": r.features, "closed": true, "note": "" }))
        .collect();
    let tgt_json: Vec<Value> = targets
        .iter()
        .map(|t| json!({ "id": t.id, "name": t.id, "whatVersion": t.version.clone().unwrap_or_default(), "partition": t.in_target, "note": "" }))
        .collect();

    json!({
        "features": feat_json,
        "releases": rel_json,
        "targets": tgt_json,
        "versions": project_versions(g, repo_root),
    })
}

/// A feature's `window.PF` shape, with acceptance pulled from the deliverable
/// that wraps it (§7.1).
fn feature_json(f: &Feature, deliverables: &[Deliverable]) -> Value {
    let acceptance: Vec<String> = deliverables
        .iter()
        .find(|d| d.feature == f.id)
        .map(|d| d.acceptance.iter().map(|a| a.statement.clone()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    json!({
        "id": f.id, "name": f.id, "sub": f.id,
        "flows": f.anchors, "footprint": f.anchors,
        "conformance": "realised", "valueAction": "", "acceptance": acceptance,
    })
}

/// §7.3 — the What/How semantic versions the graph declares.
fn project_versions(g: &DomainGraph, repo_root: &Path) -> Value {
    let what_v = g.products.first().and_then(|p| p.version.clone());
    let how = how_contract(repo_root);
    let what: Vec<Value> = what_v
        .iter()
        .map(|v| json!({ "v": v, "name": "current", "bump": "minor", "current": true, "status": "realised", "diff": "", "adds": [] }))
        .collect();
    let howv: Vec<Value> = how
        .as_ref()
        .and_then(|h| h.version.clone())
        .map(|v| {
            let realises = how.as_ref().and_then(|h| h.realises_version.clone()).unwrap_or_default();
            vec![json!({ "v": v, "name": "current", "bump": "minor", "realises": realises, "current": true, "diff": "" })]
        })
        .unwrap_or_default();
    json!({ "what": what, "how": howv })
}

// --- §4 the How: blueprint, DeployableUnits, why-cascade -------------------

pub fn project_how(g: &DomainGraph, repo_root: &Path) -> Value {
    let how = how_contract(repo_root);
    let (decisions, principles, patterns) = why_cascade(how.as_ref());
    json!({
        "blueprint": project_blueprint(g, repo_root, how.as_ref()),
        "deployableUnits": project_deployable_units(repo_root),
        "decisions": decisions,
        "principles": principles,
        "patterns": patterns,
    })
}

/// The blueprint node: its name, the parts it packages, and the systems it
/// realises (best-effort — the graph carries no explicit blueprint→system edge).
fn project_blueprint(g: &DomainGraph, repo_root: &Path, how: Option<&HowContract>) -> Value {
    let Some(name) = blueprint_names(repo_root).into_iter().next() else { return Value::Null };
    let mut packages = vec![Value::from("application contract")];
    if let Some(h) = how {
        if h.layout_model.is_some() { packages.push(json!("repository layout model")); }
        packages.push(json!(format!("{} principles", h.principles.len())));
        packages.push(json!(format!("{} patterns", h.patterns.len())));
    }
    let instances: Vec<Value> =
        g.systems.iter().map(|s| json!({ "sys": s.id, "conformance": "realised" })).collect();
    json!({
        "id": name, "name": name, "packages": packages, "instances": instances,
        "note": "the reusable How captured once where a system shape recurs (§4)",
    })
}

/// The DeployableUnits (§4.2) declared under `.product/deployable-units/`.
fn project_deployable_units(repo_root: &Path) -> Value {
    let units = load_deployable_units(repo_root);
    let v: Vec<Value> = units
        .iter()
        .map(|u| {
            let identity: Vec<String> =
                [&u.identity.domain_name, &u.identity.bundle_id, &u.identity.runtime].into_iter().flatten().cloned().collect();
            json!({
                "id": u.id,
                "system": u.deploys_system.first().cloned().unwrap_or_default(),
                "env": u.environment.clone().unwrap_or_default(),
                "identity": identity.join(" · "),
                "frozen": true,
            })
        })
        .collect();
    Value::Array(v)
}

/// The §4.1 why-cascade — (decisions, principles, patterns) — from the How.
fn why_cascade(how: Option<&HowContract>) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    let Some(h) = how else { return (vec![], vec![], vec![]) };
    let d = h.top_decisions.iter().map(|d| json!({
        "id": d.id, "title": d.decision, "why": d.rationale,
        "applies": d.applies_when.clone().unwrap_or_default(),
        "not": d.does_not_apply_when.clone().unwrap_or_default(),
        "licenses": d.licenses,
    })).collect();
    let p = h.principles.iter().map(|p| json!({
        "id": p.id, "text": p.statement,
        "enforcedBy": p.enforced_by.first().cloned().unwrap_or_default(),
        "appliedBy": p.realized_by.first().cloned().unwrap_or_default(),
    })).collect();
    let pat = h.patterns.iter().map(|p| json!({
        "id": p.id, "text": p.shape,
        "implements": p.realizes.first().cloned().unwrap_or_default(),
        "files": [], "rules": [], "units": p.applied_by,
    })).collect();
    (d, p, pat)
}
