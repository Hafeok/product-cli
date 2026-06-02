//! `[author]` section — selects which agent CLI hosts authoring sessions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorConfig {
    /// Agent CLI to launch: `claude` (default) or `copilot`.
    #[serde(default = "default_author_cli")]
    pub cli: String,
}

impl Default for AuthorConfig {
    fn default() -> Self {
        Self { cli: default_author_cli() }
    }
}

fn default_author_cli() -> String { "claude".into() }
