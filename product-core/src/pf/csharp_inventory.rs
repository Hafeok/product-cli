//! The C# inventory artefact — the versioned fact file the .NET reader emits.
//!
//! Mirrors `schema/json/csharp-inventory/inventory.schema.json` (version 5).
//! Loading refuses any `inventory_version` not in [`KNOWN_INVENTORY_VERSIONS`]
//! before another field is read. Facts only: nothing here classifies.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ProductError, Result};

/// The inventory versions this consumer understands.
pub const KNOWN_INVENTORY_VERSIONS: &[&str] = &["5"];

/// Test-framework assemblies, by name: a project referencing one is a test
/// project (CG-R-75 — classified by project, mechanically, never by reach).
pub const TEST_FRAMEWORK_ASSEMBLIES: &[&str] = &[
    "xunit.core", "xunit.v3.core", "nunit.framework", "MSTest.TestFramework",
    "Microsoft.VisualStudio.TestPlatform.TestFramework", "TUnit.Core",
];

/// The vendored schema, applied unchanged.
pub const INVENTORY_SCHEMA: &str =
    include_str!("../../../schema/json/csharp-inventory/inventory.schema.json");

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProducedBy {
    pub tool: String,
    pub tool_version: String,
    #[serde(default)]
    pub roslyn_version: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Solution {
    pub path: String,
    #[serde(default)]
    pub git_head: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub target_frameworks: Vec<String>,
    #[serde(default)]
    pub assembly: String,
    /// Every assembly the compilation references, by name (v4).
    #[serde(default)]
    pub referenced_assemblies: Vec<String>,
    /// Razor files the workspace lists; their generated classes are not in
    /// the compilation — the blind spot's extent (CG-R-78, v5).
    #[serde(default)]
    pub razor_files: u64,
    /// `@inject` lines in those files: composition edges the instrument cannot see.
    #[serde(default)]
    pub razor_inject_directives: u64,
}

impl Project {
    pub fn is_test(&self) -> bool {
        self.referenced_assemblies.iter().any(|a| TEST_FRAMEWORK_ASSEMBLIES.contains(&a.as_str()))
    }
}

/// One declared attribute with its arguments, as the compiler resolved them.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AttributeUse {
    #[serde(rename = "type")]
    pub attribute_type: String,
    #[serde(default)]
    pub positional: Vec<Value>,
    #[serde(default)]
    pub named: BTreeMap<String, Value>,
}

impl AttributeUse {
    /// Positional argument `i` as a string, if it is one.
    pub fn arg(&self, i: usize) -> Option<&str> {
        self.positional.get(i).and_then(Value::as_str)
    }

