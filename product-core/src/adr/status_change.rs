//! ADR status transitions with optional supersession bidirectional cascade.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, hash, parser, types};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StatusChangePlan {
    pub adr_id: String,
    pub adr_path: PathBuf,
    pub adr_content: String,
    pub new_status: types::AdrStatus,
    /// Successor ADR that also needs its `supersedes` list updated, when
    /// the user passes `--by <successor>`.
    pub successor_update: Option<SuccessorUpdate>,
}

#[derive(Debug, Clone)]
pub struct SuccessorUpdate {
    pub adr_id: String,
    pub adr_path: PathBuf,
    pub adr_content: String,
}

pub fn plan_status_change(
    graph: &KnowledgeGraph,
    adr_id: &str,
    new_status: types::AdrStatus,
    by: Option<&str>,
) -> Result<StatusChangePlan, ProductError> {
    let adr = graph
        .adrs
        .get(adr_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;

    let mut front = adr.front.clone();
    front.status = new_status;

    if new_status == types::AdrStatus::Accepted {
        front.content_hash = Some(hash::compute_adr_hash(&front.title, &adr.body));
    }

    let successor_update = if let Some(by_id) = by {
        if !front.superseded_by.contains(&by_id.to_string()) {
            front.superseded_by.push(by_id.to_string());
        }
        graph.adrs.get(by_id).map(|succ| {
            let mut succ_front = succ.front.clone();
            if !succ_front.supersedes.contains(&adr_id.to_string()) {
                succ_front.supersedes.push(adr_id.to_string());
            }
            let succ_content = parser::render_adr(&succ_front, &succ.body);
            SuccessorUpdate {
                adr_id: by_id.to_string(),
                adr_path: succ.path.clone(),
                adr_content: succ_content,
            }
        })
    } else {
        None
    };

    let adr_content = parser::render_adr(&front, &adr.body);
    Ok(StatusChangePlan {
        adr_id: adr_id.to_string(),
        adr_path: adr.path.clone(),
        adr_content,
        new_status,
        successor_update,
    })
}

pub fn apply_status_change(plan: &StatusChangePlan) -> Result<(), ProductError> {
    let mut writes: Vec<(&std::path::Path, &str)> = Vec::with_capacity(2);
    writes.push((&plan.adr_path, plan.adr_content.as_str()));
    if let Some(ref succ) = plan.successor_update {
        writes.push((&succ.adr_path, succ.adr_content.as_str()));
    }
    fileops::write_batch_atomic(&writes)?;
    Ok(())
}
