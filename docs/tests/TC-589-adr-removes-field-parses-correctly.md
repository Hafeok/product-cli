---
id: TC-589
title: adr_removes_field_parses_correctly
type: scenario
status: unimplemented
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_589_adr_removes_field_parses_correctly
---

## Session: ST-143 — adr-removes-field-parses-correctly

### Given
An ADR file with non-empty `removes:` array containing two strings.

### When
The parser loads the ADR.

### Then
- The parsed `Adr` struct's `removes` field equals the input array, in order.
- Round-trip serialisation produces a `removes:` block with the same values.
- An ADR with no `removes:` field parses with `removes` defaulting to `[]`.
- Round-trip serialisation of an ADR with empty `removes` does NOT emit a
  `removes:` line (no-churn rule).
