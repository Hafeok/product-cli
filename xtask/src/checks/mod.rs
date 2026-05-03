//! Convention check registry.

use std::path::Path;

use crate::check_id::CtxId;
use crate::diagnostic::Diagnostic;

mod ctx001_file_length;
mod ctx004_single_responsibility;
mod ctx005_function_length;

/// A single workspace convention check.
///
/// The trait is the contract relied on by the drift self-test: every
/// registered `Check` must have a matching `conventions/docs/<id>.md` whose
/// frontmatter `id`/`title`/`adrs` match what the implementation reports.
pub trait Check: Send + Sync {
    /// Stable identifier (e.g. `CTX001`). Used in diagnostic codes and to
    /// locate the matching convention doc.
    fn id(&self) -> CtxId;

    /// Human-readable title; must match the `title` field in the doc.
    fn title(&self) -> &'static str;

    /// Permalink to the convention doc on the canonical branch.
    fn help_url(&self) -> String {
        format!(
            "https://github.com/Hafeok/product-cli/blob/main/conventions/docs/{}.md",
            self.id()
        )
    }

    /// Run the check across the workspace rooted at `root`.
    fn run(&self, root: &Path) -> Vec<Diagnostic>;
}

pub struct Registry {
    checks: Vec<Box<dyn Check>>,
}

impl Registry {
    pub fn default_set() -> Self {
        Self {
            checks: vec![
                Box::new(ctx001_file_length::FileLengthCheck),
                Box::new(ctx004_single_responsibility::SingleResponsibilityCheck),
                Box::new(ctx005_function_length::FunctionLengthCheck),
            ],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Check> {
        self.checks.iter().map(|c| c.as_ref())
    }
}
