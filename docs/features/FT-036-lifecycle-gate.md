---
id: FT-036
title: Lifecycle Gate
phase: 1
status: planned
depends-on: []
adrs:
- ADR-034
- ADR-009
- ADR-013
- ADR-021
- ADR-032
tests:
- TC-440
- TC-441
- TC-442
- TC-443
- TC-444
- TC-445
- TC-446
- TC-447
domains: []
domains-acknowledged: {}
---

## Description

Enforce the lifecycle ordering invariant: a feature cannot reach `complete` while any linked ADR is still `proposed`. This prevents decisions from being rubber-stamped after implementation, which defeats the purpose of ADRs as governing documents.

### Validation Rules

**W017 — `product graph check`:** Warning when a feature with `status: in-progress` or `complete` has a linked ADR with `status: proposed`. Exit code 2 (warning). Fires across the entire graph on every check.

**E016 — `product verify`:** Hard gate. Before running any TCs, verify checks all linked ADRs. If any is `proposed`, emit E016 and exit 1 without running tests or updating status.

### Invariant

```
∀f:Feature, ∀a:ADR where a ∈ f.adrs:
  f.status = "complete" → a.status ≠ "proposed"
```

Only `proposed` blocks. `accepted`, `superseded`, and `abandoned` all satisfy the invariant.

### Bypass

`product verify FT-XXX --skip-adr-check` suppresses E016 for migration scenarios (retroactively linking ADRs to existing features). Does not suppress W017 in graph check.

### Interaction with ADR-032

ADR-032 writes the content-hash at the moment of acceptance. This feature ensures acceptance happens before verify marks complete. Together they create the ordering: propose → accept (hash sealed) → implement → verify (gate checks acceptance) → complete.