//! Granular mutation of a How contract — build the Why cascade plus contracts.
//!
//! Backs `product how add/set`: append a top decision, principle, pattern, or
//! interface; set the application/infrastructure contract; add a contract
//! statement or an infrastructure resource. Ids in the Why cascade are unique
//! across decisions, principles, patterns, and interfaces (they reference each
//! other by id); nested ids are unique within their contract.

use crate::error::{ProductError, Result};

use super::how::*;

fn dup(id: &str, what: &str) -> ProductError {
    ProductError::ConfigError(format!("id {:?} already exists ({})", id, what))
}

/// True if `id` already names a Why-cascade element (decision/principle/pattern/interface).
fn id_taken(c: &HowContract, id: &str) -> bool {
    c.top_decisions.iter().any(|d| d.id == id)
        || c.principles.iter().any(|p| p.id == id)
        || c.patterns.iter().any(|p| p.id == id)
        || c.interface_contracts.iter().any(|i| i.id == id)
}

pub fn add_decision(c: &mut HowContract, d: TopDecision) -> Result<()> {
    if id_taken(c, &d.id) {
        return Err(dup(&d.id, "How element"));
    }
    c.top_decisions.push(d);
    Ok(())
}

pub fn add_principle(c: &mut HowContract, p: Principle) -> Result<()> {
    if id_taken(c, &p.id) {
        return Err(dup(&p.id, "How element"));
    }
    c.principles.push(p);
    Ok(())
}

pub fn add_pattern(c: &mut HowContract, p: Pattern) -> Result<()> {
    if id_taken(c, &p.id) {
        return Err(dup(&p.id, "How element"));
    }
    c.patterns.push(p);
    Ok(())
}

pub fn add_interface(c: &mut HowContract, i: InterfaceContract) -> Result<()> {
    if id_taken(c, &i.id) {
        return Err(dup(&i.id, "How element"));
    }
    c.interface_contracts.push(i);
    Ok(())
}

/// Set the application contract (the singleton §4.2 part). Existing statements
/// are preserved unless the new contract brings its own, so the metadata can be
/// (re)set without losing statements added separately.
pub fn set_app_contract(c: &mut HowContract, mut a: ApplicationContract) {
    if a.statements.is_empty() {
        a.statements = std::mem::take(&mut c.application_contract.statements);
    }
    c.application_contract = a;
}

pub fn add_app_statement(c: &mut HowContract, s: ContractStatement) -> Result<()> {
    if c.application_contract.id.trim().is_empty() {
        return Err(ProductError::ConfigError(
            "set the application contract first (`product how set app-contract …`)".to_string(),
        ));
    }
    if c.application_contract.statements.iter().any(|x| x.id == s.id) {
        return Err(dup(&s.id, "application-contract statement"));
    }
    c.application_contract.statements.push(s);
    Ok(())
}

/// Set the infrastructure contract, preserving already-added resources unless
/// the new contract brings its own.
pub fn set_infra_contract(c: &mut HowContract, mut i: InfrastructureContract) {
    if i.resources.is_empty() {
        if let Some(old) = c.infrastructure_contract.as_mut() {
            i.resources = std::mem::take(&mut old.resources);
        }
    }
    c.infrastructure_contract = Some(i);
}

pub fn add_resource(c: &mut HowContract, r: Resource) -> Result<()> {
    let infra = c.infrastructure_contract.as_mut().ok_or_else(|| {
        ProductError::ConfigError(
            "set the infrastructure contract first (`product how set infra-contract …`)".to_string(),
        )
    })?;
    if infra.resources.iter().any(|x| x.id == r.id) {
        return Err(dup(&r.id, "infrastructure resource"));
    }
    infra.resources.push(r);
    Ok(())
}

#[cfg(test)]
#[path = "how_edit_tests.rs"]
mod tests;
