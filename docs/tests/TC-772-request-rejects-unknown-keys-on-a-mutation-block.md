---
id: TC-772
title: request rejects unknown keys on a mutation block
type: scenario
status: unimplemented
validates:
  features:
  - FT-064
  adrs: []
phase: 5
---

A mutation block carrying a key outside the closed set
`{op, field, value}` (for example `path:`, `to:`, `from:`) is
rejected with an E-class finding pointing at the offending key.
Today the unknown key is silently dropped at parse time and the
mutation either applies a malformed action or applies nothing.
