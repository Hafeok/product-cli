//! Product library — module re-exports for tests, benchmarks, integration.

pub mod author;
pub mod config;
#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
pub mod config_author;
pub mod config_cycle_times;
pub mod config_features;
pub mod config_migrate;
pub mod config_observability;
pub mod config_paths;
pub mod config_patterns;
pub mod config_planning;
pub mod config_request_builder;
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
