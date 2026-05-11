---
id: FT-065
title: Publish Product CLI to the Official MCP Registry
phase: 5
status: planned
depends-on:
- FT-021
- FT-035
- FT-061
adrs:
- ADR-020
- ADR-048
tests:
- TC-776
- TC-777
domains:
- api
- security
domains-acknowledged:
  data-model: No new artifact type, no front-matter schema change, no graph edges — `server.json` is an external packaging manifest governed by the upstream MCP registry's JSON Schema, not by Product's internal graph model. ADR-048's canonical-`.product/`-layout invariant is intentionally not violated because `server.json` is a public packaging manifest that the registry expects at the repo root (alongside `Cargo.toml`), not a piece of Product's own state.
  observability: No telemetry, metrics, or trace surface change. The release-workflow publish step emits a single CI log line on success or warning on failure; no OTel, no metrics endpoint, no logging-format change. ADR-048 (the empty-domain ADR flagged here) governs canonical repository layout and is non-applicable because `server.json` is an external packaging manifest the upstream registry requires at the repo root, not Product's own state.
  api: This feature is packaging and distribution work, not a change to the CLI / MCP API surface. The published artifact is the existing MCP server (FT-021 / ADR-020); no new tools, commands, errors, or schemas are introduced. The `api` domain is touched only insofar as the registry manifest declares the existing stdio launch invocation (`product mcp`).
  testing: This feature ships a single smoke-test TC validating that `server.json` parses, matches its declared schema URL, and tracks `product.toml`'s version. That TC follows the standard scenario-test pattern (ADR-018) and uses `cargo-test` as its runner per CLAUDE.md, but does not exercise property-based, session-based, or LLM-benchmark testing strategies — none of those modalities apply to a static manifest validation. No `testing` domain gap exists in substance.
  security: Security exposure is bounded to the publish-side authentication ritual (GitHub OIDC at release time) and the namespace-squatting guarantee that registry OIDC verification provides. No new authentication paths, no new token storage, no new trust boundaries inside the binary — the registry-installed server is byte-identical to the binary already built by the release workflow. Mitigations are documented in the Risk and Mitigation section of the feature body.
---

## Description

Publish the `product` MCP server to the official Model Context Protocol
registry at `registry.modelcontextprotocol.io` so that any MCP-capable
client (Claude Code, claude.ai, Cursor, Zed, custom agents) can discover
and install it through standard tooling instead of cloning the repo and
running `cargo install --path .`.

The MCP registry is the canonical, vendor-neutral catalog of MCP
servers (`https://github.com/modelcontextprotocol/registry`). Publishing
is a two-step ritual:

1. **Author a `server.json` manifest** at the repo root, following the
   registry's JSON Schema (`https://static.modelcontextprotocol.io/schemas/2025-09-29/server.schema.json`).
   The manifest declares the server's reverse-DNS name, version,
   description, repository, package distribution channel, runtime
   invocation hint, and any required environment variables or runtime
   arguments.
2. **Publish via the `mcp-publisher` CLI**, which authenticates the
   publisher (GitHub OIDC for `io.github.*` namespaces, DNS TXT for
   custom domains), validates the manifest against the live registry
   schema, and uploads the entry.

Because `product` is a Rust binary, the distribution channel is **GitHub
Releases** with prebuilt artifacts (already produced by the existing
release workflow) rather than npm or PyPI. The manifest points the
registry at the GitHub Release for a given version; clients pull the
binary directly from the release asset, no language runtime required.

This feature is **packaging / distribution work**, not a code change to
the MCP server itself. The on-disk MCP tool surface (FT-021, FT-046,
FT-059, FT-061, FT-062) is the artifact being published; we are simply
making it discoverable.

## Depends on

- **FT-021** — MCP Server. Owns the binary being published; the dual
  stdio + HTTP transport is the install target.
