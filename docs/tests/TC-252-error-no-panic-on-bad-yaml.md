---
id: TC-252
title: error_no_panic_on_bad_yaml
type: scenario
status: unimplemented
validates:
  features: 
  - FT-010
  - FT-026
  adrs:
  - ADR-013
phase: 1
---

feed a file with completely invalid YAML as front-matter. Assert exit code 1, structured error on stderr, no panic.