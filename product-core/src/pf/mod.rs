//! Product-Framework What-capture subsystem — domain/event model (ADR-053).
//!
//! Implements the graph behind `product author domain` (FT-109): a typed model of the
//! framework's "What" layer (§3.1 structure, §3.2 behaviour), an in-loop
//! conformance checker mirroring `schema/shapes/shapes.shacl.ttl`, Turtle
//! export + seed, the open-questions facilitation driver, and the session
//! container that the domain MCP server drives. Pure library — no MCP, no CLI.

pub mod archetype;
pub mod archetype_turtle;
pub mod bundle;
pub mod cell;
pub mod cell_validate;
pub mod decider;
pub mod decider_cel;
pub mod decider_logic;
pub mod decider_sim;
pub mod decider_turtle;
pub mod dispatch;
pub mod edit;
pub mod how;
pub mod how_edit;
pub mod layout;
pub mod layout_check;
pub mod how_turtle;
pub mod how_validate;
pub mod ids;
pub mod model;
pub mod ops;
pub mod provenance;
pub mod query;
pub mod questions;
pub mod rules_decider;
pub mod rules_how;
pub mod rules_what;
pub mod seed;
pub mod session;
pub mod sparql_rules;
pub mod turtle;
pub mod validate;
pub mod work_unit;
pub mod work_unit_validate;

pub use how::HowContract;
pub use model::DomainGraph;
pub use ops::OpResult;
pub use session::{DomainSession, Finalized};
pub use validate::Violation;
