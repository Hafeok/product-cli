---
id: TC-626
title: builder_new_create_starts_draft_and_lists_next_commands
type: scenario
status: unimplemented
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
---

## Session — builder-new-create-starts-draft

### Given

A freshly initialised fixture repo with no existing
`.product/requests/draft.yaml`.

### When

The user runs `product request new create`.

### Then

- `.product/requests/draft.yaml` is created with `type: create`
  front-matter and an empty `artifacts:` list.
- The command output names the draft path, the request type, and
  lists the next-step commands (`product request add feature|adr|tc|dep|doc`,
  plus `status`, `validate`, `submit`, `discard`).
- Exit code is 0.
