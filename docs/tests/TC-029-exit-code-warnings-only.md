---
id: TC-029
title: exit_code_warnings_only
type: exit-criteria
status: passing
validates:
  features:
  - FT-010
  adrs:
  - ADR-009
phase: 1
---

create an ADR with no feature links (orphan). Assert exit code 2.