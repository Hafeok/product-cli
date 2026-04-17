---
id: TC-493
title: request writes reason to request-log jsonl
type: scenario
status: unimplemented
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_493_request_writes_reason_to_request_log_jsonl
---

Validates FT-041 / ADR-038 decision 5.

**Setup:** fixture repo with no existing `.product/request-log.jsonl` file.

**Act:** apply a successful request with `reason: "Add rate limiting"`. Then apply a second successful request with `reason: "Link ADR-X to feature"`.

**Assert:**
- After the first apply, `.product/request-log.jsonl` exists with exactly one line
- That line is valid JSON and contains at minimum: `timestamp` (ISO 8601), `reason: "Add rate limiting"`, `request_hash` (SHA-256 of the request YAML content), `created` array, `changed` array
- After the second apply, the file has exactly two lines, the second containing `reason: "Link ADR-X to feature"`
- A failed apply (with E-class finding) does **not** append a line to the log
- Running `validate` (not `apply`) does **not** append a line to the log
