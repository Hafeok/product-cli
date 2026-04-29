---
id: TC-713
title: update_tc_status_escapes_newlines_in_failure_message
type: scenario
status: unimplemented
validates:
  features:
  - FT-023
  adrs:
  - ADR-002
  - ADR-021
phase: 1
---

**Test Type:** scenario

**Why this TC exists:**

The double-quote case is the symptom that surfaced the bug. The
deeper hazard is *any* control character in bash stderr that
breaks single-line YAML — the most common of which is a literal
newline. The current writer formats the field as a double-quoted
single-line scalar, which YAML 1.2 forbids from containing
literal `\n`. A real cargo / bash failure trace always contains
newlines, so this is the universal case.

**Setup:**

1. Build a tempdir fixture repo with:
   - Feature `FT-001`, status `in-progress`, linked to `TC-001`.
   - `TC-001` with `runner: bash` and a `runner-args` script
     that exits 1 and writes a multi-line stderr block:
     ```bash
     #!/usr/bin/env bash
     printf 'thread main panicked\n  at line 1\n  at line 2\n' >&2
     exit 1
     ```

**Execution:**

1. Run `product verify FT-001`.
2. Read `TC-001` from disk and pass to `parser::parse_test`.
3. Run `product graph check`.

**Expected:**

- `product verify FT-001` exits with code `1`.
- `TC-001`'s file is valid YAML — `serde_yaml::from_str` succeeds.
  Acceptable encodings: a double-quoted scalar with literal `\n`
  escape sequences, or a YAML block scalar (`failure-message: |-`
  followed by an indented block).
- `parser::parse_test` returns `Ok(...)` and the parsed
  `failure-message` either matches the original stderr or is a
  prefix of it (the writer is allowed to truncate at 500 bytes
  per the existing `interpret_runner_output` cap; truncation
  must still be valid YAML).
- `product graph check` does **not** emit any parse error for
  `TC-001`.
- Re-running `product verify FT-001` (idempotent re-run) leaves
  the file in the same shape — the rewriter must not double-escape
  on the second pass.

**Notes:**

- This is the regression guard for the *root cause* — most bash
  failure messages contain newlines, so this is the case that
  will fire most often in production.
- The idempotent-rewrite check matters: a fix that escapes once
  and corrupts on the second write would still leave the corpus
  broken after one more verify cycle.
