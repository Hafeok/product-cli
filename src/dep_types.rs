//! Dependency artifact types (ADR-030) — DependencyFrontMatter, DependencyType, DependencyStatus

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyFrontMatter {
    pub id: String,
    pub title: String,
    #[serde(rename = "type", default = "default_dep_type")]
    pub dep_type: DependencyType,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default = "default_dep_status")]
    pub status: DependencyStatus,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
    #[serde(rename = "availability-check", default, skip_serializing_if = "Option::is_none")]
    pub availability_check: Option<String>,
    #[serde(rename = "breaking-change-risk", default = "default_risk")]
    pub breaking_change_risk: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<InterfaceBlock>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supersedes: Vec<String>,
}

fn default_dep_type() -> DependencyType {
    DependencyType::Library
}
fn default_dep_status() -> DependencyStatus {
    DependencyStatus::Active
}
fn default_risk() -> String {
    "low".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyType {
    Library,
    Service,
    Api,
    Tool,
    Hardware,
    Runtime,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Library => write!(f, "library"),
            Self::Service => write!(f, "service"),
            Self::Api => write!(f, "api"),
            Self::Tool => write!(f, "tool"),
            Self::Hardware => write!(f, "hardware"),
            Self::Runtime => write!(f, "runtime"),
        }
    }
}

impl std::str::FromStr for DependencyType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "library" => Ok(Self::Library),
            "service" => Ok(Self::Service),
            "api" => Ok(Self::Api),
            "tool" => Ok(Self::Tool),
            "hardware" => Ok(Self::Hardware),
            "runtime" => Ok(Self::Runtime),
            _ => Err(format!("unknown dependency type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyStatus {
    Active,
    Evaluating,
    Deprecated,
    Migrating,
}

impl std::fmt::Display for DependencyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Evaluating => write!(f, "evaluating"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Migrating => write!(f, "migrating"),
        }
    }
}

impl std::str::FromStr for DependencyStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "active" => Ok(Self::Active),
            "evaluating" => Ok(Self::Evaluating),
            "deprecated" => Ok(Self::Deprecated),
            "migrating" => Ok(Self::Migrating),
            _ => Err(format!("unknown dependency status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
    #[serde(rename = "connection-string-env", default, skip_serializing_if = "Option::is_none")]
    pub connection_string_env: Option<String>,
    #[serde(rename = "health-endpoint", default, skip_serializing_if = "Option::is_none")]
    pub health_endpoint: Option<String>,
    #[serde(rename = "base-url", default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(rename = "auth-env", default, skip_serializing_if = "Option::is_none")]
    pub auth_env: Option<String>,
    #[serde(rename = "rate-limit", default, skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<String>,
    #[serde(rename = "error-model", default, skip_serializing_if = "Option::is_none")]
    pub error_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    #[serde(rename = "storage-min-gb", default, skip_serializing_if = "Option::is_none")]
    pub storage_min_gb: Option<u32>,
    #[serde(rename = "storage-device-pattern", default, skip_serializing_if = "Option::is_none")]
    pub storage_device_pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub front: DependencyFrontMatter,
    pub body: String,
    pub path: PathBuf,
}
