---
id: TC-631
title: builder_submit_blocked_on_e_class_leaves_draft_unchanged
type: scenario
status: unimplemented
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
---

## Session — builder-submit-blocked-on-e-class-errors

### Given

A draft containing a dep with no governing ADR in the draft or
the existing graph (an E013 finding at validate time).

### When

The user runs `product request submit`.

### Then

- The command refuses to apply and prints the E013 finding with a
  JSONPath `location:`.
- The draft file at `.product/requests/draft.yaml` is unchanged
  (byte-identical SHA-256 to pre-submit).
- No artifact files are written.
- No entry is appended to `.product/request-log.jsonl`.
- Exit code is 1.
