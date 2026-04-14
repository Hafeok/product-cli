## Overview

Cluster Foundation establishes the deployment baseline for Product CLI: a single statically-linked Rust binary that compiles cleanly for both ARM64 and x86_64 targets with no runtime dependencies beyond libc. This foundation exists because Product must run on developer laptops, CI pipelines, and Raspberry Pi nodes without an installer or language runtime, as decided in ADR-001.

## Tutorial

### Verify your build environment

Before working with cross-compilation targets, confirm that the default release build succeeds on your machine.

1. Build the release binary:

   ```bash
   cargo build --release
   ```

2. Confirm the binary exists:

   ```bash
   ls -lh target/release/product
   ```

3. Run the binary to verify it works:

   ```bash
   ./target/release/product --help
   ```

### Cross-compile for ARM64

If you are deploying to a Raspberry Pi or another ARM64 target, build for `aarch64-unknown-linux-gnu`.

1. Install the cross-compilation target (one-time setup):

   ```bash
   rustup target add aarch64-unknown-linux-gnu
   ```

2. Build the release binary for ARM64:

   ```bash
   cargo build --release --target aarch64-unknown-linux-gnu
   ```

3. Verify the build completes with zero errors and zero warnings.

### Cross-compile for x86_64 (musl)

For fully static Linux binaries targeting x86_64, build against musl.

1. Install the target:

   ```bash
   rustup target add x86_64-unknown-linux-musl
   ```

2. Build:

   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ```

3. Verify the build completes with zero errors and zero warnings.

### Check dynamic dependencies

After building a Linux binary, confirm it has no unexpected dynamic dependencies.

1. Run `ldd` against the compiled binary:

   ```bash
   ldd target/release/product
   ```

2. The output should list only `libc` (and its loader). Any other dynamic library indicates a dependency that violates the single-binary constraint.

## How-to Guide

### How to validate the full cluster foundation

Run all checks in sequence to confirm the deployment baseline holds.

1. Build the release binary:

   ```bash
   cargo build --release
   ```

2. Run clippy with the project's lint policy:

   ```bash
   cargo clippy -- -D warnings -D clippy::unwrap_used
   ```

3. Run the test suite:

   ```bash
   cargo test
   ```

4. Verify the feature using Product's test runner:

   ```bash
   product verify FT-012
   ```

5. Confirm all linked test criteria (TC-001 through TC-004, TC-156, TC-163, TC-164) report as passing.

### How to check that a new dependency does not break the single-binary constraint

When adding a crate to `Cargo.toml`, verify it does not pull in system libraries.

1. Add the dependency and rebuild:

   ```bash
   cargo build --release
   ```

2. Check for new dynamic dependencies:

   ```bash
   ldd target/release/product
   ```

3. If `ldd` reports libraries beyond `libc`, investigate the new crate. Prefer crates that are pure Rust or that link statically.

### How to verify the Rust edition and build cleanliness

1. Check the edition field in `Cargo.toml`:

   ```bash
   grep '^edition' Cargo.toml
   ```

   It must be `2021` or later.

2. Run a clean, warning-free build:

   ```bash
   cargo build --release 2>&1
   ```

   Zero errors and zero warnings are required.

3. Run clippy to enforce the zero-unwrap policy:

   ```bash
   cargo clippy -- -D warnings -D clippy::unwrap_used
   ```

## Reference

### Supported compilation targets

| Target triple                    | Architecture | Use case                        |
|----------------------------------|--------------|---------------------------------|
| `aarch64-unknown-linux-gnu`      | ARM64        | Raspberry Pi, ARM64 Linux hosts |
| `x86_64-unknown-linux-musl`      | x86_64       | CI pipelines, x86 Linux servers |
| Default host target              | varies       | Developer laptops               |

### Test criteria linked to FT-012

| TC ID  | Title                                        | Type          | What it validates                                                        |
|--------|----------------------------------------------|---------------|--------------------------------------------------------------------------|
| TC-001 | binary_compiles_arm64                        | scenario      | `cargo build --release --target aarch64-unknown-linux-gnu` succeeds      |
| TC-002 | binary_compiles_x86                          | scenario      | `cargo build --release --target x86_64-unknown-linux-musl` succeeds      |
| TC-003 | binary_no_deps.sh                            | scenario      | `ldd` reports no dynamic dependencies beyond libc                        |
| TC-004 | cargo build --release                        | scenario      | Default release build succeeds                                           |
| TC-156 | FT-001 core concepts validated               | exit-criteria | Aggregates core concept scenarios (TC-011 through TC-015, TC-001–TC-004) |
| TC-163 | FT-012 cluster foundation binary validated   | exit-criteria | All cluster foundation scenarios pass                                    |
| TC-164 | FT-013 Rust implementation compiles clean    | exit-criteria | Clean build, zero clippy warnings, Rust edition 2021+                    |

### Build commands

```bash
# Default release build
cargo build --release

# ARM64 cross-compilation
cargo build --release --target aarch64-unknown-linux-gnu

# x86_64 static (musl) build
cargo build --release --target x86_64-unknown-linux-musl

# Lint check (project policy)
cargo clippy -- -D warnings -D clippy::unwrap_used

# Dependency audit
ldd target/release/product
```

### Exit codes

Build and clippy commands use standard Cargo exit codes: `0` for success, non-zero for failure. Any non-zero exit from `cargo build` or `cargo clippy` means the cluster foundation constraint is not met.

## Explanation

### Why a single binary matters

Product is designed to run across heterogeneous environments — developer laptops on macOS (Apple Silicon), Linux CI runners, and ARM64 Raspberry Pi nodes in PiCloud clusters. Requiring users to install a language runtime (Node.js, Python, Go runtime) or manage system-level dependencies would create friction at every deployment point. A single binary with no dependencies beyond libc eliminates this class of deployment problems entirely.

### Why Rust (ADR-001)

ADR-001 records the decision to implement Product in Rust. The key factors were:

- **Single-binary compilation** across ARM64, x86_64, and Apple Silicon with no runtime
- **Toolchain alignment** with PiCloud, which is also written in Rust — one language, one formatter, one linter, and eventually shared libraries
- **Ecosystem fit** — `clap` for CLI parsing, `oxigraph` for embedded SPARQL, and `serde` for YAML/JSON are all native Rust crates requiring no FFI

TypeScript, Go, and Python were considered and rejected. TypeScript requires a bundled runtime; Go would fragment the toolchain from PiCloud; Python lacks a clean single-binary story. See ADR-001 for full rationale and rejected alternatives.

### The role of musl for static linking

The `x86_64-unknown-linux-musl` target produces a fully statically-linked binary, meaning `ldd` reports "not a dynamic executable." This is the strongest form of the no-dependencies guarantee. The `aarch64-unknown-linux-gnu` target links against glibc dynamically, which is acceptable because libc is universally present on Linux systems.

### How cluster foundation relates to other features

Cluster Foundation is a Phase 1 prerequisite. Every other feature in the Product CLI depends on the binary existing and running correctly. The test criteria (TC-001 through TC-004) form the base layer of the validation pyramid — if these fail, nothing else can be verified. TC-163 aggregates these scenarios into a single exit-criteria gate for FT-012, ensuring the foundation is validated as a unit before dependent features proceed.
