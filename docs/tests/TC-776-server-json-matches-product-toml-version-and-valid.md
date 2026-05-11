---
id: TC-776
title: server.json matches product.toml version and validates against pinned schema
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_776_server_json_matches_product_toml_version_and_validates_against_pinned_schema
---

## Purpose

Catch drift between `server.json` (the MCP-registry publishing manifest)
and `product.toml` (the source-of-truth version) before any release
workflow runs. Without this guard, a maintainer can ship a release tag
whose registry entry advertises a stale version, or whose manifest fails
schema validation only after the publisher CLI rejects it in CI.

## Scenario

Given a repo root containing both `server.json` and `product.toml`:

1. **Parse `product.toml`** and read the top-level `version` string.
2. **Parse `server.json`** and read:
   - `name` (must equal the expected `io.github.{owner}/product-cli`
     constant).
   - `version_detail.version`.
   - `$schema` (must equal the dated registry schema URL the project
     has pinned).
   - At least one entry under `packages`.
3. **Assert** that `server.json`'s `version_detail.version` equals
   `product.toml`'s `version` byte-for-byte.
4. **Validate** `server.json` against the offline schema fixture at
   `tests/fixtures/server.schema.json`. The fixture is a verbatim copy
   of the schema URL pinned by `$schema`; refreshing the schema is a
   deliberate fixture-update commit.
5. **Pass** on success; **fail** with a descriptive diff or
   schema-validator finding on any mismatch.

## Failure modes covered

- `server.json` has a stale `version_detail.version` after a `product.toml`
  bump.
- `server.json` has a stale `version_detail.version` after a release tag
  that updated `product.toml` but not the manifest.
- `name` typo (e.g. `product` vs `product-cli`).
- `$schema` accidentally rolled forward without refreshing the
  fixture (catches the case where the manifest claims a newer schema
  than the test fixture asserts against).
- Missing `packages` array.
- Any other schema-required field omitted (`description`,
  `repository.url`, etc.).

## Out of scope

- **Live registry queries.** The test runs offline against the
  shipped schema fixture; it does not reach out to
  `registry.modelcontextprotocol.io`.
- **Authentication checks.** GitHub OIDC behaviour is the workflow's
  responsibility; this TC validates the manifest itself, not the
  publish ritual.
- **Package-artifact existence.** The TC does not verify that the
  GitHub Release identified by `packages[0].identifier` actually has
  binaries attached — that's an end-to-end concern owned by the release
  workflow.

## Runner

`cargo-test` — the test lives in `tests/integration.rs` as
`tc_776_server_json_matches_product_toml_version_and_validates_against_pinned_schema`,
following the CLAUDE.md naming convention.
