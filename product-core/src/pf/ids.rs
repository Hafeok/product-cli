//! Identifier validation for Product-Framework nodes.

use crate::error::{ProductError, Result};

/// The node classes the What-capture session can create. Used for the
/// `sh:class` checks in the conformance mirror (an event must `changes` a
/// node whose kind is `Entity`, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    BoundedContext,
    Entity,
    ValueObject,
    Relation,
    Invariant,
    ContextMapping,
    Command,
    Event,
    ReadModel,
    WireframeStep,
    Flow,
    /// §3.2.2 — an Abstract Interaction Object: a context-independent kind of
    /// interaction a UI step is typed against (never a concrete control).
    Aio,
    /// §3.2.2 — a context of use (form factor, modality, …) reification rules
    /// are written against.
    ContextOfUse,
    /// §3.2.4 — the distinguished node of the page graph; its out-edges are the
    /// global destinations the primary navigation renders.
    ApplicationRoot,
    /// §3.2.3 — an ingested WCAG 2.2 success criterion (level + verification
    /// type) a UI step or AIO must satisfy.
    WcagCriterion,
    /// §3.2.3 — a dated, attributed record that a non-machine criterion was
    /// evaluated and met.
    Attestation,
    /// §4.6 — the swappable provider of words: resolves (content key, locale) →
    /// string for the content references a UI step carries.
    ContentStore,
    /// §4.5 — the design system: the closed catalog of CIOs + token surface a
    /// screen composes from.
    DesignSystem,
    /// §4.5 — a Concrete Interaction Object: an on-system component an AIO
    /// reifies into.
    Cio,
    /// §4.5 — a design token (colour, spacing, …); styling references tokens,
    /// never literals.
    Token,
    /// §4.5 — a reify(AIO, context) → CIO rule carrying rationale.
    ReificationRule,
    /// §3.1 — constitutive reference data: a governed set of values the
    /// behaviour depends on (the What side of the structure/data split).
    ReferenceSet,
    /// §3.1 — a validatable shape over an entity: the structure side made
    /// machine-checkable, the oracle production data is validated against.
    DataShape,
    /// §3.1 — a production dataset: the populated facts data conformance
    /// (§6.3) checks the shapes against, never specification itself.
    ProductionDataset,
    /// §3.2.5 — a first-class system: the named thing (name, kind, purpose,
    /// target platforms/classes) the page graph and flows belong to.
    System,
}

/// Every node kind, in declaration order (for `list`/iteration).
pub const ALL_KINDS: [NodeKind; 25] = [
    NodeKind::BoundedContext,
    NodeKind::Entity,
    NodeKind::ValueObject,
    NodeKind::Relation,
    NodeKind::Invariant,
    NodeKind::ContextMapping,
    NodeKind::Command,
    NodeKind::Event,
    NodeKind::ReadModel,
    NodeKind::WireframeStep,
    NodeKind::Flow,
    NodeKind::Aio,
    NodeKind::ContextOfUse,
    NodeKind::ApplicationRoot,
    NodeKind::WcagCriterion,
    NodeKind::Attestation,
    NodeKind::ContentStore,
    NodeKind::DesignSystem,
    NodeKind::Cio,
    NodeKind::Token,
    NodeKind::ReificationRule,
    NodeKind::ReferenceSet,
    NodeKind::DataShape,
    NodeKind::ProductionDataset,
    NodeKind::System,
];

/// Built-in WCAG 2.2 criteria seed: (id, level, verification-type, name). An
/// adopter may register more as `WcagCriterion` nodes; these are always known.
pub const CORE_WCAG: [(&str, &str, &str, &str); 6] = [
    ("1.1.1", "A", "machine", "Non-text Content"),
    ("1.3.1", "A", "machine", "Info and Relationships"),
    ("3.3.2", "A", "machine", "Labels or Instructions"),
    ("4.1.2", "A", "machine", "Name, Role, Value"),
    ("2.4.7", "AA", "assisted", "Focus Visible"),
    ("2.5.8", "AA", "machine", "Target Size (Minimum)"),
];

/// Built-in accessibility obligations inherited from each core AIO (§3.2.3):
/// (aio-id, &[criterion-id]). A step's full obligation is the union of these
/// over the AIOs it references, plus its own `must_satisfy`.
pub const CORE_AIO_CRITERIA: [(&str, &[&str]); 4] = [
    ("text-entry", &["3.3.2", "1.3.1", "4.1.2"]),
    ("numeric-entry", &["3.3.2", "1.3.1", "4.1.2"]),
    ("date-entry", &["3.3.2", "1.3.1", "4.1.2"]),
    ("display-value", &["1.1.1"]),
];

/// The closed-core Abstract Interaction Objects (§3.2.2 table). An adopter may
/// register additional AIOs as `Aio` nodes; these core ids are always recognised.
pub const CORE_AIOS: [&str; 10] = [
    "trigger-action",
    "single-select",
    "multi-select",
    "text-entry",
    "numeric-entry",
    "date-entry",
    "display-value",
    "display-collection",
    "navigate",
    "edit",
];

