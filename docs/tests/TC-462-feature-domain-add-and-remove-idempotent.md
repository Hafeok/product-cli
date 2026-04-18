---
id: TC-462
title: feature domain add and remove idempotent
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_462_feature_domain_add_and_remove_idempotent"
last-run: 2026-04-18T10:42:03.345580667+00:00
last-run-duration: 0.2s
---

Run `product feature domain FT-XXX --add api` twice. Assert the second call exits 0 and the `domains` list contains `api` exactly once (no duplicates). Run `product feature domain FT-XXX --remove storage` when `storage` is not in the domains list. Assert exit code 0 (no-op, not an error). Verify the file is unchanged.