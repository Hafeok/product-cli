It looks like the write permission is being blocked. Here's the complete documentation file for FT-026 — CI Integration (~240 lines). You can save it to `docs/guide/FT-026-ci-integration.md`:

---

## Overview

CI Integration makes Product a first-class gate in CI/CD pipelines. It provides machine-readable JSON output on all list and check commands, a structured exit code scheme that encodes error severity, shell completions for developer ergonomics, and a ready-made GitHub Actions workflow pattern. Together these capabilities let teams enforce knowledge graph health, specification coverage, and architectural fitness as automated PR checks — without screen-scraping or custom parsing.

## Tutorial

This walkthrough sets up Product as a CI gate that blocks PRs with broken links or specification gaps. You will run graph checks locally, interpret exit codes, parse JSON output, and create a GitHub Actions workflow.

### Step 1: Run a graph health check

```bash
product graph check
```

If the graph is clean, the command exits with code `0`. If there are errors (broken links, cycles), it exits with code `1`. If there are only warnings (orphaned artifacts, missing exit criteria), it exits with code `2`.

### Step 2: Check the exit code

```bash
product graph check
echo $?
```

A clean graph returns `0`. Try introducing a broken link by referencing a non-existent ADR in a feature's front-matter, then run the check again — you should see exit code `1` with a diagnostic on stderr.

### Step 3: Get JSON output for CI parsing

```bash
product graph check --format json
```

JSON diagnostics go to stderr, keeping stdout clean. Errors and warnings are separated with file paths, line numbers, and remediation hints.

### Step 4: Run gap analysis on changed files

```bash
product gap check --changed --format json
```

### Step 5: Verify fitness function thresholds

```bash
product metrics threshold
```

Exits `1` if any fitness function exceeds its threshold in `product.toml`.

### Step 6: Generate shell completions

```bash
product completions bash > /etc/bash_completion.d/product
```

## How-to Guide

### Gate PRs on graph health

1. Add `product graph check` as a CI step.
2. The step fails (exit `1`) on broken links, dependency cycles, or malformed front-matter.
3. To allow warnings without failing:
   ```bash
   product graph check || [ $? -eq 2 ]
   ```

### Parse JSON diagnostics in CI

1. Redirect stderr:
   ```bash
   product graph check --format json 2> diagnostics.json
   ```
2. Use `file`, `line`, and `detail` fields to post inline PR annotations.

### List features as JSON for tooling

```bash
product feature list --format json
product adr list --format json
product test list --format json
```

### Set up a GitHub Actions workflow

Create `.github/workflows/product-check.yml`:

```yaml
name: Knowledge Graph Check
on: [pull_request]

jobs:
  product-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Product
        run: cargo install --path .
      - name: Graph health
        run: product graph check --format json
      - name: Fitness thresholds
        run: product metrics threshold
      - name: Gap analysis (changed files)
        run: product gap check --changed --format json
```

### Install shell completions

- **Bash:** `product completions bash > /etc/bash_completion.d/product`
- **Zsh:** `product completions zsh > ~/.zfunc/_product`
- **Fish:** `product completions fish > ~/.config/fish/completions/product.fish`

### Distinguish errors from warnings in scripts

```bash
product graph check
case $? in
  0) echo "Clean" ;;
  1) echo "Errors — build must fail" ;;
  2) echo "Warnings only — policy decision" ;;
  3) echo "Internal error — file a bug report" ;;
esac
```

## Reference

### Exit codes

| Code | Meaning | When |
|------|---------|------|
| `0` | Clean | No errors or warnings |
| `1` | Errors | Broken links, dependency cycles, malformed front-matter, invalid IDs |
| `2` | Warnings only | Orphaned artifacts, missing exit criteria, untested features |
| `3` | Internal error | Bug in Product itself — report it |

(ADR-009, ADR-013)

### `--format json` (global flag)

