//! Unit tests for pure ADR planning functions.

use super::*;
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::types::{Adr, AdrFrontMatter, AdrScope, AdrStatus};
use std::path::PathBuf;

fn adr(id: &str, title: &str, status: AdrStatus) -> Adr {
    Adr {
        front: AdrFrontMatter {
            id: id.to_string(),
            title: title.to_string(),
            status,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope: AdrScope::Domain,
            content_hash: None,
            amendments: vec![],
            source_files: vec![],
            removes: vec![],
            deprecates: vec![],
        },
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

fn config_with_domains(names: &[&str]) -> ProductConfig {
    let domain_entries: String = names
        .iter()
        .map(|d| format!("{} = \"test domain\"", d))
        .collect::<Vec<_>>()
        .join("\n");
    let toml_str = format!("name = \"test\"\n[domains]\n{}\n", domain_entries);
    toml::from_str(&toml_str).expect("parse test config")
}

// ---- create --------------------------------------------------------------

#[test]
fn plan_create_rejects_empty_title() {
    let err = create::plan_create("  ", &[], "ADR").unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_create_generates_next_id() {
    let plan = create::plan_create("first one", &[], "ADR").expect("plan");
    assert_eq!(plan.id, "ADR-001");
    assert_eq!(plan.filename, "ADR-001-first-one.md");
    assert_eq!(plan.front.status, AdrStatus::Proposed);
    assert_eq!(plan.front.scope, AdrScope::Domain);
    assert!(plan.front.content_hash.is_none());
}

#[test]
fn plan_create_after_existing_returns_next() {
    let plan = create::plan_create(
        "x",
        &["ADR-001".to_string(), "ADR-005".to_string()],
        "ADR",
    )
    .expect("plan");
    assert_eq!(plan.id, "ADR-006");
}

// ---- field edits ---------------------------------------------------------

#[test]
fn plan_domain_edit_rejects_unknown() {
    let graph = KnowledgeGraph::build(vec![], vec![adr("ADR-001", "x", AdrStatus::Proposed)], vec![]);
    let config = config_with_domains(&["known"]);
    let err = field_edits::plan_domain_edit(
        &config,
        &graph,
        "ADR-001",
        &["nope".to_string()],
        &[],
    )
    .unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_domain_edit_adds_sorted_unique() {
    let graph = KnowledgeGraph::build(vec![], vec![adr("ADR-001", "x", AdrStatus::Proposed)], vec![]);
    let config = config_with_domains(&["a", "b", "c"]);
    let plan = field_edits::plan_domain_edit(
        &config,
        &graph,
        "ADR-001",
        &["c".to_string(), "a".to_string(), "c".to_string()],
        &[],
    )
    .expect("plan");
    assert_eq!(plan.final_domains, vec!["a", "c"]);
}

#[test]
fn plan_scope_change_records_new_scope() {
    let graph = KnowledgeGraph::build(vec![], vec![adr("ADR-001", "x", AdrStatus::Proposed)], vec![]);
    let plan = field_edits::plan_scope_change(&graph, "ADR-001", AdrScope::CrossCutting)
        .expect("plan");
    assert_eq!(plan.new_scope, AdrScope::CrossCutting);
}

#[test]
fn plan_source_files_edit_flags_missing_paths_but_still_plans() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let graph = KnowledgeGraph::build(vec![], vec![adr("ADR-001", "x", AdrStatus::Proposed)], vec![]);
    let plan = field_edits::plan_source_files_edit(
        &graph,
        tmp.path(),
        "ADR-001",
        &["does/not/exist.rs".to_string()],
        &[],
    )
    .expect("plan");
    assert_eq!(plan.final_source_files, vec!["does/not/exist.rs"]);
    assert_eq!(plan.missing_added_paths, vec!["does/not/exist.rs"]);
}

// ---- status change -------------------------------------------------------

#[test]
fn plan_status_change_accepted_computes_hash() {
    let graph = KnowledgeGraph::build(vec![], vec![adr("ADR-001", "x", AdrStatus::Proposed)], vec![]);
    let plan =
        status_change::plan_status_change(&graph, "ADR-001", AdrStatus::Accepted, None).expect("plan");
    assert_eq!(plan.new_status, AdrStatus::Accepted);
    // Rendered content should include a content-hash line
    assert!(plan.adr_content.contains("content-hash"));
    assert!(plan.successor_update.is_none());
}

#[test]
fn plan_status_change_by_links_successor_bidirectionally() {
    let a = adr("ADR-001", "old", AdrStatus::Accepted);
    let b = adr("ADR-002", "new", AdrStatus::Accepted);
    let graph = KnowledgeGraph::build(vec![], vec![a, b], vec![]);
    let plan = status_change::plan_status_change(
        &graph,
        "ADR-001",
        AdrStatus::Superseded,
        Some("ADR-002"),
    )
    .expect("plan");
    assert!(plan.adr_content.contains("ADR-002"));
    let succ = plan.successor_update.expect("successor update");
    assert!(succ.adr_content.contains("ADR-001"));
}

#[test]
fn plan_status_change_not_found() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let err = status_change::plan_status_change(&graph, "ADR-999", AdrStatus::Accepted, None)
        .unwrap_err();
    assert!(matches!(err, crate::error::ProductError::NotFound(_)));
}

// ---- supersede -----------------------------------------------------------

#[test]
fn plan_supersede_add_changes_accepted_target_to_superseded() {
    let a = adr("ADR-001", "old", AdrStatus::Accepted);
    let b = adr("ADR-002", "new", AdrStatus::Accepted);
    let graph = KnowledgeGraph::build(vec![], vec![a, b], vec![]);
    let plan = supersede::plan_supersede_add(&graph, "ADR-002", "ADR-001").expect("plan");
    assert!(plan.target_status_changed_to_superseded);
    assert!(plan.source_content.contains("ADR-001"));
    assert!(plan.target_content.contains("superseded"));
}

#[test]
fn plan_supersede_add_detects_cycle() {
    let mut a = adr("ADR-001", "a", AdrStatus::Accepted);
    let mut b = adr("ADR-002", "b", AdrStatus::Accepted);
    a.front.superseded_by = vec!["ADR-002".to_string()];
    b.front.supersedes = vec!["ADR-001".to_string()];
    let graph = KnowledgeGraph::build(vec![], vec![a, b], vec![]);
    let err = supersede::plan_supersede_add(&graph, "ADR-001", "ADR-002").unwrap_err();
    assert!(matches!(err, crate::error::ProductError::SupersessionCycle { .. }));
}

#[test]
fn plan_supersede_remove_clears_both_sides() {
    let mut a = adr("ADR-001", "a", AdrStatus::Accepted);
    let mut b = adr("ADR-002", "b", AdrStatus::Accepted);
    a.front.superseded_by = vec!["ADR-002".to_string()];
    b.front.supersedes = vec!["ADR-001".to_string()];
    let graph = KnowledgeGraph::build(vec![], vec![a, b], vec![]);
    let plan = supersede::plan_supersede_remove(&graph, "ADR-002", "ADR-001").expect("plan");
    assert!(!plan.source_content.contains("supersedes:\n- ADR-001"));
    assert!(!plan.target_content.contains("superseded-by:\n- ADR-002"));
}

// ---- seal / amend --------------------------------------------------------

#[test]
fn plan_amend_rejects_empty_reason() {
    let graph = KnowledgeGraph::build(vec![], vec![adr("ADR-001", "x", AdrStatus::Accepted)], vec![]);
    let err = seal::plan_amend(&graph, "ADR-001", "   ").unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_seal_skips_already_sealed() {
    let mut a = adr("ADR-001", "x", AdrStatus::Accepted);
    a.front.content_hash = Some("existing".to_string());
    let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
    let plan = seal::plan_seal(&graph, "ADR-001").expect("plan result");
    assert!(plan.is_none());
}

#[test]
fn plan_seal_produces_hash_for_unsealed() {
    let graph = KnowledgeGraph::build(vec![], vec![adr("ADR-001", "x", AdrStatus::Accepted)], vec![]);
    let plan = seal::plan_seal(&graph, "ADR-001")
        .expect("plan result")
        .expect("plan present");
    assert!(!plan.new_hash.is_empty());
    assert!(plan.adr_content.contains("content-hash"));
}

#[test]
fn unsealed_accepted_ids_excludes_sealed_and_proposed() {
    let a = adr("ADR-001", "a", AdrStatus::Accepted);
    let b = adr("ADR-002", "b", AdrStatus::Proposed);
    let mut c = adr("ADR-003", "c", AdrStatus::Accepted);
    c.front.content_hash = Some("h".into());
    let graph = KnowledgeGraph::build(vec![], vec![a, b, c], vec![]);
    let ids = seal::unsealed_accepted_ids(&graph);
    assert_eq!(ids, vec!["ADR-001"]);
}

// ---- conflicts -----------------------------------------------------------

#[test]
fn check_conflicts_empty_targets_returns_no_findings() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let findings = conflicts::check_conflicts(&graph, &[]).expect("ok");
    assert!(findings.is_empty());
}

#[test]
fn check_conflicts_not_found_for_missing_target() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let err = conflicts::check_conflicts(&graph, &["ADR-999".to_string()]).unwrap_err();
    assert!(matches!(err, crate::error::ProductError::NotFound(_)));
}

#[test]
fn check_conflicts_flags_supersession_asymmetry() {
    let mut a = adr("ADR-001", "a", AdrStatus::Accepted);
    a.front.superseded_by = vec!["ADR-002".to_string()];
    let b = adr("ADR-002", "b", AdrStatus::Accepted); // missing reciprocal
    let graph = KnowledgeGraph::build(vec![], vec![a, b], vec![]);
    let findings =
        conflicts::check_conflicts(&graph, &["ADR-001".to_string()]).expect("ok");
    assert!(findings.iter().any(|f| f.code == conflicts::FindingCode::W025));
}

#[test]
fn check_conflicts_flags_cross_cutting_with_features() {
    let mut a = adr("ADR-001", "a", AdrStatus::Accepted);
    a.front.scope = AdrScope::CrossCutting;
    a.front.features = vec!["FT-001".to_string()];
    let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
    let findings =
        conflicts::check_conflicts(&graph, &["ADR-001".to_string()]).expect("ok");
    assert!(findings.iter().any(|f| f.code == conflicts::FindingCode::W027));
}
