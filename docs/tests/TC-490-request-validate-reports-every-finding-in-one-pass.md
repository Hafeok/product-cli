---
id: TC-490
title: request validate reports every finding in one pass
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_490_request_validate_reports_every_finding_in_one_pass
---

Validates FT-041 / ADR-038 decision 3.

**Setup:** empty fixture.

**Act:** write a deliberately broken `type: create` request containing at least 3 distinct E-class findings:
- a feature with a domain not in `[domains]` vocabulary (E012)
- a dep with no governing ADR in the request or the graph (E013)
- a `ref:xxx` that doesn't resolve (E002)

Run `product request validate FILE`.

**Assert:**
- `validate` exits 1
- The findings array contains **all three** findings, not just the first one encountered
- Each finding has `code`, `severity: error`, `message`, and `location` fields
- Running `product request apply FILE` on the same file also reports all three findings, writes nothing, exits 1
