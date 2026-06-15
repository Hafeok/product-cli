---
id: TC-914
title: how init scaffolds and validates
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
runner: cargo-test
runner-args: tc_914_how_init_scaffolds_and_validates
---

## Scenario — scaffold then validate

**Given** a repo with no How contract,
**When** the user runs `product how init --archetype rest-api`,
**Then** the process exits 0 and writes `.product/how-contract.yaml`; a second
`init` without `--force` exits 1, and `product how validate` on the
scaffold exits 0.

## Validates

- FT-111 — product how — validate, show, and project an archetype's How contract
- ADR-054 — How contracts are file-based YAML with a native checker mirroring how.shacl.ttl