    /// Named argument as a string, if present and a string.
    pub fn named_str(&self, key: &str) -> Option<&str> {
        self.named.get(key).and_then(Value::as_str)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TypeFact {
    pub id: String,
    pub project: String,
    #[serde(default)]
    pub namespace: String,
    pub name: String,
    pub kind: String,
    pub accessibility: String,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub is_abstract: bool,
    #[serde(default)]
    pub is_partial: bool,
    #[serde(default)]
    pub base_type: Option<String>,
    #[serde(default)]
    pub interfaces: Vec<String>,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub line: u64,
    #[serde(default)]
    pub attributes: Vec<AttributeUse>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub parameter_type: String,
    /// The parameter type's type arguments (v5, P-6): `IEnumerable<IFoo>` carries `IFoo`.
    #[serde(default)]
    pub type_arguments: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MemberFact {
    pub id: String,
    pub declaring_type: String,
    pub name: String,
    pub kind: String,
    pub accessibility: String,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub is_entry_point: bool,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub return_type: Option<String>,
    /// The return/property/field type's type arguments (v5, P-6).
    #[serde(default)]
    pub return_type_arguments: Vec<String>,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub line: u64,
    #[serde(default)]
    pub attributes: Vec<AttributeUse>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Reference {
    pub from: String,
    pub to: String,
    pub kind: String,
}

/// A type declared outside the solution that an edge lands on, with the
/// member shape the role proxies read.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExternalType {
    pub id: String,
    #[serde(default)]
    pub assembly: String,
    #[serde(default)]
    pub namespace: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub is_abstract: bool,
    #[serde(default)]
    pub arity: u32,
    #[serde(default)]
    pub methods: u32,
    #[serde(default)]
    pub properties: u32,
    #[serde(default)]
    pub events: u32,
    #[serde(default)]
    pub abstract_returns: Vec<String>,
    /// The base class, itself an external type (v4).
    #[serde(default)]
    pub base_type: Option<String>,
    /// Directly declared base interfaces, each itself an external type, so
    /// member counts can be summed over the chain (v4, P-1).
    #[serde(default)]
    pub interfaces: Vec<String>,
}

/// One call on an `IServiceCollection` — or on a builder such a call
/// returned in the same statement (v4, R-2) — as written. Which calls
/// register what is decided in `csharp_di`, not here.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Registration {
    pub site: String,
    /// The receiver's type: `IServiceCollection` or the builder reached.
    #[serde(default)]
    pub receiver: String,
    pub method: String,
    pub method_name: String,
    #[serde(default)]
    pub type_arguments: Vec<String>,
    #[serde(default)]
    pub typeof_arguments: Vec<String>,
    #[serde(default)]
    pub constructs: Vec<String>,
    #[serde(default)]
    pub has_lambda: bool,
    #[serde(default)]
    pub conditional: bool,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub line: u64,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Diagnostic {
    pub severity: String,
    #[serde(default)]
    pub project: Option<String>,
    pub message: String,
}

/// The whole artefact.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Inventory {
    pub inventory_version: String,
    pub produced_by: ProducedBy,
    pub produced_at: String,
    pub solution: Solution,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub types: Vec<TypeFact>,
    #[serde(default)]
    pub members: Vec<MemberFact>,
    #[serde(default)]
    pub references: Vec<Reference>,
    #[serde(default)]
    pub external_types: Vec<ExternalType>,
    #[serde(default)]
    pub registrations: Vec<Registration>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse an inventory. The version is checked first; an unknown version is
/// refused with the offending value named, before any other field is read.
pub fn load_inventory(text: &str) -> Result<Inventory> {
    let value: Value = serde_json::from_str(text)
        .map_err(|e| ProductError::ConfigError(format!("inventory is not valid JSON: {e}")))?;
    check_version(&value)?;
    serde_json::from_value(value)
        .map_err(|e| ProductError::ConfigError(format!("inventory: {e}")))
}

fn check_version(value: &Value) -> Result<()> {
    let version = value
        .get("inventory_version")
        .and_then(Value::as_str)
        .ok_or_else(|| ProductError::ConfigError(
            "inventory has no `inventory_version` — refused rather than parsed optimistically".into(),
        ))?;
    if !KNOWN_INVENTORY_VERSIONS.contains(&version) {
        return Err(ProductError::ConfigError(format!(
            "inventory_version '{version}' is not known to this consumer (known: {})",
            KNOWN_INVENTORY_VERSIONS.join(", ")
        )));
    }
    Ok(())
}

/// Validate a parsed inventory against the vendored schema. Empty = conformant.
pub fn schema_findings(value: &Value) -> Result<Vec<String>> {
    let schema: Value = serde_json::from_str(INVENTORY_SCHEMA)
        .map_err(|e| ProductError::ConfigError(format!("inventory schema: {e}")))?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| ProductError::ConfigError(format!("inventory schema: {e}")))?;
    Ok(validator
        .iter_errors(value)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect())
}

/// Lookups over one inventory: types, members, and out-edges by source.
pub struct Index<'a> {
    pub types: BTreeMap<&'a str, &'a TypeFact>,
    pub members: BTreeMap<&'a str, &'a MemberFact>,
    pub out: BTreeMap<&'a str, Vec<&'a Reference>>,
    /// Interface/base → the types implementing/inheriting it.
    pub implementors: BTreeMap<&'a str, Vec<&'a str>>,
    /// Type → its members.
    pub members_of: BTreeMap<&'a str, Vec<&'a str>>,
    /// Types a caller cannot instantiate: interfaces and abstract classes,
    /// in the solution or outside it.
    pub abstract_types: BTreeSet<&'a str>,
    /// Abstractions declared outside the solution, by id.
    pub external: BTreeMap<&'a str, &'a ExternalType>,
    /// Projects referencing a test framework (CG-R-75): outside the primary
    /// convention, reported separately.
    pub test_projects: BTreeSet<&'a str>,
}

impl Index<'_> {
    /// Does the type live in a test project?
    pub fn is_test(&self, type_id: &str) -> bool {
        self.types.get(type_id).is_some_and(|t| self.test_projects.contains(t.project.as_str()))
    }

