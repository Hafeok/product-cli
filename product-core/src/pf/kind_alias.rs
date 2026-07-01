//! The `kind`-field alias table — the single source of truth for surface names
//! that reach a node struct's shadowed `kind` field.
//!
//! `product domain new <kind>` / `product_domain_new {kind}` consume `kind` to
//! select the node type, so any node struct that also carries a `kind` field is
//! unreachable through that surface unless its `kind` is offered under a distinct
//! alias. CLI flags (`--system-kind`, …) mirror these; the MCP layer normalizes
//! them back to `kind`.

use super::ids::NodeKind;

/// A surface alias for the `kind` struct field of a node kind (see the module doc).
#[derive(Debug, Clone, Copy)]
pub struct KindAlias {
    /// The surface arg / flag name (e.g. `system_kind`; the CLI adds `--system-kind`).
    pub alias: &'static str,
    /// The node kind whose shadowed `kind` field this alias sets.
    pub kind: NodeKind,
    /// A one-line schema description for the alias.
    pub description: &'static str,
}

/// Every node kind whose struct has a `kind` field MUST appear here — the
/// node-type router shadows it otherwise. The `every_shadowed_kind_field_has_an_alias`
/// guard fails by name if a kind is added with a `kind` field but no alias.
pub const KIND_ALIASES: [KindAlias; 4] = [
    KindAlias { alias: "system_kind", kind: NodeKind::System, description: "§3.2.5 system sub-kind: application | website | service | cli | …" },
    KindAlias { alias: "mapping_kind", kind: NodeKind::ContextMapping, description: "§3.1 context-mapping kind (e.g. shared-kernel, customer-supplier, …)" },
    KindAlias { alias: "demand_kind", kind: NodeKind::QualityDemand, description: "§3.6 quality-demand kind: runtime-bound | architectural" },
    KindAlias { alias: "token_kind", kind: NodeKind::Token, description: "§4.5 design-token kind: colour | spacing | typography | …" },
];
