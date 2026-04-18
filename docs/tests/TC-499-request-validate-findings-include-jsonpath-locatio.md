---
id: TC-499
title: request validate findings include jsonpath location
type: scenario
status: passing
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_499_request_validate_findings_include_jsonpath_location
last-run: 2026-04-18T10:42:07.406621710+00:00
last-run-duration: 0.2s
---

Validates FT-041 / ADR-038 decision 11.

**Act:** run `validate` on a request containing four deliberately-placed errors:
- Invalid domain on `artifacts[2].domains[1]`
- Missing `reason:` at the document root
- Dep-without-ADR on `artifacts[4]`
- Invalid mutation value on `changes[1].mutations[0].value`

**Assert:**
- Each finding's `location` field is a valid JSONPath string per RFC 9535
- The four locations are exactly: `$.artifacts[2].domains[1]`, `$.reason`, `$.artifacts[4]`, `$.changes[1].mutations[0].value`
- Every location string parses as valid JSONPath (validated by a reference parser in the test harness)
- Indexing is zero-based (JSONPath convention)
- Arrays use `[n]`, object keys use `.name`, root starts with `$`