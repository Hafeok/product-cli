---
id: ADR-033
title: Repository Initialization — `product init` Command
status: accepted
features:
- FT-035
supersedes: []
superseded-by: []
domains: []
scope: feature-specific
content-hash: sha256:77ec5fd0946eb51999d1d43c7dcdcdf6d118aaee06a9e1d4dac6d1e05634744e
---

**Context:** Every `product` command except `init` requires a `product.toml` discovered by walking up the directory tree (`ProductConfig::discover()`). A new user who installs the CLI and runs `product feature list` gets "No product.toml found in current directory or any parent" — a dead end with no guidance. There is no bootstrapping path: the user must hand-write a TOML file with the correct schema, keys, and directory structure before any other command works.

Other CLI tools solve this with an `init` command: `cargo init`, `npm init`, `git init`. The pattern is well-understood — create the minimal configuration and scaffolding so that every subsequent command has a valid environment. Product needs the same.

The design must handle several tensions:
1. **Interactive vs. scriptable** — humans want prompts; CI/scripts want flags and defaults.
2. **Minimal vs. complete** — a bare toml works but leaves users to discover `[domains]`, `[mcp]`, `[phases]` on their own; a full toml is noisy but self-documenting.
3. **Idempotency** — running `init` in an already-initialized repo must not silently destroy configuration.
4. **Git integration** — the generated `docs/graph/` directory should be gitignored by default (it contains generated TTL files, ADR-008).

**Decision:** Add a top-level `product init` command that creates `product.toml`, the directory skeleton, and a `.gitignore` entry. The command operates in two modes:

### 1. Interactive mode (default)

When run without `--yes`, the command prompts the user through a series of questions:

```
$ product init
Project name [my-project]: picloud
Schema version [1]:
Feature prefix [FT]:
ADR prefix [ADR]:
Test prefix [TC]:

Common concern domains (select with space, enter to confirm):
  [x] security        — Authentication, authorisation, secrets, trust boundaries
  [x] error-handling  — Error model, diagnostics, exit codes, recovery
  [ ] storage         — Persistence, durability, backup
  [ ] networking      — DNS, mTLS, service discovery, port allocation
  [x] api             — CLI surface, MCP tools, event schema
  [ ] observability   — Metrics, tracing, logging, telemetry
  [ ] data-model      — RDF, SPARQL, ontology, event sourcing
  > Add custom domain? (name=description, or enter to skip): iam=Identity, OIDC, tokens, RBAC

MCP server:
  Enable write tools by default? [y/N]: y
  HTTP port [7777]:

Created:
  product.toml
  docs/features/
  docs/adrs/
  docs/tests/
  docs/graph/
  .gitignore (appended: docs/graph/)

Run `product feature new "My First Feature"` to get started.
```

The project name defaults to the current directory name. All other fields have sensible defaults that match the existing `ProductConfig::default()` values. The user can accept all defaults by pressing enter through every prompt.

### 2. Non-interactive mode (`--yes` / `-y`)

Accepts all defaults without prompting. Suitable for CI, scripts, and quick starts:

```
$ product init --yes
$ product init --yes --name picloud --domain security="Auth, secrets"
```

Flags override defaults: `--name`, `--domain` (repeatable), `--port`, `--write-tools`. Any field not specified via flags uses the default.

### 3. Idempotency and `--force`

- If `product.toml` already exists: **hard error** with exit code 1 and a diagnostic message:
  ```
  error: product.toml already exists
    --> ./product.toml
    = hint: use `product init --force` to overwrite, or edit the file directly
  ```
- `--force` overwrites `product.toml` but does **not** delete existing artifact directories or their contents. It only replaces the configuration file.

### 4. Directory scaffolding

`init` creates every directory declared in `[paths]`:
- `docs/features/`
- `docs/adrs/`
- `docs/tests/`
- `docs/graph/`

Directories are created with `create_dir_all` — safe if they already exist. This matches the existing behavior in `feature new` which calls `create_dir_all` before writing.

### 5. Git integration

`init` manages `.gitignore` entries for generated files:
- If `.gitignore` does not exist, create it with the entry.
- If `.gitignore` exists but does not contain the entry, append it.
- If `.gitignore` already contains the entry, do nothing.

