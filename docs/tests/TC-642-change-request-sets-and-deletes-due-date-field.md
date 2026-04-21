---
id: TC-642
title: change_request_sets_and_deletes_due_date_field
type: scenario
status: unimplemented
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
---

## Session — change-request-sets-and-deletes-due-date

### Given

A feature `FT-009` with no `due-date` field.

### When

The user applies a `type: change` request:
```yaml
changes:
  - target: FT-009
    mutations:
      - { op: set, field: due-date, value: "2026-05-01" }
```

### Then

- `FT-009`'s front-matter gains `due-date: 2026-05-01`.
- `product request apply` exits 0 and writes one entry to the
  request log with the given `reason:`.

### And

A follow-up `type: change` request with `{op: delete, field:
due-date}` removes the field entirely (not "sets to null"). The
front-matter shape matches the pre-set state byte-for-byte.
