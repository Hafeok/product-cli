//! Thin adapters for ADR field edits — domain, scope, supersede, source-files.

use product_lib::{adr, error::ProductError, types};

use super::{acquire_write_lock_typed, load_graph_typed, CmdResult, Output};

pub(crate) fn adr_domain(id: &str, add: Vec<String>, remove: Vec<String>) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (config, _, graph) = load_graph_typed()?;
    let plan = adr::plan_domain_edit(&config, &graph, id, &add, &remove)?;
    adr::apply_domain_edit(&plan)?;
    Ok(Output::text(format!(
        "{} domains: [{}]",
        id,
        plan.final_domains.join(", ")
    )))
}

pub(crate) fn adr_scope(id: &str, scope_str: &str) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (_, _, graph) = load_graph_typed()?;
    let scope: types::AdrScope = scope_str
        .parse()
        .map_err(|e: String| ProductError::ConfigError(format!("error[E001]: {}", e)))?;
    let plan = adr::plan_scope_change(&graph, id, scope)?;
    adr::apply_scope_change(&plan)?;
    Ok(Output::text(format!("{} scope -> {}", id, plan.new_scope)))
}

pub(crate) fn adr_supersede(
    id: &str,
    supersedes: Option<String>,
    remove: Option<String>,
) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (_, _, graph) = load_graph_typed()?;

    if let Some(target_id) = supersedes {
        let plan = adr::plan_supersede_add(&graph, id, &target_id)?;
        adr::apply_supersede(&plan)?;
        let mut lines = vec![format!("{} supersedes {}", id, target_id)];
        if plan.target_status_changed_to_superseded {
            lines.push(format!("{} status -> superseded", target_id));
        }
        Ok(Output::text(lines.join("\n")))
    } else if let Some(target_id) = remove {
        let plan = adr::plan_supersede_remove(&graph, id, &target_id)?;
        adr::apply_supersede(&plan)?;
        Ok(Output::text(format!(
            "{} removed supersession link to {}",
            id, target_id
        )))
    } else {
        Err(ProductError::ConfigError(
            "must specify --supersedes or --remove".to_string(),
        ))
    }
}

pub(crate) fn adr_source_files(id: &str, add: Vec<String>, remove: Vec<String>) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (_, root, graph) = load_graph_typed()?;
    let plan = adr::plan_source_files_edit(&graph, &root, id, &add, &remove)?;
    // Warnings are printed to stderr before the final rendered Output so that
    // the dispatcher's stdout write remains the canonical result.
    for missing in &plan.missing_added_paths {
        eprintln!(
            "warning[W012]: path '{}' does not exist (yet) in repository",
            missing
        );
    }
    adr::apply_source_files_edit(&plan)?;
    Ok(Output::text(format!(
        "{} source-files: [{}]",
        id,
        plan.final_source_files.join(", ")
    )))
}
