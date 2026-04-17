---
id: TC-497
title: body mutation on accepted adr succeeds and surfaces e014
type: scenario
status: unimplemented
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_497_body_mutation_on_accepted_adr_surfaces_e014
---

Validates FT-041 / ADR-038 decision 9.

**Setup:** fixture with an existing accepted ADR (`ADR-X` with `status: accepted` and a computed `content-hash`).

**Act:** apply a `type: change` request that mutates ADR-X's body: `set field: body value: |...corrected prose...`. Then immediately run `product graph check`.

**Assert:**
- `apply` exits 0 — the request does not duplicate content-hash enforcement at the request layer
- The ADR file is updated on disk with the new body
- The post-apply `graph check` (step 11 of the pipeline) reports `E014` content-hash mismatch on ADR-X
- `graph check` exits 1 (because E014 is an error)
- The request itself exited 0 — the invariant in TC-496 is intentionally narrower: it covers requests that should produce a clean graph; this TC documents the one allowed exception where the request succeeds and post-check flags the ADR-032 amendment requirement
- Running `product adr accept ADR-X --amend --reason "body corrected"` clears the E014 (covered by ADR-032 tests, not this TC)
