---
id: FT-014
title: Status and Filters
phase: 2
status: planned
depends-on:
- FT-017
- FT-016
adrs:
- ADR-007
- ADR-009
tests: []
domains: []
domains-acknowledged: {}
---

`product status` provides a summary view of project health by phase, coverage, and dependency state.

```
product status                   # summary: features by phase and status
product status --phase 1         # phase 1 detail with test coverage
product status --untested        # features with no linked test criteria
product status --failing         # features with one or more failing tests
```

### Test Filters

```
product test untested            # features with no linked tests
product test list --failing      # tests currently failing
```

### Git Awareness

When regenerating the checklist, warn if modified files are uncommitted. This prevents stale checklist state from being committed alongside unrelated changes.
