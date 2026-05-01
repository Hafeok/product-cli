---
id: TC-712
title: update_tc_status_escapes_double_quotes_in_failure_message
type: scenario
status: unrunnable
validates:
  features:
  - FT-023
  adrs:
  - ADR-002
  - ADR-021
phase: 1
runner: cargo-test
runner-args: "tc_712_update_tc_status_escapes_double_quotes_in_failure_message"
---

**Test Type:** scenario

**Why this TC exists:**

Production bug 2026-04-29: a previous `product verify` run wrote
`failure-message` values containing unescaped double quotes (from
bash error output). That corrupted the YAML front-matter — the
`parser` could no longer read the file, so the TC became invisible
to `product graph check`, and the linked feature stayed
perpetually `in-progress`. This TC pins the smallest character
class that triggered the bug: a literal `"` inside the message.

**Setup:**

1. Build a tempdir fixture repo with:
   - Feature `FT-001`, status `in-progress`, linked to `TC-001`.
   - `TC-001` with `runner: bash` and `runner-args` pointing at a
     script `scripts/fail.sh` that exits 1 with stderr containing
     a literal double quote — for example:
     ```bash
     #!/usr/bin/env bash
     echo 'expected "expected" but got "actual"' >&2
     exit 1
     ```

**Execution:**

1. Run `product verify FT-001`.
2. Re-run `product graph check`.

**Expected:**

- `product verify FT-001` exits with code `1` (the TC failed).
- `TC-001`'s on-disk content can be parsed by
  `serde_yaml::from_str` against `TestCriterionFrontMatter`
  without error — every embedded `"` in the failure message is
  escaped (`\"`) inside a double-quoted YAML scalar OR the
  message is emitted as a YAML block scalar (`|-` / `>-`).
- `parser::parse_test` (the same parser used by
  `parser::load_all`) returns `Ok(TestCriterion { ... })` for
  `TC-001` after the verify write.
- `product graph check` does **not** report `TC-001` as missing
  or unparseable — the graph contains the node and the edge to
  `FT-001`.
- `TC-001`'s `failure-message` field, after parsing, equals the
  original stderr (modulo trailing whitespace) — the escape is
  lossless, not just safe.

**Notes:**

- This TC is the regression guard for the exact characters that
  broke the corpus (one literal `"`). The companion TCs cover
  newlines and backslashes.
- The assertion is **parseability**, not the literal escape
  encoding — either YAML escape (`\"`) or block scalar is
  acceptable. The contract is "round-trips cleanly", not "uses
  escape style X".
