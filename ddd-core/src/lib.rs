//! Decision-Driven Design governance graph — the `.ddd/` store library.
//!
//! Implements PRD M1 (`docs/ddd-v1-spec.md`): a repo-local YAML graph of
//! predicates, closure claims, decisions, analyzer/linter manifests, pattern
//! instances, seam declarations; ontology validation via the `pf` SPARQL rule
//! engine over a `ddd:` Turtle projection; `why` resolution from any governed
//! id to its decision chain. Own ontology, own namespace — nothing here
//! extends the What/How vocabulary. Pure library: no clap, no MCP.

pub mod basis_pin;
pub mod bicepconfig;
pub mod binding;
pub mod cargolints;
pub mod claim;
pub mod concordance;
pub mod config;
pub mod configured;
pub mod contracts;
pub mod decision;
pub mod detect;
pub mod diff;
pub mod gitrev;
pub mod editorconfig;
pub mod htmlvalidateconfig;
pub mod init;
pub mod manifest;
pub mod pattern;
pub mod predicate;
pub mod render;
pub mod report;
pub mod rules;
pub mod sarif;
pub mod seam;
pub mod seam_event;
pub mod store;
pub mod stylelintconfig;
pub mod surface;
pub mod tokenfile;
pub mod turtle;
pub mod validate;
pub mod what;
pub mod why;

pub use store::DddStore;
pub use validate::validate_store;
