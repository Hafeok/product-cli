---
id: FT-035
title: Repository Initialization
phase: 1
status: complete
depends-on: []
adrs:
- ADR-003
- ADR-008
- ADR-020
- ADR-025
- ADR-033
tests:
- TC-431
- TC-432
- TC-433
- TC-434
- TC-435
- TC-436
- TC-437
- TC-438
- TC-439
domains:
- api
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
---

## Description

`product init` bootstraps a new Product repository. It creates `product.toml` with the project configuration, scaffolds the directory structure (`docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/graph/`), and manages `.gitignore` entries for generated files.

The command operates in two modes:

- **Interactive (default):** Prompts for project name, concern domains (with a suggested common set), and MCP configuration. All fields have sensible defaults — the user can press Enter through every prompt.
- **Non-interactive (`--yes` / `-y`):** Accepts all defaults without prompting. Flags (`--name`, `--domain`, `--port`, `--write-tools`) override individual defaults.

### Safety

- Errors with exit code 1 if `product.toml` already exists (prevents accidental overwrite).
- `--force` flag overrides the existence check — replaces `product.toml` but does not delete existing artifact directories or their contents.

### CLI Surface

```
product init [OPTIONS]

Options:
  -y, --yes                Accept all defaults without prompting
      --force              Overwrite existing product.toml
      --name <NAME>        Project name (default: directory name)
      --domain <K=V>       Add a domain (repeatable)
      --port <PORT>        MCP HTTP port (default: 7777)
      --write-tools        Enable MCP write tools by default
      --path <DIR>         Target directory (default: cwd)
```

### Generated Files

**product.toml** — full config with all sections:
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

[phases]
1 = "Phase 1"

[domains]

[mcp]
write = false
port = 7777
```

**Directories:** `docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/graph/`

**.gitignore:** Appends `docs/graph/` entry (or creates the file if absent). Does not duplicate if entry already present.

---

## Functional Specification

### Inputs

- `product init [OPTIONS]` — CLI invocation from any directory.
- Flags: `-y` / `--yes` (non-interactive), `--force` (overwrite existing config), `--name NAME`, `--domain K=V` (repeatable), `--port PORT`, `--write-tools`, `--path DIR`.
- Interactive stdin prompts in default mode: project name, concern domains (multi-select from a suggested vocabulary), MCP write-tools flag, HTTP port.
- The current working directory name is used as the default project name when `--name` is not supplied.
- Presence or absence of an existing `product.toml` (or `.product/config.toml` for canonical layout) in the target directory.
- Presence or absence of an existing `.gitignore` in the target directory.

### Outputs

- **`product.toml`** (or `.product/config.toml` for canonical layout) — generated configuration file containing `name`, `schema-version`, `[paths]`, `[prefixes]`, `[phases]`, `[domains]`, and `[mcp]` sections.
- **Directories** — `docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/graph/` created via `create_dir_all` (safe if already present).
- **`.gitignore`** — entry `docs/graph/` appended if not already present; file created if absent.
- **Console output** — a list of created files and a "Run `product feature new`..." next-step hint.

### State

`product init` is a bootstrapping command; it has no pre-existing Product state to read (no `product.toml` is required). After successful execution, the target directory contains a valid configuration file that every other Product command can discover via `ProductConfig::discover()`. The command is not idempotent by default: re-running without `--force` on an already-initialised directory produces a hard error (see Error handling).

### Behaviour

1. **Interactive mode (default)** — presents prompts for project name, domain selection from a suggested vocabulary, MCP write-tools, and HTTP port. All prompts have defaults; the user can press Enter through every question to accept them. Custom domains can be added at the domain-selection step.
2. **Non-interactive mode (`--yes`)** — skips all prompts. Flags override individual defaults; unspecified flags use built-in defaults (`mcp.write = false`, `port = 7777`, empty `[domains]` section).
3. **Directory scaffolding** — all directories declared in `[paths]` are created with `create_dir_all`. This is safe to call even if directories already exist and does not touch existing artifact files.
4. **`.gitignore` management** — `docs/graph/` is appended if the file exists and does not already contain the entry; if `.gitignore` is absent, it is created. The entry is not duplicated on repeated runs with `--force`.
5. **`--force`** — overwrites the configuration file but does not delete existing artifact directories or their contents. All existing features, ADRs, and TCs are preserved.
6. **`--path DIR`** — targets a directory other than the current working directory. Useful for scripted multi-repo setups.

### Invariants

- If `product.toml` exists and `--force` is not passed, the command exits with code 1 and a diagnostic hint to use `--force`.
- The generated `product.toml` must be parseable by `ProductConfig::load()` — verified by TC-438 (property test).
- Directory creation (`create_dir_all`) never fails because directories already exist — it is always safe to re-create.
- `.gitignore` entries are never duplicated; idempotent detection checks for the entry before appending.
- Project name defaults to the directory name when `--name` is not supplied and the user accepts the default in interactive mode.

### Error handling

- **Existing config without `--force`** — exits with code 1; diagnostic message names the existing file and hints to use `--force` or edit directly.
- **Target directory does not exist** — `--path` to a non-existent directory exits with `ProductError::IoError` naming the missing path.
- **`create_dir_all` failure** — I/O errors during directory creation propagate as `ProductError::IoError`.
- **`.gitignore` write failure** — I/O errors are reported and the command exits with code 1.
- **Invalid `--domain` format** — `K=V` parsing failures produce `ProductError::ConfigError` naming the malformed argument.
- **Invalid `--port`** — clap validates the port argument as a `u16`; out-of-range values are rejected before `handle_init` is called.

### Boundaries

- `product init` creates configuration and scaffolding only. It does not create any artifact files (`FT-XXX`, `ADR-XXX`, `TC-XXX`).
- It does not install git hooks — that is `product install-hooks`. The init output message mentions `install-hooks` as a recommended next step.
- It does not run `product graph check` or any validation on existing artifacts in the target directory.
- Interactive domain selection presents a curated suggestion list; no network call or registry lookup is performed.

## Out of scope

- Template-based initialization with pre-populated features or ADRs — `product init` scaffolds an empty repository.
- Merging into an existing `product.toml` (e.g. adding missing sections) — use direct editing or `product migrate schema` for schema upgrades.
- Automatic `git init` — `product init` does not initialise a git repository; it assumes the target directory may already be a git repo.
- Multi-repository batch initialization — `product init` operates on one target directory per invocation.
- Removing or renaming directories — `product init --force` replaces only the configuration file.
