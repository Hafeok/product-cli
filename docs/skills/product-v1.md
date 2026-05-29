---
name: product
description: Use the Product CLI/MCP to read and modify a knowledge graph of features (FT-XXX), ADRs (ADR-XXX), patterns (PAT-XXX), and test criteria (TC-XXX). Trigger when working in any repo that contains a `product.toml` at the root, OR when the user asks to "author a feature/ADR/pattern", "implement FT-XXX", "verify FT-XXX", "write a TC for ...", "check graph health", "find specification gaps", or similar. The graph is the source of truth — every artifact has YAML front-matter that the CLI parses; never hand-edit CHECKLIST.md or feature/TC status fields, let the CLI manage them.
---

# Product — using the knowledge graph CLI

Product manages a file-based knowledge graph of features, architectural
decisions, patterns, and test criteria. Every artifact is a Markdown file
with YAML front-matter under `docs/` (paths configured in `product.toml`).
The CLI (`product`) and MCP server expose the same operations against this
graph.

If you are unsure whether a repo uses Product: check for `product.toml` at
the root. If present, use the workflows below. If absent, this skill does
not apply.

## Mental model — three rules that govern everything

1. **The graph is derived from front-matter on every invocation.** There is
   no persistent store. Edits to `adrs:`, `tests:`, `patterns:`, `status:`,
   `observes:`, etc. *are* the graph. (ADR-002, ADR-003)
2. **The CLI owns writes.** Never hand-edit `CHECKLIST.md`, feature
   `status:`, TC `status:`, or reciprocal links. Use `product feature
   status`, `product test status`, `product feature link`, etc. — they
   write atomically and keep both sides of every edge in sync.
3. **Test criteria define done.** A feature is not complete until every
   linked TC runner passes via `product verify FT-XXX`. Do not flip a
   status by hand to claim progress; `verify` will overwrite it.

## Phase 1 — read before writing

Before authoring or implementing anything, orient yourself in the graph:

```bash
product status                           # project summary
product feature next                     # topological next-to-implement
product feature list --status planned    # backlog
product graph check                      # structural health (must be clean)
product graph central --top 5            # the high-centrality ADRs to know
product gap report                       # specification gaps
```

For a specific artifact:

```bash
product feature show FT-XXX
product adr show ADR-XXX
product pattern show PAT-XXX
product test show TC-XXX
product context FT-XXX --depth 2         # bundle for an LLM session
product impact ADR-XXX                   # what changes if this ADR moves
product feature deps FT-XXX              # full dependency tree
```

## Phase 2 — authoring (specification work)

Product ships canonical prompts for each authoring flow. **Read the live
prompt from the repo before doing the work** — it lists the exact tool
calls and the gates that must pass before the session can close:

```bash
product prompts list
product prompts get author-feature       # for a new feature
product prompts get author-adr           # for a new decision
product prompts get author-pattern       # for a new structural template
product prompts get author-review        # for spec coverage review
```

Equivalent flow via the CLI: `product author feature`, `product author
adr`, `product author pattern`, `product author review` — these print the
matching prompt and wire up the session.

### Authoring TCs — the rules you must encode

Every scenario / session / smoke / contract TC at phase ≥ the configured
observability threshold **must** declare `observes:` in front-matter
(ADR-051, FT-072). Allowed surfaces: `file`, `graph`, `exit-code`, `tag`,
`stdout`, `stderr`, `disk-state`, `mcp-response` (plus any custom
surfaces in `[tc-observability].custom`).

```yaml
---
id: TC-778
title: mcp_feature_status_writes_to_disk
type: scenario
observes: [file, mcp-response]
runner: cargo-test
runner-args: tc_778_mcp_feature_status_writes_to_disk
---
```

- The TC body must assert on the named surface(s). `product graph check`
  rejects missing `observes:` (E032); the body-reference gate warns when
  a declared surface is never mentioned in the body.
- Scaffold with `product test new "title" --type scenario --observes file
  --observes mcp-response`.
- Configure the runner the moment you write the integration test:
  `product test runner TC-XXX --runner cargo-test --args
  tc_XXX_snake_case_title`. Without this, `product verify` skips the TC
  and five gates fail with E022.
- Assert on the **causation**, not the envelope. The FT-046 lesson:
  an MCP write that returns `{ "status": "complete" }` while the file on
  disk is unchanged is a green test of a broken feature. Always read the
  file the action claims to have mutated. See PAT-003 (`product pattern
  show PAT-003`) for the worked example.

### Closing an authoring session — the hard gates

Authoring sessions **cannot close** until the relevant gate is clean. The
host auto-commit refuses dirty graphs:

- **Feature**: `product graph check` clean AND `product preflight FT-XXX`
  status `clean` (warnings are not advisory — `product implement` will
  hard-block on the same gaps). Resolve every gap by linking the missing
  ADR/TC, or set `domains-acknowledged.<domain>` to a written reason.
- **ADR**: all five sections present (Context, Decision, Rationale,
  Rejected alternatives, Test coverage). Check conflicts via
  `product gap check` and `product impact` first.
- **Pattern**: all five required H2 sections present (`## When to use`,
  `## Prerequisites`, `## The pattern`, `## Anti-patterns`,
  `## Worked example`), at least one `adrs:` link, no `E031`
  (requires-cycle) or `W032/W033` findings in `product graph check`.

