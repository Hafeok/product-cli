---
id: TC-900
title: author domain defaults product to configured name
type: scenario
status: passing
validates:
  features:
  - FT-109
  adrs:
  - ADR-053
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_900_author_domain_defaults_product_to_config_name
---

## Scenario — the product positional defaults to the repo's configured name

**Given** a repository whose `product.toml` declares `name = "test"`,
**When** the user runs `product author domain --print-prompt` with no product
positional,
**Then** the process exits 0 and the facilitation prompt on stdout names the
configured product `test` (the positional is not required for a single-product
repo).

## Validates

- FT-109 — product author domain — facilitated What-capture MCP session
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
