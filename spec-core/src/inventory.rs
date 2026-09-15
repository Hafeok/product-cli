//! The importer's projection of a codebase, read back.
//!
//! A projection, never authority: it is rebuilt wholesale on every `import`
//! and carries no verdict, no ratification, no decision. Nothing in this crate
//! writes it — the scanner runs in the agent host, and the gate only reads.

use std::path::{Path, PathBuf};

use product_core::error::{ProductError, Result};
use serde::{Deserialize, Serialize};

/// Where the inventory lands, relative to a repo root.
pub const INVENTORY_PATH: &str = ".spec/inventory.json";

/// A declaration the scan found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub namespace: String,
    pub file: String,
    pub line: u32,
}

/// One type reaching another.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionEdge {
    pub from: String,
    pub to: String,
    pub via: String,
}

/// A place the outside world gets in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryPoint {
    pub id: String,
    pub kind: String,
    pub symbol: String,
    /// The transport-shaped address. Observed, never naming an act.
    pub transport: String,
    pub file: String,
    pub line: u32,
}

/// A claim code makes about the specification.
///
/// `slice` says this code is part of a slice built against the spec;
/// `realises-fact` says it realises a determination someone filed. Both are
/// references, and a reference to something that does not exist is broken.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecClaim {
    pub kind: String,
    pub value: String,
    pub symbol: String,
    pub file: String,
    pub line: u32,
}

/// An entry point with the slots ratification must fill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub entry_point: String,
    #[serde(default)]
    pub observed: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub unfilled_slots: Vec<String>,
}

/// What one `import` run observed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Inventory {
    pub form: String,
    pub root: String,
    pub revision: String,
    pub scanned_at: String,
    #[serde(default)]
    pub symbols: Vec<SymbolEntry>,
    #[serde(default)]
    pub composition_edges: Vec<CompositionEdge>,
    #[serde(default)]
    pub entry_points: Vec<EntryPoint>,
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    #[serde(default)]
    pub claims: Vec<SpecClaim>,
}

impl Inventory {
    /// The inventory path under a repo root.
    pub fn path(repo_root: &Path) -> PathBuf {
        repo_root.join(INVENTORY_PATH)
    }

    /// Read the inventory, or `None` when no `import` has run.
    ///
    /// Absence is not an error. A repo that has never imported is a legitimate
    /// state — greenfield is this flow with an empty import — and a gate that
    /// refused to run without one would make the empty case a special path.
    pub fn load_opt(repo_root: &Path) -> Result<Option<Self>> {
        let path = Self::path(repo_root);
        if !path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| ProductError::IoError(format!("{}: {e}", path.display())))?;
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| ProductError::ParseError {
                file: path,
                line: Some(e.line()),
                message: e.to_string(),
            })
    }

    /// Look one entry point up by id.
    pub fn entry_point(&self, id: &str) -> Option<&EntryPoint> {
        self.entry_points.iter().find(|e| e.id == id)
    }

    /// Every claim of one kind.
    pub fn claims_of(&self, kind: &str) -> Vec<&SpecClaim> {
        self.claims.iter().filter(|c| c.kind == kind).collect()
    }

    /// The symbols a given file declares, for showing an act its realisation.
    pub fn symbols_in(&self, file: &str) -> Vec<&SymbolEntry> {
        self.symbols.iter().filter(|s| s.file == file).collect()
    }
}

#[path = "inventory_tests.rs"]
#[cfg(test)]
pub(crate) mod tests;
