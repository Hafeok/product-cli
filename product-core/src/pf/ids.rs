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
}

/// Every node kind, in declaration order (for `list`/iteration).
pub const ALL_KINDS: [NodeKind; 11] = [
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
            Self::WireframeStep => "wireframe-step",
            Self::Flow => "flow",
        }
    }

    /// Parse a CLI kind name. Accepts the kebab CLI names plus the `pf:` class
    /// names, case-insensitively.
    pub fn parse(s: &str) -> Result<Self> {
        let norm = s.trim().to_lowercase();
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
