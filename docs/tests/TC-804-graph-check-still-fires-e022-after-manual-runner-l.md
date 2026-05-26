---
id: TC-804
title: graph check still fires E022 after manual runner-line delete
type: scenario
status: passing
validates:
  features:
  - FT-068
  adrs: []
phase: 5
runner: cargo-test
runner-args: tc_804_graph_check_still_fires_e022
runner-timeout: 120
last-run: 2026-05-26T12:31:19.240471574+00:00
last-run-duration: 0.2s
---

## Scenario

A feature `FT-XXX` is `in-progress` with a linked TC that has
`runner: cargo-test` and `runner-args: tc_NNN_xxx` populated. A
user manually deletes the `runner:` line from the TC's
front-matter (simulating drift from a hand-edit or a bad merge).

## When

The user runs `product graph check`.

## Then

`error[E022]: TC runner configuration missing` fires naming the
offending TC. The error renderer emits the canonical YAML
snippet (`runner: cargo-test` / `runner-args: "tc_NNN_<slug>"`)
hint matching ADR-013.

This proves the auto-fill does not leak into `product graph check`
— the gate remains strict outside the implement pipeline, so
manual edits that break the invariant are caught at CI / pre-
commit time exactly as before. Exit code is 22.