//! Code extraction into ground — the §5.2 declaration-level pipeline.
//!
//! Model-free by ruling, and rung zero: deterministic reads of a codebase,
//! per-language mapping rules, RDFS/OWL-RL entailment, CONSTRUCT rules to a
//! fixpoint for the usage-inferred relations, SHACL over the result. There is
//! no candidate-generation step in this path, so there is nothing for a model
//! to do — extraction is *constructively* closed, which is why Feature 3
//! needs no model where the loop does.
//!
//! The slice is pure. It reads no language server: the reading half fills
//! [`facts::CorpusFacts`] and hands it over, which is what keeps
//! `product-core` free of any dependency on the governance stack.
//!
//! Output is a **proposal**, never ground. Ratification is a reviewer
//! merging one file at a time.

mod apply;
mod declared;
pub mod entail;
pub mod facts;
pub(crate) mod mint;
pub mod plan;
mod propose;
mod report;
pub mod rows;
mod structural;
pub mod triple;
mod vocab;
mod write;

pub use apply::{apply_extraction, ExtractionReport};
pub use plan::{plan_extraction, ExtractionParams, ExtractionPlan, PlannedFile, RowResult};
pub use report::render as render_extraction;

/// The extractor's own version, recorded in every Reading's assurance
/// through the instrument string the reader supplies.
pub const EXTRACTOR_VERSION: &str = env!("CARGO_PKG_VERSION");
