---
id: TC-911
title: how validate flags broken trace
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
runner-args: tc_911_how_validate_flags_broken_trace
---

## Scenario — the crown trace-truth rule fires

**Given** a How contract where an applied principle (realised by a pattern with
a non-empty `applied_by`) has had its `enforced_by` removed,
**When** the user runs `product how validate`,
**Then** the process exits 1 and stderr names the §5/§4.1 rule 'the trace must
be true' for that principle.

## Validates

- FT-111 — product how — validate, show, and project an archetype's How contract
- ADR-054 — How contracts are file-based YAML with a native checker mirroring how.shacl.ttl
