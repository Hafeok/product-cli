//! Typed discharge pointers — where a decision's disposition is checked.
//!
//! §4.2's `DischargeRef` is a typed pointer, not a URL and not prose: the
//! scheme says *what kind of extra-actor* closes the decision, which is what
//! makes discharge-by-stage (§8) and the toolchain joins (§9.1) queryable.
//! An unknown scheme is a schema fault rather than an opaque string, because
//! a pointer nothing can resolve is the wiki this tool is not (§3.2).
//!
//! L0 parses and canonicalises these. Nothing *resolves* them — that is L2's
//! coverage and L4's toolchain integration.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::identity::Identity;

/// A pointer to the extra-actor that discharges a decision.
///
/// Ordering is derived and used by [`crate::canon`] to sort the discharge
/// list: a set of pointers has no authored order, so reordering them in a
/// file is formatting, not meaning.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DischargeRef {
    /// A static-analysis rule id, e.g. `analyzer:DEC001-no-float-money`.
    Analyzer(String),
    /// A test's fully-qualified name, e.g. `test:ExportIdempotencyTests`.
    Test(String),
    /// A platform policy id, e.g. `policy:deny-public-blob`.
    Policy(String),
    /// A pre-deployment assertion, e.g. `whatif:no-public-network`.
    WhatIf(String),
    /// A production telemetry metric. Its `expectation` is a sibling field on
    /// the version: a dashboard read afterward is unfalsifiable (§9.1).
    Otel(String),
    /// A named actor — the discharge for a `judgment`.
    Actor(Identity),
}

impl DischargeRef {
    /// The scheme, as it appears before the colon.
    pub fn scheme(&self) -> &'static str {
        match self {
            Self::Analyzer(_) => "analyzer",
            Self::Test(_) => "test",
            Self::Policy(_) => "policy",
            Self::WhatIf(_) => "whatif",
            Self::Otel(_) => "otel",
            Self::Actor(_) => "actor",
        }
    }

    /// Whether this pointer names a production telemetry metric, which
    /// obliges the version to carry a stated `expectation`.
    pub fn is_otel(&self) -> bool {
        matches!(self, Self::Otel(_))
    }
}

/// Every scheme this format version defines, for the error message.
const SCHEMES: &str = "analyzer | test | policy | whatif | otel | actor";

impl FromStr for DischargeRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (scheme, body) = s
            .split_once(':')
            .ok_or_else(|| format!("`{s}` has no scheme — expected one of {SCHEMES}"))?;
        let body = body.trim();
        if body.is_empty() {
            return Err(format!("`{s}` has an empty `{scheme}` payload"));
        }
        match scheme {
            "analyzer" => Ok(Self::Analyzer(body.to_string())),
            "test" => Ok(Self::Test(body.to_string())),
            "policy" => Ok(Self::Policy(body.to_string())),
            "whatif" => Ok(Self::WhatIf(body.to_string())),
            "otel" => Ok(Self::Otel(body.to_string())),
            "actor" => body.parse().map(Self::Actor),
            other => Err(format!("`{other}` is not a discharge scheme — expected one of {SCHEMES}")),
        }
    }
}

impl fmt::Display for DischargeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Analyzer(b) | Self::Test(b) | Self::Policy(b) | Self::WhatIf(b) | Self::Otel(b) => {
                write!(f, "{}:{b}", self.scheme())
            }
            Self::Actor(i) => write!(f, "actor:{i}"),
        }
    }
}

impl Serialize for DischargeRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DischargeRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

/// The pipeline stage a criterion is discharged at (§5.1's ground table).
/// Discharging later than the ground allowed is waste; earlier is fiction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    /// Static ground only: types, structure, template validity.
    Pr,
    /// Real resource behaviour, identity, RBAC.
    Dev,
    /// Prod-like data volume and shape, migration behaviour.
    Staging,
    /// Users, cohorts, real traffic, failure distribution.
    Prod,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pr => "pr",
            Self::Dev => "dev",
            Self::Staging => "staging",
            Self::Prod => "prod",
        };
        f.write_str(s)
    }
}

#[path = "discharge_tests.rs"]
#[cfg(test)]
mod tests;
