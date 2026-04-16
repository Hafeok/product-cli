---
id: ADR-036
title: Tag-Based Implementation Tracking
status: accepted
features:
- FT-037
supersedes:
- ADR-035
superseded-by:
- emk@contextand.com
domains:
- observability
- data-model
scope: domain
content-hash: sha256:2257e2c2ee0a93443099921f75ade147abf9ea6529985296c89723b71ca0d354
---

**Status:** Proposed — supersedes ADR-035

**Context:** ADR-035 proposed recording commit SHAs in feature front-matter (`implementation-commits`) to track which code implements a feature, replacing the weak `source-files` heuristic in ADR-023. While ADR-035 correctly identified the problem (speculative, static, incomplete file lists), its solution — commit SHAs in YAML — has three structural flaws:

1. **Multiple commits implement one feature.** Which do you record? All of them pollutes front-matter with implementation archaeology.
2. **One commit touches two features.** You duplicate the SHA across front-matter files.
3. **A rebase changes the SHA.** Your front-matter is now wrong. The data is ephemeral and maintenance-heavy.

A commit says "I changed these files for this reason." It belongs to the author's workflow. A commit might touch three features, fix a bug, and update a test. The commit message is about the change, not about which specification artifact is now satisfied.

The right unit is a **semantic milestone** — "this point in history is significant for this artifact." That is a git tag, not a git commit.

**Decision:** Implementation is tracked via annotated git tags in the `product/{artifact-id}/{event}` namespace. `product verify FT-XXX` creates an annotated tag when a feature transitions to `complete`. Drift detection uses `git log TAG..HEAD` on the implementation file set. The tag IS the record — no front-matter mutation required. `implementation-commits` (ADR-035) is not implemented. `source-files` (ADR-023) is deprecated. A new `product tags` command group provides lifecycle browsing.

---

### Tag Namespace

```
product/{artifact-id}/{event}
```

| Tag | Meaning |
|---|---|
| `product/FT-001/complete` | Feature completed — all TCs passing |
| `product/FT-001/complete-v2` | Re-completed after revision |
| `product/ADR-002/accepted` | ADR accepted (content hash anchor) |
| `product/ADR-002/superseded` | ADR superseded |
| `product/DEP-001/active` | Dependency confirmed active |

The namespace is unambiguous, browsable, and doesn't collide with release tags (`v1.0.0`, `release-2026-04`):

```bash
git tag | grep "^product/"        # all Product events
git tag | grep "^product/FT-"     # all feature completions
git tag | grep "^product/FT-001"  # full lifecycle of one feature
```

---

### Tag Creation by `product verify`

When `product verify FT-001` transitions a feature to `complete`:

```bash
git tag -a "product/FT-001/complete" \
  -m "FT-001 complete: 4/4 TCs passing (TC-001, TC-002, TC-003, TC-004)"
```

Annotated tag — has a message, an author, and a timestamp. The message records which TCs passed. This is richer than any YAML field.

If `product/FT-001/complete` already exists (feature re-opened, re-implemented, re-verified), the next version is created: `product/FT-001/complete-v2`, `complete-v3`, etc.

Output after tagging:

```
  TC-001 binary-compiles         PASS
  TC-002 raft-leader-election    PASS
  TC-003 raft-leader-failover    PASS
  TC-004 volume-allocation       PASS

  ✓ All TCs passing. Feature status: complete.
  ✓ Tagged: product/FT-001/complete
    Run `git push --tags` to share.
```

**Graceful degradation.** If the working directory is not a git repo, or `git` is not available, verify still works — it skips tag creation and prints a W-class warning. Tag creation is advisory, not gating.

---

### Push Discipline

Tags need to be pushed explicitly. `git push` doesn't push tags by default.

**Option A (v1, default):** Product reminds after tagging — `Run git push --tags to share.`

**Option B (opt-in via config):** `auto-push-tags = true` in `product.toml`. Product runs `git push origin <tag>` after creating it. Requires git remote to be configured.

Option A is correct for v1. It is safer and respects the user's git workflow. Option B introduces a network dependency into `product verify`.

---

### Drift Detection via Tags

When `product drift check FT-XXX` runs:

```bash
# Step 1: find the completion tag
TAG="product/FT-001/complete"

# Step 2: find files touched in commits up to that tag
FILES=$(git diff-tree --no-commit-id -r --name-only \
  $(git rev-list $TAG --max-count=20))

# Step 3: find changes to those files since the tag
CHANGES=$(git log --name-only $TAG..HEAD -- $FILES)

# Step 4: get the actual diff for LLM context
DIFF=$(git diff $TAG..HEAD -- $FILES)
```

Product passes DIFF to the LLM with the feature's context bundle. The LLM answers: "do any of these changes contradict the governing ADRs?" That's a tightly scoped, accurate question grounded in actual code changes.

**ADR-level drift** is derived transitively through linked features:

