---
id: FT-014
title: Status and Filters
phase: 2
status: complete
depends-on:
- FT-017
- FT-016
adrs:
- ADR-003
- ADR-007
- ADR-008
- ADR-009
- ADR-012
tests:
- TC-009
- TC-010
- TC-021
- TC-022
- TC-023
- TC-024
- TC-025
- TC-026
- TC-027
- TC-028
- TC-029
- TC-030
- TC-041
- TC-042
- TC-043
- TC-044
- TC-045
- TC-046
- TC-047
- TC-048
- TC-049
- TC-050
- TC-051
- TC-052
- TC-053
- TC-054
- TC-157
- TC-159
- TC-181
- TC-209
- TC-210
- TC-232
- TC-233
- TC-234
- TC-235
- TC-236
- TC-237
- TC-238
- TC-249
domains:
- api
- data-model
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
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
