# ADR-0001: 400-line per-file hard limit on Rust source

**Status:** Accepted
**Date:** 2026-05-03
**Deciders:** Engineering team
**Convention:** [CTX001](../docs/CTX001.md)

## Context

Files in this codebase have a known failure mode: they accrete unrelated
helpers over time. Once a file passes ~500 lines, it becomes harder to read,
harder to test in isolation, and harder for an LLM agent to load relevant
context without blowing past its window. The slice + adapter pattern
documented in `CLAUDE.md` only works if files stay narrow enough for
"slice" to mean something concrete.

We already had a bash-based file-length check
(`scripts/checks/file-length.sh`) and a fitness test
(`tests/code_quality_tests.rs::tc_402_*`). Both work but live outside the
canonical convention pipeline this PRD establishes.

## Decision

Every Rust source file under `src/` and `xtask/src/` must be **at most 400
lines**. Enforcement runs as an xtask check (`cargo xtask check`) so it
participates in the same diagnostic pipeline as every other CTX rule:
stable code, permalink to the doc, ADR pointer, JSON output for editors.

The bash script and the fitness test stay in place during the bootstrap
phase. They are redundant with the xtask check by design — the xtask check
is the canonical enforcement, the bash script is the legacy entry point,
and the fitness test guards against regressions in either path.

## Alternatives considered

- **300-line limit.** Rejected: too aggressive for the existing codebase;
  would force premature splits that hurt locality.
- **Soft warn-only at 400 with no hard limit.** Rejected: warnings get
  ignored. The point of a build break is that it is not optional.
- **Per-file `#[allow(ctx001)]` escape hatch.** Rejected: the limit is the
  forcing function. An escape hatch makes it advisory.
- **rustfmt `max_width`-style soft cap.** Rejected: line width is a
  different rule; file length is about module decomposition, not formatting.

## Consequences

- Files that approach 400 lines are split before they cross. The Slice +
  Adapter pattern in `CLAUDE.md` already encourages this; the rule
  formalises it.
- Some legitimate large definitions (giant enums, hand-written tables) need
  to be moved into sibling modules. This is acceptable cost.
- The xtask check duplicates the bash script for one phase. The bash script
  may be retired once CI uses `cargo xtask check` exclusively.

## References

- PRD: *In-Workspace Convention Enforcement (Rust)* — section "Enforcement
  mechanisms (in priority order)".
- `CLAUDE.md` — "Build & Test" section documenting the existing
  `tests/code_quality_tests.rs` fitness suite.
