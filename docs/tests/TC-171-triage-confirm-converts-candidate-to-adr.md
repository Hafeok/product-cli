---
id: TC-171
title: Triage confirm converts candidate to ADR
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Description

Start with a `candidates.json` containing one decision candidate (DC-001). Run `product onboard triage` and confirm (action: `c`) the candidate. Run `product onboard seed` on the triaged output. Assert that:

1. An ADR file is created with the next available ADR ID
2. The ADR body contains a **Context** section derived from the candidate's observation
3. The ADR body contains a **Decision** section derived from the candidate's title
4. The ADR front-matter has `status: proposed`

## Verification

```bash
echo 'c' | product onboard triage tests/fixtures/single-candidate.json --interactive --output /tmp/triaged.json
product onboard seed /tmp/triaged.json
# Assert: new ADR file exists in docs/adrs/
# Assert: ADR body contains observation text from DC-001
# Assert: ADR front-matter status = proposed
```

---