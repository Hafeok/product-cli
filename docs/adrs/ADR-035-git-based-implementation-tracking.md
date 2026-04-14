---
id: ADR-035
title: Git-Based Implementation Tracking
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:b15d19c192b23c8cf6a5a1c36b26d5863aeefec203fcc137191473675bf33e56
---

**Status:** Proposed

**Context:** The current drift detection model (ADR-023) resolves source files for an ADR via two mechanisms: pattern-based discovery (searching `source-roots` for files containing ADR/feature IDs) and explicit `source-files` in ADR front-matter. Both are weak signals.

**Pattern-based discovery** is heuristic. It searches for files whose path or content mentions an ADR or feature ID. Files that implement a decision without naming it are invisible. Files that mention an ID in a comment are false positives.

**`source-files` in ADR front-matter** has three problems:

1. **Speculative.** You declare `source-files` when writing the ADR, before implementation. You're guessing which files will implement it.
2. **Static.** A file rename silently breaks the link. A refactor that moves logic to a new file isn't captured. The declared files and the actual implementation diverge.
3. **Incomplete.** An ADR that governs a subsystem might touch 15 files across 5 modules. Nobody maintains a list of 15 files in an ADR front-matter field.

The result is that drift detection operates on an unreliable input set. The LLM receives files that may or may not be the ones that actually implement the decision. This undermines the value proposition of the entire drift check.

The git history already knows exactly which files implemented a feature. The commit that completes a feature is the ground truth.

**Decision:** Implementation is tracked via git commits, not declared file lists. `product verify FT-XXX` automatically records the current HEAD commit SHA when a feature transitions to `complete`. Drift detection uses `git diff-tree` on these recorded commits to determine exactly which files were changed during implementation. ADR-level drift is derived transitively through linked features. The `source-files` field on ADRs is deprecated.

---

### Feature Front-Matter: `implementation-commits`

When `product verify` transitions a feature to `complete`, it records the implementation commits automatically:

```yaml
---
id: FT-001
status: complete
completed-at: 2026-04-11T09:14:22Z
implementation-commits:
  - sha: abc123def456
    message: "implement raft consensus layer"
    date: 2026-04-11
  - sha: def456abc789
    message: "fix leader election edge case"
    date: 2026-04-12
---
```

**`completed-at`**: ISO 8601 timestamp of the moment `product verify` set status to `complete`.

**`implementation-commits`**: Array of commit objects. Each has `sha` (full 40-char hash), `message` (first line of commit message), and `date` (ISO 8601 date).

**How commits are collected:** When `product verify` transitions a feature to `complete`, it collects commits using a two-pass strategy:

1. **Commit message scan.** `git log --all --grep="FT-XXX"` finds commits that reference the feature ID in their message. This catches disciplined workflows where developers tag commits with feature IDs.
2. **Diff since last status change.** If the feature was previously `in-progress`, Product records the timestamp of the last status transition (from the `last-run` field on linked TCs or from git log of the feature file itself). All commits between that timestamp and HEAD that touch files also touched by the message-scan commits are included.

If no commits reference the feature ID, Product falls back to recording HEAD as a single implementation commit. This is the minimum viable signal — it captures what the repo looked like when the feature was verified as complete.

These fields are optional with defaults (empty list, no timestamp). Adding them is not a breaking schema change per ADR-014.

---

### Drift Detection via Implementation Commits

When `product drift check FT-XXX` runs:

```bash
# Step 1: Get every file touched by the implementation commits
git diff-tree --no-commit-id -r --name-only abc123def456
git diff-tree --no-commit-id -r --name-only def456abc789
# → union of files = the implementation file set

# Step 2: Find changes to those files since implementation
git log --name-only abc123def456..HEAD -- <implementation files>
# → if any files changed, that's the drift signal

# Step 3: For each changed file, get the actual diff
git diff abc123def456..HEAD -- <changed file>
# → this is the precise input to the LLM
```

The LLM receives: the ADR context bundle (unchanged from ADR-023) plus the **exact diffs** of changes made to implementation files since the feature was completed. The drift question becomes tightly scoped: "Do any of these changes contradict the governing ADRs?"

This is qualitatively better grounding than the current model, which sends the LLM entire file contents that may or may not be relevant.

---

### ADR-Level Drift via Feature Commits

ADRs do not need their own commit hashes. Drift on an ADR is detected transitively:

```
ADR-002 governs FT-001, FT-002, FT-005

product drift check ADR-002
  → collect implementation-commits from FT-001, FT-002, FT-005
  → union of implementation files across all three features
  → find changes to those files since the earliest implementation commit
  → ask: "do these changes violate ADR-002's decision?"
```

This is accurate because the ADR's scope is exactly the files that implemented its linked features. No over-inclusion (files that happen to mention the ADR ID in a comment), no under-inclusion (files that implement the decision without naming it).

---

### CI Integration: `product drift affected-features`

The CI trigger changes from "ADRs changed in this PR" to "files changed in this PR that are part of a feature implementation":

```bash
# Get files changed in this PR
CHANGED=$(git diff --name-only origin/main...HEAD)

# Find which completed features have implementation commits touching those files
product drift affected-features --files $CHANGED
# → FT-001, FT-003

# Check drift for those features (and their governing ADRs)
product drift check FT-001 FT-003
```

