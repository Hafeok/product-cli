//! Feature `domains` list edits — add/remove entries with vocabulary validation.

use crate::config::ProductConfig;
use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DomainEditPlan {
    pub feature_id: String,
    pub feature_path: PathBuf,
    pub feature_content: String,
    pub final_domains: Vec<String>,
}

/// Pure: plan a domain edit.
///
/// Validates that every domain in `add` exists in the config's domain
/// vocabulary (E012). Applies adds before removes; output list is sorted.
pub fn plan_domain_edit(
    config: &ProductConfig,
    graph: &KnowledgeGraph,
    feature_id: &str,
    add: &[String],
    remove: &[String],
) -> Result<DomainEditPlan, ProductError> {
    let feature = graph
        .features
        .get(feature_id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", feature_id)))?;

    for domain in add {
        if !config.domains.contains_key(domain) {
            return Err(ProductError::ConfigError(format!(
                "error[E012]: unknown domain '{}'\n   = hint: check [domains] vocabulary in product.toml",
                domain
            )));
        }
    }

    let mut front = feature.front.clone();
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
    let feature_content = parser::render_feature(&front, &feature.body);

    Ok(DomainEditPlan {
        feature_id: feature_id.to_string(),
        feature_path: feature.path.clone(),
        feature_content,
        final_domains,
    })
}

/// I/O: persist the domain edit.
pub fn apply_domain_edit(plan: &DomainEditPlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.feature_path, &plan.feature_content)?;
    Ok(())
}
