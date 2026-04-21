---
id: TC-647
title: cycle_times_excludes_features_without_complete_tag
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_647_cycle_times_excludes_features_without_complete_tag
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.6s
---

## Session — cycle-times-excludes-features-without-complete-tag

### Given

A repository fixture with:
- `FT-301` with both `started` and `complete` tags (`status:
  complete`).
- `FT-302` with only `product/FT-302/started` (`status:
  in-progress`, never verified).

### When

The user runs `product cycle-times` (without `--in-progress`).

### Then

- Output contains exactly one row: `FT-301`.
- `FT-302` is not listed — it has no completion timestamp and
  its cycle time is not yet defined.
- Running `product cycle-times --in-progress` in the same
  fixture lists `FT-302` with an elapsed-so-far column and
  `FT-301` is not in that view.