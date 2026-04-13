---
id: TC-161
title: FT-005 atomic writes and locking safe
type: exit-criteria
status: passing
validates:
  features:
  - FT-005
  adrs:
  - ADR-015
phase: 1
runner: cargo-test
runner-args: "tc_161_ft005_exit_criteria"
---

## Description

All file writes use atomic temp-file-plus-rename (ADR-015). Advisory locking on `.product.lock` serialises concurrent write commands. Stale locks from crashed processes are detected and cleared. Leftover `.product-tmp.*` files are cleaned on startup. Specifically:

- TC-066: Atomic writes produce correct content with no leftover temp files
- TC-067: Interrupted writes leave the original file unchanged and clean up temp files
- TC-068: Concurrent write commands are serialised — one succeeds, the other gets E010
- TC-069: Stale lock files (dead PID) are automatically cleared
- TC-070: Leftover temp files from prior crashes are cleaned on any command startup