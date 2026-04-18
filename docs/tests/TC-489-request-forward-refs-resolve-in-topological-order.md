---
id: TC-489
title: request forward refs resolve in topological order
type: scenario
status: passing
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_489_request_forward_refs_resolve_in_topological_order
last-run: 2026-04-18T10:42:07.406621710+00:00
last-run-duration: 0.2s
---

Validates FT-041 / ADR-038 decision 2.

**Setup:** empty fixture.

**Act:** write a `type: create` request with 5 artifacts whose cross-references form a DAG:
- `feature` with `ref: ft-a`, `adrs: [ref:adr-b, ref:adr-c]`, `tests: [ref:tc-d]`, `uses: [ref:dep-e]`
- `adr` with `ref: adr-b`
- `adr` with `ref: adr-c`, `governs: [ref:dep-e]`
- `tc` with `ref: tc-d`, `validates: { features: [ref:ft-a], adrs: [ref:adr-b] }`
- `dep` with `ref: dep-e`, `adrs: [ref:adr-c]`

Apply it.

**Assert:**
- All 5 files are created
- IDs are assigned in topological order (ADRs and DEPs whose refs have no outgoing deps get IDs first; the feature that refs them gets its ID after)
- Every occurrence of `ref:xxx` in every written file is replaced with the assigned real ID
- Bidirectional links are materialised: the ADR files list the feature in their `features:` array; the DEP file lists the feature in its `features:` array; the TC file's `validates` points to real IDs
- MCP `created` array maps each `ref` to its assigned `id`