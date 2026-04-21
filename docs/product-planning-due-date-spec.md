# Product Planning — Due Date Specification

> Standalone reference for due date support on features.
> One new front-matter field, one new git tag, two new warning codes.
> Planning is advisory — due dates never block implementation.

---

## Overview

A `due-date` field on a feature declares an external commitment date — when
a stakeholder, customer, or contract requires this feature to be complete.

That is the complete scope. Due dates are advisory signals, not gates.
They appear in `product status`, trigger W028/W029 warnings, and serve as
reference points for the forecasting model (product-forecasting-spec.md).

---

## Feature Front-Matter

One new optional field:

```yaml
---
id: FT-009
title: Rate Limiting
phase: 2
status: in-progress
due-date: 2026-05-01       # optional — ISO 8601 date (YYYY-MM-DD)
---
```

`due-date` is:
- Optional — features without it work identically to today
- A calendar date, not a datetime — precision to the day is sufficient
- Set via a change request — never by Product automatically
- Never used as a gate — W028/W029 are warnings, not errors

---

## Git Tag: `product/FT-XXX/started`

When `product request apply` transitions a feature's `status` field from
`planned` to `in-progress`, Product creates an annotated git tag:

```
product/FT-009/started
```

Tag message:
```
FT-009 started: status changed to in-progress
```

**Rules:**
- Created once, on the first `planned → in-progress` transition
- Never overwritten — if a feature reverts to `planned` and restarts,
  the original tag stands
- If a feature is created with `status: in-progress` (no prior `planned`
  state), the tag is created at apply time

The tag timestamp is the start point for cycle time calculation. Together
with `product/FT-XXX/complete`, it gives:

```
cycle_time = complete_timestamp - started_timestamp
```

This tag exists to support the forecasting model. It requires no
front-matter field and no human action — Product creates it automatically.

### Tag namespace summary

| Tag | Created by | When |
|---|---|---|
| `product/FT-XXX/started` | `product request apply` | First `in-progress` transition |
| `product/FT-XXX/complete` | `product verify FT-XXX` | All TCs passing |
| `product/FT-XXX/complete-vN` | `product verify FT-XXX` | Re-verification |

---

## Validation Codes

### W028 — Due date passed

```
warning[W028]: due date passed
  FT-009: Rate Limiting
  due-date: 2026-05-01  (3 days ago)
  status: in-progress

  This feature has passed its due date.
  Run: product verify FT-009
```

Fires when: `due-date < today` AND `status != complete`.

### W029 — Due date approaching

```
warning[W029]: due date approaching
  FT-009: Rate Limiting
  due-date: 2026-05-01  (in 2 days)
  status: in-progress

  This feature is due soon.
```

Fires when: `due-date` is within the configured warning window AND `status != complete`.

Default warning window: 3 days. Configurable:

```toml
[planning]
due-date-warning-days = 3     # warn this many days before due date
```

Both warnings are exit code 2 (W-class). They appear in `product verify`
stage 2 (graph structure) and in `product status`.

---

## `product status` with due dates

```
product status

  Phase 1 — Cluster Foundation  [OPEN — exit criteria: 2/4 passing]
    FT-001  Cluster Foundation     complete    (2026-04-11)
    FT-002  mTLS Node Comms        complete    (2026-04-13)
    FT-003  Raft Consensus         in-progress  due 2026-04-30
    FT-004  Block Storage          planned      due 2026-05-07

  Phase 2 — Products and IAM  [LOCKED]
    FT-009  Rate Limiting          planned      due 2026-05-01  ← ⚠ W029
```

Features without a `due-date` show no date column. Features with `due-date`
show it alongside status. Overdue features are flagged.

---

## Setting a Due Date

Via the builder:

```bash
product request new change
product request add target FT-009
# → Field: due-date
# → Value: 2026-05-01
# ✓ Mutation added: FT-009 / set / due-date / 2026-05-01
product request submit
```

Via direct YAML:

```yaml
type: change
reason: "Set due date for rate limiting — sprint commitment"
changes:
  - target: FT-009
    mutations:
      - op: set
        field: due-date
        value: "2026-05-01"
```

### Removing a due date

```yaml
type: change
reason: "Remove due date — commitment moved to FT-012"
changes:
  - target: FT-009
    mutations:
      - op: delete
        field: due-date
```

---

## `product tags list` — started tags included

```
product tags list --feature FT-009

  product/FT-009/started     2026-04-14T09:00:00Z   status → in-progress
  product/FT-009/complete    2026-04-21T14:22:00Z   4/4 TCs passing
```

```
product tags list --type started         # all started tags
product tags list --type complete        # all completion tags (unchanged)
```

---

## `product.toml`

```toml
[planning]
due-date-warning-days = 3    # days before due date to emit W029
                             # set 0 to disable W029
```

---

## Session Tests

```
ST-300  due-date-field-parses-correctly
ST-301  due-date-invalid-format-emits-e006
ST-302  w028-fires-when-overdue-not-complete
ST-303  w028-clear-when-complete
ST-304  w029-fires-within-warning-window
ST-305  w029-configurable-window
ST-306  w029-disabled-when-window-zero
ST-307  status-shows-due-date-column
ST-308  status-flags-overdue-features
ST-309  started-tag-created-on-in-progress-transition
ST-310  started-tag-not-recreated-on-replan
ST-311  started-tag-created-for-new-feature-already-in-progress
ST-312  tags-list-includes-started-tags
ST-313  change-request-sets-due-date
ST-314  change-request-deletes-due-date
```

---

## Invariants

- `product/FT-XXX/started` is created at most once per feature. A second
  `planned → in-progress` transition after a replan does not overwrite it.
- `due-date` has no effect on phase gate evaluation, feature completion
  status, or TC execution. It is purely a planning annotation.
- W028 and W029 are W-class (exit 2). They never cause `product verify`
  to exit 1. A missed due date does not block CI.
- Features without `due-date` emit neither W028 nor W029. The field is
  genuinely optional.
