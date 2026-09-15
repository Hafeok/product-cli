//! The flow's exit codes, as `docs/spec-flow-store-v1.md` §6 fixes them.

/// Conformant.
pub const CONFORMANT: i32 = 0;
/// Findings.
pub const FINDINGS: i32 = 1;
/// Could not run.
pub const COULD_NOT_RUN: i32 = 2;
/// Work completed, closure pending.
///
/// Distinct from [`FINDINGS`] on purpose: it is what lets an agent harness
/// report "I finished my half" without it reading as "I broke something".
/// It is not success, and `implement` returns it on every unattended run.
pub const PENDING_CLOSURE: i32 = 3;
