## Overview

CI Integration makes Product a first-class gate in continuous integration pipelines. It provides machine-readable JSON output on all list and navigation commands, a three-tier exit code scheme that distinguishes clean graphs from errors and warnings, and shell completions for interactive use. Together these capabilities let a CI pipeline run `product graph check`, interpret the result without parsing human-readable text, and fail or warn based on the severity of any issues found.

## Tutorial

### Your first CI gate

This tutorial adds Product as a pull request gate that catches broken links in your knowledge graph.

1. Confirm your repository has a valid `product.toml` and at least one feature file:

   ```bash
   product feature list
   ```

2. Run the graph health check and observe the exit code:

   ```bash
   product graph check
   echo $?
   ```

   If the graph is clean, the exit code is `0`. If there are broken links or cycles, it is `1`. If there are only warnings (orphaned artifacts, missing exit criteria), it is `2`.

3. Introduce a deliberate broken link by editing a feature file to reference a non-existent ADR (e.g., `adrs: [ADR-999]`), then run the check again:

   ```bash
   product graph check
   echo $?
   # Exit code: 1
   ```

   You see a rustc-style diagnostic on stderr pointing to the offending file, line, and reference. Stdout remains empty.

4. Now request JSON output instead:

   ```bash
   product graph check --format json
   ```

   Stderr contains a JSON object with `errors`, `warnings`, and `summary` fields. A CI tool can parse this directly to annotate a pull request.

5. Remove the broken link. Run the check once more and confirm exit code `0`.

### Adding shell completions

Install tab completions for your shell so you can discover commands interactively:

```bash
# Bash
product completions bash > /etc/bash_completion.d/product

# Zsh
product completions zsh > ~/.zfunc/_product

# Fish
product completions fish > ~/.config/fish/completions/product.fish
```

Restart your shell or source the completion file, then type `product g<TAB>` to verify.

## How-to Guide

### Gate a GitHub Actions workflow on graph health

1. Add a step that runs `product graph check --format json`:

   ```yaml
   - name: Knowledge graph check
     run: product graph check --format json
   ```

   The step fails automatically on exit code `1` (errors). Warnings (exit code `2`) also fail by default in GitHub Actions.

2. To allow warnings but fail on errors, use a conditional:

   ```yaml
   - name: Knowledge graph check (errors only)
     run: |
       product graph check --format json || [ $? -eq 2 ]
   ```

   This passes the step when the only issues are warnings (exit code `2`) but fails on hard errors (exit code `1`).

3. Add fitness function thresholds and gap analysis as additional gates:

   ```yaml
   - name: Metrics threshold
     run: product metrics threshold

   - name: Gap analysis (changed files only)
     run: product gap check --changed --format json
   ```

### Get JSON output from list commands

Run any list or navigation command with `--format json` to get structured output on stdout:

```bash
product feature list --format json
product gap check --format json
product graph check --format json
```

Pipe to `jq` for ad-hoc filtering:

```bash
product feature list --format json | jq '.[] | select(.status == "planned")'
```

### Fail CI only on new specification gaps

1. Use the `--changed` flag to scope gap analysis to files modified in the current PR:

   ```bash
   product gap check --changed --format json
   ```

2. The command exits `1` if any new gaps are found in changed files, `0` otherwise.

### Interpret exit codes in a shell script

```bash
product graph check
rc=$?
case $rc in
  0) echo "Graph is clean" ;;
  1) echo "Errors found — broken links, cycles, or parse failures" ;;
  2) echo "Warnings only — orphans, missing exit criteria" ;;
  3) echo "Internal error — this is a bug in Product" ;;
esac
```

## Reference

### Global flag

| Flag | Type | Description |
|---|---|---|
| `--format json` | Global | Emit all output (results, errors, warnings) as structured JSON. Applies to every command. |

### Exit codes

| Code | Meaning | When |
|---|---|---|
| `0` | Success / clean | No errors, no warnings |
| `1` | Error | Broken links (E002), dependency cycles (E003), supersession cycles (E004), parse failures (E001, E005, E006, E007), schema errors (E008), or other hard errors |
| `2` | Warnings only | Orphaned artifacts (W001), missing test criteria (W002), missing exit criteria (W003), missing formal blocks (W004), phase/dependency disagreements (W005), domain gaps (W010, W011), and other validation warnings |
| `3` | Internal error | Bug in Product (I001, I002). Includes source location and version. |

