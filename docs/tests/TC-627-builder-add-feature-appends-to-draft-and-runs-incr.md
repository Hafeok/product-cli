---
id: TC-627
title: builder_add_feature_appends_to_draft_and_runs_incremental_validation
type: scenario
status: passing
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
runner: cargo-test
runner-args: "tc_627_builder_add_feature_appends_to_draft_and_runs_incremental_validation"
last-run: 2026-04-21T12:40:57.330357420+00:00
last-run-duration: 0.2s
---

## Session — builder-add-feature-appends-and-validates

### Given

An open draft created by `product request new create`.

### When

The user runs
`product request add feature --title "Rate Limiting" --phase 2 --domains "api,security"`.

### Then

- `draft.yaml` gains one artifact block of `type: feature` with
  `title: Rate Limiting`, `phase: 2`, `domains: [api, security]`,
  and a `ref:` name matching `^ft-[a-z0-9-]+$`.
- The command prints the assigned `ref:` name.
- Incremental validation runs in under 100ms and reports no
  E-class findings.
- A second call with an unknown domain (`--domains chimney`)
  fails with E012 and does NOT append to the draft.