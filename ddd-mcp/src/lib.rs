//! The `ddd_*` MCP tool namespace served through product-mcp's plumbing.
//!
//! PRD §5.1/§8: language-intelligence tools (M3) plus the interceptor
//! with governance tools (M4), dispatched against the repo-local `.ddd/`
//! graph plus per-language LSP hosts. The core here never names a
//! language beyond routing into `ddd-lsp`'s adapter registry.

pub mod dispatch;
pub mod govern_tools;
pub mod intercept;
pub mod lang_tools;
pub mod serve;
pub mod state;
pub mod tools;

pub use serve::serve_stdio;
