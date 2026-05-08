---
id: TC-740
title: request accepts dot-notation on known head field
type: scenario
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-038
phase: 5
runner: cargo-test
runner-args: tc_740_request_accepts_dot_notation_on_known_head
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.6s
---

## Given

A session repository with one feature `FT-001` whose front-matter
declares `domains: [api]`.

## When

`product request apply` is called with a mutation whose `field` is
`domains-acknowledged.security` — a dot-notation path whose first
segment (`domains-acknowledged`) is a known feature field, but whose
leaf (`security`) is an open vocabulary key.

```yaml
type: change
schema-version: 1
reason: "acknowledge security"
changes:
  - target: FT-001
    mutations:
      - op: set
        field: domains-acknowledged.security
        value: "No new trust boundaries introduced."
```

## Then

- The request applies cleanly: `applied: true`, no E026 finding, exit
  code `0`.
- The post-apply `FT-001` front-matter contains
  `domains-acknowledged.security` with the supplied reasoning.
- This proves the new validation does **not** over-reach: the head
  segment matches a known field, so the leaf is accepted regardless of
  its name.
- `product graph check` exits `0`.