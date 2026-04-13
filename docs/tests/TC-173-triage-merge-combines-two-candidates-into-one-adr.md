---
id: TC-173
title: Triage merge combines two candidates into one ADR
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Description

Start with a `candidates.json` containing two candidates that describe the same decision from different angles:
- DC-001: "Database access exclusively through the repository layer" (boundary signal, evidence from `src/repo/`)
- DC-002: "No direct sqlx imports outside the repository module" (absence signal, evidence from `src/handlers/`)

Run `product onboard triage --interactive`, merge DC-002 into DC-001 (action: `m`, target: `DC-001`). Assert that:

1. The triaged output contains one merged candidate with the title from DC-001
2. The merged candidate's evidence array contains entries from both DC-001 and DC-002
3. Running `product onboard seed` creates exactly one ADR, not two
4. The ADR body references evidence from both original candidates

## Verification

```bash
printf 'm\nDC-001\nc\n' | product onboard triage tests/fixtures/merge-candidates.json --interactive --output /tmp/triaged.json
# Assert: triaged.json contains 1 candidate with combined evidence
product onboard seed /tmp/triaged.json
# Assert: exactly 1 new ADR file created
# Assert: ADR body references files from both original candidates
```

---