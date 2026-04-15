---
id: TC-462
title: feature domain add and remove idempotent
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Run `product feature domain FT-XXX --add api` twice. Assert the second call exits 0 and the `domains` list contains `api` exactly once (no duplicates). Run `product feature domain FT-XXX --remove storage` when `storage` is not in the domains list. Assert exit code 0 (no-op, not an error). Verify the file is unchanged.