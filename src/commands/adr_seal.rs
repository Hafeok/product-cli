//! Thin adapters for ADR sealing — amend, rehash.

use product_lib::{adr, error::ProductError};

use super::{acquire_write_lock_typed, load_graph_typed, CmdResult, Output};

pub fn adr_amend(id: &str, reason: Option<String>) -> CmdResult {
    let reason = reason.ok_or_else(|| {
        ProductError::ConfigError("--reason is required for amendments".to_string())
    })?;
    let _lock = acquire_write_lock_typed()?;
    let (_, _, graph) = load_graph_typed()?;
    let plan = adr::plan_amend(&graph, id, &reason)?;
    adr::apply_amend(&plan)?;
    Ok(Output::text(format!(
        "{} amended: content-hash updated to {}",
        id, plan.new_hash
    )))
}

pub fn adr_rehash(id: Option<String>, all: bool) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (_, _, graph) = load_graph_typed()?;
    if all {
        let ids = adr::unsealed_accepted_ids(&graph);
        let total_already_sealed = graph
            .adrs
            .values()
            .filter(|a| {
                a.front.status == product_lib::types::AdrStatus::Accepted
                    && a.front.content_hash.is_some()
            })
            .count();
        let mut lines: Vec<String> = Vec::new();
        let mut sealed = 0;
        for adr_id in &ids {
            if let Some(plan) = adr::plan_seal(&graph, adr_id)? {
                adr::apply_seal(&plan)?;
                lines.push(format!("  sealed {} -> {}", plan.adr_id, plan.new_hash));
                sealed += 1;
            }
        }
        lines.push(format!(
            "{} ADR(s) sealed, {} already sealed",
            sealed, total_already_sealed
        ));
        Ok(Output::text(lines.join("\n")))
    } else {
        let adr_id = id.ok_or_else(|| {
            ProductError::ConfigError("specify an ADR ID or use --all".to_string())
        })?;
        match adr::plan_seal(&graph, &adr_id)? {
            None => Ok(Output::text(format!("{} is already sealed", adr_id))),
            Some(plan) => {
                adr::apply_seal(&plan)?;
                Ok(Output::text(format!(
                    "{} sealed: content-hash = {}",
                    adr_id, plan.new_hash
                )))
            }
        }
    }
}
