---
id: TC-558
title: verify_ci_json_output
type: scenario
status: passing
validates:
  features:
  - FT-044
  adrs:
  - ADR-009
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_558_verify_ci_json_output
last-run: 2026-04-18T10:54:35.903075401+00:00
last-run-duration: 0.2s
---

## Session: ST-116 — verify-ci-json-output

**Validates:** FT-044, ADR-040, ADR-009 (`--ci` flag emits valid JSON)

### Given

Any temp repository that produces at least one finding per stage (mix of pass / warning / fail).

### When

`product verify --ci` is run.

### Then

- stdout contains a single top-level JSON object (not NDJSON, not multi-document).
- The JSON parses cleanly with `serde_json::from_str`.
- The document matches the documented schema: keys `passed` (bool), `exit` (int 0/1/2), `stages` (array of 6).
- Each stage object has `stage` (int 1–6), `name` (string), `status` (one of `pass`/`warning`/`fail`), `findings` (array).
- No ANSI colour codes in the output.