---
id: TC-626
title: builder_new_create_starts_draft_and_lists_next_commands
type: scenario
status: passing
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
runner: cargo-test
runner-args: "tc_626_builder_new_create_starts_draft_and_lists_next_commands"
last-run: 2026-04-21T12:40:57.330357420+00:00
last-run-duration: 0.2s
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