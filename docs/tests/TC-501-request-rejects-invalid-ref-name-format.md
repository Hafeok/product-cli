---
id: TC-501
title: request rejects invalid ref name format
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_501_request_rejects_invalid_ref_name_format
---

Validates FT-041 / ADR-038 decision 13.

**Act:** run `validate` on requests with each of the following `ref:` values:
1. `Ref_Upper` (uppercase, underscore) — invalid
2. `1-starts-with-digit` — invalid
3. `ref with spaces` — invalid
4. `ft-rate-limiting` — valid
5. `adr-a1-b2-c3` — valid
6. `a` — valid (single letter)

**Assert:**
- Cases 1–3 produce `code: E001` with a message naming the invalid ref and the grammar (`^[a-z][a-z0-9-]*$`)
- Cases 4–6 pass the ref-name check (may still fail other validation, but not on the ref format)
- The finding `location` points to the artifact's `ref:` field in JSONPath form
