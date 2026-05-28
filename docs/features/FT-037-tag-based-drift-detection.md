---
id: FT-037
title: Tag-Based Drift Detection
phase: 1
status: complete
depends-on: []
adrs:
- ADR-009
- ADR-013
- ADR-021
- ADR-023
- ADR-036
tests:
- TC-448
- TC-449
- TC-450
- TC-451
- TC-452
- TC-453
- TC-454
- TC-455
- TC-456
- TC-457
- TC-458
- TC-459
- TC-460
domains:
- data-model
- observability
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
---

## Description

Replace the heuristic-based drift detection model (source-files in ADR front-matter, pattern-based file discovery) with git tag-based implementation tracking. When `product verify` transitions a feature to `complete`, it creates an annotated git tag in the `product/{artifact-id}/{event}` namespace. Drift detection uses `git log TAG..HEAD` to find precise changes to implementation files since completion.

This feature implements the mechanism described in ADR-036 (superseding ADR-035).

### New Module: `src/tags.rs`

Git tag operations for the `product/` namespace:

- `is_git_repo(root)` — check if working directory is a git repo
- `create_tag(root, artifact_id, event, message)` — create annotated tag
- `tag_exists(root, tag_name)` — check existence
- `next_event_version(root, artifact_id, base_event)` — find next version (complete → complete-v2 → complete-v3)
- `find_completion_tag(root, feature_id)` — find latest completion tag for a feature
- `list_tags(root, filter)` — list all product/* tags with metadata
- `show_tag(root, tag_name)` — detailed tag info including message
- `tag_timestamp(root, tag_name)` — derive completed-at from tag
- `implementation_files(root, tag_name, depth)` — files touched in commits near the tag
- `check_drift_since_tag(root, tag_name, depth)` — full drift query (files, changes, diff)

### Verify Changes

Modify `src/implement/verify.rs`:

- When `update_feature_and_checklist` transitions a feature to `complete`, call `tags::create_tag` to create `product/FT-XXX/complete`
- Tag message includes TC count and TC IDs: `"FT-001 complete: 4/4 TCs passing (TC-001, TC-002, TC-003, TC-004)"`
- If tag already exists, use `next_event_version` to create `complete-v2`, etc.
- Print `✓ Tagged: product/FT-XXX/complete` and `Run git push --tags to share.`
- If not a git repo, print `warning[W018]: not a git repository — skipping tag creation` and continue

### Drift Detection Enhancements

Modify `src/drift/check.rs` and `src/commands/drift.rs`:

- Add `check_feature(feature_id, graph, root, baseline, config)` — tag-based drift for a feature
- Add `--all-complete` flag to drift check — iterate all features with completion tags
- Existing `product drift check ADR-XXX` gains tag-based file resolution (transitive through linked features)
- Fallback chain: completion tag → source-files → pattern discovery

### New Command Group: `product tags`

Add `src/commands/tags.rs`:

- `product tags list` — all product/* tags, table format
- `product tags list --feature FT-001` — lifecycle of one feature
- `product tags list --type complete` — filter by event type
- `product tags show FT-001` — full detail with tag message
- JSON output support via `--format json`

### Configuration

Add `[tags]` section to `ProductConfig`:

```toml
[tags]
auto-push-tags = false
implementation-depth = 20
```

### Error Model

- `W018`: not a git repository — tag creation skipped (verify still succeeds)
- `W019`: no completion tag for feature — falling back to pattern-based drift

### What Does NOT Change

- Feature front-matter schema — no new fields. Tags are external.
- `source-files` on ADR front-matter — retained for backward compatibility, used as fallback
- `product drift scan <path>` — continues to work (reverse lookup)
- `product drift suppress/unsuppress` — unchanged
- `drift.json` baseline — unchanged

---

## Functional Specification

### Inputs

- `product verify FT-XXX` — when a feature transitions to `complete`, tag creation is triggered automatically as a side-effect.
- `product tags list [--feature FT-XXX] [--type complete] [--format json]` — filters for browsing the `product/` tag namespace.
- `product tags show FT-XXX` — a feature ID for detailed tag inspection.
- `product drift check FT-XXX [--all-complete]` — feature ID or `--all-complete` for batch drift analysis.
- `product drift check ADR-XXX` — an ADR ID; drift is derived transitively through features linked to that ADR.
- `[tags]` section in `product.toml` — `auto-push-tags` (bool, default false) and `implementation-depth` (int, default 20).
- The git working directory — all tag operations call `git` as a subprocess via `src/tags.rs`.

### Outputs

- **Tag creation** — when `product verify FT-XXX` completes a feature, an annotated git tag `product/FT-XXX/complete` is created. Tag message: `"FT-001 complete: 4/4 TCs passing (TC-001, TC-002, TC-003, TC-004)"`. Subsequent re-completions create `complete-v2`, `complete-v3`, etc.
- **Console notice** — `✓ Tagged: product/FT-XXX/complete` and `Run git push --tags to share.`
- **`product tags list`** — table with columns TAG, ARTIFACT, EVENT, DATE. `--format json` emits a JSON array.
- **`product tags show FT-XXX`** — full tag detail including tag message (TC list) and timestamp.
- **`product drift check FT-XXX`** — drift report for files changed since the completion tag. Falls back to source-files or pattern-based discovery when no tag exists.
- **W018 warning** — printed to stderr when the working directory is not a git repo; verify still succeeds.
- **W019 warning** — printed when no completion tag is found for a feature; drift falls back to pattern-based discovery.

### State

Implementation milestones are recorded entirely as annotated git tags in the `product/{artifact-id}/{event}` namespace. No front-matter fields are added to feature files. `completed-at` is derived on demand from the tag object timestamp via `git log -1 --format=%aI`. The `[tags]` configuration section in `product.toml` persists the `auto-push-tags` and `implementation-depth` settings across invocations.

### Behaviour

1. **Tag creation in verify** — `src/tags.rs::create_tag` is called with `(root, feature_id, event, message)`. If the base tag already exists, `next_event_version` finds the next available `complete-vN` name.
2. **Graceful degradation** — `is_git_repo(root)` is checked before any tag operation. If not in a git repo, or if `git` is not available, tag creation is skipped with W018 and verify continues normally.
3. **Tag namespace** — format is `product/{artifact-id}/{event}`. Tag names are unambiguous and do not collide with release tags (`v1.0.0`, `release-*`).
4. **Drift detection** — `check_drift_since_tag` uses `git diff-tree` to find files touched in commits near the completion tag (up to `implementation-depth` commits), then runs `git log TAG..HEAD -- <files>` to find subsequent changes and `git diff TAG..HEAD -- <files>` to produce a diff for LLM context.
5. **Fallback chain** — drift check priority: (1) completion tag if present → (2) `source-files` in ADR front-matter → (3) pattern-based discovery. Fallback is per-feature.
6. **ADR-level drift** — `product drift check ADR-XXX` unions the implementation file sets of all features linked to that ADR, then computes changes since the earliest completion tag among them.
7. **`product tags` commands** — `list`, `list --feature`, `list --type`, `show` are read-only commands implemented in `src/commands/tags.rs` using `src/tags.rs` functions. All filter and format logic is in the command adapter.
8. **Auto-push** — when `auto-push-tags = true` in `[tags]`, verify runs `git push origin <tag>` after tag creation. Default is false (print-reminder only).

### Invariants

- Tags are created in the `product/{artifact-id}/{event}` namespace exclusively — never in other namespaces.
- A completion tag is annotated (not lightweight) — it always carries a message, an author, and a timestamp.
- Re-verification of a feature that is already complete creates a new version tag (`complete-v2`) rather than overwriting the existing tag.
- Feature front-matter schema is unchanged — no new fields are written by tag creation.
- Tag creation never blocks verify from completing — W018 and W019 are warnings, not errors.

### Error handling

- **W018** — not a git repository; tag creation skipped. Verify continues and succeeds. Warning is printed to stderr.
- **W019** — no completion tag found for feature; drift falls back to source-files or pattern discovery. Warning identifies the feature and states the fallback in use.
- **`git tag` subprocess failure** — `create_tag` returns `ProductError::IoError` with the stderr from git. Verify treats this as a non-fatal warning (the tag operation is advisory).
- **`product drift check FT-XXX` with no history** — if neither a tag nor source-files exist, drift check emits "no baseline available" and exits 0 (no drift detected, not an error).

### Boundaries

- Feature front-matter is not modified by tag creation. The tag IS the record of completion; no `completed-at` or `implementation-commits` field is written to YAML.
- `source-files` on ADR front-matter is retained for backward compatibility as a fallback; it is not deprecated from the schema.
- `product drift scan <path>` (reverse lookup) and `product drift suppress/unsuppress` are unchanged.
- `drift.json` baseline is unchanged.
- Tags in the `product/` namespace are owned by the Product CLI. Manually created tags in this namespace may interfere with version numbering (e.g. `complete-v2` being unexpected).

## Out of scope

- Recording git tags for artifacts other than features reaching `complete` (ADR acceptance tags, DEP activation tags) — the namespace supports these but they are not created automatically by this feature.
- Automatic `git push --tags` by default — opt-in via `auto-push-tags = true` in config.
- Renaming or deleting existing `product/` tags — no destructive tag operations are exposed.
- Drift analysis for features with no linked completion tag and no source-files declared — pattern-based discovery (the last fallback) continues to handle this case.
- Integration with remote git hosting APIs (GitHub, GitLab) — tag push uses standard `git` subprocess only.
