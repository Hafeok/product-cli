//! Concern domain classification for pre-flight analysis (ADR-025, ADR-026)
//!
//! Domain classification, cross-cutting scope, preflight checks,
//! coverage matrix, feature acknowledgement.

pub mod preflight;
pub mod coverage;
pub mod validation;

// Re-export public API
pub use coverage::{
    build_coverage_matrix, coverage_matrix_to_json, render_coverage_matrix,
    render_coverage_matrix_filtered, CoverageCell, CoverageMatrix,
};
pub use preflight::{
    acknowledge_adr, acknowledge_domain, preflight, render_preflight,
    CoverageStatus, CrossCuttingGap, DomainGap, PreflightResult,
};
pub use validation::validate_domains;

#[cfg(test)]
mod tests;