Entries added:
```
# Product CLI — generated files
docs/graph/
```

### 6. Generated `product.toml` structure

The full generated file (interactive mode with all options filled):

```toml
name = "picloud"
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
security = "Authentication, authorisation, secrets, trust boundaries"
error-handling = "Error model, diagnostics, exit codes, recovery"
api = "CLI surface, MCP tools, event schema"
iam = "Identity, OIDC, tokens, RBAC"

[mcp]
write = true
port = 7777
```

In `--yes` mode with no flags, the `[domains]` section is empty (`[domains]` header present, no entries) and `[mcp]` uses defaults (`write = false`, `port = 7777`).

### 7. CLI surface

```
product init [OPTIONS]

Options:
  -y, --yes                Accept all defaults without prompting
      --force              Overwrite existing product.toml
      --name <NAME>        Project name (default: directory name)
      --domain <K=V>       Add a domain (repeatable): --domain security="Auth, secrets"
      --port <PORT>        MCP HTTP port (default: 7777)
      --write-tools        Enable MCP write tools by default
      --path <DIR>         Target directory (default: current directory)
```

**Rationale:**
- **Interactive-by-default with `--yes` escape** is the standard pattern (npm, poetry, cargo). It serves both newcomers (guided setup) and power users (quick scaffold) without bifurcating documentation.
- **Hard error on existing `product.toml`** prevents accidental data loss. The `--force` flag makes overwrite intentional. This is the `git init` pattern — git also warns when re-initializing.
- **Creating directories eagerly** means `product graph check` works immediately after init with zero artifacts (empty graph, no errors). Without the directories, the parser would fail trying to read a non-existent `docs/features/`.
- **`.gitignore` management** prevents generated TTL files from being committed. The `docs/graph/` directory is regenerated from front-matter on every invocation (ADR-003) — committing it creates spurious diffs.
- **Including `[mcp]` by default** reflects the primary use case: Product is used as an MCP server for LLM agents. Omitting it forces users to discover the section exists and hand-add it. Including it with safe defaults (`write = false`) is zero-risk and self-documenting.
- **Domain suggestions in interactive mode, blank in `--yes`** balances discoverability against opinion. Interactive mode teaches the concept of domains (ADR-025) at the moment it matters. `--yes` mode doesn't impose a vocabulary — the user adds domains when they need them.
- **`--path` flag** enables `product init --path ./new-repo` for creating repos in a non-cwd directory, useful for scripted multi-repo setups.

**Rejected alternatives:**
- **`product init --template <name>`** — predefined templates (rust-cli, web-service, etc.) with curated domain sets. Rejected because it front-loads complexity before we have evidence of which templates users need. The interactive domain picker achieves 80% of the value. Templates can be added later as a non-breaking extension.
- **Merge mode for existing repos** — `product init` on an existing repo would add missing fields to the toml. Rejected because merging TOML while preserving comments and formatting is fragile. The `product migrate schema` command already handles schema upgrades. For adding new sections, direct editing is clearer than magic merging.
- **Automatic `product install-hooks`** — running hook installation during init. Rejected because git hooks are a side-effect with security implications (pre-commit hooks run arbitrary code). The user should opt in explicitly. The init output message should mention `product install-hooks` as a next step.
- **YAML config format** — using `product.yaml` instead of `product.toml`. Rejected for consistency with the existing codebase and the Rust ecosystem convention (Cargo.toml, rustfmt.toml, clippy.toml).

**Test coverage:**
- TC-431: init creates product.toml and full directory skeleton in an empty directory
- TC-432: interactive mode prompts for name and domains (stdin simulation)
- TC-433: `--yes` flag produces valid config without prompts
- TC-434: init errors with exit code 1 when product.toml exists
- TC-435: `--force` overwrites existing product.toml without error
- TC-436: init appends to existing .gitignore without duplicating entries
- TC-437: init creates .gitignore when none exists
- TC-438: property test — any combination of flags produces a toml parseable by `ProductConfig::load()`
- TC-439: exit criteria — all TC-431 through TC-438 pass