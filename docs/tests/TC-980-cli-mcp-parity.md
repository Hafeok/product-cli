---
id: TC-980
title: every CLI command is MCP-exposed or explicitly classified
type: scenario
status: passing
validates:
  features:
  - FT-118
  adrs:
  - ADR-060
phase: 6
observes:
- exit-code
runner: cargo-test
runner-args: tc_980_cli_commands_have_mcp_parity
---

## Scenario — the CLI↔MCP parity gate

**Given** the top-level CLI commands and the MCP tool registry,
**When** the fitness gate runs,
**Then** the test fails (non-zero exit code) unless every command is
MCP-exposed (`product_<cmd>_*`),
listed in the CLI-only allowlist, or listed in the documented pending-MCP debt
list; a command in none fails with its name; a pending-MCP entry that has since
gained a tool fails (so the debt list is self-cleaning); and a stale
classification entry fails.

With the debt list emptied, the gate flags exactly the unexposed command
families (`archetype`, `cell`, `dep`, `domain`, `how`, `work-unit`) — the
recurring "added a command without an MCP tool" defect.

## Validates

- FT-118 — CLI↔MCP parity gate — every command exposes an MCP tool or is classified
- ADR-060 — CLI↔MCP parity is enforced by a fitness gate with a self-cleaning debt list
