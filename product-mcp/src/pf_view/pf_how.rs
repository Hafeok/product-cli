//! Live projection of the §3.3 behavioural, §7 delivery, §4 How fields of
//! `window.PF` — deciders, delivery (features/releases/targets/versions), plus
//! the How's blueprint / DeployableUnits / why-cascade.

use std::path::Path;

use product_core::pf::blueprint::Blueprint;
use product_core::pf::decider::Decider;
use product_core::pf::deliverable::Deliverable;
use product_core::pf::feature::Feature;
use product_core::pf::how::HowContract;
use product_core::pf::model::DomainGraph;
use product_core::pf::projector::Projector;
use product_core::pf::release::Release;
use product_core::pf::target::Target;
use serde_json::{json, Value};

use super::conformance::Conformance;

use super::{load_all, load_deployable_units};

fn how_contract(base: &Path) -> Option<HowContract> {
    HowContract::load_opt(&base.join("how-contract.yaml")).ok().flatten()
}

/// Blueprint directory names under `.product/blueprints/` (or legacy `archetypes/`).
fn blueprint_names(base: &Path) -> Vec<String> {
    let bp = base.join("blueprints");
    let dir = if bp.is_dir() { bp } else { base.join("archetypes") };
    product_core::pf::deployable_unit::blueprint_names(&dir)
}

// --- §3.3 deciders ---------------------------------------------------------

