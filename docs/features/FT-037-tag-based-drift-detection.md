---
id: FT-037
title: Tag-Based Drift Detection
phase: 1
status: planned
depends-on: []
adrs:
- ADR-036
- ADR-023
- ADR-021
- ADR-013
- ADR-009
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
domains: [observability, data-model]
domains-acknowledged: {}
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