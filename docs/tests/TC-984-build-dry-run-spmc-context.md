---
id: TC-984
title: build dry-run assembles the SPMC context
type: scenario
status: passing
validates:
  features:
  - FT-130
  adrs:
  - ADR-071
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_984_build_dry_run_assembles_the_spmc_context
---

## Scenario — build assembles the frozen SPMC context

**Given** a slice over a captured What graph and a deliverable on it,
**When** the user runs `product build place-order --dry-run`,
**Then** the process exits 0 and stdout contains the `Build Context` with `## What`
(the slice subgraph, including `PlaceOrder`), `## Acceptance`, and a `Gate status`
section reporting `not done` (acceptance pending).

## Validates

- FT-130 — product build — the SPMC build orchestrator that records conformance into done
- ADR-071 — build assembles the SPMC context; conformance is recorded into done
