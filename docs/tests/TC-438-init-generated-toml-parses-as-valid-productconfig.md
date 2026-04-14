---
id: TC-438
title: init generated toml parses as valid ProductConfig
type: invariant
status: passing
validates:
  features:
  - FT-035
  adrs:
  - ADR-033
phase: 1
runner: cargo-test
runner-args: tc_438_init_generated_toml_parses_as_valid_productconfig
last-run: 2026-04-14T14:52:43.866547207+00:00
---

## Description

Property test: generate random combinations of init flags (`--name` with arbitrary strings, 0-5 `--domain` entries with arbitrary key=value pairs, `--port` with arbitrary u16, with and without `--write-tools`). For each combination, run `product init --yes` and then load the resulting `product.toml` via `ProductConfig::load()`. Assert:

1. `ProductConfig::load()` succeeds (no parse error) for every generated combination.
2. `check_schema_version()` returns Ok for every generated config.
3. All paths resolve to valid relative directory strings (no absolute paths, no `..` traversal).

## Formal Specification

⟦Σ:Types⟧{
  Flags ≜ { name: String, domains: Vec<(String,String)>, port: u16, write_tools: bool }
  Config ≜ ProductConfig
  Path ≜ String
}

⟦Γ:Invariants⟧{
  ∀f:Flags: ProductConfig::load(init(f)) ∈ Ok(Config)

  ∀f:Flags: check_schema_version(init(f)) ∈ Ok(())

  ∀f:Flags, ∀p:Config.paths(init(f)): ¬starts_with(p, "/") ∧ ¬contains(p, "..")
}

⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◇⁺⟩