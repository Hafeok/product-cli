---
id: ADR-060
title: CLI↔MCP parity is enforced by a fitness gate with a self-cleaning debt list
status: accepted
features:
- FT-118
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
content-hash: sha256:11c218832f30d5976b94fdf83507051a19329562356224f4a13fd5ba5817053d
source-files:
- product-cli/tests/code_quality_tests.rs
---

## Context

A recurring defect in this codebase: a new CLI command family is added without
exposing the same functionality as an MCP tool, so agents (which drive the
product through MCP) silently lose access to it. The convention — established
for `feature`, `adr`, `test`, `pattern`, etc. (`product_<cmd>_*` tools) — was
documented but not enforced, so it kept being missed. Dogfooding the framework
made the gap concrete: the five new What/How/cell/archetype/work-unit command
families (plus the older `dep`) ship CLI-only, with no MCP tools.

## Decision

Add a fitness gate (`tc_980_cli_commands_have_mcp_parity`) that classifies
**every** top-level CLI command into exactly one of three buckets:

1. **MCP-exposed** — a tool `product_<cmd>_*` exists in the registry (detected
   automatically).
2. **CLI-only** — an explicit allowlist of process/launch/meta commands that
   legitimately have no MCP surface (`mcp`, `author`, `implement`, `init`, …).
3. **Pending MCP** — an explicit, documented debt list of commands that *should*
   expose MCP tools but do not yet.

A command in **none** of the three fails the gate — so a newly-added command
cannot silently skip MCP; the author must add a tool or make a conscious
classification. The debt list is **self-cleaning**: an entry that later gains a
tool fails the gate until removed, and stale (non-command) entries fail too.

The rule is also recorded in product-cli's own How archetype as the
`cli-mcp-parity` principle, `enforced_by` this gate — the architecture
references the verification that protects it.

## Rationale

- Enforcing classification (rather than blanket-requiring MCP) respects that
  some commands are genuinely CLI-only, while making the omission for everything
  else a hard, visible failure at the moment a command is added.
- A self-cleaning debt list turns "we keep forgetting" into a shrinking,
  reviewable checklist that cannot rot: the gate breaks if an entry is stale.
- Recording the principle in the How ties the convention to a real verification,
  exactly as the framework's earn-their-place rule intends.

## Rejected alternatives

- **Documentation only (CLAUDE.md).** Rejected: that is the status quo that kept
  being missed; conventions that are not mechanically enforced regress.
- **Hard-require an MCP tool for every command.** Rejected: process/launch
  commands (`mcp`, `author`, `implement`) have no meaningful MCP surface; the
  allowlist captures that without weakening the gate for everything else.

## Test coverage

- TC-980 — the parity gate: every command is MCP-exposed or classified; the
  debt list is self-cleaning; demonstrated to flag the unexposed families when
  the debt list is empty.
