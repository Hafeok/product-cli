//! Product-Framework What-capture subsystem — domain/event model (ADR-053).
//!
//! Implements the graph behind `product author domain` (FT-109): a typed model of the
//! framework's "What" layer (§3.1 structure, §3.2 behaviour), an in-loop
//! conformance checker mirroring `schema/shapes/shapes.shacl.ttl`, Turtle
//! export + seed, the open-questions facilitation driver, and the session
//! container that the domain MCP server drives. Pure library — no MCP, no CLI.

pub mod edit;
pub mod ids;
pub mod model;
pub mod ops;
pub mod provenance;
pub mod query;
pub mod questions;
pub mod seed;
pub mod session;
pub mod turtle;
pub mod validate;

pub use model::DomainGraph;
pub use ops::OpResult;
pub use session::{DomainSession, Finalized};
pub use validate::Violation;
