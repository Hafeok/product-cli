---
id: TC-590
title: adr_deprecates_field_parses_correctly
type: scenario
status: unimplemented
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_590_adr_deprecates_field_parses_correctly
---

## Session: ST-144 — adr-deprecates-field-parses-correctly

### Given
An ADR file with non-empty `deprecates:` array containing two strings.

### When
The parser loads the ADR.

### Then
- The parsed `Adr` struct's `deprecates` field equals the input array.
- Round-trip serialisation preserves the values.
- An ADR with no `deprecates:` field parses with `deprecates` defaulting to
  `[]`.
- Round-trip serialisation of an ADR with empty `deprecates` does NOT emit a
  `deprecates:` line.
