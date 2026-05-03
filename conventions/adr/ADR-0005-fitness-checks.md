# ADR-0005: Fitness checks for module decomposition (file + function size, SRP)

**Status:** Accepted
**Date:** 2026-05-03
**Deciders:** Engineering team
**Conventions:** [CTX001](../docs/CTX001.md), [CTX004](../docs/CTX004.md), [CTX005](../docs/CTX005.md)

## Context

This codebase has had three "fitness" checks for years, expressed as
bash scripts under `scripts/checks/` and exercised by
`tests/code_quality_tests.rs`:

- **`file-length.sh`** — 400-line hard cap, 300-line warn.
- **`function-length.sh`** — 40-statement hard cap, 30-statement warn.
- **`single-responsibility.sh`** — `//!` doc on every file, no " and ".

They worked. They also lived outside the convention pipeline this PRD
establishes: bash output, no JSON, no permalink, no ADR pointer, no
drift self-test. Editor tooling and LLM agents that already parse cargo
diagnostics had to special-case them.

## Decision

Port all three to xtask checks under the unified diagnostic format:

- **CTX001** = file length (already landed in Phase 1).
- **CTX004** = single-responsibility module doc.
- **CTX005** = function length.

Implementation uses `syn` for proper AST inspection. CTX004 walks inner
doc attributes; CTX005 visits `ItemFn`/`ImplItemFn`/`TraitItemFn` and
counts body statement lines using `proc_macro2` span info.

The bash scripts and `tests/code_quality_tests.rs` are not removed in
this PR. They remain as a redundant safety net during the bootstrap
phase. The ADR governing them (ADR-029, in `docs/adrs/`) is unchanged —
it documents the decision to *have* fitness checks. CTX001/CTX004/CTX005
document *how* those checks are now implemented.

## Alternatives considered

- **Keep the bash scripts as the canonical implementation.** Rejected:
  bash output is not consumable by editor diagnostics; output format
  cannot match `rustc`'s; no JSON; SRP and function-length checks are
  brittle (the bash function-length script is a 30-line awk regex over
  every line of every file).
- **Use `tokei` or `scc` for line counting.** Rejected: those tools
  count "code lines" with a different definition than what the rule
  needs, and don't help with the SRP doc-comment check at all.
- **Wait until a `#[derive(BoundedModule)]` macro exists and skip
  Tier 3b entirely.** Rejected: the macro doesn't exist and writing
  it is more work than the xtask checks.

## Consequences

- Three more entries in `xtask/src/checks/` and three more docs under
  `conventions/docs/`. The drift self-test validates them automatically.
- The bash scripts and `code_quality_tests.rs` are now in maintenance
  mode. New thresholds or rule changes go to xtask first; the bash
  layer is updated only to stay in sync until it's eventually retired.
- An LLM agent that runs `cargo xtask check` gets actionable
  diagnostics with a permalink, instead of needing to re-run a bash
  script and parse its prose output.

## References

- `scripts/checks/file-length.sh`, `scripts/checks/function-length.sh`,
  `scripts/checks/single-responsibility.sh` — the precursor implementations.
- `tests/code_quality_tests.rs` — the existing test layer that exercises
  the bash scripts (TC-369..TC-380, TC-402).
- `docs/adrs/ADR-029-*.md` — the original ADR establishing fitness
  checks as a category.