## Phase 3 — implementing a feature

```bash
product implement FT-XXX                 # full pipeline (preferred)
```

This runs preflight, assembles the depth-2 context bundle (feature + ADRs
+ TCs + cited patterns + transitive `requires:`), spawns the implementer,
and gates completion on `product verify FT-XXX`. **Do not also run
`product context`** — the bundle is already in the prompt.

If implementing manually (no `product implement`):

```bash
product preflight FT-XXX                 # must be clean before you start
product context FT-XXX --depth 2         # this is your context bundle
# ... read the patterns the bundle cites, then write the code + tests ...
product test runner TC-XXX --runner cargo-test --args tc_XXX_...
product verify FT-XXX                    # promotes statuses on green
product gap check && product drift check # post-implementation health
```

When the bundle lists patterns under `## Patterns`, **read them before
the TCs**. They are operationalisations of the governing ADRs that tell
you *how* this codebase wants the work shaped. PAT-001 (slice + adapter)
and PAT-002 (MCP write parity) are particularly load-bearing for any
feature that touches the CLI or MCP surface.

## Phase 4 — verifying and committing

```bash
product verify FT-XXX                    # per-feature six-stage pipeline
product verify                           # platform-wide
product gap check                        # finding-level gaps
product drift check                      # spec-vs-implementation drift
product graph check                      # structural integrity
product checklist generate               # regenerates CHECKLIST.md
```

`product verify` updates TC `status:` from runner exit codes and promotes
the feature to `complete` when all linked TCs pass. The committing host
reads the resulting graph; do not re-commit `CHECKLIST.md` separately.

## Common operations — quick reference

| Task | Command |
|---|---|
| What's next | `product feature next` |
| See everything | `product status` |
| Read context bundle | `product context FT-XXX --depth 2` |
| Show full prompt for a flow | `product prompts get <name>` |
| New feature scaffold | `product feature new "title"` (or `product author feature`) |
| New ADR scaffold | `product adr new "title"` |
| New pattern scaffold | `product pattern new "title"` |
| New TC scaffold | `product test new "title" --observes file --observes mcp-response` |
| Configure TC runner | `product test runner TC-XXX --runner cargo-test --args tc_XXX_...` |
| Link feature → ADR | `product feature link FT-XXX --adr ADR-YYY` |
| Link feature → TC | `product feature link FT-XXX --test TC-YYY` |
| Link feature → pattern | `product feature link FT-XXX --pattern PAT-YYY` |
| Link pattern → ADR | `product pattern link PAT-X --adr ADR-Y` |
| Link pattern → prereq pattern | `product pattern link PAT-X --requires PAT-Z` |
| Link pattern → example feature | `product pattern link PAT-X --example FT-N` |
| Set feature status | `product feature status FT-XXX in-progress` (rarely needed; verify does it) |
| Set TC status | `product test status TC-XXX passing` (rarely needed; verify does it) |
| Acknowledge a gap | `product feature acknowledge FT-XXX` (interactive) |
| Preflight gate | `product preflight FT-XXX` |
| Impact analysis | `product impact ADR-XXX` |
| Most central ADRs | `product graph central --top 5` |
| Output JSON instead of text | append `--format json` to any command |

## MCP equivalents

When operating via MCP (e.g. inside `product implement`), the same
operations are exposed as `product_*` tools: `product_feature_show`,
`product_context`, `product_graph_check`, `product_feature_status`,
`product_pattern_link`, etc. PAT-002 governs every write-tool
implementation — handlers must dispatch into the shared slice's `plan_*`
+ `apply_*` so the response envelope is backed by a real disk write.
Never trust an MCP envelope alone in a TC; assert on the file or graph.

## Anti-patterns — things that look right but are wrong

- **Hand-editing `CHECKLIST.md`, `status:`, or reciprocal `tests:` /
  `adrs:` links.** The CLI keeps both sides of every edge in sync. Manual
  edits desync the graph and `graph check` will flag them.
- **TCs that assert only on a return value or response envelope.** This
  is the FT-046 failure mode — green tests over a broken feature. Read
  the file. (PAT-003)
- **Marking a feature `complete` before TCs pass.** `verify` will undo
  it; do not race the pipeline.
- **Skipping `product preflight` before implementing.** The implementer
  pipeline hard-blocks on the same gaps and you will waste a session.
- **Adding `observes:` but never reading the surface in the TC body.**
  The body-reference gate catches this; the test is decorative
  otherwise.
- **Writing MCP write-tool handlers that return `{ ok: true, note: "use
  the CLI for ..." }`.** Either dispatch into the slice and actually
  write, or do not advertise `requires_write: true`. (PAT-002)
- **Creating a new CLI subcommand whose business logic lives in the
  handler.** Extract a slice with `plan_*` + `apply_*` per PAT-001 so
  the rule is unit-testable without `assert_cmd`.

## When you hit `graph check` errors

Common codes (read the message; the CLI tells you which file to fix):

- `E003` requires-cycle on patterns
- `E016` ADR lifecycle violation
- `E022` TC missing runner config
- `E031` pattern requires-cycle
- `E032` TC missing `observes:` at required phase
- `W032` deprecated pattern still cited
- `W033` pattern body missing a required H2 section

Fix the underlying artifact, re-run `product graph check`, repeat until
clean. Do not bypass the gate.
