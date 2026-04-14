---
id: TC-176
title: Seed dry-run writes no files
type: scenario
status: passing
validates:
  features:
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_176_seed_dry_run_writes_no_files"
last-run: 2026-04-14T18:28:17.026062993+00:00
last-run-duration: 0.2s
---

## Description

Run `product onboard seed triaged.json --dry-run` with a triaged file containing 3 confirmed candidates. Assert that:

1. Zero files are created in `docs/adrs/` or `docs/features/`
2. stdout shows the proposed file paths and ADR IDs that *would* be created
3. The proposed IDs follow the correct sequence
4. Re-running without `--dry-run` creates exactly the files that were proposed

## Verification

```bash
# Count files before
BEFORE=$(ls docs/adrs/ | wc -l)
product onboard seed /tmp/triaged.json --dry-run
AFTER=$(ls docs/adrs/ | wc -l)
# Assert: BEFORE == AFTER (no files created)
# Assert: stdout contains proposed file paths

# Now run for real
product onboard seed /tmp/triaged.json
FINAL=$(ls docs/adrs/ | wc -l)
# Assert: FINAL == BEFORE + 3
```

---