---
id: TC-560
title: verify_log_integrity_stage_1
type: scenario
status: passing
validates:
  features:
  - FT-044
  adrs:
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_560_verify_log_integrity_stage_1
last-run: 2026-04-18T10:54:35.903075401+00:00
last-run-duration: 0.2s
---

## Session: ST-118 — verify-log-integrity-stage-1

**Validates:** FT-044, ADR-040 (Stage 1 detects tampered log)

### Given

A temp repository with a valid hash-chained `requests.jsonl` whose most recent entry is modified in place (simulating tampering) so the recomputed hash no longer matches the stored next-hash.

### When

`product verify` is run.

### Then

- Stage 1 (log-integrity) emits `fail` with finding code `E015` or `E016`.
- All six stages still complete — stage 1 failure does not short-circuit.
- Exit code is `1`.
- `--ci` JSON: `stages[0].status == "fail"`, `findings` contains the E-code.