    /// In-solution types implementing or inheriting `id`, outside test projects.
    pub fn production_implementors(&self, id: &str) -> impl Iterator<Item = &str> + '_ {
        self.implementors.get(id).into_iter().flatten().copied().filter(move |t| !self.is_test(t))
    }
}

impl Inventory {
    pub fn index(&self) -> Index<'_> {
        let mut out: BTreeMap<&str, Vec<&Reference>> = BTreeMap::new();
        let mut implementors: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for r in &self.references {
            out.entry(r.from.as_str()).or_default().push(r);
            if r.kind == "implement" || r.kind == "inherit" {
                implementors.entry(r.to.as_str()).or_default().push(r.from.as_str());
            }
        }
        let mut members_of: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for m in &self.members {
            members_of.entry(m.declaring_type.as_str()).or_default().push(m.id.as_str());
        }
        Index {
            types: self.types.iter().map(|t| (t.id.as_str(), t)).collect(),
            members: self.members.iter().map(|m| (m.id.as_str(), m)).collect(),
            out,
            implementors,
            members_of,
            abstract_types: self
                .types
                .iter()
                .filter(|t| t.kind == "interface" || t.is_abstract)
                .map(|t| t.id.as_str())
                .chain(self.external_types.iter().filter(|e| e.kind == "interface" || e.is_abstract).map(|e| e.id.as_str()))
                .collect(),
            external: self.external_types.iter().map(|e| (e.id.as_str(), e)).collect(),
            test_projects: self.projects.iter().filter(|p| p.is_test()).map(|p| p.id.as_str()).collect(),
        }
    }

    /// Types carrying an attribute of the given type id, with each use.
    pub fn types_with_attribute<'a>(&'a self, attribute_type: &str) -> Vec<(&'a TypeFact, &'a AttributeUse)> {
        self.types
            .iter()
            .flat_map(|t| t.attributes.iter().filter(|a| a.attribute_type == attribute_type).map(move |a| (t, a)))
            .collect()
    }

    /// Members carrying an attribute of the given type id.
    pub fn members_with_attribute<'a>(&'a self, attribute_type: &str) -> Vec<&'a MemberFact> {
        self.members.iter().filter(|m| m.attributes.iter().any(|a| a.attribute_type == attribute_type)).collect()
    }

    /// The namespaces present, each with its type count.
    pub fn namespaces(&self) -> BTreeMap<&str, usize> {
        let mut out = BTreeMap::new();
        for t in &self.types {
            *out.entry(t.namespace.as_str()).or_insert(0) += 1;
        }
        out
    }

    /// Every type id, as a set.
    pub fn type_ids(&self) -> BTreeSet<&str> {
        self.types.iter().map(|t| t.id.as_str()).collect()
    }
}

#[cfg(test)]
#[path = "csharp_inventory_tests.rs"]
mod tests;
