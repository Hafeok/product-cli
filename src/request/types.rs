//! Request document types (FT-041, ADR-038).
//!
//! A request YAML deserializes into `Request`. Artifact and change specs keep the
//! raw YAML fields on the side so the validator/applier can inspect unknown
//! fields without committing to a fully-typed schema per artifact.

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::path::PathBuf;

/// Current request schema version supported by this binary.
pub const CURRENT_REQUEST_SCHEMA: u32 = 1;

/// Top-level request document.
#[derive(Debug, Clone)]
pub struct Request {
    pub request_type: RequestType,
    pub schema_version: u32,
    pub reason: String,
    pub artifacts: Vec<ArtifactSpec>,
    pub changes: Vec<ChangeSpec>,
    /// Raw YAML source for hashing (request-log.jsonl)
    pub source_yaml: String,
}

/// Request operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    Create,
    Change,
    CreateAndChange,
}

impl std::fmt::Display for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::Change => write!(f, "change"),
            Self::CreateAndChange => write!(f, "create-and-change"),
        }
    }
}

/// An artifact to create.
#[derive(Debug, Clone)]
pub struct ArtifactSpec {
    /// Index in the request's `artifacts` array (for JSONPath locations).
    pub index: usize,
    /// Artifact type (feature, adr, tc, dep).
    pub artifact_type: ArtifactType,
    /// Optional forward-reference name (`ref: foo-bar`).
    pub ref_name: Option<String>,
    /// Raw fields excluding `type` and `ref`.
    pub fields: Mapping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactType {
    Feature,
    Adr,
    Tc,
    Dep,
}

impl ArtifactType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "feature" => Some(Self::Feature),
            "adr" => Some(Self::Adr),
            "tc" => Some(Self::Tc),
            "dep" => Some(Self::Dep),
            _ => None,
        }
    }
}

impl std::fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Feature => write!(f, "feature"),
            Self::Adr => write!(f, "adr"),
            Self::Tc => write!(f, "tc"),
            Self::Dep => write!(f, "dep"),
        }
    }
}

/// A mutation against an existing artifact.
#[derive(Debug, Clone)]
pub struct ChangeSpec {
    pub index: usize,
    pub target: String,
    pub mutations: Vec<Mutation>,
}

#[derive(Debug, Clone)]
pub struct Mutation {
    pub index: usize,
    pub op: MutationOp,
    pub field: String,
    /// Optional — `delete` does not need a value.
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationOp {
    Set,
    Append,
    Remove,
    Delete,
}

impl MutationOp {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "set" => Some(Self::Set),
            "append" => Some(Self::Append),
            "remove" => Some(Self::Remove),
            "delete" => Some(Self::Delete),
            _ => None,
        }
    }
}

impl std::fmt::Display for MutationOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Set => write!(f, "set"),
            Self::Append => write!(f, "append"),
            Self::Remove => write!(f, "remove"),
            Self::Delete => write!(f, "delete"),
        }
    }
}

/// A validation finding.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    /// JSONPath expression against the request document.
    pub location: String,
    /// Optional upgrade hint (used by schema-version errors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

impl Finding {
    pub fn error(code: &str, message: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            severity: Severity::Error,
            message: message.into(),
            location: location.into(),
            upgrade_hint: None,
        }
    }

    pub fn warning(code: &str, message: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            severity: Severity::Warning,
            message: message.into(),
            location: location.into(),
            upgrade_hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.upgrade_hint = Some(hint.into());
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{}[{}]: {}\n  at {}", sev, self.code, self.message, self.location)?;
        if let Some(ref hint) = self.upgrade_hint {
            write!(f, "\n  = hint: {}", hint)?;
        }
        Ok(())
    }
}

/// A file touched by a request apply.
#[derive(Debug, Clone)]
pub struct PlannedWrite {
    pub path: PathBuf,
    pub content: String,
    pub is_new: bool,
}
