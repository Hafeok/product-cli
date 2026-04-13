---
id: TC-169
title: Scan rejects candidates citing non-existent files
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Description

Run `product onboard scan` against a test codebase. Manually inject a candidate with a fabricated file path (`src/nonexistent.rs`) into the scan output. Assert that evidence post-validation flags the candidate with a warning and marks `evidence_valid: false` on the affected evidence entry.

Alternatively: configure a scan where the LLM is mocked to return a candidate citing `src/fake_module.rs:42`. Assert the post-validation step emits a warning on stderr and the candidate's evidence entry is flagged.

## Verification

```bash
# candidates.json contains a candidate with evidence pointing to non-existent file
product onboard scan tests/fixtures/onboard-sample/ --output /tmp/candidates.json
# Assert: any candidate with invalid evidence has a warning attached
# Assert: valid candidates are unaffected
```

---