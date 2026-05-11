---
name: release
description: Cut a new release of the Product CLI by bumping the Cargo.toml version, committing, tagging `vX.Y.Z`, and pushing the tag to trigger the cargo-dist release workflow. Use when the user asks to "cut a release", "release version X", "ship 0.1.X", "tag a release", "publish a release", or "bump and tag for release".
---

# Release the Product CLI

The release pipeline (`.github/workflows/release.yml`) is driven by `cargo-dist`. It fires when a git tag matching `**[0-9]+.[0-9]+.[0-9]+*` is pushed and builds the GitHub Release artifacts from the tagged commit.

A release therefore needs **five things to all line up** at the version `X.Y.Z`:

1. `Cargo.toml` — `version = "X.Y.Z"` (what `cargo-dist` reads at plan time)
2. `.product/config.toml` — `version = "X.Y.Z"` (Product config; source of truth per FT-065 spec)
3. `server.json` — both top-level `"version"` and `packages[0].version` set to `"X.Y.Z"` (MCP registry manifest)
4. A commit containing all three bumps on `main`
5. A pushed tag `vX.Y.Z` pointing at that commit

If `Cargo.toml` doesn't match the tag, `cargo-dist` refuses to plan (failure that caused the 0.1.1 redo in commit `9c9f828`). If `.product/config.toml` and `server.json` diverge, **TC-776** (`tc_776_server_json_matches_product_toml_version_and_validates_against_pinned_schema`) fails in `cargo t` — but **nothing currently enforces parity with `Cargo.toml`**, so a mismatch there can ship and only be caught at the MCP-registry publish step (which is non-fatal per FT-065).

## When to use

- The user asks to release a specific version (e.g. "release 0.1.3")
- The user asks to "tag the release" after a version bump commit has already landed
- The user notices a release pipeline didn't run because the tag was forgotten (this skill's origin story)

## What to do

### 1. Determine the target version

Ask the user if not specified. Otherwise infer:

- **Patch bump (default)** — `0.1.2` → `0.1.3` for bugfixes / small additions
- **Minor bump** — `0.1.x` → `0.2.0` for new features / non-breaking surface changes
- **Major bump** — reserve for breaking changes; confirm with the user before doing this

Read the current version from `Cargo.toml` (line 3 in this repo):

```bash
grep -E '^version = ' Cargo.toml | head -1
```

### 2. Verify preconditions

Run all of these in parallel before touching anything:

```bash
git status                                 # working tree must be clean
git rev-parse --abbrev-ref HEAD            # must be on main (or confirm intent)
git fetch origin && git status -sb         # must be up to date with origin/main
git tag -l "v<TARGET>"                     # must NOT already exist
```

If any check fails, surface the problem to the user rather than papering over it. A dirty tree means there's uncommitted work that needs a decision; an existing tag means the release was already attempted and you need to know why before re-running.

### 3. Decide: bump-and-tag, or tag-only?

Inspect the most recent commit:

```bash
git log -1 --format='%h %s'
grep -E '^version = ' Cargo.toml
```

- **If the `Cargo.toml` version already matches the target** and there's a "Bump version" commit at HEAD that hasn't been tagged — skip to step 5 (tag-only). This is the case the user hit today.
- **Otherwise** — proceed to step 4 to create the bump commit.

### 4. Bump and commit

Three files need editing in lockstep:

1. **`Cargo.toml`** line 3:
   ```toml
   version = "X.Y.Z"
   ```
2. **`.product/config.toml`** — the top-level `version` field:
   ```toml
   version = "X.Y.Z"
   ```
3. **`server.json`** — both occurrences (top-level `"version"` and `packages[0].version`):
   ```json
   "version": "X.Y.Z"
   ```

`Cargo.lock` will update when you next run `cargo build`, but is not required for the tag to plan correctly.

Verify the three are in agreement and TC-776 passes before committing:

```bash
grep -E '^version = ' Cargo.toml .product/config.toml
grep -E '"version"' server.json
cargo test --test integration_tests tc_776   # must pass
```

Commit using the established message style (see `git log --oneline --grep="Bump version"`):

