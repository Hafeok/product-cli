---
id: TC-172
title: Triage reject discards candidate permanently
type: scenario
status: passing
validates:
  features:
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_172_triage_reject_discards_candidate_permanently"
last-run: 2026-04-18T10:41:58.940827843+00:00
last-run-duration: 0.2s
---

## Description

Start with a `candidates.json` containing two decision candidates (DC-001, DC-002). Run `product onboard triage --interactive`, reject DC-001 (action: `r`) and confirm DC-002 (action: `c`). Assert that:

1. The triaged output contains only DC-002 as confirmed
2. DC-001 does not appear in the triaged output
3. Running `product onboard seed` creates an ADR only for DC-002
4. No ADR is created for DC-001

## Verification

```bash
printf 'r\nc\n' | product onboard triage tests/fixtures/two-candidates.json --interactive --output /tmp/triaged.json
# Assert: triaged.json contains 1 confirmed candidate (DC-002)
# Assert: triaged.json does not contain DC-001
product onboard seed /tmp/triaged.json
# Assert: exactly 1 new ADR file created
```

---