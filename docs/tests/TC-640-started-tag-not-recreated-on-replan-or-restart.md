---
id: TC-640
title: started_tag_not_recreated_on_replan_or_restart
type: scenario
status: unimplemented
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
---

## Session — started-tag-preserved-on-replan

### Given

A fixture git repo where `FT-009` already carries
`product/FT-009/started` from an earlier `planned →
in-progress` transition (timestamp T0).

### When

The feature reverts to `planned` (via a change request) and
then transitions back to `in-progress` (via another change
request) at a later time T1.

### Then

- `product/FT-009/started` still exists with its original
  timestamp T0 (not T1).
- No `product/FT-009/started-v2` is created.
- Both change requests apply with exit code 0.
