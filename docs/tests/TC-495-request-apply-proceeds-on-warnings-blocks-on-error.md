---
id: TC-495
title: request apply proceeds on warnings blocks on errors
type: scenario
status: unimplemented
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_495_request_apply_proceeds_on_warnings_blocks_on_errors
---

Validates FT-041 / ADR-038 decision 7.

**Setup:** fixture where applying a specific request produces a W-class finding (e.g. creating a new ADR that triggers a G005 advisory conflict, or creating a DEP with `breaking-change-risk: high`).

**Act 1:** run `validate` and `apply` on a request that produces W-class findings but no E-class findings.

**Assert 1:**
- `validate` exits 0 but prints the warnings
- `apply` exits 0, writes all files, prints the same warnings alongside the apply summary
- The warnings appear in the MCP output `findings` array with `severity: warning`

**Act 2:** run `apply` on a request that produces both W-class and E-class findings.

**Assert 2:**
- `apply` exits 1 (because of the E-class)
- No files are written
- Both warning and error findings appear in the output
