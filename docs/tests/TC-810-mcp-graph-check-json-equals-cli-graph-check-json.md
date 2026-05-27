---
id: TC-810
title: mcp_graph_check_json_equals_cli_graph_check_json
type: invariant
status: passing
validates:
  features:
  - FT-069
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_810_mcp_graph_check_json_equals_cli_graph_check_json
last-run: 2026-05-27T11:04:45.120555493+00:00
last-run-duration: 0.2s
---

## Invariant

For every fixture produced by the parity harness, the MCP
`product_graph_check` JSON envelope equals the CLI `product graph
check --format json` envelope **byte-for-byte** (after ordering-
normalisation: findings sorted by code then file).

## Formal

⟦Σ:Types⟧{
  Finding   ≜ ⟨code:String, tier:Tier, file:Path?, line:Nat?, detail:String?⟩
  Envelope  ≜ ⟨errors:Finding*, warnings:Finding*, summary:⟨errors:Nat, warnings:Nat⟩⟩
  Fixture   ≜ Repo
  Normalise ≜ Envelope → Envelope    (sort findings by (code, file, line, detail))
}

⟦Γ:Invariants⟧{
  ∀ f:Fixture:
    Normalise(mcp_envelope(f)) = Normalise(cli_envelope(f))
}

⟦Ε⟧⟨δ≜0.92;φ≜100;τ≜◊⁺⟩

## Implementation

The test enumerates a curated `Fixtures(graph_check)` set covering:

- Clean repo (no findings).
- W030 trigger (TC-806 fixture).
- E011 trigger (TC-807 fixture).
- W028 trigger (TC-808 fixture).
- Log-verify trigger (TC-809 fixture, with `[log].verify-on-check =
  true`).
- All-triggers-at-once compound fixture.

For each fixture, the test spawns the binary in MCP stdio mode and
in CLI `--format json` mode and compares the normalised envelopes
with `assert_eq!`. Any mismatch fails the invariant.

This TC is the parity guard: future validation layers added to the
CLI must route through `graph::full_check::run`, otherwise this
test fails immediately.