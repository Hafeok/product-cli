---
id: TC-563
title: gap_bundle_outputs_context_and_instructions
type: scenario
status: failing
validates:
  features:
  - FT-045
  adrs:
  - ADR-019
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_563_gap_bundle_outputs_context_and_instructions
last-run: 2026-04-18T10:42:24.877521871+00:00
last-run-duration: 0.1s
failure-message: "No matching test function found (0 tests ran)"
---

## Session: ST-120 — gap-bundle-outputs-context-and-instructions

**Validates:** FT-045, ADR-019 (amended), ADR-040

### Given

A temp repository with an accepted ADR-002 linked to at least one feature, and a `benchmarks/prompts/gap-analysis-v1.md` prompt file.

### When

`product gap bundle ADR-002` is run.

### Then

- stdout is a single markdown document with two top-level sections: `## Instructions` and `## Context Bundle`.
- The Instructions section lists gap codes G001 through G008 with their descriptions.
- The Context Bundle section contains the depth-2 bundle for ADR-002 (the ADR, its linked features, their TCs, and related ADRs within 2 hops).
- No HTTP / LLM API call is made (verifiable via the absence of outbound network sockets during the run — in tests, via the `gap::bundle::bundle_for_adr` function signature not touching any client).
- Exit code is `0`.