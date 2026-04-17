---
id: TC-492
title: request rejects empty reason
type: scenario
status: unimplemented
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_492_request_rejects_empty_reason
---

Validates FT-041 / ADR-038 decision 5.

**Act:** attempt three requests and run `validate` + `apply` on each:
1. A request YAML with no `reason:` field at all
2. A request with `reason: ""` (empty string)
3. A request with `reason: "   "` (whitespace only)

**Assert:**
- Each case exits 1
- The finding has `code: E011`, `severity: error`, `message` mentioning "reason required" or equivalent, and a `location` JSONPath of `$.reason`
- No file is written for any of the three cases
- A fourth case with `reason: "meaningful text"` applies successfully with exit 0
