---
id: TC-292
title: gap_stdout_stderr_separation
type: scenario
status: unimplemented
validates:
  features: 
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

gap findings are always on stdout. Analysis errors are always on stderr. Verified by piping stdout only and asserting it is valid JSON.