- **FT-061** — MCP Server and CLI Honor `.product/config.toml`
  Discovery. The registry-installed binary must respect the same
  discovery rules as a local `cargo install`; the published manifest
  documents `cwd` / discovery expectations so registry clients spawn
  the server in the right repo.
- **FT-035** — Repository Initialization. `product init` is the typical
  first command a user runs after installing from the registry, so the
  manifest's description and homepage point at the init flow.

## Governing ADRs

- **ADR-020 — MCP Server — Dual Transport (stdio and HTTP).** The
  registry-installed binary inherits the dual-transport invocation
  model; the manifest's `runtime_arguments` and `transport` fields
  declare the stdio launch shape (`product mcp`) that ADR-020 defines.
- **ADR-048 — Canonical Repository Layout — All Product State Under
  `.product/`.** `server.json` is placed at the repo root, **not** under
  `.product/`, in deliberate adherence to ADR-048's *Rule of external
  conventions* (Rule 2). The MCP registry mandates `server.json` at the
  repo root in the same way Claude Code mandates `.mcp.json` at the
  root and the multi-tool agent ecosystem mandates `AGENTS.md` at the
  root. Placing it inside `.product/` would break the convention
  scrapers rely on and force every registry interaction through a
  non-default `--server-json` flag for no benefit. ADR-048 explicitly
  carves out this case; FT-065 is the second concrete instance after
  `.mcp.json` itself.

## Scope of this feature

### In

1. **`server.json` manifest** committed at the repo root (per
   ADR-048 Rule 2), validated against the registry's published JSON
   Schema. Fields covered: `$schema`, `name`
   (`io.github.{owner}/product-cli` — `owner` determined at authoring
   time), `description`, `repository.url`, `version_detail.version`,
   one `packages` entry of type `oci` or `github-release` pointing at
   the release binary, and `runtime_arguments` enumerating the stdio
   launch shape (`product mcp`) plus optional HTTP flags (`--http`,
   `--port`, `--bind`, `--token`).
2. **Publisher identity.** GitHub OIDC is the chosen authentication
   path because the repo already lives on GitHub and the namespace
   `io.github.{owner}` is the matching reverse-DNS form. No DNS TXT
   record, no custom domain, no maintainer-list management beyond the
   GitHub repo's existing collaborator set.
3. **Release-time publish.** The existing release workflow (or a new
   `publish-mcp.yml` GitHub Actions workflow, decision deferred to
   design) runs `mcp-publisher publish server.json` after the binary
   artifacts are uploaded to the GitHub Release. Failure to publish
   does **not** fail the release — the registry is best-effort
   advertising, not a release blocker.
