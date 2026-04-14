//! Formal block parser for AISP-influenced notation in test criteria (ADR-011, ADR-016)

pub mod parser;
pub mod blocks;

// Re-export public API
pub use parser::{
    aggregate_evidence, parse_formal_blocks, parse_formal_blocks_with_diagnostics,
    FormalParseResult,
};
pub use blocks::{
    EvidenceBlock, ExitField, FormalBlock, Invariant, ScenarioBlock, Stability, TypeDef,
};

#[cfg(test)]
mod tests;
