---
id: TC-988
title: first-party worker writes artifact offline
type: scenario
status: passing
validates:
  features:
  - FT-133
  adrs:
  - ADR-074
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_988_first_party_worker_writes_artifact_offline
---

## Scenario — the first-party worker runs offline and applies a stub artifact

**Given** a scaffolded worker catalog (the `coder` role → a `code-writer`
capability with `endpoint: worker`),
**When** the user runs `product worker run coder --prompt "…"` with no model
configured (`LITELLM_BASE_URL`/`LITELLM_API_KEY` empty),
**Then** the process exits 0, reports the resolved `endpoint worker` running
`offline`, and writes a `STUB-…` artifact under `.product/build/artifacts/`.

## Validates

- FT-133 — first-party worker — a native SPMC executor (endpoint worker)
- ADR-074 — The first-party worker is a structured-output capability endpoint