4. **Versioning policy.** The manifest's `version_detail.version`
   tracks the `version` field in `product.toml` (or
   `.product/config.toml` per ADR-048's discovery fallback) and the
   git tag exactly. Pre-1.0 we publish every release tag; once stable,
   only minor and patch releases publish.
5. **Documentation.** The README gains an "Install via MCP registry"
   section showing the canonical client-side commands
   (`claude mcp install io.github.{owner}/product-cli`, the JSON entry
   for a manual `.mcp.json`, and equivalents for other clients). The
   `docs/guide/` per-feature documentation generator picks this up via
   the standard Diátaxis flow.
6. **Smoke test.** A new TC validates that the committed `server.json`
   parses, matches the schema version pinned by `$schema`, declares the
   same version string as the resolved Product config, and lists at
   least one package entry. The TC runs as part of `cargo t` so a
   drift between the config and `server.json` is caught in CI.

### Out

- **Multiple distribution channels.** v1 ships one package entry
  (GitHub Release binary or OCI image, picked at design time). npm /
  PyPI wrappers are out of scope — Rust users use `cargo install`,
  registry users use the manifest path, no third channel.
- **Custom domain namespace.** We do not register `io.product.cli`
  or similar; the `io.github.*` reverse-DNS form derived from the
  GitHub owner is sufficient and avoids the DNS-TXT verification
  ritual.
- **Co-listing every MCP tool.** The registry entry advertises the
  server as a whole; the per-tool surface is discovered at runtime via
  the MCP `tools/list` request the client already issues. No per-tool
  registry metadata.
- **Mirroring to non-official registries.** We publish only to the
  canonical `registry.modelcontextprotocol.io`. Third-party catalogs
  that scrape the official registry are welcome to redistribute; we do
  not push to them.
- **Auto-update / version negotiation.** Registry clients pull the
  version the user installs; in-place upgrades are the client's
  responsibility. We do not implement any side-channel update check
  inside `product` itself.
- **Relocating `server.json` under `.product/`.** ADR-048 Rule 2
  governs — `server.json` lives at the repo root because the registry
  mandates it there.

## Functional Specification

### Inputs

- **`server.json`** at the repo root — the registry manifest, hand-
  maintained, version-tracked alongside the Product config.
- **Product config (`product.toml` or `.product/config.toml`)** — the
  existing `version` field is the source of truth for the manifest's
  `version_detail.version`. The smoke-test TC reads whichever path
  ADR-048's discovery fallback resolves first.
- **GitHub Release artifacts** — prebuilt `product` binaries (one per
  target triple) uploaded to the release before the publish step runs.
- **`MCP_PUBLISHER_TOKEN`** — GitHub OIDC token, minted by the Actions
  workflow at publish time. Never stored in the repo, never committed.
- **`mcp-publisher` CLI** — the official publisher tool from
  `github.com/modelcontextprotocol/registry`. Pinned to a known version
  in the workflow.

### Outputs

- **Registry entry** at `https://registry.modelcontextprotocol.io/v0/servers/io.github.{owner}/product-cli`
  — JSON document the registry serves on lookup. Clients consume this
  to render an install card.
- **README install section** — Markdown documenting the registry
  install path for each common client.
- **CI log line** — `Published io.github.{owner}/product-cli@vX.Y.Z to
  registry.modelcontextprotocol.io` on a successful publish; a warning
  (not a hard error) when publish fails so the release itself still
  ships.
- **Smoke-test output** — `tc_776_server_json_matches_product_toml_version_and_validates_against_pinned_schema`
  passes/fails as a normal cargo test.

### State

- **On disk.** One new file — `server.json` at the repo root (per
  ADR-048 Rule 2). No new directories. Nothing new under `.product/`.
- **In CI.** One new (or extended) workflow step that runs after the
  release artifacts upload step. No new persistent CI state beyond the
  GitHub OIDC token, which is ephemeral per workflow run.
- **At the registry.** One new entry under the `io.github.{owner}/`
  namespace, updated on every release tag.

### Behaviour

1. **At authoring time** — a maintainer edits `server.json` only to
   bump `version_detail.version` in lockstep with the Product config
   and the release tag, or to change the package entry. All other
   fields (name, description, repository, runtime_arguments) are
   stable across releases.
2. **At release time** — the release workflow:
   a. Builds the prebuilt binaries.
   b. Creates the GitHub Release and uploads the binaries.
   c. Runs the smoke-test TC against `server.json` (failing the
      workflow if it does not match the Product config).
   d. Mints a GitHub OIDC token.
   e. Invokes `mcp-publisher publish server.json --token $TOKEN`.
   f. Logs the published version on success or a warning on failure.
3. **At client install time** — a user runs (for example)
   `claude mcp install io.github.{owner}/product-cli`. Claude Code
   queries the registry, downloads the linked GitHub Release binary,
   places it on `$PATH`, and writes a `.mcp.json` entry pointing at the
   stdio invocation `product mcp` with the appropriate `cwd`. Per
   FT-061 the binary then discovers `.product/config.toml` (or the
   ADR-048 fallback chain) from the working-directory tree.
4. **At catalog browse time** — a user browsing the registry web UI
   sees the entry's description, version, repository link, and (when
   present) the icon / homepage from the manifest. The description must
   read coherently as a one-paragraph elevator pitch.

