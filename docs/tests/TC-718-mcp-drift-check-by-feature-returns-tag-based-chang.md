---
id: TC-718
title: mcp drift check by feature returns tag-based changed files
type: scenario
status: passing
validates:
  features:
  - FT-059
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_718_mcp_drift_check_feature_tag_changed_files
last-run: 2026-05-06T13:31:35.056793027+00:00
last-run-duration: 0.3s
---

## Given

A temp Product repository under git, with one feature `FT-100` in `complete` status and a `product/FT-100` completion tag pointing at a clean commit. After tagging, one file under `src/` declared as a `source-files` entry on a linked ADR is modified.

`product` is launched in MCP stdio mode against the temp repo.

## When

The test sends a `tools/call` JSON-RPC request for `product_drift_check` with `{ "id": "FT-100" }`.

## Then

- The response is a `success` result.
- The parsed envelope has `checked.scope == "FT-100"`, `checked.tag == "product/FT-100"`, and a non-null `checked.tag_timestamp`.
- `findings` contains exactly one entry with `code == "D003"`, `severity == "medium"`, `adr_id == "FT-100"`, and `source_files` equal to the modified path list.
- `status == "findings"` and `summary.medium == 1`.
- The CLI invocation `product drift check FT-100 --format json` against the same temp repo emits a JSON document whose `changed_files` array equals the MCP envelope's `findings[0].source_files` (parity).