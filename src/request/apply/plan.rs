//! Planning — compute the full set of writes (new + mutated) before committing.

use super::super::types::*;
use super::super::validate::strip_ref_prefix;
use super::assign::{resolve_refs_in_mapping};
use super::mutate::apply_mutation;
use super::render::{render_from_mapping, render_new_artifact};
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::parser;
use serde_yaml::{Mapping, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

pub struct NewWritePlanned {
    pub path: PathBuf,
    pub content: String,
    pub assigned_id: (Option<String>, String), // (ref_name, id)
}

pub struct MutationPlanned {
    pub path: PathBuf,
    pub content: String,
    pub target_id: String,
    pub mutation_count: usize,
}

pub fn plan_writes(
    request: &Request,
    ref_to_id: &HashMap<String, String>,
    graph: &KnowledgeGraph,
    config: &ProductConfig,
    repo_root: &Path,
) -> Result<(Vec<NewWritePlanned>, Vec<MutationPlanned>), Vec<Finding>> {
    let mut new_writes: Vec<NewWritePlanned> = Vec::new();
    let mut mutated_maps: BTreeMap<String, Mapping> = BTreeMap::new();
    let mut existing_bodies: HashMap<String, String> = HashMap::new();
    let mut existing_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut mutation_counts: HashMap<String, usize> = HashMap::new();
    let mut errors: Vec<Finding> = Vec::new();

    for (id, f) in &graph.features {
        existing_bodies.insert(id.clone(), f.body.clone());
        existing_paths.insert(id.clone(), f.path.clone());
    }
    for (id, a) in &graph.adrs {
        existing_bodies.insert(id.clone(), a.body.clone());
        existing_paths.insert(id.clone(), a.path.clone());
    }
    for (id, t) in &graph.tests {
        existing_bodies.insert(id.clone(), t.body.clone());
        existing_paths.insert(id.clone(), t.path.clone());
    }
    for (id, d) in &graph.dependencies {
        existing_bodies.insert(id.clone(), d.body.clone());
        existing_paths.insert(id.clone(), d.path.clone());
    }

    // Build initial per-new-artifact maps with refs resolved.
    let mut new_maps: HashMap<usize, Mapping> = HashMap::new();
    for a in &request.artifacts {
        let id = ref_to_id
            .get(&format!("__idx_{}", a.index))
            .cloned()
            .unwrap_or_default();
        let mut resolved = resolve_refs_in_mapping(&a.fields, ref_to_id);
        resolved.insert(Value::String("id".into()), Value::String(id));
        new_maps.insert(a.index, resolved);
    }

    bidirectional::materialize(request, ref_to_id, &mut new_maps);

    // Apply changes — either to new maps or to loaded existing files.
    for c in &request.changes {
        let target_id = if let Some(ref_name) = strip_ref_prefix(&c.target) {
            match ref_to_id.get(ref_name) {
                Some(id) => id.clone(),
                None => {
                    errors.push(Finding::error(
                        "E002",
                        format!("change target 'ref:{}' not defined in request", ref_name),
                        format!("$.changes[{}].target", c.index),
                    ));
                    continue;
                }
            }
        } else {
            c.target.clone()
        };
        *mutation_counts.entry(target_id.clone()).or_insert(0) += c.mutations.len();

        // Target is a new artifact?
        let mut handled_new = false;
        for (_idx, m) in new_maps.iter_mut() {
            let this_id = m.get(Value::String("id".into())).and_then(|v| v.as_str());
            if this_id == Some(target_id.as_str()) {
                for mu in &c.mutations {
                    if let Err(f) = apply_mutation(m, mu, ref_to_id) {
                        errors.push(Finding::error(
                            "E001", f,
                            format!("$.changes[{}].mutations[{}]", c.index, mu.index),
                        ));
                    }
                }
                handled_new = true;
                break;
            }
        }
        if handled_new { continue; }

        if !existing_paths.contains_key(&target_id) {
            errors.push(Finding::error(
                "E002",
                format!("change target '{}' does not exist", target_id),
                format!("$.changes[{}].target", c.index),
            ));
            continue;
        }
        // Load or fetch the current mapping for the existing file.
        if !mutated_maps.contains_key(&target_id) {
            let m = load_existing_front(&existing_paths[&target_id])?;
            mutated_maps.insert(target_id.clone(), m);
        }
        let current = mutated_maps.get_mut(&target_id).expect("present");
        for mu in &c.mutations {
            if let Err(f) = apply_mutation(current, mu, ref_to_id) {
                errors.push(Finding::error(
                    "E001", f,
                    format!("$.changes[{}].mutations[{}]", c.index, mu.index),
                ));
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // Render new artifacts.
    for a in &request.artifacts {
        let id = ref_to_id
            .get(&format!("__idx_{}", a.index))
            .cloned()
            .unwrap_or_default();
        let title = a.fields
            .get(Value::String("title".into()))
            .and_then(|v| v.as_str())
            .unwrap_or("untitled");
        let filename = parser::id_to_filename(&id, title);
        let dir = dir_for_artifact_type(a.artifact_type, config, repo_root);
        let path = dir.join(filename);
        let map = new_maps.get(&a.index).cloned().unwrap_or_else(Mapping::new);
        let content = render_new_artifact(a.artifact_type, map, a.artifact_type.to_string().as_str())?;
        new_writes.push(NewWritePlanned {
            path,
            content,
            assigned_id: (a.ref_name.clone(), id),
        });
    }

    // Render mutation files.
    let mut mutated: Vec<MutationPlanned> = Vec::new();
    for (target_id, map) in mutated_maps {
        let path = existing_paths[&target_id].clone();
        let body = existing_bodies.get(&target_id).cloned().unwrap_or_default();
        let (body_override, new_front) = extract_body_override(map);
        let final_body = body_override.unwrap_or(body);
        let content = render_from_mapping(&new_front, &final_body)?;
        let count = mutation_counts.get(&target_id).copied().unwrap_or(0);
        mutated.push(MutationPlanned {
            path, content, target_id, mutation_count: count,
        });
    }

    Ok((new_writes, mutated))
}

fn dir_for_artifact_type(t: ArtifactType, config: &ProductConfig, repo_root: &Path) -> PathBuf {
    match t {
        ArtifactType::Feature => config.resolve_path(repo_root, &config.paths.features),
        ArtifactType::Adr => config.resolve_path(repo_root, &config.paths.adrs),
        ArtifactType::Tc => config.resolve_path(repo_root, &config.paths.tests),
        ArtifactType::Dep => config.resolve_path(repo_root, &config.paths.dependencies),
    }
}

fn extract_body_override(mut m: Mapping) -> (Option<String>, Mapping) {
    let body_key = Value::String("body".into());
    if let Some(v) = m.remove(body_key) {
        let s = match v {
            Value::String(s) => Some(s),
            _ => None,
        };
        (s, m)
    } else {
        (None, m)
    }
}

fn load_existing_front(path: &Path) -> Result<Mapping, Vec<Finding>> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        vec![Finding::error("E001", format!("failed to read {}: {}", path.display(), e), "$")]
    })?;
    let (yaml, _body) = split_front_matter(&content).ok_or_else(|| {
        vec![Finding::error("E001", format!("{} has no YAML front-matter", path.display()), "$")]
    })?;
    let parsed: Value = serde_yaml::from_str(yaml).map_err(|e| {
        vec![Finding::error("E001", format!("failed to parse front-matter: {}", e), "$")]
    })?;
    match parsed {
        Value::Mapping(m) => Ok(m),
        _ => Err(vec![Finding::error("E001", format!("front-matter of {} is not a mapping", path.display()), "$")]),
    }
}

fn split_front_matter(content: &str) -> Option<(&str, &str)> {
    let content = content.trim_start();
    if !content.starts_with("---") { return None; }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml = &rest[..end];
    let body_start = end + 4;
    let body = if body_start < rest.len() {
        rest[body_start..].trim_start_matches('\n')
    } else { "" };
    Some((yaml, body))
}

pub mod bidirectional;
