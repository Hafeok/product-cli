---
id: TC-178
title: Seeded ADRs have no G005 contradictions after gap check
type: exit-criteria
status: passing
validates:
  features:
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_178_seeded_adrs_have_no_g005_contradictions_after_gap_check"
last-run: 2026-04-13T14:45:04.298783396+00:00
---

## Description

Run the full onboard pipeline end-to-end against a test fixture codebase. After seeding, run `product gap check --all` and assert:

1. No G005 (architectural contradiction) findings among the seeded ADRs
2. G003 (missing rationale) findings are expected for candidates that were confirmed without enrichment
3. G001 (missing test coverage) findings are expected since no TCs are created during onboarding
4. The gap check completes without error (exit code 0 or 1, not 2)

This validates the primary exit criterion for onboarding: the captured decisions are internally consistent even if incomplete.

## Verification

```bash
product onboard scan tests/fixtures/onboard-realistic/ --output /tmp/candidates.json
product onboard triage /tmp/candidates.json --output /tmp/triaged.json
product onboard seed /tmp/triaged.json
product gap check --all --format json > /tmp/gaps.json
# Assert: no findings with code "G005" in output
# Assert: findings with code "G003" are present (expected)
# Assert: findings with code "G001" are present (expected)
```

---