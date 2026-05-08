---
id: TC-738
title: request rejects unknown top-level key with E025
type: scenario
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-038
phase: 5
runner: cargo-test
runner-args: tc_738_request_rejects_unknown_top_level_key_with_e025
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.6s
---

## Given

A session repository with one feature `FT-001`.

## When

`product request validate` and `product request apply` are called with a
request that carries an unknown top-level key — for example, a `patch:`
wrapper around a real change:

```yaml
type: change
schema-version: 1
reason: "tighten validation regression"
patch:
  target: FT-001
  mutations:
    - op: append
      field: domains
      value: api
changes: []
```

## Then

- `product request validate --format json` reports a finding with
  `code: "E025"` and JSONPath location `$.patch`.
- `product request apply` returns `applied: false` with the same E025
  finding; **not** `applied: true / mutations: 0`.
- Exit code is `1`.
- The on-disk `FT-001` file is byte-identical pre/post.
- `product graph check` exits `0` after the rejected call.

## Variants

- Multiple unknown top-level keys (e.g. both `patch:` and `update:`)
  produce one E025 finding per offender in a single response.
- A request with **only** known keys is unaffected by E025.