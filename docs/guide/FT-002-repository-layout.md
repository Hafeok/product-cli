It seems write permission to that directory is being denied. Here's the complete documentation for FT-002 — Repository Layout. You can save it to `docs/guide/FT-002-repository-layout.md`:

---

## Overview

Repository Layout defines the on-disk structure that Product uses to discover and manage artifacts — features, ADRs, test criteria, the generated graph index, and the generated checklist. All paths and ID prefixes are configurable through `product.toml`, which sits at the repository root. Product discovers this file by walking up from the current working directory, so commands work from any subdirectory.

## Tutorial

### Setting up a repository from scratch

1. Create a `product.toml` at the root of your project:

   ```toml
   name = "my-project"
   schema-version = "1"

   [paths]
   features = "docs/features"
   adrs = "docs/adrs"
   tests = "docs/tests"
   graph = "docs/graph"
   checklist = "docs/checklist.md"

   [prefixes]
   feature = "FT"
   adr = "ADR"
   test = "TC"
   ```

2. Create the artifact directories:

   ```bash
   mkdir -p docs/features docs/adrs docs/tests docs/graph
   ```

3. Create your first feature file at `docs/features/FT-001-hello-world.md`:

   ```markdown
   ---
   id: FT-001
   title: Hello World
   phase: 1
   status: planned
   depends-on: []
   adrs: []
   tests: []
   ---

   Describe the feature here.
   ```

4. Verify the layout is valid:

   ```bash
   product graph check
   ```

   A clean run produces no errors, confirming Product can discover `product.toml`, read the configured directories, and parse the front-matter in every artifact file.

### Adding an ADR and linking it to a feature

1. Create a new ADR:

   ```bash
   product adr new "Use SQLite for storage"
   ```

   This creates `docs/adrs/ADR-001-use-sqlite-for-storage.md` with skeleton front-matter.

2. Link the ADR to your feature:

   ```bash
   product feature link FT-001 --adr ADR-001
   ```

3. Rebuild the graph index and verify:

   ```bash
   product graph rebuild
   product graph check
   ```

## How-to Guide

### Customize directory paths

Edit the `[paths]` section of `product.toml`. All paths are relative to the repository root (the directory containing `product.toml`).

1. Open `product.toml`.
2. Change the path values under `[paths]`:
   ```toml
   [paths]
   features = "specs/features"
   adrs = "specs/decisions"
   tests = "specs/criteria"
   graph = "specs/graph"
   checklist = "specs/checklist.md"
   ```
3. Move your existing artifact files to match the new paths.
4. Run `product graph check` to confirm everything resolves.

### Customize ID prefixes

Edit the `[prefixes]` section to change the identifiers used in filenames and front-matter:

1. Open `product.toml`.
2. Set your preferred prefixes:
   ```toml
   [prefixes]
   feature = "FEAT"
   adr = "DEC"
   test = "TEST"
   ```
3. Rename existing files and update their `id` fields to match the new prefixes.
4. Run `product graph check` to verify all cross-references resolve.

### Define project phases

Add a `[phases]` section to label each phase number with a human-readable name:

```toml
[phases]
1 = "Foundation"
2 = "Core Features"
3 = "Polish"
```

Phases are used by `product status --phase <N>` and `product context --phase <N>` to scope output.

### Migrate from monolithic documents

If you have an existing PRD or ADR document, use the migration commands to split them into individual artifact files that follow the repository layout:

1. Dry-run the migration to preview what will be created:
   ```bash
   product migrate from-prd docs/prd.md --validate
   ```
2. Execute the migration:
   ```bash
   product migrate from-prd docs/prd.md --execute
   ```
3. Repeat for ADRs:
   ```bash
   product migrate from-adrs docs/adrs.md --execute
   ```

### Regenerate derived files

The graph index (`docs/graph/index.ttl`) and checklist (`docs/checklist.md`) are generated — never hand-edit them.

- Rebuild the graph index:
  ```bash
  product graph rebuild
  ```
- Regenerate the checklist:
  ```bash
  product checklist generate
  ```

## Reference

