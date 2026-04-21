---
id: TC-639
title: started_tag_created_on_first_in_progress_transition
type: scenario
status: unimplemented
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
---

## Session — started-tag-created-on-in-progress-transition

### Given

A fixture repo that is a git repo with `FT-009` currently at
`status: planned` and no existing `product/FT-009/*` tags.

### When

The user applies a change request setting `FT-009.status` to
`in-progress`.

### Then

- `product request apply` writes the feature's front-matter with
  the new status.
- An annotated tag `product/FT-009/started` exists with message
  `FT-009 started: status changed to in-progress`.
- The tag's author timestamp is within 5 seconds of the apply
  timestamp.
- Apply exit code is 0.

### And

When the fixture repo is NOT a git repo, the same request
applies cleanly but prints a W-class warning about skipped tag
creation; no E-class finding, no failure.

### And

A feature created directly with `status: in-progress`
(never passing through `planned`) also receives
`product/FT-XXX/started` at apply time.
