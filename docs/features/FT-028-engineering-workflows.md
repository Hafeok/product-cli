---
id: FT-028
title: Engineering Workflows
phase: 5
status: complete
depends-on: []
adrs:
- ADR-023
- ADR-024
- ADR-035
tests:
- TC-121
- TC-122
- TC-123
- TC-124
- TC-125
- TC-126
- TC-127
- TC-128
- TC-129
- TC-130
- TC-131
domains: []
domains-acknowledged: {}
---

### Drift Detection

`product drift` checks whether the codebase matches what the ADRs decided. The LLM receives the ADR's context bundle plus the source files most likely to implement it (resolved via configurable path patterns in `product.toml`).

```toml
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/"]
```

Drift codes:

| Code | Severity | Description |
|---|---|---|
| D001 | high | Decision not implemented — ADR says X, no code implements X |
| D002 | high | Decision overridden — code does Y, ADR says do X |
| D003 | medium | Partial implementation — some aspects of the decision implemented |
| D004 | low | Implementation ahead of spec — code does X but no ADR documents why |

Drift findings follow the same baseline/suppression model as gap findings (`drift.json`). `product drift scan src/consensus/` is the reverse direction — given source code, identify which ADRs govern it. Useful for onboarding and code review.

### Fitness Functions

`product metrics record` snapshots the current repository health into `metrics.jsonl` (one JSON line per run, committed to the repo):

```json
{"date":"2026-04-11","spec_coverage":0.87,"test_coverage":0.72,"exit_criteria_coverage":0.61,"phi":0.68,"gap_density":0.4,"gap_resolution_rate":0.75,"centrality_stability":0.02}
```

Thresholds declared in `product.toml` are checked by `product metrics threshold` in CI — this is the architectural fitness function gate. A declining `phi` below 0.70 fails CI just as a broken link does.

`product metrics trend` renders an ASCII chart to terminal for quick visual inspection.

### Pre-Commit Review

`product install-hooks` installs a pre-commit hook that runs `product adr review --staged` before every commit. The hook is advisory — it prints findings but does not block the commit. The CI gap analysis gate is the enforcement point; pre-commit is the fast-feedback loop.

The review checks locally (no LLM, instant):
- Required sections present
- At least one linked feature and one linked TC
- Status field is set
- Evidence blocks present on formal blocks

Then a single LLM call checks:
- Internal consistency of rationale
- Contradiction with linked ADRs
- Obvious missing tests given the claims made

---