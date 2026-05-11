---
id: TC-773
title: request op:remove on list-valued field removes the entry
type: scenario
status: unimplemented
validates:
  features:
  - FT-064
  adrs: []
phase: 5
---

Regression cover for the headline symptom. Given a feature
`FT-001` with `tests: [TC-001, TC-002]`, applying:

```yaml
type: change
reason: "remove TC-002"
changes:
  - target: FT-001
    mutations:
      - op: remove
        field: tests
        value: TC-002
```

must result in the on-disk feature file's `tests:` line containing
exactly `[TC-001]` and the apply summary reporting
`changed: [{id: FT-001, mutations: 1, ...}]`. This complements
ST-014 (TC-669) which covers the same shape on `domains` — this TC
covers the `tests` field on a feature, the path that the user hit
in the wild.
