//! Core artifact types — Feature, ADR, TestCriterion (ADR-002, ADR-005, ADR-011)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Feature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFrontMatter {
    pub id: String,
    pub title: String,
    #[serde(default = "default_phase")]
    pub phase: u32,
    #[serde(default = "default_feature_status")]
    pub status: FeatureStatus,
    #[serde(rename = "depends-on", default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
    #[serde(default)]
    pub tests: Vec<String>,
    /// Concern domains this feature touches (ADR-025)
    #[serde(default)]
    pub domains: Vec<String>,
    /// Acknowledged domain gaps with reasoning (ADR-025)
    #[serde(rename = "domains-acknowledged", default)]
    pub domains_acknowledged: std::collections::HashMap<String, String>,
    /// Bundle measurement metrics (written by `product context --measure`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<BundleMetrics>,
}

/// Metrics captured by `product context --measure`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetrics {
    #[serde(rename = "depth-1-adrs")]
    pub depth_1_adrs: usize,
    pub tcs: usize,
    pub domains: Vec<String>,
    #[serde(rename = "tokens-approx")]
    pub tokens_approx: usize,
    #[serde(rename = "measured-at")]
    pub measured_at: String,
}

fn default_phase() -> u32 {
    1
}
fn default_feature_status() -> FeatureStatus {
    FeatureStatus::Planned
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureStatus {
    Planned,
    InProgress,
    Complete,
    Abandoned,
}

impl std::fmt::Display for FeatureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::InProgress => write!(f, "in-progress"),
            Self::Complete => write!(f, "complete"),
            Self::Abandoned => write!(f, "abandoned"),
        }
    }
}

impl std::str::FromStr for FeatureStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "planned" => Ok(Self::Planned),
            "in-progress" => Ok(Self::InProgress),
            "complete" => Ok(Self::Complete),
            "abandoned" => Ok(Self::Abandoned),
            _ => Err(format!("unknown feature status: {}", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// ADR
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdrFrontMatter {
    pub id: String,
    pub title: String,
    #[serde(default = "default_adr_status")]
    pub status: AdrStatus,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub supersedes: Vec<String>,
    #[serde(rename = "superseded-by", default)]
    pub superseded_by: Vec<String>,
    /// Concern domains this ADR governs (ADR-025)
    #[serde(default)]
    pub domains: Vec<String>,
    /// Scope: cross-cutting, domain, or feature-specific (ADR-025)
    #[serde(default = "default_scope")]
    pub scope: AdrScope,
    /// Content hash for immutability enforcement (ADR-032)
    #[serde(rename = "content-hash", default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// Amendment audit trail (ADR-032)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amendments: Vec<Amendment>,
    /// Source files governed by this ADR
    #[serde(rename = "source-files", default, skip_serializing_if = "Vec::is_empty")]
    pub source_files: Vec<String>,
    /// Things this ADR mandates be removed (ADR-041)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removes: Vec<String>,
    /// Things this ADR deprecates (ADR-041)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecates: Vec<String>,
}

/// Amendment record for accepted ADR edits (ADR-032)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amendment {
    pub date: String,
    pub reason: String,
    #[serde(rename = "previous-hash")]
    pub previous_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdrScope {
    CrossCutting,
    Domain,
    #[default]
    FeatureSpecific,
}

fn default_scope() -> AdrScope {
    AdrScope::FeatureSpecific
}

impl std::fmt::Display for AdrScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CrossCutting => write!(f, "cross-cutting"),
            Self::Domain => write!(f, "domain"),
            Self::FeatureSpecific => write!(f, "feature-specific"),
        }
    }
}

impl std::str::FromStr for AdrScope {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "cross-cutting" => Ok(Self::CrossCutting),
            "domain" => Ok(Self::Domain),
            "feature-specific" => Ok(Self::FeatureSpecific),
            _ => Err(format!(
                "unknown scope: '{}'. Valid values: cross-cutting, domain, feature-specific",
                s
            )),
        }
    }
}

