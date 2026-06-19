---
id: TC-991
title: build verify fix loop converges
type: scenario
status: passing
validates:
  features:
  - FT-131
  adrs:
  - ADR-071
phase: 6
observes:
- stdout
- disk-state
runner: cargo-test
runner-args: tc_991_build_verify_fix_loop_converges
---

## Scenario — the verify fix loop re-dispatches, escalates, and converges

**Given** a deliverable with a shell acceptance runner and a scripted worker
(`PRODUCT_MOCK_DIR`) whose first response fails the runner and whose second
passes it,
**When** the user runs `product build conv --role coder`,
**Then** the verify gate runs the runner, sees it fail, **re-dispatches up the
capability ladder** (`code-writer` → `code-writer-heavy`), the second response
makes the runner pass, and the deliverable reaches `DONE` — proving the
deterministic diagnose→fix→re-run loop converges without a live model.

## Validates

- FT-131 — product build gates — the escalating verify fix loop
- ADR-071 — verification is recorded, not judged; done is a predicate over recorded checks
