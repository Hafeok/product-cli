---
id: FT-027
title: Context Bundle
phase: 5
status: complete
depends-on: []
adrs:
- ADR-026
tests:
- TC-140
- TC-141
- TC-142
- TC-143
- TC-144
- TC-145
- TC-146
- TC-147
- TC-148
- TC-149
- TC-150
- TC-151
domains:
- api
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
---

{BUNDLE}
```

The test status table is generated fresh at invocation time — the agent sees which TCs are currently passing and which are not.

---