# ADR-0003: Adopt cargo-deny for supply-chain policy

**Status:** Accepted
**Date:** 2026-05-03
**Deciders:** Engineering team
**Convention:** [CTX-DEPS](../docs/CTX-DEPS.md)

## Context

The `product` CLI ships as prebuilt binaries via `cargo-dist` and runs as a
trusted subprocess inside Dagger pipelines. Every transitive dependency we
pull in becomes part of the trust boundary of every consumer. We need a
deterministic check at PR time that:

- Surfaces RustSec advisories before they reach a release.
- Enforces a license allow-list so we don't accidentally ship under a
  copyleft we haven't reviewed.
- Forbids wildcard versions and unknown sources, which break reproducible
  builds.

`cargo audit` covers advisories alone. `cargo about` checks licenses but
requires manual configuration. `cargo-deny` consolidates all four
concerns (advisories, licenses, bans, sources) into one config file and
one CI step. It is the de-facto standard in the Rust ecosystem.

## Decision

Adopt `cargo-deny` with `deny.toml` at the workspace root. CI runs
`cargo deny check --workspace` on every PR via
`EmbarkStudios/cargo-deny-action@v2`. The policy starts strict (deny on
unknown sources, deny on wildcards, explicit license allow-list) and
relaxes only when a specific dependency requires it.

## Alternatives considered

- **`cargo audit` only.** Rejected: covers advisories but not licenses or
  source restrictions. We'd still need a second tool.
- **No supply-chain check at all, rely on Dependabot.** Rejected:
  Dependabot reacts to advisories *after* they're published; it doesn't
  enforce licenses or source policies, and it operates by opening PRs
  rather than blocking them.
- **Custom xtask check that wraps `cargo metadata`.** Rejected: would
  duplicate years of work in cargo-deny for no upside. Tier 3c is the
  right home for this rule.

## Consequences

- Adding a dependency with a new license requires updating `deny.toml`
  in the same PR. This adds friction by design.
- A new RustSec advisory on an existing dependency breaks CI until we
  bump or explicitly ignore. Acceptable cost.
- The `EmbarkStudios/cargo-deny-action` adds ~30s to CI. Acceptable.

## References

- cargo-deny: <https://embarkstudios.github.io/cargo-deny/>
- RustSec: <https://rustsec.org/>
- `dist-workspace.toml` — release pipeline that ships the binaries.
