//! ID assignment for new artifacts (FT-041, ADR-038).

use super::super::types::*;
use super::super::validate::strip_ref_prefix;
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;

/// Assign real IDs to every new artifact. Returns a map from ref-name → ID and
/// also indexes by `__idx_<index>` so artifacts without a ref can still be
/// resolved.
pub fn assign_ids(
    artifacts: &[ArtifactSpec],
    graph: &KnowledgeGraph,
    config: &ProductConfig,
) -> Result<HashMap<String, String>, Finding> {
    let n = artifacts.len();
    let name_to_index: HashMap<String, usize> = artifacts
        .iter()
        .filter_map(|a| a.ref_name.as_ref().map(|n| (n.clone(), a.index)))
        .collect();

    let mut deps: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, a) in artifacts.iter().enumerate() {
        for t in collect_ref_targets(&a.fields) {
            if let Some(&idx) = name_to_index.get(&t) {
                if idx != i {
                    deps[i].push(idx);
                }
            }
        }
    }
    let order = topo_or_declaration(n, &deps);

    let mut ref_to_id: HashMap<String, String> = HashMap::new();
    let mut next_ft = next_existing(&config.prefixes.feature, graph.features.keys());
    let mut next_adr = next_existing(&config.prefixes.adr, graph.adrs.keys());
    let mut next_tc = next_existing(&config.prefixes.test, graph.tests.keys());
    let mut next_dep = next_existing(&config.prefixes.dependency, graph.dependencies.keys());

    for &i in &order {
        let a = &artifacts[i];
        let (prefix, counter) = match a.artifact_type {
            ArtifactType::Feature => (&config.prefixes.feature, &mut next_ft),
            ArtifactType::Adr => (&config.prefixes.adr, &mut next_adr),
            ArtifactType::Tc => (&config.prefixes.test, &mut next_tc),
            ArtifactType::Dep => (&config.prefixes.dependency, &mut next_dep),
        };
        let id = format!("{}-{:03}", prefix, *counter);
        *counter += 1;
        if let Some(ref n) = a.ref_name {
            ref_to_id.insert(n.clone(), id.clone());
        }
        ref_to_id.insert(format!("__idx_{}", a.index), id);
    }
    Ok(ref_to_id)
}

fn topo_or_declaration(n: usize, deps: &[Vec<usize>]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(n);
    let mut processed = vec![false; n];
    let mut queue: std::collections::VecDeque<usize> =
        (0..n).filter(|&i| deps[i].is_empty()).collect();
    while let Some(i) = queue.pop_front() {
        if processed[i] { continue; }
        processed[i] = true;
        out.push(i);
        for j in 0..n {
            if !processed[j] && deps[j].iter().all(|&d| processed[d]) {
                queue.push_back(j);
            }
        }
    }
    for (i, done) in processed.iter().enumerate() {
        if !done { out.push(i); }
    }
    out
}

fn next_existing<'a, I: Iterator<Item = &'a String>>(prefix: &str, ids: I) -> u32 {
    let max = ids
        .filter_map(|id| {
            id.strip_prefix(prefix)
                .and_then(|r| r.strip_prefix('-'))
                .and_then(|n| n.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    max + 1
}

/// Collect every `ref:<name>` target appearing anywhere inside a mapping.
pub fn collect_ref_targets(m: &Mapping) -> Vec<String> {
    let mut out = Vec::new();
    collect(&Value::Mapping(m.clone()), &mut out);
    out
}

fn collect(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => {
            if let Some(r) = strip_ref_prefix(s) {
                out.push(r.to_string());
            }
        }
        Value::Sequence(seq) => {
            for item in seq {
                collect(item, out);
            }
        }
        Value::Mapping(m) => {
            for (_, v) in m.iter() {
                collect(v, out);
            }
        }
        _ => {}
    }
}

/// Resolve `ref:<name>` strings to real IDs throughout a value.
pub fn resolve_refs(v: &Value, ref_to_id: &HashMap<String, String>) -> Value {
    match v {
        Value::String(s) => {
            if let Some(r) = strip_ref_prefix(s) {
                if let Some(id) = ref_to_id.get(r) {
                    return Value::String(id.clone());
                }
            }
            Value::String(s.clone())
        }
        Value::Sequence(seq) => {
            Value::Sequence(seq.iter().map(|i| resolve_refs(i, ref_to_id)).collect())
        }
        Value::Mapping(m) => {
            let mut out = Mapping::new();
            for (k, v) in m.iter() {
                out.insert(k.clone(), resolve_refs(v, ref_to_id));
            }
            Value::Mapping(out)
        }
        _ => v.clone(),
    }
}

pub fn resolve_refs_in_mapping(m: &Mapping, ref_to_id: &HashMap<String, String>) -> Mapping {
    let mut out = Mapping::new();
    for (k, v) in m.iter() {
        out.insert(k.clone(), resolve_refs(v, ref_to_id));
    }
    out
}
