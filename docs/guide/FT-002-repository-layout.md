## Overview

Repository Layout defines the standard directory structure and file naming conventions that Product expects when scanning a repository for artifacts. Product derives its knowledge graph entirely from YAML front-matter in markdown files located in configured directories. Getting the layout right is a prerequisite for every other Product command — `graph check`, `context`, `verify`, and `checklist generate` all depend on finding artifacts where the configuration says they should be.

## Tutorial

### Setting up your first Product repository

This walkthrough creates a minimal repository with one feature, one ADR, and one test criterion.

1. Create the project root and initialize git:

   ```bash
   mkdir my-project && cd my-project
   git init
   ```

2. Create the configuration file:

   ```bash
   cat > product.toml << 'EOF'
   name = "my-project"
   prefix = "MP"

   [paths]
   features = "docs/features"
   adrs = "docs/adrs"
   tests = "docs/tests"
   EOF
   ```

3. Create the directory structure:

   ```bash
   mkdir -p docs/features docs/adrs docs/tests
   ```

4. Add a feature file at `docs/features/FT-001-initial-setup.md`:

   ```markdown
   ---
   id: FT-001
   title: Initial Setup
   status: draft
   phase: 1
   adrs:
     - ADR-001
   tests:
     - TC-001
   ---

   ## Description

   The initial setup feature covers bootstrapping the project.
   ```

5. Add an ADR file at `docs/adrs/ADR-001-use-rust.md`:

   ```markdown
   ---
   id: ADR-001
   title: Use Rust
   status: accepted
   features:
     - FT-001
   ---

   ## Context

   We need a compiled language for the CLI.

   ## Decision

   Use Rust.
   ```

6. Add a test criterion at `docs/tests/TC-001-binary-compiles.md`:

   ```markdown
   ---
   id: TC-001
   title: Binary compiles
   type: scenario
   status: unknown
   validates:
     features:
       - FT-001
   ---

   ## Description

   Assert the binary compiles without errors.
   ```

7. Validate the repository layout:

   ```bash
   product graph check
   ```

   If everything is wired correctly, the command exits with code 0 and reports no broken links. If an ID reference is wrong — say `FT-001` lists `ADR-999` which does not exist — the check reports the broken link and exits with code 1.

## How-to Guide

### Customize directory paths

1. Open `product.toml` in your repository root.
2. Edit the `[paths]` section to point to your preferred directories:

   ```toml
   [paths]
   features = "specs/features"
   adrs = "specs/decisions"
   tests = "specs/tests"
   ```

3. Move your existing artifact files into the new directories.
4. Run `product graph check` to confirm the graph still resolves correctly.

### Add a new feature to the repository

1. Determine the next feature ID. Product assigns IDs sequentially (FT-001, FT-002, ...).
2. Create a new file in the features directory following the naming convention `FT-XXX-short-name.md`.
3. Add YAML front-matter with at minimum `id`, `title`, `status`, and `phase`.
4. Reference related ADRs and test criteria by ID in the front-matter.
5. Run `product graph check` to verify all references resolve.

### Add a new ADR to the repository

1. Create a file in the ADRs directory: `docs/adrs/ADR-XXX-short-title.md`.
2. Include front-matter with `id`, `title`, `status`, and `features` (the features this decision applies to).
3. Use optional fields `supersedes` and `superseded-by` if this ADR replaces or is replaced by another.
4. Run `product graph check` to verify.

### Add a new test criterion

1. Create a file in the tests directory: `docs/tests/TC-XXX-short-title.md`.
2. Include front-matter with `id`, `title`, `type`, `status`, and `validates.features`.
3. If the test has an integration test, add `runner: cargo-test` and `runner-args` to the front-matter.
4. Run `product graph check` to verify.

### Verify the repository is well-formed

1. Run `product graph check` to detect broken links, missing required fields, and malformed front-matter.
2. Run `product gap check` to find specification gaps (features without tests, ADRs without features).
3. Run `product drift check` to detect spec-vs-code drift.

