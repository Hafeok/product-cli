---
id: TC-706
title: verify_allows_missing_runner_when_feature_planned
type: scenario
status: passing
validates:
  features:
  - FT-058
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: "tc_706_verify_allows_missing_runner_when_feature_planned"
last-run: 2026-04-30T09:25:29.076293260+00:00
last-run-duration: 0.3s
---

**Test Type:** scenario

**Why this TC exists:**

FT-058 makes runner config required only once a feature reaches
`in-progress`. TCs sketched out during planning must remain
authorable without runner config — otherwise the authoring flow
breaks for every new feature. This TC pins the exemption:
features in `planned` (and `abandoned`) status never trigger
E022.

**Setup:**

1. Build a tempdir fixture repo with a feature `FT-001` whose
   status is `planned`.
2. Link one TC `TC-001` with NO `runner` and NO `runner-args` in
   its front-matter.

**Execution (planned):**

1. Run `product verify FT-001`.

**Expected (planned):**

- Exit code `0`.
- Stdout contains the line `TC-001  <title>  UNIMPLEMENTED (no
  runner configured)` — the pre-FT-058 soft-skip behaviour is
  preserved for planning-stage TCs.
- Stderr contains the W001-style warning indicating no runnable
  TCs were found.
- No `error[E022]` text in stderr.
- Feature status remains `planned`.

**Execution (abandoned):**

1. Mutate `FT-001` status to `abandoned`.
2. Run `product verify FT-001`.

**Expected (abandoned):**

- Exit code `0`.
- Same soft-skip behaviour as `planned`: no E022 fires.

**Notes:**

- This is the negative case for TC-705. Together they pin the
  exact status boundary at which the gate engages.
- The exemption deliberately covers `abandoned` so retroactive
  enforcement does not break against features whose work was
  cancelled before runner config became mandatory.