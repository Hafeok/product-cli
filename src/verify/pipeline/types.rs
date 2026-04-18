//! Shared pipeline types (FT-044).

use serde::Serialize;

/// Status of a single pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StageStatus {
    Pass,
    Warning,
    Fail,
}

impl StageStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pass => "\u{2713}",    // ✓
            Self::Warning => "\u{26A0}", // ⚠
            Self::Fail => "\u{2717}",    // ✗
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warning => "warning",
            Self::Fail => "fail",
        }
    }

    /// Combine two statuses: worst wins (Fail > Warning > Pass).
    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Fail, _) | (_, Self::Fail) => Self::Fail,
            (Self::Warning, _) | (_, Self::Warning) => Self::Warning,
            _ => Self::Pass,
        }
    }
}

/// A finding produced by a stage. Stages 1–4 emit diagnostic codes (strings).
/// Stage 5 and 6 emit TC-result objects.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Finding {
    /// Simple diagnostic code string (stages 1–4).
    Code(String),
    /// Per-TC result object (stages 5 and 6).
    Tc {
        tc: String,
        feature: Option<String>,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

/// Result of a single stage.
#[derive(Debug, Clone, Serialize)]
pub struct StageResult {
    pub stage: u8,
    pub name: &'static str,
    pub status: StageStatus,
    pub findings: Vec<Finding>,
    /// Optional one-line summary shown in pretty mode.
    #[serde(skip)]
    pub summary: String,
}

/// Aggregate pipeline result.
#[derive(Debug, Clone, Serialize)]
pub struct PipelineResult {
    pub passed: bool,
    pub exit: i32,
    pub stages: Vec<StageResult>,
}

impl PipelineResult {
    /// Overall exit code: 1 if any stage is Fail, 2 if any Warning, 0 otherwise.
    pub fn exit_code(&self) -> i32 {
        let mut worst = StageStatus::Pass;
        for s in &self.stages {
            worst = worst.merge(s.status);
        }
        match worst {
            StageStatus::Fail => 1,
            StageStatus::Warning => 2,
            StageStatus::Pass => 0,
        }
    }
}

/// Scope flags for the pipeline.
#[derive(Debug, Clone, Default)]
pub struct PipelineScope {
    /// If set, stage 5 runs only features in this phase.
    pub phase: Option<u32>,
}
