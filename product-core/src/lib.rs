//! Product library — module re-exports for tests, benchmarks, integration.

pub mod author;
pub mod config;
pub mod config_sections;
pub mod demo;
pub mod error;
pub mod fileops;
pub mod guide;
pub mod pf;
pub mod root;

// Wrapper modules for canonical module structure (ADR-029)
pub mod io;
pub mod parse;