Available on all commands. Errors/warnings go to **stderr** as JSON. Command results go to **stdout** as JSON.

### JSON diagnostic schema

```json
{
  "errors": [
    {
      "code": "E002",
      "tier": "graph",
      "message": "broken link",
      "file": "docs/features/FT-003-rdf-projection.md",
      "line": 4,
      "context": "adrs: [ADR-001, ADR-002, ADR-099]",
      "detail": "ADR-099 does not exist",
      "hint": "create the file with `product adr new` or remove the reference"
    }
  ],
  "warnings": [...],
  "summary": { "errors": 1, "warnings": 0 }
}
```

### Error and warning codes

| Code | Tier | Description |
|------|------|-------------|
| E001 | Parse | Malformed YAML front-matter |
| E002 | Graph | Broken link — referenced artifact does not exist |
| E003 | Graph | Dependency cycle in `depends-on` DAG |
| E004 | Graph | Supersession cycle in ADR `supersedes` chain |
| E005 | Parse | Invalid artifact ID format |
| E006 | Parse | Missing required front-matter field |
| E007 | Parse | Unknown artifact type |
| E008 | Schema | Schema version exceeds binary support |
| E009 | Orchestration | `product implement` blocked by unsuppressed gaps |
| E010 | Concurrency | Repository locked by another Product process |
| E011 | Domain | `domains-acknowledged` entry with empty reasoning |
| E012 | Domain | Unknown domain not in `product.toml` vocabulary |
| W001 | Validation | Orphaned artifact — no incoming links |
| W002 | Validation | Feature has no linked test criteria |
| W003 | Validation | Feature has no exit-criteria type test |
| W004 | Validation | Invariant/chaos test missing formal block |
| W005 | Validation | Phase label disagrees with dependency order |
| W010 | Validation | Cross-cutting ADR not acknowledged by feature |
| W011 | Validation | Domain gap without acknowledgement |

### `product completions`

```
product completions <SHELL>
```

| Argument | Required | Values |
|----------|----------|--------|
| `SHELL` | yes | `bash`, `zsh`, `fish` |

### Interactive vs. structured error format

**Interactive (default):**
```
error[E002]: broken link
  --> docs/features/FT-003-rdf-projection.md
   |
 4 | adrs: [ADR-001, ADR-002, ADR-099]
   |                          ^^^^^^^ ADR-099 does not exist
   |
   = hint: create the file with `product adr new` or remove the reference
```

**Structured (`--format json`):** JSON on stderr (see schema above).

## Explanation

### Why exit codes instead of JSON-only output?

Exit codes are the native CI signal. Every CI system understands non-zero exit as failure without configuration. JSON output via `--format json` is available for richer annotation, but the exit code alone is sufficient for a basic gate (ADR-009).

### Why separate errors (code 1) from warnings (code 2)?

A broken link (E002) means the graph is structurally inconsistent. An orphaned artifact (W001) means something is disconnected but the graph is still valid. The two-code scheme lets teams set tolerance: strict teams fail on any non-zero exit; pragmatic teams use `|| [ $? -eq 2 ]` to allow warnings during active development (ADR-009).

### Why stderr for errors and stdout for results?

Unix convention that makes piping reliable. `product context FT-001 > bundle.md` produces a clean file even when warnings are present. In CI, stdout and stderr are often handled by different log processors (ADR-013).

### Why rustc-style diagnostics?

The interactive format mirrors rustc and clang — file path, line number, offending content, and a remediation hint in one message. The `hint` field tells the developer exactly what command to run, reducing the feedback loop from "error → search docs → fix" to "error → fix" (ADR-013).

### How domain checks integrate with CI

Domain validation (ADR-025) adds W010 (unacknowledged cross-cutting ADR) and W011 (domain gap) to `product graph check`. These surface as warnings (exit code 2), not errors, so they do not block CI by default. Teams that want to enforce domain coverage can fail on any non-zero exit.
