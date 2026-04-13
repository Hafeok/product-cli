---
id: TC-067
title: atomic_write_interrupted
type: scenario
status: unimplemented
validates:
  features:
  - FT-005
  adrs:
  - ADR-015
phase: 1
---

simulate a write failure after temp file creation (inject error before rename). Assert the target file is unchanged. Assert the temp file is deleted.