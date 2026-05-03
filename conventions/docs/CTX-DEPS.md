---
id: CTX-DEPS
title: Supply-chain rules enforced by cargo-deny
severity: deny
tier: 3
mechanism: cargo-deny
adrs: [ADR-0003]
applies_to: ["Cargo.toml", "Cargo.lock", "deny.toml"]
exclude: []
---

# CTX-DEPS — Supply-chain rules

`cargo deny check` runs in CI and fails the build on:

- **Vulnerable dependencies.** Any crate with a matching RustSec advisory.
- **Yanked dependencies.** Warning only — yanks are usually maintainer
  errors, but they break reproducible builds.
- **Disallowed licenses.** The allow-list in `deny.toml` is the only set of
  licenses we ship under. Adding a new one requires a deliberate PR with
  rationale.
- **Wildcard version requirements.** `foo = "*"` in `Cargo.toml` is denied;
  versions must be pinned.
- **Unknown registries or git sources.** Only crates.io is allowed by
  default. Adding a git source requires updating `deny.toml`.

## Why

The CLI is shipped as prebuilt binaries (see `dist-workspace.toml`) and is
embedded in CI pipelines (Dagger module under `dagger/`). A vulnerable
transitive dependency or an incompatibly-licensed crate becomes everyone's
problem the moment we publish a release. Detecting these at PR time is the
only point in the pipeline where a rejection is cheap.

See [ADR-0003](../adr/ADR-0003-cargo-deny.md) for rationale and rejected
alternatives.

## How to fix a violation

| Failure | Action |
|---|---|
| `RUSTSEC-####` advisory | Bump the dependency. If no fix exists, file an issue and add an `ignore` entry in `deny.toml` with a deadline. |
| `license not allowed` | Confirm the license is acceptable, then add it to `deny.toml [licenses] allow`. Open a separate PR for the policy change. |
| `wildcard version` | Pin the version explicitly in `Cargo.toml`. |
| `unknown source` | Either add the registry/git URL to `deny.toml` or replace the dependency with a crates.io equivalent. |
| `multiple versions` | Try `cargo update -p <crate>` to converge. If two versions are unavoidable (transitive lock), the warning is acceptable. |

## Enforcement

- **Tier:** 3c (cargo-deny).
- **Configuration:** `deny.toml` at the workspace root.
- **CI step:** `cargo deny check --workspace` via
  `EmbarkStudios/cargo-deny-action@v2`.
- **Promotion path:** rules here cannot move to a higher tier — supply-chain
  policy is inherently external.
