//! `add dep` — optionally creates a governing ADR in the same step.

use super::add::{AddDepArgs, AddedArtifact};
use super::add_helpers::{make_ref_name, resolve_id_or_ref, restore_from_yaml, validate_draft};
use super::draft::{Draft, DraftKind};
use crate::config::ProductConfig;
use crate::request::Finding;
use serde_yaml::{Mapping, Value};

pub fn add_dep(
    draft: &mut Draft,
    args: AddDepArgs,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Result<AddedArtifact, Vec<Finding>> {
    if draft.kind() != Some(DraftKind::Create) {
        return Err(vec![Finding::error(
            "E006",
            "add dep requires a 'create' draft — run `product request new create`",
            "$.type",
        )]);
    }
    let dep_ref = args
        .ref_name
        .clone()
        .unwrap_or_else(|| make_ref_name("dep", &args.title, draft));
    let mut dep_m = Mapping::new();
    dep_m.insert(Value::String("type".into()), Value::String("dep".into()));
    dep_m.insert(Value::String("ref".into()), Value::String(dep_ref.clone()));
    dep_m.insert(Value::String("title".into()), Value::String(args.title.clone()));
    dep_m.insert(Value::String("dep-type".into()), Value::String(args.dep_type));
    if let Some(v) = &args.version {
        dep_m.insert(Value::String("version".into()), Value::String(v.clone()));
    }

    let mut refs = vec![dep_ref.clone()];
    let mut note: Option<String> = None;
    let mut new_adr_mapping: Option<Mapping> = None;

    match args.adr.as_deref() {
        Some("new") => {
            let adr_title = args
                .adr_title
                .clone()
                .unwrap_or_else(|| format!("Governs {}", args.title));
            let adr_ref = make_ref_name("adr", &adr_title, draft);
            let mut adr_m = Mapping::new();
            adr_m.insert(Value::String("type".into()), Value::String("adr".into()));
            adr_m.insert(Value::String("ref".into()), Value::String(adr_ref.clone()));
            adr_m.insert(Value::String("title".into()), Value::String(adr_title));
            adr_m.insert(
                Value::String("scope".into()),
                Value::String("feature-specific".into()),
            );
            adr_m.insert(
                Value::String("governs".into()),
                Value::Sequence(vec![Value::String(format!("ref:{dep_ref}"))]),
            );
            new_adr_mapping = Some(adr_m);
            refs.push(adr_ref);
            note = Some("E013 satisfied — dep has governing ADR in draft".to_string());
        }
        Some(existing) if !existing.is_empty() => {
            dep_m.insert(
                Value::String("adrs".into()),
                Value::Sequence(vec![resolve_id_or_ref(existing)]),
            );
        }
        _ => {}
    }

    let snapshot = draft.to_yaml();
    if let Some(adr_m) = new_adr_mapping {
        draft.artifacts_mut().push(Value::Mapping(adr_m));
    }
    draft.artifacts_mut().push(Value::Mapping(dep_m));

    let findings = validate_draft(draft, config, graph);
    if findings.iter().any(|f| f.is_error()) {
        let _ = restore_from_yaml(draft, &snapshot);
        return Err(findings);
    }
    Ok(AddedArtifact { refs, findings, note })
}