fn default_adr_status() -> AdrStatus {
    AdrStatus::Proposed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Superseded,
    Abandoned,
}

impl std::fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Accepted => write!(f, "accepted"),
            Self::Superseded => write!(f, "superseded"),
            Self::Abandoned => write!(f, "abandoned"),
        }
    }
}

impl std::str::FromStr for AdrStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "superseded" => Ok(Self::Superseded),
            "abandoned" => Ok(Self::Abandoned),
            _ => Err(format!("unknown adr status: {}", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// Test Criterion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFrontMatter {
    pub id: String,
    pub title: String,
    #[serde(rename = "type", default = "default_test_type")]
    pub test_type: TestType,
    #[serde(default = "default_test_status")]
    pub status: TestStatus,
    #[serde(default)]
    pub validates: ValidatesBlock,
    #[serde(default = "default_phase")]
    pub phase: u32,
    /// Content hash for immutability enforcement (ADR-032)
    #[serde(rename = "content-hash", default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// TC runner name (e.g. cargo-test)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner: Option<String>,
    /// TC runner arguments (e.g. test function name)
    #[serde(rename = "runner-args", default, skip_serializing_if = "Option::is_none")]
    pub runner_args: Option<String>,
    /// TC runner timeout in seconds
    #[serde(rename = "runner-timeout", default, skip_serializing_if = "Option::is_none")]
    pub runner_timeout: Option<u64>,
    /// TC prerequisites
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// Last run timestamp
    #[serde(rename = "last-run", default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<String>,
    /// Last failure message
    #[serde(rename = "failure-message", default, skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,
    /// Last run duration (e.g. "4.2s")
    #[serde(rename = "last-run-duration", default, skip_serializing_if = "Option::is_none")]
    pub last_run_duration: Option<String>,
}

fn default_test_type() -> TestType {
    TestType::Scenario
}
fn default_test_status() -> TestStatus {
    TestStatus::Unimplemented
}

/// TC type — ADR-042 structural / descriptive partition.
///
/// Structural (reserved, compiled-in): `ExitCriteria`, `Invariant`, `Chaos`, `Absence`.
/// Built-in descriptive: `Scenario`, `Benchmark`.
/// Custom descriptive: `Custom(String)` — declared in `[tc-types].custom` in product.toml.
///
/// This type is no longer `Copy` because `Custom` carries a `String`.
/// Comparisons via `==` remain cheap and work as before.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestType {
    // Structural (reserved) — drive Product mechanics by exact-string match.
    ExitCriteria,
    Invariant,
    Chaos,
    Absence,
    // Built-in descriptive — no mechanics beyond bundle inclusion.
    Scenario,
    Benchmark,
    /// Custom descriptive type — treated identically to `Scenario` in mechanics.
    Custom(String),
}