### JSON error schema (`--format json` on stderr)

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
  "warnings": [],
  "summary": { "errors": 1, "warnings": 0 }
}
```

### Shell completions command

```
product completions <SHELL>
```

| Argument | Values |
|---|---|
| `SHELL` | `bash`, `zsh`, `fish` |

Writes the completion script to stdout. Redirect to the appropriate file for your shell.

### CI-relevant commands

| Command | Purpose | Typical CI usage |
|---|---|---|
| `product graph check` | Validate graph integrity | Gate: fail on broken links or cycles |
| `product graph check --format json` | Same, with structured output | Parse errors/warnings for PR annotations |
| `product metrics threshold` | Check fitness function thresholds | Gate: fail if metrics exceed bounds |
| `product gap check --changed --format json` | Find specification gaps in changed files | Gate: fail if PR introduces new gaps |
| `product feature list --format json` | List features as JSON | Reporting, dashboards |

### Interactive error format (default)

When `--format json` is not set, errors and warnings are written to stderr in rustc-style diagnostic format:

```
error[E002]: broken link
  --> docs/features/FT-003-rdf-projection.md
   |
 4 | adrs: [ADR-001, ADR-002, ADR-099]
   |                          ^^^^^^^ ADR-099 does not exist
   |
   = hint: create the file with `product adr new` or remove the reference
```

Every diagnostic includes: error code, description, file path, line number (where applicable), offending content, and a remediation hint.

### Output routing

| Stream | Content |
|---|---|
| stdout | Command results: context bundles, lists, query results, JSON output |
| stderr | Errors and warnings (both interactive and `--format json` modes) |

This separation ensures that piping (`product context FT-001 > bundle.md`) produces clean files even when warnings are present.

## Explanation

### Why exit codes instead of structured output only

Exit codes are the native signaling mechanism of Unix processes. A CI pipeline step fails or passes based on the exit code without any parsing logic. The three-tier scheme (0/1/2) lets teams express nuanced policies: "fail on broken links but tolerate missing exit criteria" is a one-liner (`|| [ $? -eq 2 ]`), while achieving the same with JSON-only output would require a `jq` filter or custom script in every pipeline.

The exit code convention follows `grep` (0 = match, 1 = no match, 2 = error) and lint tools like `clippy`, so engineers arrive with existing intuition about what the codes mean. See ADR-009 for the full rationale.

### Structured errors for machine and human consumption

Product maintains two rendering paths for the same underlying error data. The interactive format (rustc-style diagnostics) is optimized for a developer reading a terminal. The JSON format is optimized for CI tools that annotate pull requests or feed dashboards. Both formats carry identical information: error code, tier, file path, line number, context, detail, and hint. The `--format json` flag is global — it applies to every command, not just `graph check` — so CI pipelines can use a single flag to get machine-readable output everywhere. See ADR-013 for the error taxonomy and format specification.

### Stdout/stderr separation

All errors and warnings go to stderr. Stdout is reserved for command output. This is a Unix convention that makes Product composable: `product context FT-001 > bundle.md` produces a clean Markdown file even if the graph has warnings. CI pipelines that capture stderr separately can surface diagnostics in PR comments without contaminating the primary output. ADR-013 discusses the rejected alternative of writing everything to stdout and why it was not adopted.

### Graph check as the primary CI gate

`product graph check` is the single command designed to answer "is this knowledge graph consistent?" It validates referential integrity (no broken links), structural soundness (no dependency or supersession cycles), and completeness (exit criteria, formal blocks, domain acknowledgements). The exit code directly encodes the answer. This makes it the natural choice for a CI gate — a single line in a workflow file that enforces graph health on every pull request.

### Domain and cross-cutting validation in CI

When ADRs declare `scope: cross-cutting` or `scope: domain` (ADR-025), `product graph check` enforces that features acknowledge them. In CI, this surfaces as W010 (unacknowledged cross-cutting ADR) or W011 (domain gap without acknowledgement). These are warnings (exit code 2), not errors — they prompt review without blocking merges during active development. Teams that want stricter enforcement can treat exit code 2 as a failure in their pipeline configuration.
