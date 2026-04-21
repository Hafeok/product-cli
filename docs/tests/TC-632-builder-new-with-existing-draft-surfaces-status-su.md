---
id: TC-632
title: builder_new_with_existing_draft_surfaces_status_submit_discard_continue
type: scenario
status: unimplemented
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
---

## Session — builder-new-with-existing-draft

### Given

A working directory where `.product/requests/draft.yaml`
already exists from a prior session.

### When

The user runs `product request new create`.

### Then

- The command does NOT overwrite the existing draft.
- The output warns that an active draft exists and lists the
  options: `status`, `submit`, `discard`, `continue`.
- Exit code is 0 (informational, not an error).
- The draft file's mtime is unchanged.
