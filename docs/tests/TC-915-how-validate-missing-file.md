---
id: TC-915
title: how validate without file is a clear error
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
- stderr
runner: cargo-test
runner-args: tc_915_how_validate_without_file_is_a_clear_error
---

## Scenario — a missing contract file is a clear error

**Given** a repo with no How contract file,
**When** the user runs `product how validate`,
**Then** the process exits 1 and stderr explains there is no how-contract,
pointing at `product how init`.

## Validates

- FT-111 — product how — validate, show, and project an archetype's How contract
- ADR-054 — How contracts are file-based YAML with a native checker mirroring how.shacl.ttl
