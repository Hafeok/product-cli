---
id: FT-004
title: Artifact Authoring
phase: 2
status: planned
depends-on:
- FT-003
- FT-016
adrs:
- ADR-002
- ADR-005
- ADR-015
tests: []
domains: []
domains-acknowledged: {}
---

Scaffold, link, and update artifacts from the command line. These commands are the write-side counterpart to the read-only navigation commands in Phase 1.

### Scaffold

```
product feature new "Cluster Foundation"   # scaffold FT-XXX with next auto-incremented ID
product adr new "Use openraft for consensus"
product test new "Raft leader election" --type scenario
```

Scaffolded files include all required front-matter fields with sensible defaults. The ID is auto-incremented from the highest existing ID of that artifact type.

### Link

```
product feature link FT-001 --adr ADR-002   # add edge (mutates front-matter)
product feature link FT-001 --test TC-002
```

Linking validates that no `depends-on` cycles are introduced (E003). Front-matter is updated atomically using `fileops::atomic_write`.

### Status Update

```
product adr status ADR-002 accepted
product test status TC-002 passing
product feature status FT-001 complete
```

ADR supersession triggers an impact report. Front-matter validation on write — type checking, ID format, unknown fields preserved.