impl NodeKind {
    /// The `pf:` class local name as emitted in Turtle / the ontology.
    pub fn class_name(self) -> &'static str {
        match self {
            Self::BoundedContext => "BoundedContext",
            Self::Entity => "Entity",
            Self::ValueObject => "ValueObject",
            Self::Relation => "Relation",
            Self::Invariant => "Invariant",
            Self::ContextMapping => "ContextMapping",
            Self::Command => "Command",
            Self::Event => "Event",
            Self::ReadModel => "ReadModel",
            Self::WireframeStep => "WireframeStep",
            Self::Flow => "Flow",
            Self::Aio => "Aio",
            Self::ContextOfUse => "ContextOfUse",
            Self::ApplicationRoot => "ApplicationRoot",
            Self::WcagCriterion => "WcagCriterion",
            Self::Attestation => "Attestation",
            Self::ContentStore => "ContentStore",
            Self::DesignSystem => "DesignSystem",
            Self::Cio => "Cio",
            Self::Token => "Token",
            Self::ReificationRule => "ReificationRule",
            Self::ReferenceSet => "ReferenceSet",
            Self::DataShape => "DataShape",
            Self::ProductionDataset => "ProductionDataset",
            Self::System => "System",
        }
    }

    /// The kebab-case CLI name (e.g. `value-object`, `read-model`).
    pub fn cli_name(self) -> &'static str {
        match self {
            Self::BoundedContext => "context",
            Self::Entity => "entity",
            Self::ValueObject => "value-object",
            Self::Relation => "relation",
            Self::Invariant => "invariant",
            Self::ContextMapping => "mapping",
            Self::Command => "command",
            Self::Event => "event",
            Self::ReadModel => "read-model",
            Self::WireframeStep => "ui-step",
            Self::Flow => "flow",
            Self::Aio => "aio",
            Self::ContextOfUse => "context-of-use",
            Self::ApplicationRoot => "application-root",
            Self::WcagCriterion => "wcag-criterion",
            Self::Attestation => "attestation",
            Self::ContentStore => "content-store",
            Self::DesignSystem => "design-system",
            Self::Cio => "cio",
            Self::Token => "token",
            Self::ReificationRule => "reification-rule",
            Self::ReferenceSet => "reference-set",
            Self::DataShape => "data-shape",
            Self::ProductionDataset => "production-dataset",
            Self::System => "system",
        }
    }

    /// Parse a CLI kind name. Accepts the kebab CLI names plus the `pf:` class
    /// names, case-insensitively.
    pub fn parse(s: &str) -> Result<Self> {
        let norm = s.trim().to_lowercase();
        // `wireframe-step` is the deprecated alias for the superseding `ui-step`.
        if norm == "wireframe-step" {
            return Ok(Self::WireframeStep);
        }
        ALL_KINDS
            .into_iter()
            .find(|k| k.cli_name() == norm || k.class_name().to_lowercase() == norm)
            .ok_or_else(|| {
                let names: Vec<&str> = ALL_KINDS.iter().map(|k| k.cli_name()).collect();
                ProductError::ConfigError(format!(
                    "unknown kind {:?} — expected one of: {}",
                    s,
                    names.join(", ")
                ))
            })
    }
}

/// Cardinality of a relation (§3.1). Mirrors the `cardinality` enum in
/// `product-author-domain.tools.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

impl Cardinality {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "one-to-one" => Ok(Self::OneToOne),
            "one-to-many" => Ok(Self::OneToMany),
            "many-to-one" => Ok(Self::ManyToOne),
            "many-to-many" => Ok(Self::ManyToMany),
            other => Err(ProductError::ConfigError(format!(
                "invalid cardinality {:?} — use one-to-one | one-to-many | many-to-one | many-to-many",
                other
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneToOne => "one-to-one",
            Self::OneToMany => "one-to-many",
            Self::ManyToOne => "many-to-one",
            Self::ManyToMany => "many-to-many",
        }
    }
}

/// Validate a node identifier against the schema pattern
/// `^[A-Za-z][A-Za-z0-9_-]*$` (see `$defs/id` in the tool schema). A stable,
/// unique-within-the-graph token; never contains spaces or `:`.
pub fn validate_id(id: &str) -> Result<()> {
    let mut chars = id.chars();
    let first_ok = chars.next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false);
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if first_ok && rest_ok {
        Ok(())
    } else {
        Err(ProductError::ConfigError(format!(
            "invalid id {:?} — must match ^[A-Za-z][A-Za-z0-9_-]*$ (letter first, then letters/digits/_/-)",
            id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_ids() {
        for id in ["Task", "user_account", "ctx-1", "A", "Subscription2"] {
            assert!(validate_id(id).is_ok(), "{id} should be valid");
        }
    }

    #[test]
    fn rejects_invalid_ids() {
        for id in ["", "1abc", "has space", "ns:Task", "-leading", "emoji😀"] {
            assert!(validate_id(id).is_err(), "{id} should be invalid");
        }
    }

    #[test]
    fn node_kind_parses_cli_and_class_names() {
        assert_eq!(NodeKind::parse("entity").expect("e"), NodeKind::Entity);
        assert_eq!(NodeKind::parse("value-object").expect("vo"), NodeKind::ValueObject);
        assert_eq!(NodeKind::parse("context").expect("c"), NodeKind::BoundedContext);
        assert_eq!(NodeKind::parse("BoundedContext").expect("cls"), NodeKind::BoundedContext);
        assert_eq!(NodeKind::parse("read-model").expect("rm"), NodeKind::ReadModel);
        assert!(NodeKind::parse("widget").is_err());
        // cli_name round-trips through parse for every kind
        for k in ALL_KINDS {
            assert_eq!(NodeKind::parse(k.cli_name()).expect("rt"), k);
        }
    }

    #[test]
    fn cardinality_round_trips() {
        for s in ["one-to-one", "one-to-many", "many-to-one", "many-to-many"] {
            assert_eq!(Cardinality::parse(s).expect("parse").as_str(), s);
        }
        assert!(Cardinality::parse("lots").is_err());
    }
}
