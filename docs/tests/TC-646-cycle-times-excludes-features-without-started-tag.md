---
id: TC-646
title: cycle_times_excludes_features_without_started_tag
type: scenario
status: unimplemented
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_646_cycle_times_excludes_features_without_started_tag
---

## Session — cycle-times-excludes-features-without-started-tag

### Given

A repository fixture with:
- `FT-201` with both `started` and `complete` tags.
- `FT-202` with only `product/FT-202/complete` — no started tag
  (the feature was created before FT-053 shipped or the tag was
  manually deleted).

### When

The user runs `product cycle-times`.

### Then

- Output contains exactly one row: `FT-201`.
- `FT-202` is silently excluded — no warning, no error. The
  feature is not "complete but missing data"; it simply cannot
  contribute a cycle time and the report only includes features
  whose cycle time is computable.
- `count` in the summary equals 1, not 2.
