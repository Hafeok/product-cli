//! §11.3 PREVIEW — the design-system manifest profile (FT-141).
//!
//! Reads the canonical YAML manifest a design system publishes to plug in as the
//! Concrete-UI layer, validates its internal wholeness, and confirms it couples
//! to a captured What graph (every referenced AIO has a reifying CIO for each
//! declared context). Non-normative: a derived view of §3.2.2/§3.2.3/§4.5 from
//! the design system's side of the seam (ADR-085).

use super::model::{ContextOfUse, DomainGraph};
use super::wcag22::is_wcag_22;
use std::collections::BTreeMap;

/// The §11.3 contexts-of-use space the design system claims to reify into.
#[derive(Debug, Clone, serde::Deserialize, Default, PartialEq)]
pub struct ContextSpace {
    #[serde(default)]
    pub form_factor: Vec<String>,
    #[serde(default)]
    pub modality: Vec<String>,
}

/// A WCAG guarantee a component discharges by construction.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Satisfies {
    pub criterion: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub via: String,
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

/// One reify(aio, when) → cio rule. `when` is a partial predicate over context
/// dimensions; an unconstrained dimension is a wildcard, an empty `when` matches
/// every context.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Reify {
    pub aio: String,
    #[serde(default)]
    pub when: BTreeMap<String, String>,
    pub cio: String,
    #[serde(default)]
    pub rationale: String,
}

/// One design token on the surface.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Token {
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
}

/// The body of the §11.3 manifest (everything under the `design_system:` key).
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct DesignSystem {
    pub id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub wcag_target: String,
    #[serde(default)]
    pub contexts_supported: ContextSpace,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub reification: Vec<Reify>,
    #[serde(default)]
    pub tokens: Vec<Token>,
}

/// The §11.3 design-system manifest (the canonical YAML shape).
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct DsManifest {
    pub design_system: DesignSystem,
}

/// Parse a canonical YAML design-system manifest, pointing the user at the
/// expected shape on a schema mismatch.
pub fn parse_ds(yaml_src: &str) -> Result<DsManifest, String> {
    serde_yaml::from_str(yaml_src).map_err(|e| {
        format!("manifest does not match the §11.3 design-system schema: {e}\n\
                 expected: design_system: id; contexts_supported: {{form_factor, modality}}; \
                 components: [{{id, tokens, satisfies}}]; reification: [{{aio, when, cio}}]; \
                 tokens: [{{id, type}}]")
    })
}

/// Internal wholeness (§11.3): every reified cio is in the catalog, every
/// component token is declared, every claimed criterion is a real WCAG 2.2 entity.
pub fn validate_ds(m: &DsManifest) -> Vec<String> {
    let ds = &m.design_system;
    let mut findings = Vec::new();
    for r in &ds.reification {
        if !ds.components.iter().any(|c| c.id == r.cio) {
            findings.push(format!("reification reify({}) names cio '{}', absent from components", r.aio, r.cio));
        }
    }
    for c in &ds.components {
        for t in &c.tokens {
            if !ds.tokens.iter().any(|tok| &tok.id == t) {
                findings.push(format!("component '{}' references undeclared token '{}'", c.id, t));
            }
        }
        for s in &c.satisfies {
            if !is_wcag_22(&s.criterion) {
                findings.push(format!("component '{}' claims '{}', which is not a WCAG 2.2 criterion", c.id, s.criterion));
            }
        }
    }
    findings
}

/// Whether a reification rule's `when` predicate is compatible with a context of
/// use — an unconstrained dimension is a wildcard (§11.3).
fn applies(when: &BTreeMap<String, String>, ctx: &ContextOfUse) -> bool {
    match (&ctx.dimension, &ctx.value) {
        (Some(d), Some(v)) => when.get(d).is_none_or(|wv| wv == v),
        _ => when.is_empty() || when.values().any(|wv| wv == &ctx.id),
    }
}

/// Coupling (§11.2/§11.3): every AIO the What's UI steps reference must have a
/// reifying CIO for each context of use the What declares — the design-system
/// analogue of command coverage. A gap makes the system non-conforming for that
/// context.
pub fn couple_ds(m: &DsManifest, graph: &DomainGraph) -> Vec<String> {
    let aios: std::collections::BTreeSet<&str> = graph
        .wireframe_steps
        .iter()
        .flat_map(|s| s.surfaces.iter().map(|x| x.aio.as_str()).chain(s.offers.iter().map(|o| o.aio.as_str())))
        .collect();
    let mut findings = Vec::new();
    for ctx in &graph.contexts_of_use {
        for aio in &aios {
            let covered = m.design_system.reification.iter().any(|r| &r.aio == aio && applies(&r.when, ctx));
            if !covered {
                findings.push(format!("non-conforming for context '{}': no reify({aio}) rule applies", ctx.id));
            }
        }
    }
    findings
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
