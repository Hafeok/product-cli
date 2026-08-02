//! Repo-level `.ddd/config.yaml` — interception mode plus ignore globs.
//!
//! M1 only reads the file for validity; the `intercept` mode is consumed by
//! M4's edit interceptor (`enforce | warn | off`, PRD §8). Defaults to `warn`
//! per PRD §11's adoption mitigation.

use serde::{Deserialize, Serialize};

/// The `.ddd/config.yaml` contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DddConfig {
    /// Entry format version (v1 is current).
    pub format: u32,
    /// Interception mode: `enforce | warn | off`.
    #[serde(default = "default_intercept")]
    pub intercept: String,
    /// Globs the interceptor plus detection skip.
    #[serde(default)]
    pub ignore: Vec<String>,
}

fn default_intercept() -> String {
    "warn".to_string()
}

impl Default for DddConfig {
    fn default() -> Self {
        Self { format: 1, intercept: default_intercept(), ignore: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intercept_defaults_to_warn() {
        let c: DddConfig = serde_yaml::from_str("format: 1\n").expect("parse");
        assert_eq!(c.intercept, "warn");
        assert!(c.ignore.is_empty());
    }
}
