//! ADR sealing — amend records an audit entry; seal computes the first hash.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, hash, parser, types};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AmendPlan {
    pub adr_id: String,
    pub adr_path: PathBuf,
    pub adr_content: String,
    pub new_hash: String,
}

#[derive(Debug, Clone)]
pub struct SealPlan {
    pub adr_id: String,
    pub adr_path: PathBuf,
    pub adr_content: String,
    pub new_hash: String,
}

pub fn plan_amend(
    graph: &KnowledgeGraph,
    adr_id: &str,
    reason: &str,
) -> Result<AmendPlan, ProductError> {
    if reason.trim().is_empty() {
        return Err(ProductError::ConfigError(
            "amendment reason cannot be empty".to_string(),
        ));
    }
    let adr = graph
        .adrs
        .get(adr_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;

    let (new_hash, amendment) = hash::amend_adr(adr, reason)?;
    let mut front = adr.front.clone();
    front.content_hash = Some(new_hash.clone());
    front.amendments.push(amendment);
    let adr_content = parser::render_adr(&front, &adr.body);

    Ok(AmendPlan {
        adr_id: adr_id.to_string(),
        adr_path: adr.path.clone(),
        adr_content,
        new_hash,
    })
}

pub fn apply_amend(plan: &AmendPlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.adr_path, &plan.adr_content)?;
    Ok(())
}

pub fn plan_seal(graph: &KnowledgeGraph, adr_id: &str) -> Result<Option<SealPlan>, ProductError> {
    let adr = graph
        .adrs
        .get(adr_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;
    if adr.front.content_hash.is_some() {
        return Ok(None);
    }
    let new_hash = hash::seal_adr(adr)?;
    let mut front = adr.front.clone();
    front.content_hash = Some(new_hash.clone());
    let adr_content = parser::render_adr(&front, &adr.body);
    Ok(Some(SealPlan {
        adr_id: adr_id.to_string(),
        adr_path: adr.path.clone(),
        adr_content,
        new_hash,
    }))
}

pub fn apply_seal(plan: &SealPlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.adr_path, &plan.adr_content)?;
    Ok(())
}

/// Pure helper: yield every accepted ADR that has no `content_hash` yet.
/// Callers iterate, call `plan_seal` on each, and apply the resulting plans.
pub fn unsealed_accepted_ids(graph: &KnowledgeGraph) -> Vec<String> {
    let mut ids: Vec<String> = graph
        .adrs
        .values()
        .filter(|a| a.front.status == types::AdrStatus::Accepted && a.front.content_hash.is_none())
        .map(|a| a.front.id.clone())
        .collect();
    ids.sort();
    ids
}
