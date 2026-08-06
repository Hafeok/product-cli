//! Decision-Driven Design governance graph — the `.ddd/` store library.
//!
//! Implements PRD M1 (`docs/ddd-cli-prd.md`): a repo-local YAML graph of
//! predicates, closure claims, decisions, analyzer/linter manifests, pattern
//! instances, seam declarations; ontology validation via the `pf` SPARQL rule
//! engine over a `ddd:` Turtle projection; `why` resolution from any governed
//! id to its decision chain. Own ontology, own namespace — nothing here
//! extends the What/How vocabulary. Pure library: no clap, no MCP.

pub mod bicepconfig;
pub mod cargolints;
pub mod claim;
pub mod config;
pub mod configured;
pub mod decision;
pub mod detect;
pub mod diff;
pub mod editorconfig;
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
pub mod turtle;
pub mod validate;
pub mod why;

pub use store::DddStore;
pub use validate::validate_store;
