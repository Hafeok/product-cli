---
id: TC-630
title: builder_submit_applies_and_archives_draft_on_success
type: scenario
status: unimplemented
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
---

## Session — builder-submit-archives-on-success

### Given

A validated draft with five clean artifacts and a non-empty
`reason:` field.

### When

The user runs `product request submit`.

### Then

- `product request apply` is invoked on the draft file and every
  artifact file is written with a resolved real ID.
- The draft file is moved to
  `.product/requests/archive/<ISO-timestamp>-draft.yaml`.
- `.product/request-log.jsonl` gains exactly one apply entry whose
  `reason:` matches the draft's `reason:`.
- Command output lists each created artifact's assigned ID.
- A follow-up `product request status` reports "no active draft".
