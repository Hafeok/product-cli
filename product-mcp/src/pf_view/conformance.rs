//! Per-node conformance level (described → realised → verified → delivered) for
//! the live projection, so the UI's status dots reflect reality instead of a
//! hardcoded value. Levels are computed from real signals where they exist —
//! `feature_done` (§7.2), a Decider's recorded `.conform.json` verdict (§6),
//! release membership — and fall back to structural reachability otherwise.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use product_core::pf::decider::Decider;
use product_core::pf::deliverable::Deliverable;
use product_core::pf::done::feature_done;
use product_core::pf::feature::Feature;
use product_core::pf::model::DomainGraph;
use product_core::pf::projector::Projector;
use product_core::pf::release::Release;
use serde_json::Value;

use super::load_all;

const RANK: [&str; 4] = ["described", "realised", "verified", "delivered"];
fn rank(level: &str) -> usize {
    RANK.iter().position(|l| *l == level).unwrap_or(0)
}

/// A computed id → conformance-level map.
pub struct Conformance(HashMap<String, String>);

impl Conformance {
    /// The level for a node id, defaulting to `described` (it merely exists).
    pub fn level(&self, id: &str) -> String {
        self.0.get(id).cloned().unwrap_or_else(|| "described".to_string())
    }

    /// Compute the map from the graph + `.product/` artifacts.
    pub fn compute(g: &DomainGraph, repo_root: &Path) -> Self {
        let mut c = Conformance(HashMap::new());
        c.reachable_is_realised(g);
        let (features, deliverables) = (
            load_all(repo_root, "features", Feature::from_yaml),
            load_all(repo_root, "deliverables", Deliverable::from_yaml),
        );
        let deciders = load_all(repo_root, "deciders", Decider::from_yaml);
        let projectors = load_all(repo_root, "projectors", Projector::from_yaml);
        let conformed = conformed_deciders(repo_root);
        c.verified_deciders(&deciders, &conformed);
        c.done_features_are_verified(g, &features, &deliverables, &deciders, &conformed, &projectors, repo_root);
        c
    }

    /// Raise `id` to `level` (never downgrades).
    fn set(&mut self, id: &str, level: &str) {
        let cur = self.0.get(id).map(|l| rank(l)).unwrap_or(0);
        if rank(level) >= cur {
            self.0.insert(id.to_string(), level.to_string());
        }
    }

    /// Structural pass: a system, the domains it references, and the product are
    /// at least `realised` (they have a surface / composition).
    fn reachable_is_realised(&mut self, g: &DomainGraph) {
        for s in &g.systems {
            self.set(&s.id, "realised");
            for d in &s.references_domain { self.set(d, "realised"); }
        }
        for p in &g.products { self.set(&p.id, "realised"); }
    }

    /// A Decider with a passing `.conform.json` (and its aggregate) is verified.
    fn verified_deciders(&mut self, deciders: &[Decider], conformed: &BTreeSet<String>) {
        for d in deciders {
            if conformed.contains(&d.id) {
                self.set(&d.id, "verified");
                self.set(&d.decides_for, "verified");
            } else {
                self.set(&d.id, "realised");
            }
        }
    }

    /// A feature that is `done` (§7.2) — and its anchored nodes — is verified;
    /// if every deliverable in a release is done, they are delivered.
    #[allow(clippy::too_many_arguments)]
    fn done_features_are_verified(
        &mut self,
        g: &DomainGraph,
        features: &[Feature],
        deliverables: &[Deliverable],
        deciders: &[Decider],
        conformed: &BTreeSet<String>,
        projectors: &[Projector],
        repo_root: &Path,
    ) {
        let mut done_feature: BTreeSet<String> = BTreeSet::new();
        for d in deliverables {
            let Some(f) = features.iter().find(|f| f.id == d.feature) else { continue };
            let fd = feature_done(d, f, g, deciders, conformed, projectors);
            let level = if fd.done { "verified" } else { "realised" };
            self.set(&f.id, level);
            for a in &f.anchors { self.set(a, level); }
            if fd.done { done_feature.insert(f.id.clone()); }
        }
        for r in load_all::<Release>(repo_root, "releases", Release::from_yaml).iter() {
            if !r.features.is_empty() && r.features.iter().all(|df| {
                deliverables.iter().find(|d| &d.id == df).map(|d| done_feature.contains(&d.feature)).unwrap_or(false)
            }) {
                for df in &r.features {
                    if let Some(d) = deliverables.iter().find(|d| &d.id == df) {
                        self.set(&d.feature, "delivered");
                        if let Some(f) = features.iter().find(|f| f.id == d.feature) {
                            for a in &f.anchors { self.set(a, "delivered"); }
                        }
                    }
                }
            }
        }
    }
}

/// Decider ids with a recorded passing conformance verdict (`<id>.conform.json`).
fn conformed_deciders(repo_root: &Path) -> BTreeSet<String> {
    let dir = repo_root.join(".product").join("deciders");
    let mut out = BTreeSet::new();
    let Ok(entries) = std::fs::read_dir(&dir) else { return out };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") { continue; }
        let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else { continue };
        let passing = std::fs::read_to_string(&p).ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| v.get("conformant").and_then(|c| c.as_bool()))
            .unwrap_or(false);
        if passing { out.insert(stem.trim_end_matches(".conform").to_string()); }
    }
    out
}
