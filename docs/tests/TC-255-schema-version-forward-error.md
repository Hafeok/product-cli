---
id: TC-255
title: schema_version_forward_error
type: scenario
status: unimplemented
validates:
  features: 
  - FT-003
  - FT-008
  - FT-020
  adrs:
  - ADR-014
phase: 1
---

write `schema-version = "99"` to `product.toml`. Run any command. Assert exit code 1 and error E008 with the upgrade hint.