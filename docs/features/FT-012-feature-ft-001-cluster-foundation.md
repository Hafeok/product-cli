---
id: FT-012
title: 'Feature: FT-001 — Cluster Foundation'
phase: 1
status: complete
depends-on: []
adrs:
- ADR-001
tests:
- TC-001
- TC-002
- TC-003
- TC-004
- TC-163
domains: []
domains-acknowledged:
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
---

[full content of FT-001-cluster-foundation.md, front-matter stripped]

---

---

## Description

FT-012 represents the cluster-foundation baseline: the Product binary compiles and ships as a single self-contained executable for both ARM64 (aarch64-unknown-linux-gnu) and x86_64 (x86_64-unknown-linux-musl) targets, with no dynamic dependencies beyond `libc`. This validates the deployment constraint from ADR-001 (Rust as Implementation Language) — a single binary, no runtime, no installer.

## Functional Specification

### Inputs

- Source tree at the repository root
- Target architecture spec (`aarch64-unknown-linux-gnu` or `x86_64-unknown-linux-musl`)
- `cargo build --release` invocation

### Outputs

- A compiled release binary (`product`) at `target/<triple>/release/product`
- Exit code 0 on success; non-zero (with compiler diagnostics to stderr) on failure

### State

Stateless at runtime. The build process produces a binary artefact on disk; the feature itself describes a compilation and deployment property, not runtime state.

### Behaviour

1. `cargo build --release --target aarch64-unknown-linux-gnu` completes with zero errors and zero warnings (TC-001).
2. `cargo build --release --target x86_64-unknown-linux-musl` completes with zero errors and zero warnings (TC-002).
3. `ldd product` on the Linux binary reports no dynamic dependencies beyond `libc` — any additional shared library dependency is a test failure (TC-003).
4. `cargo build --release` (default host target) succeeds (TC-004).
5. All TC-163 exit criteria pass: the binary meets the single-binary, no-runtime deployment constraint across all targets.

### Invariants

- The compiled binary must link only against `libc`; no other shared libraries are permitted (ADR-001).
- `cargo build --release` must produce zero compiler errors and zero clippy warnings (`-D warnings -D clippy::unwrap_used`).
- Both ARM64 and x86_64 targets must compile successfully from the same source tree.

### Error handling

- Compilation errors surface as non-zero `cargo` exit codes with rustc diagnostics to stderr; no Product-specific error handling applies at this level.
- If a new dependency introduces a dynamic link beyond libc, TC-003 fails and the CI gate blocks the commit.

### Boundaries

- This feature governs the binary compilation and deployment constraint only; it does not specify runtime behaviour.
- Cross-compilation toolchain availability (e.g. `cross`, linker configuration) is a CI infrastructure concern, not part of this feature.

## Out of scope

- Runtime feature behaviour (covered by other FT entries).
- Windows or macOS binary artefacts (not currently targeted).
- `cargo install` packaging or release distribution (covered by the release workflow).
