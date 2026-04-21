//! Draft lifecycle — load, save, archive (FT-052, ADR-044).
//!
//! The draft is a YAML file at `.product/requests/draft.yaml`. Its existence
//! is the lock; there is no sidecar state.

use serde_yaml::{Mapping, Value};
use std::path::{Path, PathBuf};

/// Draft kind — matches the `type:` field in the request YAML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftKind {
    Create,
    Change,
}

impl DraftKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "create" => Some(Self::Create),
            "change" => Some(Self::Change),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Change => "change",
        }
    }
}

/// The canonical draft file name under `.product/requests/`.
pub const DRAFT_FILE: &str = "draft.yaml";

/// Relative path to the active draft, rooted at the repo root.
pub fn draft_path(root: &Path) -> PathBuf {
    root.join(".product/requests").join(DRAFT_FILE)
}

/// Archive directory for submitted drafts.
pub fn archive_dir(root: &Path) -> PathBuf {
    root.join(".product/requests/archive")
}

/// In-memory representation of a draft — the YAML `Mapping` with convenience
/// accessors. The mapping is the source of truth; anything we need to show
/// the user is derived from it.
pub struct Draft {
    pub path: PathBuf,
    pub doc: Mapping,
}

impl Draft {
    /// Create a new empty draft in memory at the canonical path with the
    /// given kind. Does not write anything to disk.
    pub fn new(root: &Path, kind: DraftKind) -> Self {
        let mut doc = Mapping::new();
        doc.insert(Value::String("type".into()), Value::String(kind.as_str().into()));
        doc.insert(Value::String("schema-version".into()), Value::Number(1.into()));
        doc.insert(Value::String("reason".into()), Value::String(String::new()));
        match kind {
            DraftKind::Create => {
                doc.insert(Value::String("artifacts".into()), Value::Sequence(Vec::new()));
            }
            DraftKind::Change => {
                doc.insert(Value::String("changes".into()), Value::Sequence(Vec::new()));
            }
        }
        Self { path: draft_path(root), doc }
    }

    /// Read a draft from disk. Returns `None` if the draft file does not
    /// exist; returns `Some(Err)` if it exists but is malformed.
    pub fn load(root: &Path) -> Option<std::io::Result<Self>> {
        let path = draft_path(root);
        if !path.exists() {
            return None;
        }
        Some(Self::load_at(path))
    }

    fn load_at(path: PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let v: Value = serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        let doc = match v {
            Value::Mapping(m) => m,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "draft document must be a YAML mapping",
                ));
            }
        };
        Ok(Self { path, doc })
    }

    /// Does an active draft exist at the canonical path?
    pub fn exists(root: &Path) -> bool {
        draft_path(root).exists()
    }

    /// Canonical draft path as a string, for rendering.
    pub fn draft_path_str(root: &Path) -> String {
        draft_path(root).display().to_string()
    }

    /// Declared request kind from the draft's `type:` field.
    pub fn kind(&self) -> Option<DraftKind> {
        self.doc
            .get(Value::String("type".into()))
            .and_then(|v| v.as_str())
            .and_then(DraftKind::parse)
    }

    /// The `reason:` field (may be empty).
    pub fn reason(&self) -> &str {
        self.doc
            .get(Value::String("reason".into()))
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Set the `reason:` field.
    pub fn set_reason(&mut self, reason: &str) {
        self.doc.insert(
            Value::String("reason".into()),
            Value::String(reason.to_string()),
        );
    }

    /// Borrow the `artifacts:` sequence, creating it if missing.
    pub fn artifacts_mut(&mut self) -> &mut Vec<Value> {
        let entry = self
            .doc
            .entry(Value::String("artifacts".into()))
            .or_insert(Value::Sequence(Vec::new()));
        if !matches!(entry, Value::Sequence(_)) {
            *entry = Value::Sequence(Vec::new());
        }
        match entry {
            Value::Sequence(s) => s,
            _ => unreachable!("ensured sequence above"),
        }
    }

    /// Borrow the `changes:` sequence, creating it if missing.
    pub fn changes_mut(&mut self) -> &mut Vec<Value> {
        let entry = self
            .doc
            .entry(Value::String("changes".into()))
            .or_insert(Value::Sequence(Vec::new()));
        if !matches!(entry, Value::Sequence(_)) {
            *entry = Value::Sequence(Vec::new());
        }
        match entry {
            Value::Sequence(s) => s,
            _ => unreachable!("ensured sequence above"),
        }
    }

    /// Read-only view of the artifacts sequence.
    pub fn artifacts(&self) -> &[Value] {
        match self.doc.get(Value::String("artifacts".into())) {
            Some(Value::Sequence(s)) => s.as_slice(),
            _ => &[],
        }
    }

    /// Read-only view of the changes sequence.
    pub fn changes(&self) -> &[Value] {
        match self.doc.get(Value::String("changes".into())) {
            Some(Value::Sequence(s)) => s.as_slice(),
            _ => &[],
        }
    }

    /// Count of artifacts by type (feature, adr, tc, dep).
    pub fn type_counts(&self) -> Vec<(String, usize)> {
        let mut counts: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for a in self.artifacts() {
            if let Value::Mapping(m) = a {
                if let Some(Value::String(t)) = m.get(Value::String("type".into())) {
                    *counts.entry(t.clone()).or_insert(0) += 1;
                }
            }
        }
        counts.into_iter().collect()
    }

    /// Serialise the draft to a YAML string using `serde_yaml`'s emit format.
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(&Value::Mapping(self.doc.clone())).unwrap_or_default()
    }

    /// Atomically write the draft to disk at its canonical path.
    pub fn save(&self) -> crate::error::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::error::ProductError::WriteError {
                    path: self.path.clone(),
                    message: format!("failed to create drafts dir: {e}"),
                }
            })?;
        }
        crate::fileops::write_file_atomic(&self.path, &self.to_yaml())
    }

    /// Remove the draft file from disk (discard).
    pub fn delete(root: &Path) -> std::io::Result<bool> {
        let p = draft_path(root);
        if p.exists() {
            std::fs::remove_file(&p)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Move the draft to the archive directory with a timestamp prefix.
    pub fn archive(root: &Path) -> std::io::Result<Option<PathBuf>> {
        let src = draft_path(root);
        if !src.exists() {
            return Ok(None);
        }
        let dir = archive_dir(root);
        std::fs::create_dir_all(&dir)?;
        let ts = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S");
        let dest = dir.join(format!("{ts}-draft.yaml"));
        std::fs::rename(&src, &dest)?;
        Ok(Some(dest))
    }
}