### Invariants

- **`server.json` version always equals the resolved Product config
  version.** Enforced by the smoke-test TC and by the CI publish step.
  The TC uses `ProductConfig::discover` (ADR-048 Rule 3) so it works
  identically on legacy `product.toml`-at-root layouts and modern
  `.product/config.toml` layouts.
- **`server.json` validates against the pinned `$schema` URL.**
  Enforced by the smoke-test TC (which parses the schema URL out of
  the manifest, fetches it offline from the registry-shipped copy in
  `tests/fixtures/`, and validates). No `mcp-publisher` invocation
  happens against an invalid manifest.
- **`name` is exactly `io.github.{owner}/product-cli`.** Enforced by
  the schema and re-asserted by the smoke-test TC so a typo (e.g.
  `product` vs `product-cli`) cannot ship.
- **`server.json` stays at the repo root.** Per ADR-048 Rule 2. The
  smoke-test TC asserts the path; moving it inside `.product/` would
  fail the test.
- **No secrets in `server.json`.** The manifest is committed to the
  repo and served publicly by the registry; no tokens, no internal
  URLs. Enforced by code review.
- **Publish failure is non-fatal.** The release itself ships even when
  the registry is unreachable. Asserted by the workflow's
  `continue-on-error: true` on the publish step.

### Error handling

- **Schema validation failure** → smoke-test TC fails locally and in
  CI before any release artifacts are built. The maintainer fixes the
  manifest and re-runs.
- **Version mismatch** (`server.json` vs Product config) → smoke-test
  TC fails with a diff between the two version strings. Failure mode:
  the maintainer forgot to bump one of them.
- **Authentication failure** at publish time → the publish step logs a
  warning and the workflow continues. The maintainer re-runs the
  publish manually with a fresh token, or files a follow-up issue.
- **Registry rejection** (duplicate version, namespace clash,
  rate-limit) → same as auth failure: warning, continue, manual
  follow-up.
- **No new `ProductError` codes.** This feature touches packaging and
  CI workflows, not the Rust error surface.

### Boundaries

- **In** — `server.json` authoring at the repo root, smoke-test TC,
  release workflow extension, README install section, documentation
  update.
- **Out** — any change to the MCP tool surface itself, any change to
  the CLI command set, any change to the Product config schema, any
  change to ADR-048's `.product/` layout decisions, any change to the
  GitHub Release workflow's binary-building stage. The binary
  artifacts are produced by existing automation; this feature only
  adds the post-release publish step.
- **Caller responsibilities** — the maintainer cutting a release is
  responsible for bumping `version` in the Product config; the
  smoke-test TC catches any forgotten `server.json` bump.

## Acceptance criteria

A maintainer cutting release `vX.Y.Z` can:

1. Bump `version = "X.Y.Z"` in the Product config and
   `version_detail.version` in `server.json`, run `cargo t`, and
   observe the smoke-test TC pass.
2. Tag the release and push; the release workflow builds the binaries,
   uploads them to the GitHub Release, runs the publish step, and logs
   `Published io.github.{owner}/product-cli@vX.Y.Z`.
3. Visit `https://registry.modelcontextprotocol.io/v0/servers/io.github.{owner}/product-cli`
   and observe the new version listed.
4. In Claude Code, run `claude mcp install io.github.{owner}/product-cli`,
   observe the binary download and the `.mcp.json` entry written, and
   then invoke any `product_*` MCP tool against a repo with a
   `.product/config.toml`.

In addition:

5. The README documents the registry install path with copy-pasteable
   commands for at least Claude Code and a generic stdio `.mcp.json`.
6. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
   and `cargo build` all pass.
7. `product graph check` exits clean after the feature, ADR, and TCs
   land.

## Out of scope

