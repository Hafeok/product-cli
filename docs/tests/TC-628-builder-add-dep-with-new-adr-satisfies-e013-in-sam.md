---
id: TC-628
title: builder_add_dep_with_new_adr_satisfies_e013_in_same_step
type: scenario
status: unimplemented
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
---

## Session — builder-add-dep-with-new-adr-closes-e013

### Given

An open create-mode draft that already contains one feature
artifact.

### When

The user runs
`product request add dep --title Redis --dep-type service --version ">=7" --adr new --adr-title "Redis for rate limit state"`.

### Then

- `draft.yaml` gains two artifact blocks: one `type: dep` with a
  `ref:` and one `type: adr` with a `ref:` whose `governs` list
  references the dep's `ref:`.
- Incremental validation reports E013 as satisfied within the
  draft (no governing-ADR finding for the new dep).
- The command output names both refs and states
  `E013 satisfied — dep has governing ADR in draft`.
