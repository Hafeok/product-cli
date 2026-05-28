---
id: FT-026
title: CI Integration
phase: 3
status: complete
depends-on:
- FT-018
- FT-024
adrs:
- ADR-009
- ADR-013
tests:
- TC-181
domains:
- api
- error-handling
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
---

Machine-readable output formats and CI/CD integration points that make Product a first-class CI gate.

### JSON Output

`--format json` output on all list and navigation commands. Structured JSON to stdout for CI annotation and tooling integration.

```
product feature list --format json
product graph check --format json     # structured stderr for CI
product gap check --format json       # structured JSON to stdout for CI annotation
```

### Shell Completions

```
product completions bash > /etc/bash_completion.d/product
product completions zsh > ~/.zfunc/_product
product completions fish > ~/.config/fish/completions/product.fish
```

### GitHub Actions

Example GitHub Actions workflow that gates PRs on:
- `product graph check --format json` — zero errors
- `product metrics threshold` — fitness functions within bounds
- `product gap check --changed --format json` — no new gaps

### Exit Criteria

`product graph check` CI gate fails on a PR with a broken link. All list commands produce valid JSON with `--format json`.

---

## Description

CI Integration makes Product a first-class CI gate by providing machine-readable JSON output on all list and navigation commands, a structured three-tier exit code scheme (0 = clean, 1 = errors, 2 = warnings only per ADR-009), and shell completions for bash, zsh, and fish. The `--format json` flag on graph check, gap check, and list commands allows CI pipelines to parse results, annotate PRs, and enforce thresholds without text parsing. The exit code scheme allows a CI step to distinguish hard errors (broken links) from soft warnings (coverage gaps) and configure its tolerance accordingly.

## Functional Specification

### Inputs

- **`--format json`**: flag accepted by `product feature list`, `product graph check`, `product gap check`, `product adr list`, and other list/navigation commands
- **`product completions bash|zsh|fish`**: shell name argument
- **GitHub Actions workflow**: calls `product graph check --format json`, `product metrics threshold`, and `product gap check --changed --format json` as CI steps
- **`product.toml` `[metrics.thresholds]`**: threshold configuration read by `product metrics threshold`

### Outputs

- **JSON to stdout**: structured output for all `--format json` commands; format is stable and versioned
- **Exit codes**: 0 (clean), 1 (errors — broken links, malformed front-matter, E-class errors), 2 (warnings only — orphaned artifacts, coverage gaps, W-class warnings) per ADR-009
- **Shell completion scripts**: to stdout, suitable for redirect to the appropriate completion directory
- **`product graph check --format json`**: JSON object with `errors` and `warnings` arrays; exit 1 if any errors, exit 2 if warnings only
- **`product gap check --format json`**: JSON finding objects (same schema as non-CI mode) to stdout; exit 1 on new unsuppressed high-severity findings

### State

Stateless. No data is retained between invocations. The exit code and JSON output are derived fresh from the current graph state on each invocation.

### Behaviour

1. Any command with `--format json` writes valid JSON to stdout. Errors and informational messages continue to go to stderr. This separation allows CI to capture structured output via `>` while still seeing diagnostic messages.
2. `product graph check` uses the three-tier exit code: exit 0 for a clean graph, exit 1 for hard errors (broken links, cycles, malformed front-matter), exit 2 for warnings only (orphaned artifacts, features without exit criteria). A CI pipeline can choose its tolerance: `product graph check` (fail on errors and warnings) or `product graph check || [ $? -eq 2 ]` (fail on errors only).
3. `product completions bash` writes a Bash completion script to stdout. The operator redirects it to the system completion directory. The script is generated from the Clap definition and is always in sync with the current command surface.
4. Example GitHub Actions workflow gates PRs on three checks: `product graph check --format json` (zero errors), `product metrics threshold` (fitness functions within bounds), `product gap check --changed --format json` (no new gaps).
5. `product gap check --changed` identifies changed ADR files via `git diff --name-only HEAD~1`, expands to 1-hop graph neighbours, and runs analysis only on the affected ADR subgraph. CI cost is proportional to change scope, not repository size.

### Invariants

- All list and navigation commands that support `--format json` write valid, parseable JSON to stdout when the flag is present. Malformed JSON output is a bug.
- Exit codes are stable and documented (ADR-009). A command that exits 1 on warnings instead of 2 is a bug.
- `product graph check` exit 1 on a PR with a broken link — this is the primary CI gate invariant (TC-181).
- stdout and stderr are always separated: structured results to stdout, diagnostics to stderr. This is required for CI pipeline JSON capture.

### Error handling

- **`--format json` with graph errors**: errors appear in the JSON `errors` array; exit code is 1. The JSON is always written even when there are errors — the CI step receives both the exit code and the structured data.
- **Model errors in `product gap check`**: gap analysis exits 2 (warning) rather than 1 (error) on model API failures — a transient model error never fails CI as if new gaps were found.
- **Invalid threshold configuration**: `product metrics threshold` exits 1 with a configuration error if `product.toml` contains unrecognised threshold keys.

### Boundaries

- Product does not write GitHub PR annotations or comments. The CI step reads JSON output and uses the GitHub Actions API to annotate if desired.
- Product does not configure CI workflow files. The example GitHub Actions workflow in the docs is a reference; operators copy and adapt it.
- Shell completions are generated from the Clap definition at compile time — they reflect the actual command surface and cannot drift independently.

## Out of scope

- GitHub PR annotation or comment posting (CI harness responsibility)
- CI workflow file generation or management
- Test result reporting formats beyond exit codes and JSON (JUnit XML, TAP, etc.)
- Parallel CI step execution coordination
- CI caching of graph state between steps (the graph is rebuilt fresh each invocation per ADR-003)
