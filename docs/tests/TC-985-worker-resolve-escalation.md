---
id: TC-985
title: worker resolve escalates by trigger
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
runner-args: tc_985_worker_resolve_escalates_by_trigger
---

## Scenario — role resolution applies the escalation ladder

**Given** a scaffolded worker catalog (`product worker init`),
**When** the user runs `product worker resolve implementer`,
**Then** it exits 0 and resolves to the default capability `claude-code`.

**And when** the user runs `product worker resolve implementer --trigger
stakes_foundational`, **then** it escalates up the ladder to `deep-reasoning`.

## Validates

- FT-131 — worker capability catalog — role to capability with escalation, claude or litellm
- ADR-072 — Workers are capabilities resolved by role with escalation; runners are claude or litellm
