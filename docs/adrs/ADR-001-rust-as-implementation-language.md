---
id: ADR-001
title: Rust as Implementation Language
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:9c60e574568e8093eaa6b90eccb22267b4c227478a42d5a6e2731d03803cb19c
---

**Status:** Accepted

**Context:** Product must ship as a single binary with no runtime dependencies. It needs to run on developer laptops (ARM64 Apple Silicon, x86_64 Linux), in CI pipelines, and eventually on ARM64 Raspberry Pi nodes alongside PiCloud itself. The tool parses files, builds an in-memory graph, and assembles markdown output — none of which are exotic requirements, but the deployment constraint (single binary, no installer, no runtime) is strict.

Additionally, Product is a companion tool for PiCloud, which is written in Rust. Shared language means shared tooling, shared CI patterns, and the ability to eventually share libraries (particularly the Oxigraph integration).

**Decision:** Implement Product in Rust.

**Rationale:**
- Single binary compilation to ARM64, x86_64, and Apple Silicon with no runtime
- Cargo cross-compilation is well-understood; CI matrix builds across targets are straightforward
- `clap` provides a production-quality CLI argument parser with shell completion generation
- `oxigraph` for embedded SPARQL is a Rust-native library — no FFI required
- `gray_matter` and `pulldown-cmark` handle YAML front-matter and markdown parsing
- Alignment with PiCloud's toolchain — one language, one formatter, one linter across the project
- LLMs produce high-quality Rust when given explicit type contracts and clear architectural context

**Rejected alternatives:**
- **TypeScript/Node** — natural first choice for a CLI that iterates fast; `gray-matter` is excellent. Rejected because it requires Node.js to be installed, which violates the single-binary constraint. `pkg`/`bun` can bundle Node apps, but the result is a large binary with bundled runtime, not a native binary.
- **Go** — would satisfy the single-binary constraint and has good CLI tooling (`cobra`). Rejected because it would fragment the toolchain from PiCloud. The development overhead of maintaining two language ecosystems on a small project is not justified.
- **Python** — fast iteration, good parsing libraries. Rejected due to runtime dependency and the absence of a clean single-binary story.