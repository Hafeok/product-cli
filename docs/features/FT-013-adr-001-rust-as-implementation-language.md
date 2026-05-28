---
id: FT-013
title: ADR-001 — Rust as Implementation Language
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
- TC-164
domains: []
domains-acknowledged:
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

[full content of ADR-001-rust-language.md, front-matter stripped]

---

---

## Description

FT-013 validates ADR-001 — the decision to implement Product in Rust. The feature ensures that the codebase compiles cleanly with `cargo build --release`, passes `cargo clippy -- -D warnings -D clippy::unwrap_used` with zero warnings, and uses the Rust 2021 edition or newer. It provides confidence that the Rust-as-implementation-language decision is actively exercised and the code quality bar enforced by the toolchain is maintained.

## Functional Specification

### Inputs

- The Product source tree (`src/`, `Cargo.toml`, `Cargo.lock`)
- `cargo build --release` and `cargo clippy -- -D warnings -D clippy::unwrap_used` invocations
- `Cargo.toml` edition field

### Outputs

- Zero-error, zero-warning release build
- Zero clippy findings under `-D warnings -D clippy::unwrap_used`
- `edition = "2021"` (or later) confirmed in `Cargo.toml`

### State

Stateless. This feature describes a code quality and toolchain property of the repository, not runtime state.

### Behaviour

1. `cargo build --release` completes with zero errors and zero warnings (inherits from TC-001 through TC-004, confirmed by TC-164).
2. `cargo clippy -- -D warnings -D clippy::unwrap_used` produces zero diagnostics — the zero-unwrap policy (`#![deny(clippy::unwrap_used)]`) is enforced at the compiler level.
3. `Cargo.toml` declares `edition = "2021"` or a later Rust edition.
4. The binary is a native Rust executable with no FFI runtime dependencies beyond `libc`.

### Invariants

- `clippy::unwrap_used` is denied crate-wide; any `.unwrap()` call in production code is a compilation error.
- `-D warnings` ensures no deprecated API usage, unused imports, or unreachable code reaches the main branch.
- The pinned toolchain version in `rust-toolchain.toml` ensures consistent behaviour across local and CI environments.

### Error handling

- Clippy violations surface as compiler errors (due to `-D warnings`), blocking the build.
- Any new dependency that introduces `unsafe` or `unwrap()` calls must be addressed before the build succeeds.

### Boundaries

- This feature describes language and toolchain requirements, not functional CLI behaviour.
- It does not govern which Rust crates are chosen — that is covered by the specific ADRs that select individual libraries (e.g. ADR-008 for Oxigraph).

## Out of scope

- CI pipeline configuration details (covered by CI workflow files, not this feature).
- Cross-compilation targets (covered by FT-012).
- Runtime performance benchmarks (covered by FT-024 and the bench suite).
