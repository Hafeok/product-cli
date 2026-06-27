//! Archetype aggregate — a reusable pre-filled How for a system shape.
//!
//! An archetype assembles the three §4/§5 parts for one system shape: its How
//! contract (the Why cascade + the two contracts + interfaces), its repository
//! layout model, and the task types (cells) it dispatches. This module loads
//! the parts from a directory and validates the whole assembly — each part
//! against its own shapes, plus the cross-part coherence (cells belong to the
//! archetype; the How's layout reference resolves; cells apply real patterns).

use std::path::Path;

use crate::error::{ProductError, Result};

use super::archetype_turtle::archetype_to_turtle;
use super::cell::TaskType;
use super::cell_validate::validate_cell;
use super::how::HowContract;
use super::how_validate::validate_how;
use super::layout::{validate_layout, LayoutModel};
use super::model::DomainGraph;
use super::rules_how::archetype_rules;
use super::sparql_rules::run_rules;
use super::validate::Violation;

/// An assembled archetype.
#[derive(Debug, Clone, Default)]
pub struct Archetype {
    pub name: String,
    pub how: Option<HowContract>,
    pub layout: Option<LayoutModel>,
    /// (source name, task type) for each cell file.
    pub cells: Vec<(String, TaskType)>,
}

impl Archetype {
    /// Load an archetype from `<dir>`: `how-contract.yaml`, `layout.yaml`, and
    /// every `cells/*.yaml`.
    pub fn load_from_dir(dir: &Path, name: &str) -> Result<Self> {
        if !dir.is_dir() {
            return Err(ProductError::NotFound(format!(
                "no archetype directory at {} — scaffold one with `product archetype init {}`",
                dir.display(),
                name
            )));
        }
        let how = read_opt(&dir.join("how-contract.yaml"))?
            .map(|t| HowContract::from_yaml(&t))
            .transpose()?;
        let layout = read_opt(&dir.join("layout.yaml"))?
            .map(|t| LayoutModel::from_yaml(&t))
            .transpose()?;
        let cells = load_cells(&dir.join("cells"))?;
        Ok(Self { name: name.to_string(), how, layout, cells })
    }

    /// Scaffold a new archetype directory: a starter How contract, a layout
    /// model, and one example cell. Returns the written file paths. Shared by the
    /// CLI (`product archetype init`) and the MCP tool so the skeleton is laid
    /// down from one place and cannot drift between the two surfaces.
    pub fn scaffold(dir: &Path, name: &str) -> Result<Vec<String>> {
        let cells = dir.join("cells");
        std::fs::create_dir_all(&cells)
            .map_err(|e| ProductError::Internal(format!("create {}: {e}", cells.display())))?;
        let files = [
            (dir.join("how-contract.yaml"), HowContract::scaffold(name).to_yaml()?),
            (dir.join("layout.yaml"), LayoutModel::scaffold(name).to_yaml()?),
            (cells.join("example-task.yaml"), TaskType::scaffold("example-task", name).to_yaml()?),
        ];
        let mut written = Vec::new();
        for (path, text) in files {
            crate::fileops::write_file_atomic(&path, &text)?;
            written.push(path.display().to_string());
        }
        Ok(written)
    }

    /// Validate the whole archetype. `domain` enables cells' `domain:X`
    /// cross-checks against the captured What graph.
    pub fn validate(&self, domain: Option<&DomainGraph>) -> Vec<Violation> {
        let mut out = Vec::new();
        match &self.how {
            None => out.push(blocking("archetype", "how",
                "§4 An archetype must declare a How contract (how-contract.yaml).")),
            Some(how) => {
                tag("how", validate_how(how), &mut out);
                self.check_layout_reference(how, &mut out);
            }
        }
        if let Some(layout) = &self.layout {
            tag("layout", validate_layout(layout), &mut out);
            self.check_layout_archetype(layout, &mut out);
            self.check_enforces_resolve(&mut out);
        }
        for (src, cell) in &self.cells {
            tag(src, validate_cell(cell, domain, self.how.as_ref()), &mut out);
            self.check_cell_archetype(src, cell, &mut out);
        }
        out
    }

    fn check_layout_reference(&self, how: &HowContract, out: &mut Vec<Violation>) {
        if how.layout_model.is_some() && self.layout.is_none() {
            out.push(warning("how", "layout_model",
                "the How references a layout_model but no layout.yaml was found in the archetype."));
        }
    }

    /// §4.3 Guard 1 + §5 honesty: a layout rule's `enforces` must resolve to a
    /// principle or decision the How actually defines — else the rationale it
    /// claims is a dangling reference. Run as a SPARQL rule over the combined
    /// How+layout projection (`sparql_rules::ENFORCES_RESOLVES`) rather than a
    /// native field-walk, so the constraint lives in the graph.
    fn check_enforces_resolve(&self, out: &mut Vec<Violation>) {
        let Some(how) = &self.how else { return };
        let ttl = archetype_to_turtle(how, self.layout.as_ref());
        for mut v in run_rules(&ttl, archetype_rules()) {
            v.focus = format!("layout/{}", v.focus);
            out.push(v);
        }
    }

    fn check_layout_archetype(&self, layout: &LayoutModel, out: &mut Vec<Violation>) {
        if let Some(a) = &layout.archetype {
            if a != &self.name {
                out.push(warning("layout", "archetype",
                    &format!("layout declares archetype '{a}' but lives under '{}'.", self.name)));
            }
        }
    }

    fn check_cell_archetype(&self, src: &str, cell: &TaskType, out: &mut Vec<Violation>) {
        match &cell.archetype {
            Some(a) if a == &self.name => {}
            Some(a) => out.push(warning(src, "archetype",
                &format!("task type declares archetype '{a}' but lives under '{}'.", self.name))),
            None => out.push(warning(src, "archetype", "task type declares no archetype.")),
        }
    }
}

/// Read a file to a string, returning `None` if it does not exist.
fn read_opt(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ProductError::IoError(format!("{}: {}", path.display(), e))),
    }
}

/// Load every `*.yaml` task type under `cells/`, sorted by filename.
fn load_cells(dir: &Path) -> Result<Vec<(String, TaskType)>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return Ok(Vec::new()),
    };
    let mut paths: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    paths.sort();
    let mut cells = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| ProductError::IoError(format!("{}: {}", path.display(), e)))?;
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("cell").to_string();
        cells.push((name, TaskType::from_yaml(&text)?));
    }
    Ok(cells)
}

/// Re-tag a part's violations with a `part/focus` focus so the source is clear.
fn tag(part: &str, violations: Vec<Violation>, out: &mut Vec<Violation>) {
    for mut v in violations {
        v.focus = format!("{part}/{}", v.focus);
        out.push(v);
    }
}

fn blocking(focus: &str, path: &str, message: &str) -> Violation {
    sev(focus, path, message, "violation")
}
fn warning(focus: &str, path: &str, message: &str) -> Violation {
    sev(focus, path, message, "warning")
}
fn sev(focus: &str, path: &str, message: &str, severity: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: severity.to_string(),
    }
}

#[cfg(test)]
#[path = "archetype_tests.rs"]
mod tests;
