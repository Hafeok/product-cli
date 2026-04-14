## Overview

FT-013 captures the decision to implement Product in Rust (ADR-001). This is a foundational architectural choice driven by the single-binary deployment constraint: Product must ship as one executable with no runtime dependencies, targeting ARM64 Apple Silicon, x86_64 Linux, and ARM64 Raspberry Pi. Rust satisfies this constraint while aligning with the PiCloud toolchain, enabling shared CI patterns, formatting, and linting across both projects.

## Tutorial

### Verifying the Rust toolchain

Before working on Product, confirm your Rust toolchain is set up correctly.

1. Check that Rust is installed and reports a supported edition:

   ```bash
   rustc --version
   ```

2. Compile Product in debug mode:

   ```bash
   cargo build
   ```

3. Run the full test suite to confirm everything passes:

   ```bash
   cargo test
   ```

4. Run the linter with the project's zero-unwrap policy:

   ```bash
   cargo clippy -- -D warnings -D clippy::unwrap_used
   ```

   A clean run produces no output. Any warning is treated as an error.

5. Build a release binary:

   ```bash
   cargo build --release
   ```

   The resulting binary is at `target/release/product`.

### Confirming the single-binary property

After building a release binary on Linux, verify it has no unexpected dynamic dependencies:

```bash
ldd target/release/product
```

The output should list only `libc` (and its loader). Any additional shared library indicates a dependency violation.

## How-to Guide

### Build for a specific target architecture

#### ARM64 (aarch64)

1. Add the cross-compilation target:

   ```bash
   rustup target add aarch64-unknown-linux-gnu
   ```

2. Build the release binary:

   ```bash
   cargo build --release --target aarch64-unknown-linux-gnu
   ```

3. The binary is at `target/aarch64-unknown-linux-gnu/release/product`.

#### x86_64 (musl, fully static)

1. Add the musl target:

   ```bash
   rustup target add x86_64-unknown-linux-musl
   ```

2. Build:

   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ```

3. The binary is at `target/x86_64-unknown-linux-musl/release/product`.

### Run all quality checks before committing

All three checks must pass before any commit:

1. Build:

   ```bash
   cargo build
   ```

2. Test:

   ```bash
   cargo test
   ```

3. Lint:

   ```bash
   cargo clippy -- -D warnings -D clippy::unwrap_used
   ```

### Check that the Rust edition is current

1. Open `Cargo.toml` and look for the `edition` key:

   ```toml
   [package]
   edition = "2021"
   ```

2. The edition must be 2021 or later. This is validated by TC-164.

## Reference

### Build commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Debug build |
| `cargo build --release` | Optimized release build |
| `cargo build --release --target aarch64-unknown-linux-gnu` | ARM64 cross-compile |
| `cargo build --release --target x86_64-unknown-linux-musl` | x86_64 static binary |
| `cargo test` | Run all tests (unit, integration, property) |
| `cargo clippy -- -D warnings -D clippy::unwrap_used` | Lint with zero-unwrap policy |
| `cargo bench` | Run benchmarks |

### Key dependencies enabling the Rust choice

| Crate | Role |
|-------|------|
| `clap` | CLI argument parsing with shell completion generation |
| `oxigraph` | Embedded SPARQL engine (Rust-native, no FFI) |
| `serde` / `serde_yaml` / `serde_json` / `toml` | Serialization for YAML front-matter, JSON, and TOML config |
| `axum` / `tokio` | HTTP server for the MCP endpoint |
| `sha2` | Content hashing |
| `fd-lock` | Advisory file locking for atomic writes |

### Supported targets

| Target triple | Architecture | Notes |
|---------------|-------------|-------|
| `aarch64-unknown-linux-gnu` | ARM64 Linux | Raspberry Pi, ARM servers |
| `x86_64-unknown-linux-musl` | x86_64 Linux (static) | CI pipelines, Linux desktops |
| `aarch64-apple-darwin` | ARM64 macOS | Apple Silicon laptops |

### Related test criteria

| TC | Title | Type |
|----|-------|------|
| TC-164 | Rust implementation compiles clean | exit-criteria |
| TC-001 | binary_compiles_arm64 | scenario |
| TC-002 | binary_compiles_x86 | scenario |
| TC-003 | binary_no_deps.sh | scenario |
| TC-004 | cargo build --release | scenario |
| TC-156 | FT-001 core concepts validated | exit-criteria |
| TC-163 | FT-012 cluster foundation binary validated | exit-criteria |

## Explanation

### Why Rust over other languages

The single-binary constraint is the primary driver. Product must run on developer laptops, CI pipelines, and Raspberry Pi nodes without requiring users to install a runtime. This eliminates TypeScript/Node (requires Node.js), Python (requires the Python interpreter), and narrows the field to compiled languages that produce static or near-static binaries.

Go would satisfy the binary constraint and has mature CLI tooling via Cobra. However, PiCloud — the companion project — is already written in Rust. Maintaining two language ecosystems on a small project doubles the tooling surface (formatters, linters, CI configurations, dependency management) without proportional benefit. A single-language stack means one `rustfmt`, one `clippy`, one `Cargo.toml` dependency resolution strategy across both projects.

### The zero-unwrap policy

The project enforces `#![deny(clippy::unwrap_used)]`. Every fallible operation must use `?`, `.ok_or()`, `.unwrap_or_default()`, or explicit `match`. This eliminates panics from `.unwrap()` calls and ensures all error paths flow through the `ProductError` enum in `error.rs`, where each variant maps to a specific exit code. See ADR-001 for the original decision context.

### Ecosystem alignment

Several Rust crates directly enable Product's architecture:

- **Oxigraph** provides an embedded SPARQL engine written in Rust. Because it is a native library rather than an FFI binding, it compiles into the single binary without external dependencies. This powers the RDF export and SPARQL query features.
- **Clap** gives Product its CLI structure, including automatic help text, shell completions, and subcommand routing — all derived from Rust type definitions.
- **Serde** with its YAML, JSON, and TOML backends handles all the front-matter parsing that drives the knowledge graph.

### LLM code generation

ADR-001 notes that LLMs produce high-quality Rust when given explicit type contracts and clear architectural context. Product's design leans into this: the `ProductError` enum, strongly typed artifact structs in `types.rs`, and the graph model in `graph.rs` all provide the kind of explicit contracts that make LLM-assisted development reliable. This is a deliberate architectural choice, not an incidental benefit.
