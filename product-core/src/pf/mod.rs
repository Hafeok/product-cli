//! Product-Framework What-capture subsystem — domain/event model (ADR-053).
//!
//! Implements the graph behind `product author domain` (FT-109): a typed model of the
//! framework's "What" layer (§3.1 structure, §3.2 behaviour), an in-loop
//! conformance checker mirroring `schema/shapes/shapes.shacl.ttl`, Turtle
//! export + seed, the open-questions facilitation driver, and the session
//! container that the domain MCP server drives. Pure library — no MCP, no CLI.

pub mod archetype;
pub mod archetype_turtle;
pub mod build;
pub mod build_metrics;
pub mod build_spmc;
pub mod bundle;
pub mod capability;
pub mod cell;
pub mod cell_validate;
pub mod decider;
pub mod decider_cel;
pub mod decider_conform;
pub mod decider_logic;
pub mod decider_sim;
pub mod decider_turtle;
pub mod deliverable;
pub mod dispatch;
pub mod done;
pub mod edit;
pub mod how;
pub mod how_edit;
pub mod layout;
pub mod layout_check;
pub mod manifest;
pub mod manifest_content;
pub mod lsp;
pub mod how_turtle;
pub mod how_validate;
pub mod ids;
pub mod data_check;
pub mod build_seam;
pub mod model;
pub mod model_data;
pub mod model_product;
pub mod model_ui;
pub mod ops;
pub mod primitive;
pub mod projector;
pub mod projector_logic;
pub mod projector_sim;
pub mod provenance;
pub mod query;
pub mod questions;
pub mod release;
pub mod render_contract;
pub mod rules_data;
pub mod rules_decider;
pub mod rules_how;
pub mod rules_pattern;
pub mod rules_reify;
pub mod rules_ui;
pub mod rules_what;
pub mod run;
pub mod schedule;
pub mod seam;
pub mod seed;
pub mod seed_canon;
pub mod seed_data;
pub mod seed_ui;
pub mod session;
pub mod feature;
pub mod sparql_rules;
pub mod target;
pub mod turtle;
pub mod turtle_data;
pub mod turtle_product;
pub mod turtle_ui;
pub mod validate;
pub mod validate_product;
pub mod verify;
pub mod viz;
pub mod wcag22;
pub mod work_unit;
pub mod work_unit_validate;
pub mod worker;
pub mod workflow;

pub use how::HowContract;
pub use model::DomainGraph;
pub use ops::OpResult;
pub use session::{DomainSession, Finalized};
pub use validate::Violation;
pub use workflow::{Phase, WorkflowSession};