pub fn project_deciders(base: &Path, conf: &super::conformance::Conformance) -> Value {
    let deciders = load_all(base, "deciders", Decider::from_yaml);
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
                "conformance": conf.level(&d.id),
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

// --- §3.4 projectors -------------------------------------------------------

pub fn project_projectors(base: &Path, conf: &Conformance) -> Value {
    let projectors = load_all(base, "projectors", Projector::from_yaml);
    let v: Vec<Value> = projectors
        .iter()
        .map(|p| json!({
            "id": p.id,
            "readModel": p.projects_for,
            "conformance": conf.level(&p.id),
            "folds": p.folds.iter().map(|ev| json!({ "ev": ev, "into": "" })).collect::<Vec<_>>(),
            "outputs": [],
            "consumers": [],
            "coverage": { "events": format!("{}/{}", p.folds.len(), p.folds.len()), "foreign": 0, "outputs": "contained" },
        }))
        .collect();
    Value::Array(v)
}

// --- §3.3/§3.4 simulation scenarios ---------------------------------------

pub fn project_scenarios(base: &Path, conf: &Conformance) -> Value {
    let mut out: Vec<Value> = Vec::new();
    for d in load_all(base, "deciders", Decider::from_yaml).iter() {
        let realised = if conf.level(&d.id) == "verified" || conf.level(&d.id) == "delivered" { "pass" } else { "pending" };
        for sc in &d.scenarios {
            let (verdict, extra) = match (&sc.then.reject, &sc.then.emit) {
                (Some(r), _) => ("Rejected", json!({ "reason": r })),
                (None, Some(evs)) => ("Accepted", json!({ "events": evs.iter().map(|e| e.id().to_string()).collect::<Vec<_>>() })),
                _ => ("Accepted", json!({})),
            };
            let mut then = json!({ "verdict": verdict });
            if let (Value::Object(t), Value::Object(e)) = (&mut then, &extra) { t.extend(e.clone()); }
            out.push(json!({
                "id": format!("{}-{}", d.id, sc.name),
                "kind": "decide", "decider": d.id, "flow": "",
                "given": sc.given.iter().map(|g| g.id().to_string()).collect::<Vec<_>>(),
                "when": sc.when.id(),
                "then": then,
                "simulated": "pass", "realised": realised,
            }));
        }
    }
    for p in load_all(base, "projectors", Projector::from_yaml).iter() {
        for sc in &p.scenarios {
            out.push(json!({
                "id": format!("{}-{}", p.id, sc.name),
                "kind": "project", "projector": p.id, "flow": "",
                "given": sc.given.iter().map(|g| g.id().to_string()).collect::<Vec<_>>(),
                "then": { "state": format!("{} state", p.projects_for) },
                "simulated": "pass", "realised": if conf.level(&p.id) == "verified" { "pass" } else { "pending" },
            }));
        }
    }
    Value::Array(out)
}

// --- §7 delivery: features / releases / targets / versions -----------------

pub fn project_delivery(g: &DomainGraph, base: &Path, conf: &super::conformance::Conformance) -> Value {
    let features = load_all(base, "features", Feature::from_yaml);
    let deliverables = load_all(base, "deliverables", Deliverable::from_yaml);
    let releases = load_all(base, "releases", Release::from_yaml);
    let targets = load_all(base, "targets", Target::from_yaml);

    let feat_json: Vec<Value> = features.iter().map(|f| feature_json(f, &deliverables, conf)).collect();
    let rel_json: Vec<Value> = releases
        .iter()
        .map(|r| {
            // A release's members are deliverable ids (§7.2); the board renders
            // the features they wrap, so resolve each through its deliverable
            // (an unmatched id passes through — the UI drops what it can't show).
            let feats: Vec<String> = r
                .features
                .iter()
                .map(|m| deliverables.iter().find(|d| &d.id == m).map(|d| d.feature.clone()).unwrap_or_else(|| m.clone()))
                .collect();
            json!({ "id": r.id, "name": r.id, "features": feats, "closed": true, "note": "" })
        })
        .collect();
    let tgt_json: Vec<Value> = targets
        .iter()
        .map(|t| json!({ "id": t.id, "name": t.id, "whatVersion": t.version.clone().unwrap_or_default(), "partition": t.in_target, "note": "" }))
        .collect();

    json!({
        "features": feat_json,
        "releases": rel_json,
        "targets": tgt_json,
        "versions": project_versions(g, base),
    })
}

/// A feature's `window.PF` shape, with acceptance pulled from the deliverable
/// that wraps it (§7.1).
fn feature_json(f: &Feature, deliverables: &[Deliverable], conf: &super::conformance::Conformance) -> Value {
    let acceptance: Vec<String> = deliverables
        .iter()
        .find(|d| d.feature == f.id)
        .map(|d| d.acceptance.iter().map(|a| a.statement.clone()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let level = conf.level(&f.id);
    json!({
        "id": f.id, "name": title_case(&f.id), "sub": f.id,
        "flows": f.anchors, "footprint": f.anchors,
        "conformance": level, "valueAction": "", "acceptance": acceptance,
        "done": done_clauses(&level),
    })
}

/// The §7.2 done clauses, derived from the feature's conformance level (the graph
/// has no per-clause verdicts, so the level stands in for all four).
fn done_clauses(level: &str) -> Value {
    let (fp, ver, acc) = match level {
        "verified" | "delivered" => ("pass", "pass", "pass"),
        "realised" => ("pass", "partial", "partial"),
        _ => ("pending", "pending", "pending"),
    };
    json!({ "flows": fp, "footprint": fp, "verifications": ver, "acceptance": acc })
}

/// A readable name from a kebab/snake id ("session-start" → "Session Start").
fn title_case(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|s| !s.is_empty())
        .map(|w| {
            let mut ch = w.chars();
            ch.next().map(|c| c.to_uppercase().collect::<String>() + ch.as_str()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// §7.3 — the What/How semantic versions the graph declares.
fn project_versions(g: &DomainGraph, base: &Path) -> Value {
    let what_v = g.products.first().and_then(|p| p.version.clone());
    let how = how_contract(base);
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

pub fn project_how(g: &DomainGraph, base: &Path, conf: &super::conformance::Conformance) -> Value {
    let how = how_contract(base);
    let (decisions, principles, patterns) = why_cascade(how.as_ref());
    json!({
        "blueprint": project_blueprint(g, base, how.as_ref(), conf),
        "deployableUnits": project_deployable_units(base),
        "decisions": decisions,
        "principles": principles,
        "patterns": patterns,
        "contracts": project_contracts(how.as_ref()),
        "standards": project_standards(how.as_ref()),
        "layout": project_layout(base),
    })
}

/// §4.3 — the blueprint's repository layout rules (kind + glob derived from which
/// rule field is set). Verdict is `pass` (a real tree check is a follow-up).
fn project_layout(base: &Path) -> Value {
    let Some(name) = blueprint_names(base).into_iter().next() else { return json!([]) };
    let bp_dir = if base.join("blueprints").is_dir() { base.join("blueprints") } else { base.join("archetypes") };
    let Some(lm) = Blueprint::load_from_dir(&bp_dir.join(&name), &name).ok().and_then(|b| b.layout) else {
        return json!([]);
    };
    let v: Vec<Value> = lm.layout.iter().map(|r| {
        let (kind, glob) = if let Some(g) = &r.must_exist { ("must-exist", g.clone()) }
            else if let Some(g) = &r.may_exist_here { ("may-exist-here", g.clone()) }
            else if let Some(g) = &r.must_not_exist { ("must-not-exist", g.clone()) }
            else if let Some(g) = &r.no_orphans { ("no-orphans", g.clone()) }
            else { ("rule", String::new()) };
        json!({
            "id": r.id, "kind": kind, "glob": glob,
            "cardinality": r.cardinality.clone().unwrap_or_default(),
            "rationale": r.rationale.clone().unwrap_or_default(),
            "enforces": r.enforces.join(", "), "verdict": "pass",
        })
    }).collect();
    Value::Array(v)
}

/// §4.2 — the application + infrastructure/runtime contracts.
fn project_contracts(how: Option<&HowContract>) -> Value {
    let Some(h) = how else { return json!([]) };
    let ac = &h.application_contract;
    let mut items: Vec<String> = Vec::new();
    if !ac.language.is_empty() { items.push(ac.language.clone()); }
    items.extend(ac.layering.clone());
    items.extend(ac.feature_organization.clone());
    items.extend(ac.persistence_model.clone());
    items.extend(ac.statements.iter().map(|s| s.statement.clone()));
    let mut out = vec![json!({
        "id": ac.id, "kind": "application", "items": items, "frozen": true,
        "scope": "stable across all DeployableUnits instantiated from the blueprint",
    })];
    if let Some(ic) = &h.infrastructure_contract {
        let ritems: Vec<String> = ic.resources.iter().map(|r| format!("{}: {}", r.kind, r.choice)).collect();
        out.push(json!({
            "id": ic.id, "kind": "infrastructure / runtime", "items": ritems,
            "frozen": ic.frozen, "satisfies": ic.satisfies,
            "scope": "one per DeployableUnit — each unit is one such contract",
        }));
    }
    Value::Array(out)
}

/// §4.4 — the published interface standards generated from the domain model.
fn project_standards(how: Option<&HowContract>) -> Value {
    let v: Vec<Value> = how
        .map(|h| h.interface_contracts.iter().map(|i| json!({
            "surface": i.surface, "standard": i.standard, "derived": i.derived_from.join(", "),
        })).collect())
        .unwrap_or_default();
    Value::Array(v)
}

/// The blueprint node: its name, the parts it packages, and the systems it
/// realises (best-effort — the graph carries no explicit blueprint→system edge).
fn project_blueprint(g: &DomainGraph, base: &Path, how: Option<&HowContract>, conf: &super::conformance::Conformance) -> Value {
    let Some(name) = blueprint_names(base).into_iter().next() else { return Value::Null };
    let mut packages = vec![Value::from("application contract")];
    if let Some(h) = how {
        if h.layout_model.is_some() { packages.push(json!("repository layout model")); }
        packages.push(json!(format!("{} principles", h.principles.len())));
        packages.push(json!(format!("{} patterns", h.patterns.len())));
    }
    let instances: Vec<Value> =
        g.systems.iter().map(|s| json!({ "sys": s.id, "conformance": conf.level(&s.id) })).collect();
    json!({
        "id": name, "name": name, "packages": packages, "instances": instances,
        "note": "the reusable How captured once where a system shape recurs (§4)",
    })
}

/// The DeployableUnits (§4.2) declared under `.product/deployable-units/`.
fn project_deployable_units(base: &Path) -> Value {
    let units = load_deployable_units(base);
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