`product drift affected-features --files <paths>` is a new subcommand. It loads all completed features, resolves their implementation file sets from `implementation-commits`, and returns features whose file sets intersect with the given paths.

`product drift check --changed` is updated to use this mechanism internally: it resolves changed files from the current branch vs. main, then checks affected features.

---

### Deprecation of `source-files`

The `source-files` field on ADR front-matter is deprecated. It is retained in the schema for backward compatibility — Product will not strip it on write (per ADR-014: unknown/deprecated fields are preserved). However:

- When `implementation-commits` are present on linked features, `source-files` is **ignored** by drift detection.
- When no linked features have `implementation-commits` (pre-migration or incomplete features), `source-files` is used as a fallback, preserving current behavior.
- `product drift scan <path>` (reverse lookup) continues to work — it now uses the implementation commit file sets as its index rather than `source-files` declarations.
- New ADR templates omit `source-files`. Existing ADRs are not migrated — the field is simply ignored when better data exists.

Pattern-based discovery (searching source-roots for ID mentions) is also demoted to a fallback. The priority order for drift file resolution becomes:

1. **Implementation commits** on linked features (if any feature is `complete` with commits)
2. **`source-files`** in ADR front-matter (legacy fallback)
3. **Pattern-based discovery** (last resort, for ADRs with no linked complete features)

---

### Rename Safety

Git tracks file renames natively. `git log --follow` handles renames correctly. When a file from an implementation commit is later renamed:

- `git diff-tree` on the original commit still returns the original path
- `git log <sha>..HEAD -- <original-path>` detects the rename as a change
- The rename diff is included in the LLM context, which can reason about whether the rename is benign or breaks an architectural constraint

This is a significant improvement over `source-files`, which silently points to a non-existent path after a rename.

---

### Multi-Feature Files

A single file touched by multiple features is handled naturally:

- File `src/graph.rs` is in implementation commits for FT-001 and FT-003
- A change to `src/graph.rs` triggers drift review for both features
- Both features' governing ADRs are checked

This is correct behavior — a change to a shared file may affect multiple features and their governing decisions.

---

### Edge Cases

**Feature completed before this ADR is implemented:** `implementation-commits` is empty. Drift detection falls back to `source-files` or pattern-based discovery. No regression.

**Feature with no commits referencing its ID:** `product verify` records HEAD as a single commit. This captures the repo state at completion time. The implementation file set is the full diff of that commit, which is broader than ideal but still a concrete anchor point.

**Amended feature (re-opened, modified, re-completed):** On the second `product verify` transition to `complete`, the implementation commits are **replaced**, not appended. The new completion represents the current implementation. Historical commits from the first completion are not preserved in front-matter (they remain in git history of the feature file itself).

**Squashed merges:** If the team uses squash merges, the implementation may collapse into a single commit. This is fine — `git diff-tree` on that single commit returns all files touched, which is the complete implementation set.

---

**Rationale:**
- **Zero maintenance.** The author doesn't declare anything about source files. `product verify` captures commits automatically at the moment of completion. The implementation file set is always accurate because it comes from git, not from a human's memory of what they changed.
- **Accurate scope.** The files in the implementation commits are exactly the files that were changed to implement the feature. No heuristic, no guesswork.
- **Better LLM grounding.** Instead of "does this codebase generally match this ADR" (current model: full file contents, uncertain relevance), the drift check becomes "here are the exact diffs of changes to implementation files since completion — do any contradict the governing ADRs?" This is a tightly scoped, answerable question.
- **Consistent with ADR-002.** Implementation commits are recorded in YAML front-matter, preserving front-matter as the sole source of truth for the graph. The data happens to be auto-populated rather than manually authored, but the storage model is unchanged.
- **Non-breaking per ADR-014.** `implementation-commits` and `completed-at` are optional fields with empty defaults. No schema version increment required.

**Rejected alternatives:**
- **Keep `source-files` as primary, add git as supplementary.** This preserves the maintenance burden. The author still has to declare files, and the git data is only used to augment. Rejected: the git data is strictly superior — it should be primary, not supplementary.
- **Record all commits since feature creation.** Too broad. A feature file may exist for weeks before implementation starts. Commits during that period are unrelated. The commit-message-scan + diff-since-status-change strategy bounds the set to relevant commits.
- **Store implementation file sets instead of commit SHAs.** File lists have the same rename problem as `source-files`. Storing commit SHAs and resolving file sets on demand via `git diff-tree` is rename-safe and always current.
- **Require developers to tag commits with feature IDs.** Too much process. The fallback to HEAD-at-completion ensures the system works even with zero commit discipline. Good commit messages improve the signal but are not required.
- **Per-ADR implementation commits.** ADRs don't have a clear "implemented" moment the way features do. Features have `product verify` as the transition point. ADR drift is derived transitively through features, which is both simpler and more accurate.

**Test coverage:**
- TC for `product verify` recording commits on completion transition
- TC for `product drift check FT-XXX` using implementation commits to resolve files
- TC for `product drift check ADR-XXX` deriving files transitively from linked features
- TC for `product drift affected-features --files` returning correct features
- TC for fallback to `source-files` when no implementation commits exist
- TC for `product drift check --changed` using the new mechanism in CI