- npm / PyPI / Homebrew wrappers around the binary.
- Auto-update inside the `product` binary itself.
- Co-listing into third-party MCP catalogs.
- Registry namespace changes (custom domain, org rename).
- Multi-architecture matrix expansion beyond what the release workflow
  already builds.
- A "browse the registry from inside product" subcommand. Discovery
  happens in the user's MCP client, not in our CLI.
- Relocating `server.json` under `.product/` — ADR-048 Rule 2 governs.

## Implementation notes

- **`server.json` location.** Repo root, alongside `Cargo.toml`,
  `AGENTS.md`, `CLAUDE.md`, and `.mcp.json`. This is the company
  ADR-048 Rule 2 puts it in: files whose location is dictated by a
  consuming tool stay at the root. The registry's documented
  convention is the repo root; deeper locations require extra
  `--server-json` flags at publish time and break the convention
  registry scrapers rely on.
- **Schema pinning.** The `$schema` field embeds a dated URL
  (`/schemas/2025-09-29/server.schema.json` at the time of writing).
  Bumping the schema is a one-line manifest edit plus a refresh of
  the offline fixture used by the smoke-test TC.
- **Packages entry shape.** For a GitHub Release binary the entry is
  approximately:
  ```json
  "packages": [{
    "registry_type": "github-release",
    "identifier": "{owner}/product-cli",
    "version": "X.Y.Z",
    "transport": { "type": "stdio" },
    "runtime_arguments": [
      { "type": "positional", "value": "mcp" }
    ]
  }]
  ```
  The exact shape is fixed by the live schema; the smoke-test TC
  validates against it rather than us hand-asserting fields.
- **OIDC workflow snippet** lives in `.github/workflows/release.yml`
  (extension) or `publish-mcp.yml` (new file). Permission required:
  `id-token: write`. The token is minted with the audience the
  registry expects (`https://registry.modelcontextprotocol.io` per
  the publisher's documented audience).
- **Smoke-test TC** — `tests/integration.rs::tc_776_server_json_matches_product_toml_version_and_validates_against_pinned_schema`
  reads both files, parses YAML / JSON respectively (via
  `ProductConfig::discover` so the ADR-048 fallback chain is
  honoured), asserts the versions match, and that the manifest's
  `name` matches the expected string. The schema-fetch fixture lives
  in `tests/fixtures/server.schema.json` to keep the test offline.
  Bumping the schema is a fixture refresh.
- **No code changes in `src/`.** Confirmed by inspection — the MCP
  server is already production-ready (FT-021, FT-061); the work here
  is wholly in `server.json`, the release workflow, the README, and
  one TC.
- **Runner config.** The smoke-test TC gets `runner: cargo-test` and
  `runner-args: "tc_776_server_json_matches_product_toml_version_and_validates_against_pinned_schema"`
  at the same time the test is written, per CLAUDE.md.

## Risk and mitigation

- **Registry namespace squatting.** Mitigated by GitHub OIDC: only the
  repo owner can publish under `io.github.{owner}/product-cli`. No
  external actor can pre-empt the namespace.
- **Registry outage at release time.** Mitigated by the `continue-on-error`
  on the publish step; the GitHub Release ships regardless.
- **Schema drift.** Mitigated by pinning `$schema` to a dated URL and
  refreshing the offline fixture deliberately on each bump.
- **Stale version in registry.** Mitigated by the smoke-test TC
  enforcing version parity with the resolved Product config, plus the
  release workflow publishing on every tag.
- **Binary compatibility on user machines.** The release workflow
  already builds for the standard target triples; if a user's
  architecture is unsupported they fall back to `cargo install` from
  source. The README documents this fallback alongside the registry
  install path.
- **ADR-048 drift.** If a future ADR relocates `.mcp.json` /
  `AGENTS.md` / `CLAUDE.md` under `.product/`, `server.json` follows
  in lockstep — but until then it stays at the root with its
  external-convention cousins.
