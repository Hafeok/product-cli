---
id: TC-861
title: adrs_rejected_reintroduces_gap_as_intentional
type: scenario
status: passing
validates:
  features:
  - FT-104
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_861_adrs_rejected_reintroduces_gap_as_intentional
observes:
- file
- exit-code
- stdout
---

## Description

A feature that genuinely disagrees with a default-acknowledged
cross-cutting ADR must surface the disagreement explicitly. The
`adrs-rejected:` frontmatter field — written via
`product feature reject` — re-introduces the gap with a distinct
status (`intentional`) so reviewers can tell "we discussed this"
apart from "we forgot."

This TC asserts:

1. `product feature reject ADR-001 --feature FT-001 --reason "..."`
   writes the `adrs-rejected:` block to FT-001 on disk.
2. `product preflight FT-001 --format json` returns the ADR in
   `cross_cutting_gaps` with `status: "intentional"` and the
   reason string preserved.
3. The text preflight render contains `INTENTIONAL` and the
   reason snippet.
4. An empty `--reason` is rejected at the CLI (non-zero exit;
   E011 — the reason is load-bearing).
5. Re-running `feature reject` with a new reason is idempotent:
   the field is not duplicated; the reason is updated in place.

## Formal specification

⟦Λ:Scenario⟧
Given a repo with default-ack list containing ADR-001 and
feature FT-001 with no link to ADR-001,
When `product feature reject ADR-001 --feature FT-001
--reason "..."` is run,
Then FT-001's frontmatter contains `adrs-rejected:` with
ADR-001 and the reason.
When `product preflight FT-001 --format json` is run,
Then exit code is 1,
And `cross_cutting_gaps` contains an entry with
`status: "intentional"` and the reason preserved.
When `product preflight FT-001` is run (text),
Then stdout contains "INTENTIONAL" and the reason snippet.
When an empty reason is supplied,
Then the CLI exits non-zero.
When `feature reject` is rerun with a new reason,
Then the field appears once and carries the new reason.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
