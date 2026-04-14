---
id: TC-177
title: End-to-end onboard produces graph with no structural errors
type: exit-criteria
status: passing
validates:
  features:
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_177_end_to_end_onboard_produces_graph_with_no_structural_errors"
last-run: 2026-04-14T11:01:04.789091923+00:00
---

## Description

Run the full onboard pipeline end-to-end (scan → triage → seed) against a realistic test fixture codebase with at least 20 source files containing deliberate architectural patterns. After seeding, run `product graph check` and assert:

1. Exit code is 0 (clean) or 2 (warnings only)
2. No E001 (malformed front-matter) errors
3. No E002 (broken link) errors
4. No E003 (dependency cycle) errors
5. W001 (orphaned artifacts) and W002 (no tests) warnings are acceptable and expected

This validates that the full onboarding pipeline produces a structurally valid knowledge graph.

## Verification

```bash
product onboard scan tests/fixtures/onboard-realistic/ --output /tmp/candidates.json
# Confirm all candidates
product onboard triage /tmp/candidates.json --output /tmp/triaged.json  # batch confirm
product onboard seed /tmp/triaged.json
product graph check
# Assert: exit code is 0 or 2
# Assert: no E-class errors in output
```

---