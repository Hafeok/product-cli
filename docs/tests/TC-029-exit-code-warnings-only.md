---
id: TC-029
title: exit_code_warnings_only
type: exit-criteria
status: passing
validates:
  features:
  - FT-010
  - FT-014
  adrs:
  - ADR-009
phase: 1
runner: cargo-test
runner-args: "tc_029_exit_code_warnings_only"
---

create an ADR with no feature links (orphan). Assert exit code 2.