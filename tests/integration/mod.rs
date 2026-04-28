//! Integration test registry — declares topic submodules.
//! See tests/integration/harness.rs for the shared Harness/Output/fixtures.

pub mod harness;

mod adr;
mod agent_context;
mod author;
mod checklist;
mod context;
mod dep;
mod drift;
mod error_codes;
mod feature;
mod ft_exit_criteria;
mod gap;
mod graph;
mod init;
mod link;
mod mcp;
mod metrics;
mod misc;
mod parser;
mod planning;
mod preflight;
mod product_cli;
mod request;
mod request_log;
mod schema;
mod slice;
mod status;
mod tags;
mod tc;
mod verify;
mod warnings;
