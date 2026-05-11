---
id: TC-771
title: request rejects change with empty mutations list
type: scenario
status: unimplemented
validates:
  features:
  - FT-064
  adrs: []
phase: 5
---

A change with `mutations: []` (or no `mutations:` key at all) is
rejected. The intent is undecidable — there is no point shipping a
change that performs zero mutations. Expected error: **E006** with
a clear message naming the offending change index.

Today this is silently accepted and the apply summary reports
`mutations: 0` for that change — the exact pathology the user hit.