## Reference

### Default directory structure

```
project-root/
  product.toml              # Repository configuration
  docs/
    features/               # Feature specifications
      FT-001-name.md
      FT-002-name.md
    adrs/                   # Architectural Decision Records
      ADR-001-name.md
      ADR-002-name.md
    tests/                  # Test criteria
      TC-001-name.md
      TC-002-name.md
    graph/
      index.ttl             # Generated — never hand-edit
  CHECKLIST.md              # Generated — never hand-edit
```

### File naming convention

| Artifact type | Pattern | Example |
|---|---|---|
| Feature | `FT-XXX-short-name.md` | `FT-002-repository-layout.md` |
| ADR | `ADR-XXX-short-name.md` | `ADR-002-yaml-front-matter.md` |
| Test criterion | `TC-XXX-short-name.md` | `TC-005-frontmatter-parse-feature.md` |

All files are CommonMark markdown with YAML front-matter.

### `product.toml` path configuration

```toml
[paths]
features = "docs/features"    # Directory scanned for FT-XXX files
adrs = "docs/adrs"            # Directory scanned for ADR-XXX files
tests = "docs/tests"          # Directory scanned for TC-XXX files
```

Subdirectory names and file prefixes are configurable. Paths are relative to the repository root.

### Generated files

| File | Generator | Notes |
|---|---|---|
| `CHECKLIST.md` | `product checklist generate` or `product verify` | Auto-regenerated; tracks feature completion status |
| `docs/graph/index.ttl` | `product rdf export` | RDF/Turtle export of the knowledge graph |

These files must never be hand-edited. They are overwritten on regeneration.

### Required front-matter fields by artifact type

| Artifact | Required fields |
|---|---|
| Feature | `id`, `title`, `status`, `phase` |
| ADR | `id`, `title`, `status` |
| Test criterion | `id`, `title`, `type`, `status`, `validates` |

### ID format

- Features: `FT-XXX` (zero-padded three-digit number)
- ADRs: `ADR-XXX`
- Test criteria: `TC-XXX`

IDs are assigned sequentially. Product detects ID conflicts and can auto-increment or fill gaps.

## Explanation

### Why YAML front-matter instead of a separate graph file?

The knowledge graph is derived entirely from YAML front-matter declared in each artifact file (ADR-002). This means every file is self-describing — open any feature, ADR, or test criterion and you immediately see its identity and all its relationships. There is no separate graph file that can fall out of sync.

The alternative — maintaining a `links.toml` or similar index alongside the documents — creates a synchronisation problem. In practice, contributors update the document and forget the graph file. By embedding the graph edges in front-matter, the graph cannot drift from the documents because it is always recomputed from them.

### Why markdown specifically?

All artifact files are CommonMark markdown (ADR-004). This choice is driven by three requirements:

- **Renderability**: Markdown renders natively on GitHub and GitLab with no documentation pipeline.
- **LLM context injection**: Markdown is the native input format for LLM context windows. The context bundle assembly step strips front-matter (a trivial string operation) and passes the body directly. No format conversion is needed.
- **Authoring ergonomics**: Markdown is widely understood, and LLM-assisted editors have first-class support for it.

AsciiDoc, TOML, and Org-mode were considered and rejected for lacking one or more of these properties.

### No persistent graph store

The graph is rebuilt from front-matter on every CLI invocation. This is a deliberate design choice (ADR-002) that trades a small amount of startup time for the guarantee that the graph always reflects the current state of the files. There is no stale cache to invalidate and no migration to run when the schema changes.

### Relationship between layout and the rest of the system

Every Product command depends on the repository layout:

- `product graph check` scans the configured directories and validates all front-matter references.
- `product context FT-XXX` assembles a bundle by following links from the feature through the graph.
- `product verify FT-XXX` locates test criteria via front-matter `validates` edges.
- `product checklist generate` enumerates all features found in the features directory.

If a file is placed outside the configured paths, Product does not see it. If front-matter references a non-existent ID, `graph check` reports the broken link.
