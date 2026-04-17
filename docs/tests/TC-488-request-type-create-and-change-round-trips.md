---
id: TC-488
title: request type create-and-change round-trips
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_488_request_type_create_and_change_round_trips
---

Validates FT-041 / ADR-038 decision 1.

**Setup:** a fixture repo containing an existing feature FT-X.

**Act:** write a `type: create-and-change` request that:
- Creates a TC with `ref: tc-new` validating `features: [FT-X]`
- In the `changes:` section, appends `ref:tc-new` to `FT-X`'s `tests` field

Apply it.

**Assert:**
- `apply` exits 0
- A new TC file exists with `validates.features: [FT-X]`
- FT-X's `tests` array now includes the new TC's assigned ID (not `ref:tc-new`)
- MCP output `created` contains the new TC with its `ref` → `id` mapping; `changed` contains FT-X
- No half-applied state is possible: if either the create or the change fails validation, neither is written (covered by TC-498)
