---
id: TC-777
title: FT-065 exit criteria — product-cli is discoverable and installable from the MCP registry
type: exit-criteria
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Purpose

Define the user-observable acceptance gate for FT-065. The feature is
"complete" only when an end user — not the maintainer running CI — can
discover and install `product-cli` through the official MCP registry's
standard tooling.

## Exit criteria

Each of the following must hold simultaneously against a published
release tag:

1. **Registry lookup succeeds.** Hitting
   `https://registry.modelcontextprotocol.io/v0/servers/io.github.{owner}/product-cli`
   returns HTTP 200 with a JSON document whose
   `version_detail.version` matches the latest published release tag.
2. **Browse-from-client succeeds.** A user running an MCP client that
   supports registry browsing (Claude Code's `claude mcp` family, or
   equivalent) can search for "product-cli" or
   "io.github.{owner}/product-cli" and see the entry with its
   description, repository link, and current version.
3. **Install-from-client succeeds.** The same client's install command
   (e.g. `claude mcp install io.github.{owner}/product-cli`) downloads
   the binary asset linked by the manifest's `packages[0]` entry and
   produces a working `.mcp.json` configuration that spawns
   `product mcp` in the user's repo.
4. **First MCP call succeeds.** After install, calling any read-only
   MCP tool (`product_feature_list`, `product_graph_check`) against a
   repo containing a valid `.product/config.toml` returns a non-error
   response.
5. **Version parity holds.** The version returned by `product --version`
   inside the installed binary equals the
   `version_detail.version` field served by the registry, equals the
   `version` field in `product.toml` at the release tag, equals the git
   tag itself (minus any `v` prefix).
6. **Smoke-test TC (TC-776) passes** on the release-tagged commit
   under `cargo t`.

## Verification approach

This is an **exit-criteria TC**, not an integration test that runs on
every commit. Verification is performed manually (or via a release
post-flight script) once per release:

1. Cut and push the release tag.
2. Wait for the release workflow to publish to the registry.
3. From a clean machine (or container), install via the MCP client.
4. Walk criteria 1–6 above and record the outcomes in the release
   notes.

The TC has no `runner` field — it is exit-criteria validated by hand
or by release post-flight, and tracked by setting the TC status to
`passing` when the manual check completes for a given release.

## Out of scope

- **Continuous validation across all clients.** We validate against
  one canonical client per release (Claude Code is the chosen
  reference). Other clients are expected to work by spec compliance,
  not by our active testing.
- **Multi-version coexistence.** We do not validate that two installs
  of different `product-cli` versions can coexist on the same machine.
- **Upgrade paths.** Upgrade behaviour is the MCP client's
  responsibility.

## Linkage

This TC validates the user-visible outcome of FT-065. The companion
smoke-test TC-776 validates the static manifest at every commit; this
TC validates the end-to-end pipeline at every release.
