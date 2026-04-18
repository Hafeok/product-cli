---
id: TC-566
title: gap_check_structural_only_no_llm_call
type: invariant
status: passing
validates:
  features:
  - FT-045
  adrs:
  - ADR-019
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_566_gap_check_structural_only_no_llm_call
last-run: 2026-04-18T14:07:23.114790413+00:00
last-run-duration: 0.2s
---

## Invariant: ST-123 — gap-check-structural-only-no-llm-call

**Validates:** FT-045, ADR-019 (amended), ADR-040 (LLM boundary invariant)

⟦Γ:Invariants⟧{
  gap_check_makes_zero_outbound_network_calls
  gap_check_does_not_read_api_key_environment_variables
  gap_check_completes_in_under_one_second_on_realistic_repos
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

### Evidence

- The `gap::check_structural` function has no network client in its transitive call graph (static check by inspecting `Cargo.toml` transitive dependencies).
- Running `product gap check` in a sandbox with no network access completes successfully.
- Running `product gap check` with a strace / dtrace filter on `connect()` syscalls produces zero `connect()` calls to any non-loopback address.
- Wall-clock time on a repository with 500 combined artifacts is under one second.