```bash
git add Cargo.toml .product/config.toml server.json
git commit -m "$(cat <<'EOF'
Bump version to X.Y.Z

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
git push origin main
```

The commit must be pushed **before** the tag — otherwise the tag would point at a commit GitHub Actions can't see, and the release run can't check it out.

### 5. Tag and push

```bash
git tag vX.Y.Z <commit-sha>     # omit the SHA to tag HEAD
git push origin vX.Y.Z
```

The tag must be `v`-prefixed to match the workflow trigger pattern and the existing tag convention (`v0.1.0`, `v0.1.1`, `v0.1.2`). The pipeline accepts other formats per the cargo-dist regex, but stay consistent with what's already there.

### 6. Verify the pipelines started

Two workflows run for a release:

```bash
gh run list --workflow=release.yml --limit 3       # cargo-dist build + GitHub Release
gh run list --workflow=publish-mcp.yml --limit 3   # MCP registry publish (fires after the Release is created)
```

The most recent `release.yml` run should show the tag you just pushed and a status of `queued` or `in_progress`. `publish-mcp.yml` is triggered by the `release.published` event, so it won't show until the `host` job in `release.yml` has called `gh release create`. Watch with:

```bash
gh run watch
```

If `release.yml` fails immediately with a "version mismatch" error in `dist plan`, the `Cargo.toml` version on the tagged commit does not match the tag. Fix by bumping `Cargo.toml`, committing, **deleting the tag** (`git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`), and re-tagging on the new commit.

If `publish-mcp.yml` fails at the "Verify server.json version matches the release tag" step, one of `Cargo.toml`, `server.json` top-level, or `server.json` packages[0] is out of step. The release itself has already shipped — fix the laggard and either rerun the workflow (`gh run rerun <id>`) or run `mcp-publisher publish` manually from a checkout of the tag.

## Gotchas

- **Cargo.toml version must equal the tag's version.** `cargo-dist` reads `Cargo.toml` from the tagged commit and refuses to plan if it doesn't match. This is the single most common failure mode — happened on 0.1.1 (commit `9c9f828`), which is the only reason we know.
- **Three-file lockstep.** `Cargo.toml`, `.product/config.toml`, and `server.json` (twice) must all carry the same version string. Forgetting `.product/config.toml` and `server.json` is what shipped a stale `0.1` to the MCP registry through 0.1.0 and 0.1.1 — they were inherited untouched from the original config and never bumped. TC-776 enforces parity between the latter two; nothing enforces parity with `Cargo.toml`, so check by eye before committing.
- **`git push` does not push tags.** You must push the tag separately with `git push origin vX.Y.Z` (or `git push --tags`, but explicit is better — avoids pushing stale local tags).
- **Don't tag before the commit is pushed.** GitHub Actions checks out the tagged ref from the remote; an unpushed commit isn't fetchable.
- **The `v0.1.0+<hash>` tags are nightlies, not releases.** Don't reuse that format for an intentional release — it'll match the cargo-dist regex but pollute the tag list. Plain `vX.Y.Z` only.
- **Tag deletion is safe only if the workflow hasn't published a GitHub Release yet.** Once the Release is created, deleting the tag won't delete the Release — you'd have to delete that separately via `gh release delete`.
- **No CHANGELOG.md to update.** This repo lets cargo-dist generate the release body from commit messages; there's no manual changelog step.
- **Registry namespace is case-sensitive.** `server.json` `name` must be `io.github.Hafeok/product-cli` with the GitHub username's canonical case (`Hafeok`, not `hafeok`). The MCP registry binds publish permission to the OIDC `sub` claim, which carries the actual case. A case mismatch returns HTTP 403 at the publish step — observed on the v0.1.3 attempt. TC-776 pins the expected name; do not lower-case it.

## Skill output

After the tag is pushed, report:

1. The version released (`vX.Y.Z`)
2. The commit SHA the tag points at
3. The URL or status of the running workflow (`gh run list --workflow=release.yml --limit 1`)
4. Anything unusual encountered (mismatched version, dirty tree, etc.) — don't hide problems in the success message
