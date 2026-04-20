//! ADR supersession — bidirectional link management with cycle detection.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser, types};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SupersedePlan {
    pub source_id: String,
    pub source_path: PathBuf,
    pub source_content: String,
    pub target_id: String,
    pub target_path: PathBuf,
    pub target_content: String,
    /// `true` when the target transitioned to `superseded` (was `accepted`
    /// before). CLI callers emit an extra status line in that case.
    pub target_status_changed_to_superseded: bool,
}

pub fn plan_supersede_add(
    graph: &KnowledgeGraph,
    source_id: &str,
    target_id: &str,
) -> Result<SupersedePlan, ProductError> {
    let source = graph
        .adrs
        .get(source_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", source_id)))?;
    let target = graph
        .adrs
        .get(target_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", target_id)))?;

    let mut source_front = source.front.clone();
    if !source_front.supersedes.contains(&target_id.to_string()) {
        source_front.supersedes.push(target_id.to_string());
    }

    let mut target_front = target.front.clone();
    if !target_front.superseded_by.contains(&source_id.to_string()) {
        target_front.superseded_by.push(source_id.to_string());
    }

    let mut test_adrs: Vec<types::Adr> = graph.adrs.values().cloned().collect();
    test_adrs.retain(|a| a.front.id != source_id && a.front.id != target_id);
    test_adrs.push(types::Adr {
        front: source_front.clone(),
        body: source.body.clone(),
        path: source.path.clone(),
    });
    test_adrs.push(types::Adr {
        front: target_front.clone(),
        body: target.body.clone(),
        path: target.path.clone(),
    });
    let test_graph = KnowledgeGraph::build(vec![], test_adrs, vec![]);
    if let Some(cycle) = test_graph.detect_supersession_cycle() {
        return Err(ProductError::SupersessionCycle { cycle });
    }

    let target_status_changed_to_superseded = target_front.status == types::AdrStatus::Accepted;
    if target_status_changed_to_superseded {
        target_front.status = types::AdrStatus::Superseded;
    }

    let source_content = parser::render_adr(&source_front, &source.body);
    let target_content = parser::render_adr(&target_front, &target.body);

    Ok(SupersedePlan {
        source_id: source_id.to_string(),
        source_path: source.path.clone(),
        source_content,
        target_id: target_id.to_string(),
        target_path: target.path.clone(),
        target_content,
        target_status_changed_to_superseded,
    })
}

pub fn plan_supersede_remove(
    graph: &KnowledgeGraph,
    source_id: &str,
    target_id: &str,
) -> Result<SupersedePlan, ProductError> {
    let source = graph
        .adrs
        .get(source_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", source_id)))?;
    let target = graph
        .adrs
        .get(target_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", target_id)))?;

    let mut source_front = source.front.clone();
    source_front.supersedes.retain(|s| s != target_id);

    let mut target_front = target.front.clone();
    target_front.superseded_by.retain(|s| s != source_id);

    let source_content = parser::render_adr(&source_front, &source.body);
    let target_content = parser::render_adr(&target_front, &target.body);

    Ok(SupersedePlan {
        source_id: source_id.to_string(),
        source_path: source.path.clone(),
        source_content,
        target_id: target_id.to_string(),
        target_path: target.path.clone(),
        target_content,
        target_status_changed_to_superseded: false,
    })
}

pub fn apply_supersede(plan: &SupersedePlan) -> Result<(), ProductError> {
    let writes: Vec<(&std::path::Path, &str)> = vec![
        (&plan.source_path, plan.source_content.as_str()),
        (&plan.target_path, plan.target_content.as_str()),
    ];
    fileops::write_batch_atomic(&writes)?;
    Ok(())
}
