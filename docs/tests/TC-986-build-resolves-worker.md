---
id: TC-986
title: build resolves worker by role
type: scenario
status: passing
validates:
  features:
  - FT-131
  adrs:
  - ADR-072
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_986_build_resolves_worker_by_role
---

## Scenario — build dispatches by role → capability

**Given** a deliverable over a captured What graph and a scaffolded worker
catalog,
**When** the user runs `product build place-order --dry-run`,
**Then** it exits 0 and stdout reports the resolved worker — a `--- Worker ---`
section naming capability `claude-code`.

## Validates

- FT-131 — worker capability catalog — role to capability with escalation, claude or litellm
- ADR-072 — Workers are capabilities resolved by role with escalation; runners are claude or litellm
