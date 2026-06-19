---
id: TC-993
title: build lsp fix loop converges
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
runner-args: tc_993_build_lsp_fix_loop_converges
---

## Scenario — the LSP diagnose→fix loop re-dispatches, escalates, converges

**Given** a scripted worker and scripted diagnostics (`PRODUCT_MOCK_LSP`) whose
first diagnose reports a lint and whose second is clean,
**When** the user runs `product build conv --role coder --lsp --no-verify`,
**Then** the LSP gate surfaces the diagnostic, **re-dispatches up the capability
ladder** to fix it, the next diagnose is clean, and the file is marked clean —
giving the diagnose→fix loop deterministic CI coverage without a cargo project +
rust-analyzer.

## Validates

- FT-131 — product build gates — the LSP diagnose→fix loop
- ADR-071 — verification is recorded, not judged
