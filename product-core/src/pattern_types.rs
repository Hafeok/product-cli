//! Pattern artifact types (FT-070, ADR-050).
//!
//! `PatternFrontMatter`, `PatternStatus`, and `Pattern` mirror the shape of
//! the existing feature/ADR/TC types. Patterns are reusable implementation
//! knowledge — peer to FT/ADR/TC/DEP in the graph.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// YAML front-matter on a pattern markdown file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFrontMatter {
    pub id: String,
    pub title: String,
    #[serde(default = "default_pattern_status")]
    pub status: PatternStatus,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(
        rename = "deprecated-by",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub deprecated_by: Option<String>,
}

fn default_pattern_status() -> PatternStatus {
    PatternStatus::Live
}

/// Lifecycle state of a pattern (ADR-050). Patterns evolve by accretion —
/// the only transitions are `live ↔ deprecated`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PatternStatus {
    Live,
    Deprecated,
}

impl std::fmt::Display for PatternStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Live => write!(f, "live"),
            Self::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl std::str::FromStr for PatternStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "live" => Ok(Self::Live),
            "deprecated" => Ok(Self::Deprecated),
            _ => Err(format!(
                "unknown pattern status: '{}'. Valid values: live, deprecated",
                s
            )),
        }
    }
}

/// Loaded pattern — front-matter, body, and file path.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub front: PatternFrontMatter,
    pub body: String,
    pub path: PathBuf,
}
