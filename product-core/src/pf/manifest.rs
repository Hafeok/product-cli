//! §11.3 PREVIEW — the design-system manifest profile (FT-141).
//!
//! Reads the TOML manifest a design system publishes to plug in as the
//! Concrete-UI layer, validates its internal wholeness, and confirms it couples
//! to a captured What graph (reification coverage over the core AIOs × the
//! contexts the application declares). Non-normative: a derived view of
//! §3.2.2/§3.2.3/§4.5 from the design system's side of the seam (ADR-085).

use super::ids::CORE_AIOS;
use super::model::DomainGraph;
use super::wcag22::is_wcag_22;

/// A WCAG guarantee a component discharges by construction.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Satisfies {
    pub criterion: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub verification: String,
}

/// One concrete interaction object in the catalog.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Component {
    pub id: String,
    #[serde(default)]
    pub tokens: Vec<String>,
    #[serde(default)]
    pub satisfies: Vec<Satisfies>,
}

/// One reify(aio, when) → cio rule.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Reify {
    pub aio: String,
    pub when: String,
    pub cio: String,
    #[serde(default)]
    pub rationale: String,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct DesignSystemHeader {
    pub id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub wcag_target: String,
    #[serde(default)]
    pub contexts_supported: Vec<String>,
    #[serde(default)]
    pub tokens: Vec<String>,
}

/// The §11.3 design-system manifest.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct DsManifest {
    pub design_system: DesignSystemHeader,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub reification: Vec<Reify>,
}

/// Parse a TOML design-system manifest, pointing the user at the expected shape
/// on a schema mismatch.
pub fn parse_ds(toml_src: &str) -> Result<DsManifest, String> {
    toml::from_str(toml_src).map_err(|e| {
        format!("manifest does not match the §11.3 design-system schema: {e}\n\
                 expected: [design_system] id; contexts_supported = [..]; tokens = [..]; \
                 [[components]] id/tokens/satisfies; [[reification]] aio/when/cio")
    })
}

/// Internal wholeness (§11.3): every reified cio is in the catalog, every
/// component token is declared, every claimed criterion is a real WCAG 2.2 entity.
pub fn validate_ds(m: &DsManifest) -> Vec<String> {
    let mut findings = Vec::new();
    for r in &m.reification {
        if !m.components.iter().any(|c| c.id == r.cio) {
            findings.push(format!(
                "reification reify({}, {}) names cio '{}', absent from components",
                r.aio, r.when, r.cio
            ));
        }
    }
    for c in &m.components {
        for t in &c.tokens {
            if !m.design_system.tokens.contains(t) {
                findings.push(format!("component '{}' references undeclared token '{}'", c.id, t));
            }
        }
        for s in &c.satisfies {
            if !is_wcag_22(&s.criterion) {
                findings.push(format!(
                    "component '{}' claims '{}', which is not a WCAG 2.2 criterion",
                    c.id, s.criterion
                ));
            }
        }
    }
    findings
}

/// Coupling (§11.3): the manifest must reify every core AIO across every context
/// of use the What declares — the design-system analogue of command coverage.
pub fn couple_ds(m: &DsManifest, graph: &DomainGraph) -> Vec<String> {
    let mut findings = Vec::new();
    for ctx in &graph.contexts_of_use {
        for aio in CORE_AIOS {
            let covered = m.reification.iter().any(|r| r.aio == aio && r.when == ctx.id);
            if !covered {
                findings.push(format!(
                    "non-conforming for context '{}': no reify({aio}, {}) rule",
                    ctx.id, ctx.id
                ));
            }
        }
    }
    findings
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
