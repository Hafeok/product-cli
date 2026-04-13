---
id: FT-006
title: Impact Analysis
phase: 2
status: planned
depends-on:
- FT-016
adrs:
- ADR-012
tests: []
domains: []
domains-acknowledged: {}
---

`product impact` performs reverse-graph reachability analysis to show the full affected set when an artifact changes.

```
product impact ADR-002    # full affected set if this decision changes
product impact FT-001     # what depends on this feature completing
product impact TC-003     # what depends on this test criterion
```

Impact analysis traverses all five edge types in reverse to find every artifact reachable from the target. The output lists affected features, ADRs, and test criteria grouped by hop distance.

This is used by:
- ADR supersession (auto-triggered when `product adr status ADR-XXX superseded` is called)
- Pre-implementation review (`product implement` step 2 references impact for drift context)
- Manual change assessment before modifying shared decisions
