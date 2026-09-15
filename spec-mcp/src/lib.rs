//! The specification flow's MCP surface.
//!
//! A **strict subset** of the CLI, never a mirror. The subset boundary is the
//! accountability leg: a model may do everything up to the decision, and may
//! not commit it. `accept`, `reject`, `close` and `policy set` each name a
//! principal, so none of them is here.
//!
//! What this boundary is, honestly: a registry that does not carry those
//! tools, plus a dispatcher that refuses their names. It is not a linkage
//! boundary — this process links `spec_core`, which contains the write paths —
//! so it is a weaker guarantee than the `spec-flow` host's, where the code to
//! write a closure is absent from the binary. The `principal` schema and the
//! `S002` gate are what hold underneath either of them.

pub mod dispatch;
pub mod serve;
pub mod tools;

pub use serve::{build_registry, serve_stdio};
