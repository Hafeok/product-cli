//! §12.2 PREVIEW — the content-store manifest profile (FT-142).
//!
//! Reads the TOML manifest a content store publishes to plug in as the words
//! provider, validates its internal wholeness, and confirms it couples to a
//! captured What graph (resolves every (content key, locale) the application's
//! UI steps reference). The store is to words what a design system is to
//! components; locale is its context dimension. Non-normative (ADR-085).

use super::model::DomainGraph;
use std::collections::BTreeMap;

/// One content entry: a stable key + role + a resolved string per locale.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Entry {
    pub key: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub values: BTreeMap<String, String>,
}

/// The body of the §12.2 manifest (everything under the `content_store:` key).
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct ContentStore {
    pub id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub locales_supported: Vec<String>,
    #[serde(default)]
    pub entries: Vec<Entry>,
}

/// The §12.2 content-store manifest (the canonical YAML shape).
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct ContentManifest {
    pub content_store: ContentStore,
}

/// Roles whose text must be non-empty and actionable.
const ACTIONABLE_ROLES: [&str; 2] = ["error-message", "empty-message"];

/// Parse a canonical YAML content-store manifest, pointing the user at the
/// expected shape on a schema mismatch.
pub fn parse_content(yaml_src: &str) -> Result<ContentManifest, String> {
    serde_yaml::from_str(yaml_src).map_err(|e| {
        format!("manifest does not match the §12.2 content-store schema: {e}\n\
                 expected: content_store: id; locales_supported: [..]; \
                 entries: [{{key, role, values: {{<locale>: \"…\"}}}}]")
    })
}

/// Internal wholeness (§12.2): every entry carries a role, every claimed locale
/// has a value for every key, and every actionable role resolves to non-empty text.
pub fn validate_content(m: &ContentManifest) -> Vec<String> {
    let mut findings = Vec::new();
    for e in &m.content_store.entries {
        if e.role.trim().is_empty() {
            findings.push(format!("entry '{}' has no role", e.key));
        }
        for loc in &m.content_store.locales_supported {
            match e.values.get(loc) {
                None => findings.push(format!("entry '{}' has no value for locale '{loc}'", e.key)),
                Some(v) if ACTIONABLE_ROLES.contains(&e.role.as_str()) && v.trim().is_empty() => {
                    findings.push(format!("entry '{}' ({}) resolves to empty text in '{loc}'", e.key, e.role));
                }
                Some(_) => {}
            }
        }
    }
    findings
}

/// Coupling (§12.2): the manifest must resolve every (content key, locale) the
/// application's UI steps reference — the content analogue of reification coverage.
pub fn couple_content(m: &ContentManifest, graph: &DomainGraph) -> Vec<String> {
    let mut findings = Vec::new();
    let keys: std::collections::BTreeSet<&str> = graph
        .wireframe_steps
        .iter()
        .flat_map(|s| s.content_refs.iter().map(|r| r.key.as_str()))
        .collect();
    for key in keys {
        for loc in &m.content_store.locales_supported {
            let resolved = m.content_store.entries.iter().any(|e| e.key == key && e.values.contains_key(loc));
            if !resolved {
                findings.push(format!(
                    "non-conforming for locale '{loc}': cannot resolve ({key}, {loc})"
                ));
            }
        }
    }
    findings
}

#[cfg(test)]
#[path = "manifest_content_tests.rs"]
mod tests;
