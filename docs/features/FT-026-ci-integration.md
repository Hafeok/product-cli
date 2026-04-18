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
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
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
