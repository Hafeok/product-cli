---
id: TC-910
title: how validate passes on conformant contract
type: scenario
status: passing
validates:
  features:
  - FT-111
  adrs:
  - ADR-054
phase: 6
observes:
- exit-code
- stdout
- stderr
runner: cargo-test
runner-args: tc_910_how_validate_passes_on_conformant_contract
---

## Scenario — validate a conformant How contract

**Given** the bundled example How contract at `.product/how-contract.yaml`,
**When** the user runs `product how validate`,
**Then** the process exits 0, stdout reports `conformant` with the decision/
principle/pattern counts, and stderr carries the one soft warning (a decision
licenses a principle id it does not define).

## Validates

- FT-111 — product how — validate, show, and project an archetype's How contract
- ADR-054 — How contracts are file-based YAML with a native checker mirroring how.shacl.ttl
