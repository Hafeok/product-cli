---
id: TC-494
title: request rejects unknown schema version with upgrade hint
type: scenario
status: passing
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_494_request_rejects_unknown_schema_version_with_upgrade_hint
last-run: 2026-04-17T11:51:10.501467904+00:00
last-run-duration: 0.2s
---

Validates FT-041 / ADR-038 decision 6.

**Act:** run `validate` on three request YAMLs:
1. No `schema-version:` field at all (should default to 1 — apply succeeds)
2. `schema-version: 1` (apply succeeds)
3. `schema-version: 99` (apply fails)

**Assert:**
- Cases 1 and 2 exit 0
- Case 3 exits 1 with finding `code: E001` (or a dedicated version code TBD in implementation), a clear message naming the declared version and the supported version(s), and an `upgrade_hint` field or equivalent guidance string (e.g. "this request was written for schema v99; upgrade Product, or rewrite the request for schema v1")
- The location is `$.schema-version`