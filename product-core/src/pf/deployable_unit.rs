//! DeployableUnit — the concrete artifact a blueprint produces (§4, §4.2).
//!
//! A **DeployableUnit** is the concrete deployable thing — an app, a service, a
//! CLI binary — that a blueprint (a reusable How) is *instantiated as* to realise
//! a system (§3.2.5), carrying that system's §4.2 deployment identity (domain
//! name, bundle id, runtime). The blueprint → system → DeployableUnit mapping is
//! 1:1:1 in the common case but may fan out (a frontend + a backend-for-frontend
//! from one blueprint; or several systems shipped as one unit), so `deploys_system`
//! is a list. Two environments (staging, production) are two DeployableUnits of
//! the same system on the same blueprint — which is where §4.2's "deployment
//! identity varies per environment" attaches. Borrowed from DORA/DevOps, where
//! flow to production is measured *per deployable unit*.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::ids::NodeKind;
use super::model::DomainGraph;
use super::validate::Violation;

/// §4.2 — the deployment identity a DeployableUnit carries (per environment).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeploymentIdentity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
}

impl DeploymentIdentity {
    /// True when the identity carries no address at all — the §4.2 debt case.
    pub fn is_empty(&self) -> bool {
        self.domain_name.is_none() && self.bundle_id.is_none() && self.runtime.is_none()
    }
}

/// A concrete deployable artifact instantiated from a blueprint (§4).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeployableUnit {
    pub id: String,
    /// The blueprint (reusable How) this unit is built from (`built_from`).
    pub built_from: String,
    /// The system(s) (§3.2.5) this unit deploys/realises; a list to allow the
    /// monolith fan-out (several systems shipped as one unit).
    #[serde(default)]
    pub deploys_system: Vec<String>,
    /// The deployment environment this unit exists in (e.g. production, staging).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// The §4.2 deployment identity this unit carries.
    #[serde(default)]
    pub identity: DeploymentIdentity,
}

impl DeployableUnit {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid deployable-unit YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize deployable-unit: {}", e)))
    }

    /// A scaffold unit for `product deployable-unit new` when fields are sparse.
    pub fn scaffold(id: &str, built_from: &str, system: &str) -> Self {
        Self {
            id: id.to_string(),
            built_from: built_from.to_string(),
            deploys_system: if system.is_empty() { vec![] } else { vec![system.to_string()] },
            environment: Some("production".to_string()),
            identity: DeploymentIdentity::default(),
        }
    }
}

/// Validate a DeployableUnit (§4/§4.2). `graph` (when present) resolves
/// `deploys_system` to real System nodes; `known_blueprints` (when non-empty)
/// resolves `built_from` to a real blueprint.
pub fn validate_deployable_unit(
    du: &DeployableUnit,
    graph: Option<&DomainGraph>,
    known_blueprints: &[String],
) -> Vec<Violation> {
    let mut out = Vec::new();

    // built_from — the blueprint it instantiates.
    if du.built_from.trim().is_empty() {
        out.push(v(&du.id, "built_from",
            "§4 A DeployableUnit must be built_from a blueprint (a reusable How)."));
    } else if !known_blueprints.is_empty() && !known_blueprints.iter().any(|b| b == &du.built_from) {
        out.push(v(&du.id, "built_from",
            &format!("§4 built_from '{}' is not a known blueprint under .product/blueprints/.", du.built_from)));
    }

    // deploys_system — the system(s) it realises.
    if du.deploys_system.is_empty() {
        out.push(v(&du.id, "deploys_system",
            "§4 A DeployableUnit must deploy at least one system (§3.2.5)."));
    }
    if let Some(g) = graph {
        for s in &du.deploys_system {
            if !g.is_kind(s, NodeKind::System) {
                out.push(v(&du.id, "deploys_system",
                    &format!("§3.2.5 deploys_system '{s}' is not a System node in the captured What graph.")));
            }
        }
    }

    // carries_identity — §4.2 deployment identity is what a unit carries.
    if du.identity.is_empty() {
        out.push(v(&du.id, "identity",
            "§4.2 A DeployableUnit must carry a deployment identity (domain_name, bundle_id, or runtime)."));
    }

    out
}

/// Load every `*.yaml` DeployableUnit under `dir` (`.product/deployable-units/`),
/// sorted by filename. Unreadable/invalid files are skipped.
pub fn load_dir(dir: &Path) -> Vec<DeployableUnit> {
    let mut paths: Vec<_> = match std::fs::read_dir(dir) {
        Ok(it) => it.flatten().map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml")).collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    paths.iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .filter_map(|t| DeployableUnit::from_yaml(&t).ok())
        .collect()
}

/// Blueprint names on disk: directory entries under `blueprints_dir`, sorted.
pub fn blueprint_names(blueprints_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(blueprints_dir) {
        Ok(it) => it.flatten().filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok()).collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: "violation".to_string(),
    }
}

#[cfg(test)]
#[path = "deployable_unit_tests.rs"]
mod tests;
