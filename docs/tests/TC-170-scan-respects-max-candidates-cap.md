---
id: TC-170
title: Scan respects max-candidates cap
type: scenario
status: passing
validates:
  features:
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_170_scan_respects_max_candidates_cap"
last-run: 2026-04-13T14:45:04.298783396+00:00
---

## Description

Run `product onboard scan` with `--max-candidates 5` against a codebase that contains at least 10 discoverable patterns. Assert that the output `candidates.json` contains at most 5 candidates. Assert that the candidates are ordered by consequence severity (the LLM's assessment of violation impact), not by file order or alphabetical title.

## Verification

```bash
product onboard scan tests/fixtures/onboard-large/ --max-candidates 5 --output /tmp/candidates.json
# Assert: len(candidates) <= 5
# Assert: candidates are the highest-consequence subset
```

---