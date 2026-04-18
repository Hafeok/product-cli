---
id: TC-596
title: w023_names_deprecating_adr
type: scenario
status: unimplemented
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
---

## Session: ST-150 — w023-names-deprecating-adr

### Given
Two accepted ADRs, both with `deprecates:` lists, deprecating different
fields (e.g. ADR-X deprecates `foo`, ADR-Y deprecates `bar`). A repository
file uses both.

### When
`product graph check` runs.

### Then
- Two W023 warnings are emitted.
- The warning for `foo` names ADR-X.
- The warning for `bar` names ADR-Y.
- The warning text follows the format
  `warning[W023]: deprecated field 'FIELD' in ARTIFACT (deprecated by ADR-NNN)`.
