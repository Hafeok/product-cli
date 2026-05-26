---
id: TC-778
title: mcp_feature_status_writes_to_disk
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_778_mcp_feature_status_writes_to_disk
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.2s
---

## Description

Compose a temp repo containing FT-X (status `planned`) via
`product request apply`. Invoke `product_feature_status` over MCP
with `id: FT-X, status: complete`. Assert: (a) the MCP response is
a success envelope with `{ id: "FT-X", status: "complete" }` and
no `note` field, (b) the on-disk front-matter `status:` field reads
`complete`, (c) `product graph check` exits 0, (d) a parallel
invocation of `product feature status FT-X complete` against a
sibling temp repo produces a byte-identical feature file.