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
}

fn default_test_type() -> TestType {
    TestType::Scenario
}
fn default_test_status() -> TestStatus {
    TestStatus::Unimplemented
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestType {
    Scenario,
    Invariant,
    Chaos,
    ExitCriteria,
}

impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scenario => write!(f, "scenario"),
            Self::Invariant => write!(f, "invariant"),
            Self::Chaos => write!(f, "chaos"),
            Self::ExitCriteria => write!(f, "exit-criteria"),
        }
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
            _ => Err(format!("unknown test type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestStatus {
    Unimplemented,
    Implemented,
    Passing,
    Failing,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unimplemented => write!(f, "unimplemented"),
            Self::Implemented => write!(f, "implemented"),
            Self::Passing => write!(f, "passing"),
            Self::Failing => write!(f, "failing"),
        }
    }
}

impl std::str::FromStr for TestStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "unimplemented" => Ok(Self::Unimplemented),
            "implemented" => Ok(Self::Implemented),
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
}

#[allow(dead_code)]
impl Artifact {
    pub fn id(&self) -> &str {
        match self {
            Self::Feature(f) => &f.front.id,
            Self::Adr(a) => &a.front.id,
            Self::Test(t) => &t.front.id,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Feature(f) => &f.front.title,
            Self::Adr(a) => &a.front.title,
            Self::Test(t) => &t.front.title,
        }
    }
}
