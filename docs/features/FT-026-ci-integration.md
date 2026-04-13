---
id: FT-026
title: CI Integration
phase: 3
status: planned
depends-on:
- FT-018
- FT-024
adrs:
- ADR-009
- ADR-013
tests:
- TC-181
domains: []
domains-acknowledged: {}
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
