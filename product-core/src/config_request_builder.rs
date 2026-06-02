//! `[request-builder]` section (FT-052, ADR-044).

use serde::{Deserialize, Serialize};

/// Interactive request builder configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBuilderConfig {
    /// Prompt for missing fields when stdin is a TTY.
    #[serde(default = "default_true")]
    pub interactive: bool,
    /// How to handle W-class findings at submit time: always | warn | block.
    #[serde(rename = "warn-on-warnings", default = "default_warn_policy")]
    pub warn_on_warnings: String,
    /// Optional `$EDITOR` override used by `product request edit`.
    #[serde(default)]
    pub editor: Option<String>,
}

impl Default for RequestBuilderConfig {
    fn default() -> Self {
        Self {
            interactive: true,
            warn_on_warnings: default_warn_policy(),
            editor: None,
        }
    }
}

fn default_true() -> bool { true }
fn default_warn_policy() -> String { "warn".into() }
