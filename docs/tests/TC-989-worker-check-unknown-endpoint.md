---
id: TC-989
title: worker check flags an unknown endpoint
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
- stderr
runner: cargo-test
runner-args: tc_989_worker_check_flags_unknown_endpoint
---

## Scenario — the catalog validator rejects an unknown runner endpoint

**Given** a capability catalog with a capability whose `endpoint` is `bogus`,
**When** the user runs `product worker check`,
**Then** the process exits 1 and stderr names the unknown `endpoint` `bogus` —
the guard against misconfiguring the runner (e.g. pointing `LITELLM_BASE_URL` at
a provider API instead of the proxy).

## Validates

- FT-131 — worker capability catalog — role to capability with escalation, claude or litellm
- ADR-072 — Workers are capabilities resolved by role with escalation; runners are claude or litellm
