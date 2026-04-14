## Overview

`product status` provides a dashboard view of project health — features grouped by phase, their completion status, and test coverage gaps. Combined with `product test` filters, it lets developers and CI pipelines quickly identify what needs attention: untested features, failing tests, and phase-level progress. The feature builds on the derived graph model (ADR-003) so every invocation reflects the current file state, and on the generated checklist convention (ADR-007) so status is always authoritative.

## Tutorial

### Checking overall project health

After installing Product and initializing a repository with `product init`, the first thing to do is see where the project stands:

```bash
product status
```

This prints a summary of all features grouped by phase, showing how many are planned, in-progress, or complete. You will see output similar to:

```
Phase 1: 5 features (3 complete, 1 in-progress, 1 planned)
Phase 2: 8 features (2 complete, 4 in-progress, 2 planned)
Phase 3: 3 features (0 complete, 0 in-progress, 3 planned)
```

### Drilling into a specific phase

To focus on a single phase and see per-feature detail including test coverage:

```bash
product status --phase 1
```

This shows each feature in phase 1 along with its status and how many test criteria are linked.

### Finding features that lack tests

Features with no linked test criteria are invisible to `product verify`. Find them early:

```bash
product status --untested
```

Or equivalently using the test subcommand:

```bash
product test untested
```

Both commands list features that have zero `validatedBy` edges in the knowledge graph.

### Finding features with failing tests

When tests break, you need to know which features are affected:

```bash
product status --failing
```

Or list the failing tests directly:

```bash
product test list --failing
```

### Verifying graph health before committing

Run the graph health check to catch broken links, orphaned artifacts, or missing test criteria:

```bash
product graph check
```

The exit code tells you the result without parsing output:

- **0** — clean graph, no issues
- **1** — errors (broken links, cycles, malformed front-matter)
- **2** — warnings only (orphans, untested features)

## How-to Guide

### Check phase progress for a milestone review

1. Run `product status --phase 2` to see all phase-2 features with their status.
2. Note any features still in `planned` or `in-progress` state.
3. Run `product status --untested` to check if any of those features lack test criteria.
4. Run `product status --failing` to check if any have failing tests.

### Identify untested features for a coverage sweep

1. Run `product status --untested` to get the list of features with no linked TCs.
2. For each feature, run `product context FT-XXX --depth 1` to understand what the feature covers.
3. Create TC files in `docs/tests/` with appropriate `validates` front-matter linking back to the feature.
4. Run `product verify FT-XXX` to execute the new tests and update status.

### Use status checks as a CI gate

1. Add `product graph check` as a pipeline step.
2. To fail only on hard errors (broken links, cycles) but allow warnings (orphans, untested features):
   ```bash
   product graph check || [ $? -eq 2 ]
   ```
3. To fail on both errors and warnings (strict mode), use the bare command — any non-zero exit fails the step:
   ```bash
   product graph check
   ```
4. For machine-readable output in CI, use JSON format:
   ```bash
   product graph check --format json
   ```

### Regenerate the checklist after status changes

1. Run `product verify FT-XXX` for each feature you have worked on — this updates front-matter status and regenerates `CHECKLIST.md`.
2. Alternatively, regenerate the checklist directly:
   ```bash
   product checklist generate
   ```
3. If modified files are uncommitted, Product warns you to prevent stale checklist state from being committed alongside unrelated changes.
4. Never hand-edit `CHECKLIST.md` — it is a generated artifact (ADR-007).

## Reference

### `product status`

Displays a summary of features grouped by phase, with status counts.

| Flag | Description |
|------|-------------|
| `--phase N` | Filter to a single phase; shows per-feature detail with test coverage |
| `--untested` | Show only features with no linked test criteria |
| `--failing` | Show only features with one or more failing tests |

**Examples:**

```bash
product status                   # full summary by phase
product status --phase 1         # phase-1 detail with test coverage
product status --untested        # features missing test criteria
product status --failing         # features with failing tests
```

### `product test`

Subcommands for filtering test criteria.

| Subcommand | Description |
|------------|-------------|
| `untested` | List features with no linked test criteria |
| `list --failing` | List test criteria currently in `failing` status |

**Examples:**

```bash
product test untested            # features with no linked tests
product test list --failing      # tests currently failing
```

### `product graph check`

Validates the knowledge graph for structural integrity.

| Exit Code | Meaning |
|-----------|---------|
| `0` | Clean graph — no issues |
| `1` | Errors — broken links, supersession cycles, malformed front-matter |
| `2` | Warnings only — orphaned artifacts, untested features |

| Flag | Description |
|------|-------------|
| `--format json` | Output results as JSON for machine consumption |

### `product checklist generate`

Regenerates `CHECKLIST.md` from current front-matter status. Warns if modified files are uncommitted.

### Feature status values

Status is stored in the `status` field of each feature's YAML front-matter:

| Value | Meaning |
|-------|---------|
| `planned` | Not yet started |
| `in-progress` | Implementation underway |
| `complete` | All linked test criteria passing |
| `abandoned` | Deliberately dropped |

### Git awareness

When `product checklist generate` runs, it checks for uncommitted modifications. If the working tree is dirty, a warning is emitted to stderr. This prevents committing a freshly regenerated checklist that does not reflect uncommitted code changes.

## Explanation

### Why status lives in front-matter, not the checklist

ADR-007 establishes that `CHECKLIST.md` is a generated view, never a source of truth. Feature status is owned by the `status` field in each artifact's YAML front-matter. This eliminates the divergence problem where a developer updates one place but forgets the other. The checklist can be regenerated at any time with `product checklist generate` without losing information.

### Why the graph is rebuilt on every invocation

ADR-003 mandates that the knowledge graph is derived from front-matter on every command run, never persisted. For repositories at Product's target scale (under 500 artifacts), parsing all files takes under 50ms. This means `product status` always reflects the actual file state — there is no cache to invalidate and no stale index to worry about. If you edit a feature file outside the CLI and then run `product status`, you see the current truth.

### Exit codes as a CI contract

ADR-009 defines a three-tier exit code scheme specifically designed for CI integration. The error/warning distinction (exit 1 vs. exit 2) lets pipeline authors express policy: strict pipelines fail on any issue, lenient pipelines tolerate coverage gaps but block on broken links. This convention follows established tools like `grep` and `clippy`, so engineers arrive with prior intuition about what the codes mean.

### Relationship to graph capabilities

The status and filter commands sit at the query layer above the knowledge graph defined in ADR-012. `--untested` is a graph query: features with zero `validatedBy` edges. `--failing` filters on test criterion status. `--phase N` groups by the `phase` field on feature nodes. These are all read-only projections of the derived graph — they never mutate state. Mutations happen through `product verify` (which updates test status) and direct front-matter edits.

### Git awareness as a safety net

The uncommitted-file warning during checklist regeneration is a lightweight guard against a specific failure mode: a developer runs `product checklist generate`, commits the checklist, but has not yet committed the code changes that caused the status update. The resulting commit contains a checklist that claims a feature is complete while the implementing code is still unstaged. The warning does not block the operation — it surfaces the risk and lets the developer decide.
