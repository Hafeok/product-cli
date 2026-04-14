//! Product library — module re-exports for tests, benchmarks, integration.

pub mod author;
pub mod checklist;
pub mod config;
pub mod context;
pub mod domains;
pub mod drift;
pub mod error;
pub mod fileops;
pub mod formal;
pub mod gap;
pub mod graph;
pub mod hash;
pub mod implement;
pub mod mcp;
pub mod metrics;
pub mod migrate;
pub mod onboard;
pub mod parser;
pub mod rdf;
pub mod types;

// Wrapper modules for canonical module structure (ADR-029)
pub mod io;
pub mod parse;
pub mod verify;
