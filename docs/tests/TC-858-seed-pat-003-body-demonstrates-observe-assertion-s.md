---
id: TC-858
title: seed_pat_003_body_demonstrates_observe_assertion_shape
type: scenario
status: passing
validates:
  features:
  - FT-075
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_858_seed_pat_003_body_demonstrates_observe_assertion_shape
last-run: 2026-05-27T15:36:27.790359954+00:00
last-run-duration: 0.2s
---

## Description

PAT-003's body section "The pattern" must include a concrete
code snippet demonstrating the file-observation assertion shape
from FT-066's TC-778 family. Read the seed file and scan its
body.

Assert:

1. The body contains a fenced code block within "## The
   pattern".
2. The code block references a file read or filesystem-level
   assertion (regex match for one of: `fs::read`, `assert!.*path`,
   `read_to_string`, `metadata`, or similar surface markers).
3. The body's "## Anti-patterns" section explicitly names "TC
   asserts on Ok(_) shape only" (or equivalent verbatim).
4. The body's "## Worked example" section references at least
   one of TC-778, TC-779, TC-787 by id.

## Formal specification

⟦Λ:Scenario⟧
Given the PAT-003 seed file on disk,
When the file body is parsed,
Then "## The pattern" contains a code block exercising
  filesystem-level assertion,
And "## Anti-patterns" names the envelope-only TC anti-pattern,
And "## Worked example" references at least one of TC-778, TC-
  779, TC-787 by id.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