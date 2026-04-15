---
id: TC-460
title: tag_based_drift_detection_exit
type: exit-criteria
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Exit Criteria

FT-037 is complete when all of the following hold:

1. `product verify FT-XXX` creates an annotated `product/FT-XXX/complete` tag when transitioning a feature to complete (TC-448)
2. Version incrementing works: re-verification creates `complete-v2`, `complete-v3`, etc. (TC-449)
3. Verify degrades gracefully outside git repos — no crash, warning only (TC-450)
4. `product tags list` displays all product/* tags with correct metadata (TC-451)
5. `product tags list --feature` and `--type` filters work correctly (TC-452, TC-453)
6. `product tags show` displays full tag detail including message (TC-454)
7. `product drift check FT-XXX` uses completion tags for file resolution (TC-455)
8. Drift check falls back to pattern-based discovery when no tag exists (TC-456)
9. `product drift check --all-complete` iterates all tagged features (TC-457)
10. `[tags]` config section is optional with correct defaults (TC-458)
11. All tags follow the `product/{artifact-id}/{event}` namespace (TC-459)
12. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass