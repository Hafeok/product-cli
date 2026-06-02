//! Single-field ADR edits — domains, scope, source-files.

use crate::config::ProductConfig;
use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser, types};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DomainEditPlan {
    pub adr_id: String,
    pub adr_path: PathBuf,
    pub adr_content: String,
    pub final_domains: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScopeChangePlan {
    pub adr_id: String,
    pub adr_path: PathBuf,
    pub adr_content: String,
    pub new_scope: types::AdrScope,
}

#[derive(Debug, Clone)]
pub struct SourceFilesEditPlan {
    pub adr_id: String,
    pub adr_path: PathBuf,
    pub adr_content: String,
    pub final_source_files: Vec<String>,
    /// Paths in `add` that do not exist in the working tree (warnings, non-fatal).
    pub missing_added_paths: Vec<String>,
}

pub fn plan_domain_edit(
    config: &ProductConfig,
    graph: &KnowledgeGraph,
    adr_id: &str,
    add: &[String],
    remove: &[String],
) -> Result<DomainEditPlan, ProductError> {
    let adr = graph
        .adrs
        .get(adr_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;

    for domain in add {
        if !config.domains.contains_key(domain) {
            return Err(ProductError::ConfigError(format!(
                "error[E012]: unknown domain '{}'\n   = hint: check [domains] vocabulary in product.toml",
                domain
            )));
        }
    }

    let mut front = adr.front.clone();
    for domain in add {
        if !front.domains.contains(domain) {
            front.domains.push(domain.clone());
        }
    }
    for domain in remove {
        front.domains.retain(|d| d != domain);
    }
    front.domains.sort();

    let final_domains = front.domains.clone();
    let adr_content = parser::render_adr(&front, &adr.body);

    Ok(DomainEditPlan {
        adr_id: adr_id.to_string(),
        adr_path: adr.path.clone(),
        adr_content,
        final_domains,
    })
}

pub fn apply_domain_edit(plan: &DomainEditPlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.adr_path, &plan.adr_content)?;
    Ok(())
}

pub fn plan_scope_change(
    graph: &KnowledgeGraph,
    adr_id: &str,
    new_scope: types::AdrScope,
) -> Result<ScopeChangePlan, ProductError> {
    let adr = graph
        .adrs
        .get(adr_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;
    let mut front = adr.front.clone();
    front.scope = new_scope;
    let adr_content = parser::render_adr(&front, &adr.body);
    Ok(ScopeChangePlan {
        adr_id: adr_id.to_string(),
        adr_path: adr.path.clone(),
        adr_content,
        new_scope,
    })
}

pub fn apply_scope_change(plan: &ScopeChangePlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.adr_path, &plan.adr_content)?;
    Ok(())
}

pub fn plan_source_files_edit(
    graph: &KnowledgeGraph,
    repo_root: &Path,
    adr_id: &str,
    add: &[String],
    remove: &[String],
) -> Result<SourceFilesEditPlan, ProductError> {
    let adr = graph
        .adrs
        .get(adr_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;

    let missing_added_paths: Vec<String> = add
        .iter()
        .filter(|p| !repo_root.join(p).exists())
        .cloned()
        .collect();

    let mut front = adr.front.clone();
    for path_str in add {
        if !front.source_files.contains(path_str) {
            front.source_files.push(path_str.clone());
        }
    }
    for path_str in remove {
        front.source_files.retain(|s| s != path_str);
    }
    front.source_files.sort();

    let final_source_files = front.source_files.clone();
    let adr_content = parser::render_adr(&front, &adr.body);

    Ok(SourceFilesEditPlan {
        adr_id: adr_id.to_string(),
        adr_path: adr.path.clone(),
        adr_content,
        final_source_files,
        missing_added_paths,
    })
}

pub fn apply_source_files_edit(plan: &SourceFilesEditPlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.adr_path, &plan.adr_content)?;
    Ok(())
}
