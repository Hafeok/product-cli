---
id: TC-739
title: request rejects unknown mutation field with E026
type: scenario
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-038
phase: 5
runner: cargo-test
runner-args: tc_739_request_rejects_unknown_mutation_field_with_e026
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.6s
---

## Given

A session repository with one feature `FT-001`.

## When

`product request apply` is called with a `change` request whose mutation
uses the camelCase typo `dependsOn`:

```yaml
type: change
schema-version: 1
reason: "typo case"
changes:
  - target: FT-001
    mutations:
      - op: append
        field: dependsOn
        value: FT-002
```

## Then

- The response reports a finding with `code: "E026"` and a JSONPath
  location pointing to `$.changes[0].mutations[0].field`.
- The finding's message contains the suggestion `did you mean
  'depends-on'?` (Levenshtein distance 2 from `dependsOn`).
- `applied: false` and exit code is `1`.
- The on-disk `FT-001` file is byte-identical.

## Variants

- A garbage field with no nearby match (e.g. `totally-bogus-name`) still
  emits E026 but without a `did you mean` suggestion.
- The pseudo-field `body` is always accepted.