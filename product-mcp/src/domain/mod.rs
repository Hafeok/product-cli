//! Domain-authoring MCP server (the What-capture tool surface; ADR-053).
//!
//! Exposes the 17 tools of `product-author-domain.tools.json` over stdio,
//! driving a stateful [`product_core::pf`] session: structured graph
//! operations that validate in-loop, never raw Turtle emission. Launched by
//! `product author domain` as the MCP server an LLM facilitation client talks
//! to. Separate from the main `product mcp` surface (a different graph).

mod args;
mod handlers;
pub mod registry;
mod session_handlers;
pub mod stdio;
pub mod tools;

pub use registry::DomainRegistry;
pub use stdio::run_domain_stdio;
