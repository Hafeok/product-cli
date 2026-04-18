---
id: FT-035
title: Repository Initialization
phase: 1
status: complete
depends-on: []
adrs:
- ADR-033
- ADR-003
- ADR-008
- ADR-020
- ADR-025
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
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
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
