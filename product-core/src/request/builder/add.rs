//! `product request add …` — append artifacts or mutations to the draft.
//!
//! Every `add` subcommand constructs one YAML `Mapping` that maps 1:1 to the
//! same artifact or change block a hand-written request would contain, and
//! appends it to the draft. Incremental validation runs against the draft
//! plus the existing graph and surfaces findings; on an E-class finding, the
//! append is rolled back.

use super::add_helpers::{
    append_artifact_transactional, make_ref_name, resolve_id_or_ref, restore_from_yaml,
    validate_draft,
};
use super::draft::{Draft, DraftKind};
use crate::config::ProductConfig;
use crate::request::Finding;
use serde_yaml::{Mapping, Value};

/// Result of a successful `add`. Contains the ref names appended so the
/// caller can surface them to the user.
pub struct AddedArtifact {
    pub refs: Vec<String>,
    pub findings: Vec<Finding>,
    pub note: Option<String>,
}

pub use super::add_helpers::validate_draft as validate_incremental;

// ---------------------------------------------------------------------------
// add feature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct AddFeatureArgs {
    pub title: String,
    pub phase: u32,
    pub domains: Vec<String>,
    pub ref_name: Option<String>,
}

pub fn add_feature(
    draft: &mut Draft,
    args: AddFeatureArgs,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Result<AddedArtifact, Vec<Finding>> {
    if draft.kind() != Some(DraftKind::Create) {
        return Err(vec![Finding::error(
            "E006",
            "add feature requires a 'create' draft — run `product request new create`",
            "$.type",
        )]);
    }
    let ref_name = args
        .ref_name
        .unwrap_or_else(|| make_ref_name("ft", &args.title, draft));
    let mut m = Mapping::new();
    m.insert(Value::String("type".into()), Value::String("feature".into()));
    m.insert(Value::String("ref".into()), Value::String(ref_name.clone()));
    m.insert(Value::String("title".into()), Value::String(args.title));
    m.insert(Value::String("phase".into()), Value::Number(args.phase.into()));
    m.insert(
        Value::String("domains".into()),
        Value::Sequence(
            args.domains
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
    append_artifact_transactional(draft, m, config, graph)
        .map(|findings| AddedArtifact { refs: vec![ref_name], findings, note: None })
}

// ---------------------------------------------------------------------------
// add adr
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct AddAdrArgs {
    pub title: String,
    pub domains: Vec<String>,
    pub scope: Option<String>,
    pub governs: Vec<String>,
    pub ref_name: Option<String>,
}

pub fn add_adr(
    draft: &mut Draft,
    args: AddAdrArgs,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Result<AddedArtifact, Vec<Finding>> {
    if draft.kind() != Some(DraftKind::Create) {
        return Err(vec![Finding::error(
            "E006",
            "add adr requires a 'create' draft — run `product request new create`",
            "$.type",
        )]);
    }
    let ref_name = args
        .ref_name
        .clone()
        .unwrap_or_else(|| make_ref_name("adr", &args.title, draft));
    let mut m = Mapping::new();
    m.insert(Value::String("type".into()), Value::String("adr".into()));
    m.insert(Value::String("ref".into()), Value::String(ref_name.clone()));
    m.insert(Value::String("title".into()), Value::String(args.title));
    if !args.domains.is_empty() {
        m.insert(
            Value::String("domains".into()),
            Value::Sequence(args.domains.into_iter().map(Value::String).collect()),
        );
    }
    if !args.governs.is_empty() {
        m.insert(
            Value::String("governs".into()),
            Value::Sequence(
                args.governs
                    .into_iter()
                    .map(|g| Value::String(format!("ref:{g}")))
                    .collect(),
            ),
        );
    }
    let scope = args.scope.unwrap_or_else(|| "feature-specific".to_string());
    m.insert(Value::String("scope".into()), Value::String(scope));
    append_artifact_transactional(draft, m, config, graph)
        .map(|findings| AddedArtifact { refs: vec![ref_name], findings, note: None })
}

// ---------------------------------------------------------------------------
// add tc
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct AddTcArgs {
    pub title: String,
    pub tc_type: String,
    pub validates_features: Vec<String>,
    pub validates_adrs: Vec<String>,
    pub ref_name: Option<String>,
}

pub fn add_tc(
    draft: &mut Draft,
    args: AddTcArgs,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Result<AddedArtifact, Vec<Finding>> {
    if draft.kind() != Some(DraftKind::Create) {
        return Err(vec![Finding::error(
            "E006",
            "add tc requires a 'create' draft — run `product request new create`",
            "$.type",
        )]);
    }
    let ref_name = args
        .ref_name
        .clone()
        .unwrap_or_else(|| make_ref_name("tc", &args.title, draft));
    let mut m = Mapping::new();
    m.insert(Value::String("type".into()), Value::String("tc".into()));
    m.insert(Value::String("ref".into()), Value::String(ref_name.clone()));
    m.insert(Value::String("title".into()), Value::String(args.title));
    m.insert(Value::String("tc-type".into()), Value::String(args.tc_type));
    let mut validates = Mapping::new();
    if !args.validates_features.is_empty() {
        validates.insert(
            Value::String("features".into()),
            Value::Sequence(
                args.validates_features
                    .iter()
                    .map(|f| resolve_id_or_ref(f))
                    .collect(),
            ),
        );
    }
    if !args.validates_adrs.is_empty() {
        validates.insert(
            Value::String("adrs".into()),
            Value::Sequence(
                args.validates_adrs
                    .iter()
                    .map(|f| resolve_id_or_ref(f))
                    .collect(),
            ),
        );
    }
    if !validates.is_empty() {
        m.insert(Value::String("validates".into()), Value::Mapping(validates));
    }
    append_artifact_transactional(draft, m, config, graph)
        .map(|findings| AddedArtifact { refs: vec![ref_name], findings, note: None })
}

// ---------------------------------------------------------------------------
// add dep — optionally creates a governing ADR in the same step
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct AddDepArgs {
    pub title: String,
    pub dep_type: String,
    pub version: Option<String>,
    pub adr: Option<String>,
    pub adr_title: Option<String>,
    pub ref_name: Option<String>,
}

pub use super::add_dep::add_dep;

// ---------------------------------------------------------------------------
// add doc — structural convenience mapping to add_adr
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct AddDocArgs {
    pub title: String,
    pub domains: Vec<String>,
    pub ref_name: Option<String>,
}

pub fn add_doc(
    draft: &mut Draft,
    args: AddDocArgs,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Result<AddedArtifact, Vec<Finding>> {
    add_adr(
        draft,
        AddAdrArgs {
            title: args.title,
            domains: args.domains,
            scope: Some("feature-specific".into()),
            governs: Vec::new(),
            ref_name: args.ref_name,
        },
        config,
        graph,
    )
}

// ---------------------------------------------------------------------------
// add target + acknowledgement (change-mode drafts)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct AddTargetArgs {
    pub target: String,
    pub mutations: Vec<MutationArg>,
}

#[derive(Debug, Clone)]
pub struct MutationArg {
    pub op: String,
    pub field: String,
    pub value: Option<Value>,
}

pub fn add_target(
    draft: &mut Draft,
    args: AddTargetArgs,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Result<AddedArtifact, Vec<Finding>> {
    if draft.kind() != Some(DraftKind::Change) {
        return Err(vec![Finding::error(
            "E006",
            "add target requires a 'change' draft — run `product request new change`",
            "$.type",
        )]);
    }
    let mut change = Mapping::new();
    change.insert(Value::String("target".into()), Value::String(args.target.clone()));
    let muts: Vec<Value> = args
        .mutations
        .iter()
        .map(|mu| {
            let mut m = Mapping::new();
            m.insert(Value::String("op".into()), Value::String(mu.op.clone()));
            m.insert(Value::String("field".into()), Value::String(mu.field.clone()));
            if let Some(v) = &mu.value {
                m.insert(Value::String("value".into()), v.clone());
            }
            Value::Mapping(m)
        })
        .collect();
    change.insert(Value::String("mutations".into()), Value::Sequence(muts));

    let snapshot = draft.to_yaml();
    draft.changes_mut().push(Value::Mapping(change));
    let findings = validate_draft(draft, config, graph);
    if findings.iter().any(|f| f.is_error()) {
        let _ = restore_from_yaml(draft, &snapshot);
        return Err(findings);
    }
    Ok(AddedArtifact { refs: vec![args.target], findings, note: None })
}

#[derive(Debug, Clone, Default)]
pub struct AddAckArgs {
    pub target: String,
    pub domain: String,
    pub reason: String,
}

pub fn add_acknowledgement(
    draft: &mut Draft,
    args: AddAckArgs,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Result<AddedArtifact, Vec<Finding>> {
    if draft.kind() != Some(DraftKind::Change) {
        return Err(vec![Finding::error(
            "E006",
            "add acknowledgement requires a 'change' draft — run `product request new change`",
            "$.type",
        )]);
    }
    add_target(
        draft,
        AddTargetArgs {
            target: args.target.clone(),
            mutations: vec![MutationArg {
                op: "set".into(),
                field: format!("domains-acknowledged.{}", args.domain),
                value: Some(Value::String(args.reason)),
            }],
        },
        config,
        graph,
    )
}
