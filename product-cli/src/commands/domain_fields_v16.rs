//! §3.0/§3.6/§7.3 `--field` flags for the product-boundary node family.
//!
//! A second flattened `clap` group beside [`super::domain_fields::NodeFields`],
//! split out to keep each under the 400-line gate. Carries the flags for the
//! `product`, `journey`, and `quality-demand` kinds (and a system's
//! `references-domain`); merged into the same JSON field map `pf::edit` consumes.

use clap::Args;
use serde_json::{json, Map, Value};

/// The 1.6.0 product-boundary flags (product / journey / quality demand).
#[derive(Args, Default)]
pub struct V16Fields {
    /// §3.0 the bounded contexts (domains) a product owns (comma-separated)
    #[arg(long = "owns-domain", value_delimiter = ',')]
    owns_domain: Option<Vec<String>>,
    /// §3.0 the systems a product owns (comma-separated)
    #[arg(long = "owns-system", value_delimiter = ',')]
    owns_system: Option<Vec<String>>,
    /// §7.3 the product's What-version (e.g. `1.0.0`)
    #[arg(long = "what-version")]
    what_version: Option<String>,
    /// §3.0/§3.2.5 whole domains a system references (comma-separated)
    #[arg(long = "references-domain", value_delimiter = ',')]
    references_domain: Option<Vec<String>>,
    /// §3.0.1 the product a journey belongs to
    #[arg(long = "owner")]
    owner: Option<String>,
    /// §3.0.1 the single-system flows a journey composes (comma-separated)
    #[arg(long = "composes-flow", value_delimiter = ',')]
    composes_flow: Option<Vec<String>>,
    /// §3.0.1 the Translation triggers a journey crosses via (comma-separated)
    #[arg(long = "crosses-via", value_delimiter = ',')]
    crosses_via: Option<Vec<String>>,
    /// §3.6 quality-demand kind (`runtime-bound` or `architectural`)
    #[arg(long = "demand-kind")]
    demand_kind: Option<String>,
    /// §4.5 design-token kind (`colour` | `spacing` | `typography` | …)
    #[arg(long = "token-kind")]
    token_kind: Option<String>,
    /// §3.6 the checkable bound or constraint (e.g. `p99 latency ≤ 200ms`)
    #[arg(long = "bound")]
    bound: Option<String>,
    /// §3.6 the element a quality demand scopes (system/flow/ui-step/decider id)
    #[arg(long = "scopes")]
    scopes: Option<String>,
    /// §3.6 a runtime bound's telemetry source it is measured against
    #[arg(long = "measured-by")]
    measured_by: Option<String>,
    /// §3.6 an architectural constraint's How-side contract it binds
    #[arg(long = "constrains")]
    constrains: Option<String>,
}

impl V16Fields {
    /// Merge the provided flags into the field map keyed by model field names.
    pub(crate) fn merge_into(&self, m: &mut Map<String, Value>) {
        let mut put = |k: &str, v: Value| { m.insert(k.to_string(), v); };
        if let Some(v) = &self.owns_domain { put("owns_domain", json!(v)); }
        if let Some(v) = &self.owns_system { put("owns_system", json!(v)); }
        if let Some(v) = &self.what_version { put("version", json!(v)); }
        if let Some(v) = &self.references_domain { put("references_domain", json!(v)); }
        if let Some(v) = &self.owner { put("product", json!(v)); }
        if let Some(v) = &self.composes_flow { put("composes_flow", json!(v)); }
        if let Some(v) = &self.crosses_via { put("crosses_via", json!(v)); }
        if let Some(v) = &self.demand_kind { put("kind", json!(v)); }
        if let Some(v) = &self.token_kind { put("kind", json!(v)); }
        if let Some(v) = &self.bound { put("bound", json!(v)); }
        if let Some(v) = &self.scopes { put("scopes", json!(v)); }
        if let Some(v) = &self.measured_by { put("measured_by", json!(v)); }
        if let Some(v) = &self.constrains { put("constrains", json!(v)); }
    }
}
