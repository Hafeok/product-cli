---
id: TC-770
title: request rejects unknown keys on a change block
type: scenario
status: unimplemented
validates:
  features:
  - FT-064
  adrs: []
phase: 5
---

A `type: change` request whose top-level change entry carries `op:`,
`field:`, or `value:` at the change level (instead of inside a
`mutations:` list) is rejected with an E-class finding. Concretely:

```yaml
type: change
reason: "remove TC-002 from FT-001"
changes:
  - target: FT-001
    op: remove          # <-- misplaced
    field: tests        # <-- misplaced
    value: TC-002       # <-- misplaced
```

Expected: validation fails, apply writes nothing, exit code 1 (CLI)
/ `applied: false` (MCP). The finding identifies the misplaced keys
with a JSONPath location and hints "did you mean to nest these
inside a `mutations:` list?".

Today this request validates clean and applies with
`changed: [{id: FT-001, mutations: 0, ...}]` — the user sees a
successful response, the file is untouched.
