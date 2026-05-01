---
id: TC-715
title: failing_tc_remains_visible_to_graph_check_after_verify
type: scenario
status: unrunnable
validates:
  features:
  - FT-018
  - FT-023
  adrs:
  - ADR-002
  - ADR-021
phase: 1
runner: cargo-test
runner-args: "tc_715_failing_tc_remains_visible_to_graph_check_after_verify"
---

**Test Type:** scenario

**Why this TC exists:**

End-to-end regression of the **observed** production failure
mode: 13 TCs went silently invisible because the YAML in their
front-matter was corrupted by `product verify`'s
`failure-message` writer. The companion encoding TCs (quotes,
newlines, backslashes) check the writer in isolation; this TC
checks the *outcome*: after any failing verify run with
arbitrary bash stderr, the TC must remain a first-class graph
node and the graph health must be unchanged-or-better, never
degraded by parse errors.

**Setup:**

1. Build a tempdir fixture repo with:
   - Feature `FT-001`, status `in-progress`, linked to **three**
     TCs (`TC-001`, `TC-002`, `TC-003`) all configured with
     `runner: bash`.
   - `TC-001`'s script exits 1 with stderr containing every
     hazard at once: literal `"`, literal `\`, a real newline,
     a tab, and a non-ASCII byte (`é`).
   - `TC-002`'s script exits 0 (passing).
   - `TC-003` has no `runner` configured (will be reported as
     `UNIMPLEMENTED` per existing behaviour).
2. Snapshot the output of `product graph check --format json`
   **before** running verify — record the set of node IDs and
   the warning/error counts.

**Execution:**

1. Run `product verify FT-001`.
2. Run `product graph check --format json` again.
3. For each TC in `[TC-001, TC-002, TC-003]`, attempt to read
   and parse the file with `parser::parse_test`.

**Expected:**

- `parser::parse_test` succeeds for **all three** TCs after
  verify — none of them have unparseable front-matter.
- The post-verify `graph check` JSON contains every TC ID that
  the pre-verify run contained. No TC has been silently
  dropped from the graph.
- The post-verify `graph check` reports zero new `E001`
  (`PARSE_ERROR`) findings compared to the pre-verify
  snapshot. Other findings (W016 unimplemented, etc.) are
  permitted to differ.
- `TC-001`'s parsed `status` is `failing` and its
  `failure-message` is non-empty.
- `TC-002`'s parsed `status` is `passing`.
- `TC-003`'s parsed `status` is unchanged from setup (the
  no-runner soft-skip path is exercised but the TC is not
  corrupted).
- Re-running `product verify FT-001` a second time leaves
  every TC file byte-identical to the first verify (fix-point;
  no double-escape drift).

**Notes:**

- This is the highest-value TC of the four because it asserts
  the *operational* invariant: "a failing verify must never
  make a TC invisible." The encoding-specific TCs above can
  all pass while this one fails (e.g. if some new field added
  later forgets the same escape rule); conversely this one can
  pass for the wrong reason on a clean stderr. We keep both
  layers.
- The pre/post `graph check` snapshot comparison is the cheapest
  way to assert "no degradation" without enumerating every
  graph invariant by hand.
- If the implementation chooses block-scalar encoding, every
  assertion here still holds — the test is intentionally
  encoding-agnostic.
