//! Feature field-edit adapters — `acknowledge`, `domain`, `depends-on`.
//!
//! Extracted from `feature_write.rs` to keep that file under the 400-line
//! fitness cap. Each adapter is a thin wrapper over the corresponding
//! slice function in `product_lib::feature` (or `product_lib::domains`
//! for acknowledgement).

use product_lib::{domains, error::ProductError, feature as feat, fileops, parser};

use super::{acquire_write_lock, acquire_write_lock_typed, load_graph, load_graph_typed, BoxResult, CmdResult, Output};

pub(crate) fn feature_acknowledge(
    id: &str,
    domain: Option<String>,
    adr: Option<String>,
    reason: Option<String>,
    remove: bool,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;
    let feature = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

    if remove {
        let key = if let Some(ref d) = domain {
            d.clone()
        } else if let Some(ref a) = adr {
            a.clone()
        } else {
            return Err(Box::new(ProductError::ConfigError(
                "must specify --domain or --adr with --remove".to_string(),
            )));
        };
        let mut front = feature.front.clone();
        front.domains_acknowledged.remove(&key);
        let content = parser::render_feature(&front, &feature.body);
        fileops::write_file_atomic(&feature.path, &content)?;
        println!("{} removed acknowledgement for '{}'", id, key);
        return Ok(());
    }

    let reason_str = reason.unwrap_or_default();
    if reason_str.trim().is_empty() {
        return Err(Box::new(ProductError::ConfigError(
            "error[E011]: acknowledgement requires non-empty --reason".to_string(),
        )));
    }

    let updated_front = if let Some(ref domain_name) = domain {
        domains::acknowledge_domain(feature, domain_name, &reason_str)?
    } else if let Some(ref adr_id) = adr {
        let adr_obj = graph
            .adrs
            .get(adr_id.as_str())
            .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;
        domains::acknowledge_adr(feature, adr_obj, &reason_str)?
    } else {
        return Err(Box::new(ProductError::ConfigError(
            "must specify --domain or --adr".to_string(),
        )));
    };

    let content = parser::render_feature(&updated_front, &feature.body);
    fileops::write_file_atomic(&feature.path, &content)?;
    if let Some(ref d) = domain {
        println!("{} acknowledged domain '{}': {}", id, d, reason_str);
    } else if let Some(ref a) = adr {
        println!("{} acknowledged ADR '{}': {}", id, a, reason_str);
    }
    Ok(())
}

pub(crate) fn feature_domain(id: &str, add: Vec<String>, remove: Vec<String>) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (config, _, graph) = load_graph_typed()?;
    let plan = feat::plan_domain_edit(&config, &graph, id, &add, &remove)?;
    feat::apply_domain_edit(&plan)?;
    Ok(Output::text(format!(
        "{} domains: [{}]",
        id,
        plan.final_domains.join(", ")
    )))
}

/// FT-062 — `product feature depends-on` adapter. Mirrors the field-edit
/// pattern: thin wrapper over `plan_depends_on_edit` + `apply_depends_on_edit`.
pub(crate) fn feature_depends_on(id: &str, add: Vec<String>, remove: Vec<String>) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (_config, _, graph) = load_graph_typed()?;
    let plan = feat::plan_depends_on_edit(&graph, id, &add, &remove)?;
    if plan.is_changed() {
        feat::apply_depends_on_edit(&plan)?;
    }
    let json = serde_json::json!({
        "id": id,
        "depends_on": plan.final_depends_on,
        "added": plan.added,
        "removed": plan.removed,
        "changed": plan.is_changed(),
    });
    let mut text = format!(
        "{} depends-on: [{}]",
        id,
        plan.final_depends_on.join(", ")
    );
    if !plan.added.is_empty() {
        text.push_str(&format!("\n  added: {}", plan.added.join(", ")));
    }
    if !plan.removed.is_empty() {
        text.push_str(&format!("\n  removed: {}", plan.removed.join(", ")));
    }
    if !plan.is_changed() {
        text.push_str("\n  (no changes)");
    }
    Ok(Output::both(text, json))
}
