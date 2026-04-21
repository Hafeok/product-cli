---
id: TC-629
title: builder_status_renders_all_artifacts_with_indicators_and_counts
type: scenario
status: passing
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
runner: cargo-test
runner-args: "tc_629_builder_status_renders_all_artifacts_with_indicators_and_counts"
last-run: 2026-04-21T12:40:57.330357420+00:00
last-run-duration: 0.3s
---

## Session — builder-status-shows-indicators

### Given

A draft containing one feature (clean), one ADR with a W-class
finding (no TC linked yet), and one TC (clean).

### When

The user runs `product request status`.

### Then

- The output renders each artifact on its own block with a status
  glyph: `✓` for clean, `⚠` for W-class, `✗` for E-class.
- Each artifact line names its `ref:`, title, and key fields
  (phase/domains for feature, scope/domain for ADR, type/level for
  TC).
- Cross-references between artifacts in the draft are rendered
  using `ref:` names, not placeholder IDs.
- A footer reports `Warnings: N` and the specific W-class code(s).
- Exit code is 0 even when warnings exist.