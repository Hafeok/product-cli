---
id: TC-174
title: Seed creates ADR files with correct front-matter
type: scenario
status: passing
validates:
  features:
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_174_seed_creates_adr_files_with_correct_front_matter"
last-run: 2026-04-14T18:29:03.925615608+00:00
last-run-duration: 0.3s
---

## Description

Run the full onboard pipeline (scan → triage with all confirmed → seed) against a test fixture codebase. Assert that each seeded ADR file has correct YAML front-matter:

1. `id` follows the `ADR-XXX` pattern with the next available sequence number
2. `status` is `proposed`
3. `features` is empty (linked later during feature stub creation or manually)
4. `supersedes` and `superseded-by` are empty
5. The front-matter is valid YAML parseable by Product's own parser
6. `product graph check` does not report E001 (malformed front-matter) for any seeded file

## Verification

```bash
# Full pipeline against fixture
product onboard scan tests/fixtures/onboard-sample/ --output /tmp/candidates.json
printf 'c\nc\nc\n' | product onboard triage /tmp/candidates.json --interactive --output /tmp/triaged.json
product onboard seed /tmp/triaged.json
# Assert: each new ADR file has valid front-matter
# Assert: product graph check reports no E001 errors
```

---