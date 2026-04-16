---
id: TC-485
title: aggregate bundle metrics exit criteria
type: exit-criteria
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Exit Criteria

FT-040 is complete when:

1. `product graph stats` shows a "Bundle size" section with mean, median, p95, max, min token counts when at least one feature has a `bundle` block — validated by TC-480
2. `product graph stats` shows "No bundle measurements" when no features have bundle blocks — validated by TC-481
3. `product context --measure-all` measures all features, writes bundle blocks and metrics.jsonl entries — validated by TC-482
4. `product context --measure-all --depth N` respects the depth flag — validated by TC-483
5. `product context --measure-all` prints only the aggregate summary table, not full bundle content — validated by TC-484