```
ADR-002 governs FT-001, FT-002, FT-005

product drift check ADR-002
  → find completion tags for FT-001, FT-002, FT-005
  → union of implementation files across all three features
  → find changes since the earliest completion tag
  → ask: "do these changes violate ADR-002's decision?"
```

**Fallback.** When no completion tag exists (pre-implementation, incomplete features, or repos that adopted Product after features were already complete), drift detection falls back to the existing source-files/pattern-based discovery from ADR-023. The priority order:

1. **Completion tag** on the feature (if exists)
2. **`source-files`** in ADR front-matter (legacy fallback)
3. **Pattern-based discovery** (last resort)

---

### What Disappears from Feature Front-Matter

```yaml
# ADR-035 proposed this — NOT implemented:
implementation-commits:
  - sha: abc123def
    message: "implement raft consensus layer"

# ADR-036 replaces it with — nothing. The tag is the record.
# completed-at is derived from the tag timestamp:
# git log -1 --format=%aI product/FT-001/complete
```

Feature front-matter gains no new fields. `completed-at` is derived from the tag object rather than written to YAML — one less thing that can go stale.

---

### `product tags` Commands

```bash
product tags list                     # all product/* tags
product tags list --feature FT-001    # lifecycle of one feature  
product tags list --type complete     # all completions
product tags show FT-001              # full tag detail with TC list
```

`product tags list` renders a table:

```
TAG                            ARTIFACT  EVENT     DATE
product/FT-001/complete        FT-001    complete  2026-04-11T09:14:22Z
product/FT-002/complete        FT-002    complete  2026-04-12T14:30:00Z
product/ADR-002/accepted       ADR-002   accepted  2026-04-10T08:00:00Z
```

`product tags show FT-001` renders full detail including the tag message (which TCs passed).

---

### `product drift check` Enhancements

```bash
product drift check FT-001           # tag-based drift for one feature
product drift check --all-complete   # all complete features with tags
product drift check ADR-001          # transitive through linked features
```

`product drift check FT-XXX` is a new primary path. The existing `product drift check ADR-XXX` continues to work but now derives files transitively through feature tags when available.

`product drift check --all-complete` iterates all features with completion tags and reports drift for each.

---

### Configuration

```toml
[tags]
auto-push-tags = false    # v1: never auto-push, print reminder
implementation-depth = 20  # commits to scan backward for implementation files
```

`implementation-depth` controls how many commits before the tag are scanned to build the implementation file set. Default 20 is sufficient for most features; large features may need more.

---

### Rename Safety

Git tracks renames natively. `git log --follow` handles renames. When a file from an implementation commit is later renamed:

- `git diff-tree` on the original commit returns the original path
- `git log <tag>..HEAD -- <original-path>` detects the rename as a change
- The rename diff is included in the LLM context

This is a significant improvement over `source-files`, which silently points to a non-existent path after a rename.

---

### Edge Cases

**Not a git repo:** Tag creation is skipped with a W-class warning. Drift check falls back to existing behavior. Product never requires git — it degrades gracefully.

**Feature completed before this feature is implemented:** No completion tag exists. Drift check falls back to source-files/pattern discovery. No regression from current behavior.

**Re-opened feature:** When a complete feature is re-opened (status changed back to in-progress), the existing tag is preserved. On re-completion, `product/FT-XXX/complete-v2` is created. The latest tag is always used for drift detection.

**Squashed merges:** The implementation collapses into one commit. `git diff-tree` on that commit returns all files touched — the complete implementation set.

---

**Rationale:**
- **Rebase-safe.** Annotated tags survive history rewrites. Commit SHAs don't.
- **Zero front-matter mutation.** The tag IS the record. `product verify` doesn't write implementation data to YAML. The filesystem stays clean. Consistent with the principle that front-matter is for graph relationships, not implementation archaeology.
- **Clean drift query.** `git log TAG..HEAD -- <files>` is a single, precise query. No YAML parsing, no stale file lists, no heuristic pattern matching.
- **Browsable lifecycle.** `git tag | grep "^product/FT-001"` shows the full lifecycle. This is richer and more standard than any YAML field.
- **Non-breaking.** No new front-matter fields. No schema version increment. Tag creation is opt-in (happens automatically in verify but is skipped gracefully outside git).

**Rejected alternatives:**
- **Commit SHAs in front-matter (ADR-035).** Rebases invalidate them. Multiple commits per feature pollute front-matter. One commit touching two features duplicates the hash. Superseded by this ADR.
- **Keep `source-files` as primary.** Speculative, static, incomplete. Tags are strictly superior when available. `source-files` is retained as fallback only.
- **Lightweight tags.** No message, no author, no timestamp. Annotated tags carry the TC list and provenance. Worth the extra `git tag -a`.
- **Auto-push by default.** Introduces network dependency into `product verify`. Print-reminder only is correct for v1.
- **Per-ADR implementation tags.** ADRs don't have a clear "implemented" moment. Features have `product verify`. ADR drift is derived transitively through features.
- **Store implementation file sets in YAML.** Same rename problem as `source-files`. Deriving file sets on-demand from `git diff-tree` is rename-safe and always current.