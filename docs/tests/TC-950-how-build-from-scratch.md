---
id: TC-950
title: how build full contract from scratch
type: scenario
status: passing
validates:
  features:
  - FT-115
  adrs:
  - ADR-057
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_950_how_build_full_contract_from_scratch
---

## Scenario — build a complete How element by element

**Given** a repo with no How contract,
**When** the user runs `product how add` for a decision, principle, pattern,
interface, an app-statement under a `set app-contract`, and a resource under a
`set infra-contract`,
**Then** every command exits 0; `how show` stdout reports the counts (including
`infrastructure-contract: infra satisfies app (1 resource(s))`); and
`how validate` exits 0 — the built-up How is conformant.

## Validates

- FT-115 — product how add/set — granular authoring of the Why cascade and contracts
- ADR-057 — How elements are authored granularly via add/set on the contract file
