---
id: TC-779
title: mcp_test_status_writes_to_disk
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_779_mcp_test_status_writes_to_disk
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.2s
---

## Description

Compose a temp repo containing TC-X (status `unimplemented`) via
`product request apply`. Invoke `product_test_status` over MCP
with `id: TC-X, status: passing`. Assert: (a) the MCP response is
a success envelope with `{ id: "TC-X", status: "passing" }` and
no `note` field, (b) the on-disk front-matter `status:` field reads
`passing`, (c) a parallel invocation of `product test status TC-X
passing` against a sibling temp repo produces a byte-identical TC
file.