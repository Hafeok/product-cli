//! MCP read handlers for the framework families that read `.product/` artifacts:
//! `archetype`, `cell`, `how`, `work-unit` — CLI↔MCP parity.

use std::path::{Path, PathBuf};

use product_core::pf::archetype::Archetype;
use product_core::pf::cell::TaskType;
use product_core::pf::cell_validate::validate_cell;
use product_core::pf::how::HowContract;
use product_core::pf::how_turtle::how_to_turtle;
use product_core::pf::how_validate::validate_how;
use product_core::pf::layout_check::check_layout;
use product_core::pf::work_unit::WorkUnit;
use product_core::pf::work_unit_validate::validate_work_unit;
use serde_json::{json, Value};

use crate::pf_mcp::{graph_of, load_yaml, pdir, req_str};

fn verdict(violations: &[product_core::pf::validate::Violation]) -> Value {
    json!({ "ok": !violations.iter().any(|v| v.severity == "violation"), "violations": violations })
}

// --- archetype -------------------------------------------------------------

fn archetypes_dir(r: &Path) -> PathBuf {
    pdir(r).join("archetypes")
}

pub fn handle_archetype_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    let mut names: Vec<String> = match std::fs::read_dir(archetypes_dir(repo_root)) {
        Ok(it) => it.flatten().filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok()).collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    Ok(json!({ "archetypes": names }))
}

fn load_archetype(repo_root: &Path, name: &str) -> Result<Archetype, String> {
    Archetype::load_from_dir(&archetypes_dir(repo_root).join(name), name).map_err(|e| format!("{e}"))
}

pub fn handle_archetype_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let name = req_str(args, "name")?;
    let a = load_archetype(repo_root, &name)?;
    Ok(json!({
        "name": name,
        "how": a.how.is_some(),
        "layout_rules": a.layout.as_ref().map(|l| l.layout.len()).unwrap_or(0),
        "cells": a.cells.iter().map(|(src, _)| src.clone()).collect::<Vec<_>>(),
    }))
}

pub fn handle_archetype_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let a = load_archetype(repo_root, &req_str(args, "name")?)?;
    let domain = graph_of(args, repo_root).ok();
    Ok(verdict(&a.validate(domain.as_ref())))
}

pub fn handle_archetype_check(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let a = load_archetype(repo_root, &req_str(args, "name")?)?;
    let Some(layout) = &a.layout else {
        return Ok(json!({ "ok": true, "violations": [], "note": "no layout model to check" }));
    };
    Ok(verdict(&check_layout(layout, repo_root)))
}

// --- cell ------------------------------------------------------------------

fn load_cell(repo_root: &Path) -> Result<TaskType, String> {
    let path = pdir(repo_root).join("cell.yaml");
    let text = std::fs::read_to_string(&path).map_err(|_| format!("no cell at {}", path.display()))?;
    TaskType::from_yaml(&text).map_err(|e| format!("{e}"))
}

pub fn handle_cell_show(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    let c = load_cell(repo_root)?;
    Ok(json!({ "name": c.name, "slots": c.slots.len(), "cells": c.cells.len() }))
}

pub fn handle_cell_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let c = load_cell(repo_root)?;
    let domain = graph_of(args, repo_root).ok();
    let how = load_yaml(&pdir(repo_root), "how-contract", HowContract::from_yaml).ok();
    Ok(verdict(&validate_cell(&c, domain.as_ref(), how.as_ref())))
}

// --- how -------------------------------------------------------------------

fn load_how(repo_root: &Path) -> Result<HowContract, String> {
    load_yaml(&pdir(repo_root), "how-contract", HowContract::from_yaml)
}

pub fn handle_how_show(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    let h = load_how(repo_root)?;
    Ok(json!({
        "application_contract": h.application_contract.id,
        "decisions": h.top_decisions.len(),
        "principles": h.principles.len(),
        "patterns": h.patterns.len(),
        "interfaces": h.interface_contracts.len(),
    }))
}

pub fn handle_how_validate(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(verdict(&validate_how(&load_how(repo_root)?)))
}

pub fn handle_how_export(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "turtle": how_to_turtle(&load_how(repo_root)?) }))
}

// --- work-unit -------------------------------------------------------------

fn load_work_unit(repo_root: &Path) -> Result<WorkUnit, String> {
    load_yaml(&pdir(repo_root), "work-unit", WorkUnit::from_yaml)
}

pub fn handle_work_unit_show(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    let w = load_work_unit(repo_root)?;
    Ok(json!({ "id": w.id, "produces": w.produces.artifact, "applies": w.applies }))
}

pub fn handle_work_unit_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let w = load_work_unit(repo_root)?;
    let domain = graph_of(args, repo_root).ok();
    let how = load_how(repo_root).ok();
    Ok(verdict(&validate_work_unit(&w, domain.as_ref(), how.as_ref())))
}

#[cfg(test)]
#[path = "framework_read_handlers_tests.rs"]
mod tests;
