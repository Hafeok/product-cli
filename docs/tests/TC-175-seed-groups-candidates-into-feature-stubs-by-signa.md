---
id: TC-175
title: Seed groups candidates into feature stubs by signal proximity
type: scenario
status: passing
validates:
  features:
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_175_seed_groups_candidates_into_feature_stubs_by_signal_proximity"
last-run: 2026-04-14T11:01:04.789091923+00:00
---

## Description

Run the full onboard pipeline against a test fixture with candidates spanning two distinct areas:
- 3 candidates with evidence files in `src/api/` (consistency and convention signals)
- 2 candidates with evidence files in `src/storage/` (boundary and constraint signals)

After seeding, assert that:

1. At least 2 feature stubs are created (one for each evidence cluster)
2. The API-related feature stub links to the 3 API-related ADRs
3. The storage-related feature stub links to the 2 storage-related ADRs
4. No feature stub contains ADRs from both clusters (unless evidence files overlap)
5. Feature stubs have `status: planned` and empty test lists

## Verification

```bash
product onboard seed /tmp/triaged.json
# Assert: >= 2 feature files created in docs/features/
# Assert: each feature's adrs list contains only ADRs from the same evidence cluster
# Assert: all feature stubs have status: planned
```

---