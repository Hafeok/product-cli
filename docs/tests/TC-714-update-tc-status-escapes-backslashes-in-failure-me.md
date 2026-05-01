---
id: TC-714
title: update_tc_status_escapes_backslashes_in_failure_message
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
runner-args: "tc_714_update_tc_status_escapes_backslashes_in_failure_message"
---

**Test Type:** scenario

**Why this TC exists:**

In a YAML 1.2 double-quoted scalar, `\` is the escape character.
The previous writer escaped `"` → `\"` but did not escape `\`,
so a literal `\` in stderr became the start of an invalid escape
sequence in the YAML scalar. Bash error output frequently
contains backslashes — Windows paths echoed back, regex traces,
`\n` already-rendered text, ANSI-stripped `\x1b` sequences, etc.
This TC pins that `\` round-trips through `update_tc_status`
without breaking the YAML.

**Setup:**

1. Build a tempdir fixture repo with:
   - Feature `FT-001`, status `in-progress`, linked to `TC-001`.
   - `TC-001` with `runner: bash` and `runner-args` script:
     ```bash
     #!/usr/bin/env bash
     printf 'failed at C:\\Users\\test path with \\n literal' >&2
     exit 1
     ```
     The stderr therefore contains both literal `\` characters
     and the two-character sequence `\n` (a backslash followed
     by `n`, not a real newline).

**Execution:**

1. Run `product verify FT-001`.
2. Read `TC-001` from disk and pass to `parser::parse_test`.

**Expected:**

- `TC-001`'s file is valid YAML — `serde_yaml::from_str` succeeds.
- Inside a double-quoted scalar, every literal `\` from the
  stderr is encoded as `\\`. Inside a block scalar, no escape is
  required.
- `parser::parse_test` returns `Ok(...)`. The parsed
  `failure-message` field contains the original `\` characters
  verbatim — the escape is lossless. In particular, the
  two-character sequence `\n` in stderr **must not** be decoded
  into a real newline.
- `product graph check` does not emit a parse error for
  `TC-001`.

**Notes:**

- Together with the quote and newline TCs, this TC closes the
  practical character-class matrix: `"` (the symptom), `\n` (the
  most common case), `\` (the trickiest case because it is
  itself the escape char).
- If the implementation switches to a YAML block scalar
  (`failure-message: |-`), this TC passes trivially — block
  scalars require no `\` or `"` escaping. That is an acceptable
  fix; the assertion is parseability, not a specific encoding.
