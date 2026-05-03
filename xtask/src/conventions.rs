//! Convention doc frontmatter parser.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub tier: u8,
    pub mechanism: String,
    #[serde(default)]
    pub adrs: Vec<String>,
    #[serde(default)]
    pub applies_to: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Read and parse the YAML frontmatter from a convention doc.
pub fn read(path: &Path) -> Result<Frontmatter, String> {
    let body = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let inner = body
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n").map(|(fm, _)| fm))
        .ok_or_else(|| format!("{}: missing YAML frontmatter delimited by ---", path.display()))?;
    serde_yaml::from_str(inner).map_err(|e| format!("{}: {e}", path.display()))
}

/// Path to `conventions/docs/<id>.md` under the workspace root.
pub fn doc_path(root: &Path, id: &str) -> PathBuf {
    root.join("conventions").join("docs").join(format!("{id}.md"))
}

/// Path to an ADR file under `conventions/adr/`. Matches by `ADR-####` prefix
/// since ADR file names include a slug after the id.
pub fn adr_exists(root: &Path, adr_id: &str) -> bool {
    let dir = root.join("conventions").join("adr");
    let Ok(entries) = fs::read_dir(&dir) else {
        return false;
    };
    let prefix = format!("{adr_id}-");
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) || name == format!("{adr_id}.md") {
            return true;
        }
    }
    false
}
