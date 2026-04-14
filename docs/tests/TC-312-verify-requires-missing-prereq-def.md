---
id: TC-312
title: verify_requires_missing_prereq_def
type: scenario
status: unimplemented
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
---

TC requires a prerequisite not defined in `product.toml`. Assert E-class error with the prerequisite name and a hint to add it to `[verify.prerequisites]`.