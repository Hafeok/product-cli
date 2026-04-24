---
id: TC-686
title: w030_respects_required_from_phase
type: scenario
status: unimplemented
validates:
  features:
  - FT-055
  adrs:
  - ADR-047
phase: 1
---

**Covers session test ST-345** — `w030-respects-required-from-phase`.

Verifies that features with `phase < [features].required-from-phase` are exempt from W030 — useful for stubs produced by early migration.

**Setup:**

- `product.toml` sets `[features].required-from-phase = 2`.
- Fixture contains two features:
  - FT-A with `phase: 1` and an empty body (no sections). Should be **exempt**.
  - FT-B with `phase: 2` and an empty body. Should **trigger W030**.

**Steps:**

1. Run `product graph check --format json`.

**Assertions:**

- Exactly one W030 warning is emitted, for FT-B.
- No W030 warning is emitted for FT-A (phase below threshold).
- Changing `required-from-phase` back to `1` and re-running surfaces W030 for both.