impl TestType {
    /// The four reserved structural TC-type names (ADR-042).
    /// Each drives a specific Product mechanic by exact-string match.
    pub const RESERVED: &'static [&'static str] =
        &["exit-criteria", "invariant", "chaos", "absence"];

    /// The two built-in descriptive TC-type names (ADR-042).
    pub const BUILTIN_DESCRIPTIVE: &'static [&'static str] = &["scenario", "benchmark"];

    /// Returns `true` for the four structural variants.
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            Self::ExitCriteria | Self::Invariant | Self::Chaos | Self::Absence
        )
    }

    /// Returns `true` for descriptive variants (built-in + custom).
    pub fn is_descriptive(&self) -> bool {
        !self.is_structural()
    }

    /// Returns `true` if the variant is the built-in or structural set
    /// (i.e. not `Custom`).
    pub fn is_builtin(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }

    /// The canonical string spelling, matching FromStr round-trip.
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

    /// Bundle sort key (ADR-042). Built-in types sort before custom types;
    /// custom types sort alphabetically among themselves.
    ///
    /// Returns `(category, position, name)`.
    /// `category`: 0 for built-in, 1 for custom.
    /// `position`: fixed index for built-in types, 0 for custom.
    /// `name`: the canonical string, used to break ties alphabetically.
    pub fn bundle_sort_key(&self) -> (u8, u8, String) {
        let (cat, pos) = match self {
            Self::ExitCriteria => (0, 0),
            Self::Invariant => (0, 1),
            Self::Chaos => (0, 2),
            Self::Absence => (0, 3),
            Self::Scenario => (0, 4),
            Self::Benchmark => (0, 5),
            Self::Custom(_) => (1, 0),
        };
        (cat, pos, self.as_str().to_string())
    }

    /// Parse a string into a `TestType` without consulting the config's custom
    /// list. Unknown strings become `Custom(s)`. Validation against the
    /// configured custom list is performed separately (E006).
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
        match s {
            "scenario" => Ok(Self::Scenario),
            "invariant" => Ok(Self::Invariant),
            "chaos" => Ok(Self::Chaos),
            "exit-criteria" => Ok(Self::ExitCriteria),
            "absence" => Ok(Self::Absence),
            "benchmark" => Ok(Self::Benchmark),
            other => {
                // Validate custom name shape — same as builtin (lowercase, kebab)
                let re = regex::Regex::new(r"^[a-z][a-z0-9-]*$").expect("constant regex");
                if !re.is_match(other) {
                    Err(format!(
                        "invalid tc type: '{}' — must be one of {} | {} or a custom name matching ^[a-z][a-z0-9-]*$",
                        other,
                        Self::RESERVED.join(" | "),
                        Self::BUILTIN_DESCRIPTIVE.join(" | "),
                    ))
                } else {
                    Ok(Self::Custom(other.to_string()))
                }
            }
        }
    }
}

impl Serialize for TestType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TestType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Permissive: accept any valid-shaped name; validation against the
        // configured custom list happens in graph/request validators.
        Ok(Self::parse_permissive(&s))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestStatus {
    Unimplemented,
    Implemented,
    Passing,
    Failing,
    Unrunnable,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unimplemented => write!(f, "unimplemented"),
            Self::Implemented => write!(f, "implemented"),
            Self::Passing => write!(f, "passing"),
            Self::Failing => write!(f, "failing"),
            Self::Unrunnable => write!(f, "unrunnable"),
        }
    }
}

impl std::str::FromStr for TestStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "unimplemented" => Ok(Self::Unimplemented),
            "implemented" => Ok(Self::Implemented),
            "unrunnable" => Ok(Self::Unrunnable),
            "passing" => Ok(Self::Passing),
            "failing" => Ok(Self::Failing),
            _ => Err(format!("unknown test status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidatesBlock {
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
}

// Re-export Dependency types from dep_types module (ADR-030)
pub use crate::dep_types::*;

// ---------------------------------------------------------------------------
// Loaded artifact — front-matter + body + file path
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Feature {
    pub front: FeatureFrontMatter,
    pub body: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Adr {
    pub front: AdrFrontMatter,
    pub body: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TestCriterion {
    pub front: TestFrontMatter,
    pub body: String,
    pub path: PathBuf,
    pub formal_blocks: Vec<crate::formal::FormalBlock>,
}

// ---------------------------------------------------------------------------
// Artifact enum for unified handling
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Artifact {
    Feature(Feature),
    Adr(Adr),
    Test(TestCriterion),
    Dependency(Dependency),
}

#[allow(dead_code)]
impl Artifact {
    pub fn id(&self) -> &str {
        match self {
            Self::Feature(f) => &f.front.id,
            Self::Adr(a) => &a.front.id,
            Self::Test(t) => &t.front.id,
            Self::Dependency(d) => &d.front.id,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Feature(f) => &f.front.title,
            Self::Adr(a) => &a.front.title,
            Self::Test(t) => &t.front.title,
            Self::Dependency(d) => &d.front.title,
        }
    }
}
