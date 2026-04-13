The file write was denied. Here's the complete documentation for FT-012 — Cluster Foundation:

---

## Overview

FT-012 (Cluster Foundation) validates that the Product CLI satisfies its fundamental deployment constraint: a single statically-linked Rust binary that compiles for ARM64 (Apple Silicon, Raspberry Pi), x86_64 (Linux CI), and has no dynamic dependencies beyond libc. This feature exists because Product is a companion tool for PiCloud, which runs on a heterogeneous cluster of ARM64 and x86_64 nodes — every target must produce a working binary from a single `cargo build --release`. The decision to use Rust for this purpose is recorded in ADR-001.

## Tutorial

### Verifying the cluster foundation on your machine

1. Confirm the project compiles cleanly with a release build:

   ```bash
   cargo build --release
   ```

   This must exit with zero errors and zero warnings.

2. Run `product verify` against FT-012 to execute all linked test criteria:

   ```bash
   product verify FT-012
   ```

   Product runs the four scenario tests (TC-001 through TC-004) and the exit-criteria test (TC-163). If all pass, the feature status remains `complete` and `CHECKLIST.md` is regenerated.

3. Inspect the context bundle for the feature to see what ADRs and tests back it:

   ```bash
   product context FT-012
   ```

   The output includes ADR-001 (Rust as Implementation Language) and all five test criteria. This bundle is what `product implement` would pass to a spawned agent if the feature were being implemented.

### Cross-compiling for a specific target

4. Install the ARM64 cross-compilation target (one-time setup):

   ```bash
   rustup target add aarch64-unknown-linux-gnu
   ```

5. Build for ARM64:

   ```bash
   cargo build --release --target aarch64-unknown-linux-gnu
   ```

   TC-001 asserts this completes with zero errors and zero warnings.

6. Build for x86_64 musl (static linking):

   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ```

   TC-002 asserts this completes with zero errors and zero warnings.

## How-to Guide

### Run the full cluster foundation validation

```bash
product verify FT-012
```

This executes all five test criteria linked to FT-012 via their `runner: cargo-test` configuration. On success, each TC's `status` field in its front-matter is updated to `passing`.

### Check the binary for unexpected dynamic dependencies

On a Linux host, after building the release binary:

```bash
ldd target/release/product
```

The output must show no dynamic dependencies beyond `libc`. Any additional shared library is a failure condition (TC-003).

### View the cluster foundation's position in the knowledge graph

```bash
product context FT-012 --depth 2
```

At depth 2, the bundle includes transitive artifacts — other features that share ADR-001 and their test criteria — giving visibility into how the single-binary constraint connects to the rest of the project.

### Check for specification drift

```bash
product drift check
```

If ADR-001 has been modified or if the binary target list has changed without updating the test criteria, drift detection flags the inconsistency.

### Regenerate the checklist after verification

Verification automatically regenerates `CHECKLIST.md`. To regenerate it manually:

```bash
product checklist generate
```

## Reference

### Feature metadata

| Field | Value |
|-------|-------|
| ID | FT-012 |
| Title | Feature: FT-001 — Cluster Foundation |
| Phase | 1 |
| Status | Complete |
| Depends on | (none) |
| ADRs | ADR-001 |
| Test criteria | TC-001, TC-002, TC-003, TC-004, TC-163 |

### Test criteria

| TC | Title | Type | What it validates |
|----|-------|------|-------------------|
| TC-001 | binary_compiles_arm64 | scenario | `cargo build --release --target aarch64-unknown-linux-gnu` exits cleanly |
| TC-002 | binary_compiles_x86 | scenario | `cargo build --release --target x86_64-unknown-linux-musl` exits cleanly |
| TC-003 | binary_no_deps.sh | scenario | `ldd` reports no dynamic deps beyond libc |
| TC-004 | cargo build --release | scenario | Default release build succeeds |
| TC-163 | FT-012 cluster foundation binary validated | exit-criteria | All four scenario TCs pass |

### TC runner configuration

Each TC uses the `cargo-test` runner. The `runner-args` field contains the integration test function name:

```yaml
runner: cargo-test
runner-args: "tc_001_binary_compiles_arm64"
```

The function name follows the pattern `tc_XXX_snake_case_title` and must match a `#[test] fn` in `tests/integration.rs`.

### Compilation targets

| Target triple | Architecture | Use case |
|---------------|-------------|----------|
| `aarch64-unknown-linux-gnu` | ARM64 | Raspberry Pi cluster nodes, Apple Silicon (Linux) |
| `x86_64-unknown-linux-musl` | x86_64 | CI pipelines, Linux developer machines (static) |
| Default host target | Host | Local development (`cargo build --release`) |

### Relevant commands

| Command | Purpose |
|---------|---------|
| `product verify FT-012` | Run all TCs and update status |
| `product context FT-012` | Assemble the context bundle |
| `product gap check` | Detect specification gaps |
| `product drift check` | Detect spec-vs-code drift |
| `product checklist generate` | Regenerate CHECKLIST.md |

## Explanation

### Why a dedicated feature for binary validation

The single-binary deployment constraint is not a nice-to-have — it is the reason Rust was chosen over TypeScript, Go, and Python (ADR-001). If any target fails to compile, or if a dynamic dependency creeps in, the tool cannot be deployed to the PiCloud cluster. FT-012 codifies this constraint as a first-class feature with its own test criteria so that regressions are caught by `product verify`, not by a failed deployment.

### Relationship to FT-001 (Core Concepts)

FT-012 and FT-001 share test criteria TC-001 through TC-004. FT-001 defines the core artifact types and relationships; FT-012 focuses specifically on the binary compilation and dependency constraints that underpin deployment. The overlap in test criteria is intentional — both features require a working binary, but FT-012 makes the deployment constraint explicit and verifiable on its own.

### Why exit-criteria wrap scenario tests

TC-163 is an exit-criteria that aggregates TC-001 through TC-004. This two-level structure serves the verification pipeline: `product verify FT-012` can check a single exit-criteria to determine feature completeness, while the individual scenario tests provide granular diagnostics when something fails. The pattern — scenario tests for individual assertions, exit-criteria for rollup — is defined in ADR-011.

### The single-binary constraint (ADR-001)

ADR-001 mandates Rust specifically because it produces native binaries for ARM64 and x86_64 without a bundled runtime. The rejected alternatives (TypeScript/Node, Go, Python) each failed this constraint in different ways: Node requires a runtime, Go would fragment the toolchain from PiCloud, and Python has no clean single-binary story. The cluster foundation tests are the operational proof that this decision holds — they run on every verification cycle and would fail immediately if a runtime dependency were introduced.

### Cross-compilation and CI

The two explicit target triples (`aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-musl`) reflect the actual deployment targets. ARM64 covers the Raspberry Pi 5 nodes in the PiCloud cluster. x86_64 musl covers CI runners and Linux developer machines with full static linking. The default host target covers local development. All three must succeed for FT-012 to be complete.

---

The document is ~170 lines, covering all five Diataxis sections. Would you like to grant write permission so I can save it to `docs/guide/FT-012-feature-ft-001-cluster-foundation.md`?
