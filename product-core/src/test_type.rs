//! `TestType` — TC type enum (ADR-011, ADR-041, ADR-042).
//!
//! Split out of `types.rs` for file-length hygiene.

use serde::{Deserialize, Serialize};

/// TC type — ADR-042 structural / descriptive partition.
///
/// Structural (reserved, compiled-in): `ExitCriteria`, `Invariant`, `Chaos`,
/// `Absence` (FT-047 / ADR-041). Built-in descriptive: `Scenario`,
/// `Benchmark`. Custom descriptive: `Custom(String)` — declared in
/// `[tc-types].custom` in product.toml.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestType {
    ExitCriteria,
    Invariant,
    Chaos,
    /// Negative assertion — "the thing is gone". FT-047 / ADR-041.
    Absence,
    Scenario,
    Benchmark,
    /// Custom descriptive type (ADR-042).
    Custom(String),
}

impl TestType {
    /// The four reserved structural TC-type names (ADR-042).
    pub const RESERVED: &'static [&'static str] =
        &["exit-criteria", "invariant", "chaos", "absence"];

    /// The two built-in descriptive TC-type names (ADR-042).
    pub const BUILTIN_DESCRIPTIVE: &'static [&'static str] = &["scenario", "benchmark"];

    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            Self::ExitCriteria | Self::Invariant | Self::Chaos | Self::Absence
        )
    }

    pub fn is_descriptive(&self) -> bool {
        !self.is_structural()
    }

    pub fn is_builtin(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::ExitCriteria => "exit-criteria",
            Self::Invariant => "invariant",
            Self::Chaos => "chaos",
            Self::Absence => "absence",
            Self::Scenario => "scenario",
            Self::Benchmark => "benchmark",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Bundle sort key (ADR-042). Built-in types sort before custom; custom
    /// types sort alphabetically among themselves.
    pub fn bundle_sort_key(&self) -> (u8, u8, String) {
        let (cat, pos) = match self {
            Self::ExitCriteria => (0u8, 0u8),
            Self::Invariant => (0, 1),
            Self::Chaos => (0, 2),
            Self::Absence => (0, 3),
            Self::Scenario => (0, 4),
            Self::Benchmark => (0, 5),
            Self::Custom(_) => (1, 0),
        };
        (cat, pos, self.as_str().to_string())
    }

    /// Parse permissively — unknown names become `Custom(s)`.
    pub fn parse_permissive(s: &str) -> Self {
        match s {
            "exit-criteria" => Self::ExitCriteria,
            "invariant" => Self::Invariant,
            "chaos" => Self::Chaos,
            "absence" => Self::Absence,
            "scenario" => Self::Scenario,
            "benchmark" => Self::Benchmark,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for TestType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        let re = regex::Regex::new(r"^[a-z][a-z0-9-]*$").expect("constant regex");
        if !re.is_match(s) {
            return Err(format!(
                "invalid tc type: '{}' — must match ^[a-z][a-z0-9-]*$",
                s
            ));
        }
        Ok(Self::parse_permissive(s))
    }
}

impl Serialize for TestType {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TestType {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::parse_permissive(&s))
    }
}

/// Helper called from `ProductConfig::is_known_tc_type`.
pub fn is_known_tc_type(custom: &[String], name: &str) -> bool {
    TestType::RESERVED.contains(&name)
        || TestType::BUILTIN_DESCRIPTIVE.contains(&name)
        || custom.iter().any(|s| s == name)
}

/// Helper called from `ProductConfig::tc_type_hint`.
pub fn tc_type_hint(custom: &[String]) -> String {
    let builtin: Vec<&str> = TestType::RESERVED
        .iter()
        .chain(TestType::BUILTIN_DESCRIPTIVE.iter())
        .copied()
        .collect();
    let custom_str = if custom.is_empty() {
        "(none)".to_string()
    } else {
        custom.join(", ")
    };
    format!("valid types: {} (custom: {})", builtin.join(", "), custom_str)
}
