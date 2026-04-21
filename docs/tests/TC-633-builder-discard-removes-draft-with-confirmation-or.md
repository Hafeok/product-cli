---
id: TC-633
title: builder_discard_removes_draft_with_confirmation_or_force
type: scenario
status: unimplemented
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
---

## Session — builder-discard-removes-draft

### Given

A working directory with an active draft containing three
artifacts.

### When

The user runs `product request discard --force`.

### Then

- `.product/requests/draft.yaml` no longer exists on disk.
- No archive entry is created (discard is not a submit).
- Exit code is 0.
- A follow-up `product request status` reports "no active draft".

### And

Without `--force`, the command prompts for confirmation and
aborts without deletion on a negative response.