### `product.toml` schema

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | string | *(required)* | Project name |
| `version` | string | `"0.1"` | Project version |
| `schema-version` | string | `"1"` | Config schema version — must not exceed the binary's supported version |
| `schema-version-warning` | bool | `true` | Warn when schema version is behind current |

#### `[paths]`

All paths are relative to the directory containing `product.toml`.

| Key | Default | Description |
|---|---|---|
| `features` | `docs/features` | Directory containing feature files (`FT-XXX-*.md`) |
| `adrs` | `docs/adrs` | Directory containing ADR files (`ADR-XXX-*.md`) |
| `tests` | `docs/tests` | Directory containing test criterion files (`TC-XXX-*.md`) |
| `graph` | `docs/graph` | Directory for the generated `index.ttl` |
| `checklist` | `docs/checklist.md` | Path to the generated checklist file |

#### `[prefixes]`

| Key | Default | Description |
|---|---|---|
| `feature` | `FT` | Prefix for feature IDs (e.g., `FT-001`) |
| `adr` | `ADR` | Prefix for ADR IDs (e.g., `ADR-001`) |
| `test` | `TC` | Prefix for test criterion IDs (e.g., `TC-001`) |

#### `[phases]`

A map of phase numbers (as strings) to human-readable names. Optional.

```toml
[phases]
1 = "Cluster Foundation"
2 = "Products and IAM"
```

### Default directory layout

```
<repo-root>/
  product.toml
  docs/
    features/
      FT-001-some-feature.md
      FT-002-another-feature.md
    adrs/
      ADR-001-some-decision.md
    tests/
      TC-001-some-test.md
    graph/
      index.ttl              ← generated, never hand-edited
    checklist.md             ← generated, never hand-edited
```

### File naming convention

Artifact files follow the pattern `{PREFIX}-{NNN}-{slug}.md`, where:

- `{PREFIX}` matches the configured prefix (`FT`, `ADR`, or `TC`)
- `{NNN}` is a zero-padded numeric ID (e.g., `001`, `042`)
- `{slug}` is a kebab-case title derived from the artifact's title

### Repository discovery

Product finds `product.toml` by walking up the directory tree from the current working directory. The directory containing `product.toml` is treated as the repository root. All configured paths are resolved relative to this root.

If no `product.toml` is found in any parent directory, Product exits with a configuration error.

### Schema version compatibility

The `schema-version` field in `product.toml` is checked on every invocation:

- If the declared version exceeds the binary's supported version, Product exits with an error.
- If the declared version is behind the current version, a warning is emitted (unless `schema-version-warning` is set to `false`).
- Run `product migrate schema` to upgrade the schema (use `--dry-run` to preview changes).

## Explanation

### Why front-matter instead of a separate graph file?

The knowledge graph is derived entirely from YAML front-matter in each artifact file. There is no persistent graph store — the graph is rebuilt on every invocation. This design, documented in ADR-002, eliminates synchronisation drift between documents and graph data. When you add a link from a feature to an ADR, you edit one field in one file. Git diffs are clean and reviewable.

The trade-off is that graph operations must scan and parse all artifact files on every run. For repositories with hundreds of artifacts, this remains fast because YAML front-matter parsing is a lightweight string operation.

### Why markdown?

ADR-004 mandates CommonMark markdown with YAML front-matter as the sole document format. Markdown renders natively on GitHub and GitLab, requires no build pipeline, and can be injected directly into LLM context windows without conversion. Front-matter stripping (removing the `---` block) is a trivial operation during context bundle assembly.

### Generated files should not be edited

Two artifacts in the layout are generated and managed by the CLI:

- **`index.ttl`** (in the graph directory) — a Turtle/RDF export of the knowledge graph, regenerated by `product graph rebuild`. Editing it by hand has no effect because it will be overwritten on the next rebuild.
- **`checklist.md`** — a feature completion checklist, regenerated by `product checklist generate` and automatically updated after `product verify`. Manual edits will be lost.

### Configuration is optional

A minimal `product.toml` requires only the `name` field. All paths, prefixes, and phases have sensible defaults. This means you can bootstrap a repository with a single line:

```toml
name = "my-project"
```

Product will look for artifacts in `docs/features`, `docs/adrs`, and `docs/tests` using the `FT`, `ADR`, and `TC` prefixes.
