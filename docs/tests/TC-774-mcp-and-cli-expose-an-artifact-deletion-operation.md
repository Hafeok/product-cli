---
id: TC-774
title: MCP and CLI expose an artifact-deletion operation
type: scenario
status: unimplemented
validates:
  features:
  - FT-064
  adrs: []
phase: 5
---

The MCP write surface and the CLI both expose an operation that
removes an artifact file (feature / ADR / TC / dep) from the
tracked artifact directories and records the deletion in
`requests.jsonl` with the same hash-chain link as every other
request entry. The exact spelling (`type: delete`, `deletions:`
section, `product request delete <ID>`) is part of design — the
TC asserts the **capability**: round-trip create → delete → graph
check exits 0 with the artifact absent, and `product request log`
shows the deletion entry.

Today: no such operation exists; deletion requires manual `rm`
and breaks the audit trail.
