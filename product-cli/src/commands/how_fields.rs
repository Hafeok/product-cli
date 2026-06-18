//! Shared `--field` flags for `product how add`/`set`, with typed builders.

use clap::Args;
use product_core::pf::how::*;

/// The superset of fields used to build any How element from CLI flags.
#[derive(Args, Default)]
pub struct HowFields {
    #[arg(long)]
    pub decision: Option<String>,
    #[arg(long)]
    pub rationale: Option<String>,
    #[arg(long = "applies-when")]
    pub applies_when: Option<String>,
    #[arg(long = "does-not-apply-when")]
    pub does_not_apply_when: Option<String>,
    #[arg(long)]
    pub statement: Option<String>,
    #[arg(long)]
    pub shape: Option<String>,
    #[arg(long)]
    pub surface: Option<String>,
    #[arg(long)]
    pub standard: Option<String>,
    #[arg(long)]
    pub language: Option<String>,
    #[arg(long)]
    pub runtime: Option<String>,
    #[arg(long = "feature-organization")]
    pub feature_organization: Option<String>,
    #[arg(long = "persistence-model")]
    pub persistence_model: Option<String>,
    #[arg(long)]
    pub satisfies: Option<String>,
    #[arg(long = "satisfies-statement")]
    pub satisfies_statement: Option<String>,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub choice: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub licenses: Vec<String>,
    #[arg(long = "licensed-by", value_delimiter = ',')]
    pub licensed_by: Vec<String>,
    #[arg(long = "realizes", value_delimiter = ',')]
    pub realizes: Vec<String>,
    #[arg(long = "realized-by", value_delimiter = ',')]
    pub realized_by: Vec<String>,
    #[arg(long = "applied-by", value_delimiter = ',')]
    pub applied_by: Vec<String>,
    #[arg(long = "enforced-by", value_delimiter = ',')]
    pub enforced_by: Vec<String>,
    #[arg(long = "derived-from", value_delimiter = ',')]
    pub derived_from: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub layering: Vec<String>,
    #[arg(long = "cross-cutting", value_delimiter = ',')]
    pub cross_cutting: Vec<String>,
    #[arg(long = "depends-on", value_delimiter = ',')]
    pub depends_on: Vec<String>,
}

impl HowFields {
    pub fn decision(&self, id: &str) -> TopDecision {
        TopDecision {
            id: id.to_string(),
            decision: self.decision.clone().unwrap_or_default(),
            rationale: self.rationale.clone().unwrap_or_default(),
            applies_when: self.applies_when.clone(),
            does_not_apply_when: self.does_not_apply_when.clone(),
            licenses: self.licenses.clone(),
            enforced_by: self.enforced_by.clone(),
        }
    }

    pub fn principle(&self, id: &str) -> Principle {
        Principle {
            id: id.to_string(),
            statement: self.statement.clone().unwrap_or_default(),
            licensed_by: self.licensed_by.clone(),
            realized_by: self.realized_by.clone(),
            enforced_by: self.enforced_by.clone(),
        }
    }

    pub fn pattern(&self, id: &str) -> Pattern {
        Pattern {
            id: id.to_string(),
            shape: self.shape.clone().unwrap_or_default(),
            realizes: self.realizes.clone(),
            applied_by: self.applied_by.clone(),
            enforced_by: self.enforced_by.clone(),
        }
    }

    pub fn interface(&self, id: &str) -> InterfaceContract {
        InterfaceContract {
            id: id.to_string(),
            surface: self.surface.clone().unwrap_or_default(),
            standard: self.standard.clone().unwrap_or_default(),
            derived_from: self.derived_from.clone(),
        }
    }

    pub fn app_contract(&self, id: &str) -> ApplicationContract {
        ApplicationContract {
            id: id.to_string(),
            language: self.language.clone().unwrap_or_default(),
            runtime: self.runtime.clone(),
            layering: self.layering.clone(),
            feature_organization: self.feature_organization.clone(),
            persistence_model: self.persistence_model.clone(),
            cross_cutting: self.cross_cutting.clone(),
            statements: vec![],
        }
    }

    pub fn app_statement(&self, id: &str) -> ContractStatement {
        ContractStatement {
            id: id.to_string(),
            statement: self.statement.clone().unwrap_or_default(),
            enforced_by: self.enforced_by.first().cloned(),
        }
    }

    pub fn infra_contract(&self, id: &str) -> InfrastructureContract {
        InfrastructureContract {
            id: id.to_string(),
            satisfies: self.satisfies.clone().unwrap_or_default(),
            frozen: true, // §4.2: chosen per instance and frozen at Discovery
            resources: vec![],
        }
    }

    pub fn resource(&self, id: &str) -> Resource {
        Resource {
            id: id.to_string(),
            kind: self.kind.clone().unwrap_or_default(),
            choice: self.choice.clone().unwrap_or_default(),
            satisfies_statement: self.satisfies_statement.clone(),
            depends_on: self.depends_on.clone(),
        }
    }
}
