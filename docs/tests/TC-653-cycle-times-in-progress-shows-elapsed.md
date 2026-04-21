---
id: TC-653
title: cycle_times_in_progress_shows_elapsed
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-045
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_653_cycle_times_in_progress_shows_elapsed
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.8s
---

## Session — cycle-times-in-progress-shows-elapsed

### Given

A fixture at simulated `now = 2026-05-22T12:00Z` with:
- 5 complete features, recent-5 median 4.01d.
- `FT-015` with `product/FT-015/started` @ 2026-05-20T07:00Z
  and no `complete` tag (`status: in-progress`).
- `FT-016` with `product/FT-016/started` @ 2026-05-21T08:00Z,
  also in-progress.

### When

The user runs `product cycle-times --in-progress`.

### Then

- Output header is `Feature   Started       Status        Elapsed`.
- `FT-015` row shows elapsed `2.2d` (within 0.1 of
  `(now - started).num_seconds() / 86400.0`).
- `FT-016` row shows elapsed `1.2d` similarly.
- The footer reference line reads approximately
  `Reference: median cycle time (recent 5) is 4.0d`.
- Complete features do not appear in this table.