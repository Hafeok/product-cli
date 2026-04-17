//! Materialize bidirectional cross-links between newly-created artifacts.

use super::super::super::types::*;
use super::super::super::validate::strip_ref_prefix;
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;

struct Delta {
    target_idx: usize,
    key: String,
    is_validates_inner: bool,
    ids_to_add: Vec<String>,
}

pub fn materialize(
    request: &Request,
    ref_to_id: &HashMap<String, String>,
    new_maps: &mut HashMap<usize, Mapping>,
) {
    let ref_to_index: HashMap<String, usize> = request
        .artifacts
        .iter()
        .filter_map(|a| a.ref_name.as_ref().map(|n| (n.clone(), a.index)))
        .collect();

    let mut deltas: Vec<Delta> = Vec::new();

    for a in &request.artifacts {
        let idx = a.index;
        let this_id = match ref_to_id.get(&format!("__idx_{}", idx)) {
            Some(id) => id.clone(),
            None => continue,
        };

        match a.artifact_type {
            ArtifactType::Feature => {
                for tref in extract_ref_targets(&a.fields, "adrs") {
                    if let Some(tidx) = ref_to_index.get(&tref) {
                        deltas.push(Delta {
                            target_idx: *tidx,
                            key: "features".into(),
                            is_validates_inner: false,
                            ids_to_add: vec![this_id.clone()],
                        });
                    }
                }
                for tref in extract_ref_targets(&a.fields, "tests") {
                    if let Some(tidx) = ref_to_index.get(&tref) {
                        deltas.push(Delta {
                            target_idx: *tidx,
                            key: "features".into(),
                            is_validates_inner: true,
                            ids_to_add: vec![this_id.clone()],
                        });
                    }
                }
                for tref in extract_ref_targets(&a.fields, "uses") {
                    if let Some(tidx) = ref_to_index.get(&tref) {
                        deltas.push(Delta {
                            target_idx: *tidx,
                            key: "features".into(),
                            is_validates_inner: false,
                            ids_to_add: vec![this_id.clone()],
                        });
                    }
                }
            }
            ArtifactType::Adr => {
                for tref in extract_ref_targets(&a.fields, "features") {
                    if let Some(tidx) = ref_to_index.get(&tref) {
                        deltas.push(Delta {
                            target_idx: *tidx,
                            key: "adrs".into(),
                            is_validates_inner: false,
                            ids_to_add: vec![this_id.clone()],
                        });
                    }
                }
                for tref in extract_ref_targets(&a.fields, "governs") {
                    if let Some(tidx) = ref_to_index.get(&tref) {
                        deltas.push(Delta {
                            target_idx: *tidx,
                            key: "adrs".into(),
                            is_validates_inner: false,
                            ids_to_add: vec![this_id.clone()],
                        });
                    }
                }
            }
            ArtifactType::Tc => {
                for tref in extract_validates_refs(&a.fields, "features") {
                    if let Some(tidx) = ref_to_index.get(&tref) {
                        deltas.push(Delta {
                            target_idx: *tidx,
                            key: "tests".into(),
                            is_validates_inner: false,
                            ids_to_add: vec![this_id.clone()],
                        });
                    }
                }
            }
            ArtifactType::Dep => {}
        }
    }

    for d in deltas {
        if let Some(m) = new_maps.get_mut(&d.target_idx) {
            if d.is_validates_inner {
                let vk = Value::String("validates".into());
                let mut vmap = match m.get(&vk) {
                    Some(Value::Mapping(inner)) => inner.clone(),
                    _ => Mapping::new(),
                };
                append_to_list(&mut vmap, &d.key, &d.ids_to_add);
                m.insert(vk, Value::Mapping(vmap));
            } else {
                append_to_list(m, &d.key, &d.ids_to_add);
            }
        }
    }

    // Drop synthetic fields that are not stored on their own artifacts.
    for a in &request.artifacts {
        if let Some(m) = new_maps.get_mut(&a.index) {
            if a.artifact_type == ArtifactType::Feature {
                m.remove(Value::String("uses".into()));
            }
            if a.artifact_type == ArtifactType::Adr {
                m.remove(Value::String("governs".into()));
            }
        }
    }
}

fn extract_ref_targets(m: &Mapping, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(Value::Sequence(seq)) = m.get(Value::String(key.into())) {
        for item in seq {
            if let Value::String(s) = item {
                if let Some(r) = strip_ref_prefix(s) {
                    out.push(r.to_string());
                }
            }
        }
    }
    out
}

fn extract_validates_refs(m: &Mapping, inner_key: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(Value::Mapping(v)) = m.get(Value::String("validates".into())) {
        if let Some(Value::Sequence(seq)) = v.get(Value::String(inner_key.into())) {
            for item in seq {
                if let Value::String(s) = item {
                    if let Some(r) = strip_ref_prefix(s) {
                        out.push(r.to_string());
                    }
                }
            }
        }
    }
    out
}

fn append_to_list(m: &mut Mapping, key: &str, values: &[String]) {
    let k = Value::String(key.into());
    let existing = m.get(&k).cloned().unwrap_or_else(|| Value::Sequence(Vec::new()));
    let mut seq = match existing {
        Value::Sequence(s) => s,
        _ => Vec::new(),
    };
    for v in values {
        let candidate = Value::String(v.clone());
        if !seq.contains(&candidate) {
            seq.push(candidate);
        }
    }
    m.insert(k, Value::Sequence(seq));
}
