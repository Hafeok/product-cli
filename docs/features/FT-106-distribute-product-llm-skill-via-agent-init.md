---
id: FT-106
title: Distribute Product LLM Skill via agent-init
phase: 6
status: planned
depends-on:
- FT-072
- FT-075
adrs:
- ADR-031
- ADR-048
- ADR-043
- ADR-051
- ADR-042
- ADR-018
- ADR-047
- ADR-050
tests:
- TC-876
- TC-877
- TC-878
- TC-879
- TC-880
- TC-881
- TC-882
- TC-883
- TC-884
domains:
- api
domains-acknowledged:
  ADR-041: FT-106 is purely additive — no surface is removed or deprecated.
  ADR-049: FT-106 does not touch the context bundle assembly path.
  ADR-040: FT-106 ships a distribution surface; it does not modify the verify pipeline.
patterns:
- PAT-001
- PAT-003
---

## Description

Ship a `SKILL.md` agent instructions file alongside the CLI so
that any LLM-driven coding assistant working in a Product-managed
repo can be taught the correct authoring, implementation, and
verification workflows by simply discovering the file at one of
the well-known skill paths it already scans at session start — no
per-repo onboarding prompt required.

Claude Code and GitHub Copilot's cloud agent both consume the
same `SKILL.md` format (YAML front-matter with `name`,
`description`, optionally `license` and `allowed-tools`, followed
by markdown body). Copilot scans three locations and so does
Claude Code's project-local skill discovery:

| Target name | Path                                | Discovered by   |
|-------------|-------------------------------------|-----------------|
| claude      | `.claude/skills/product/SKILL.md`   | Claude Code + Copilot |
| copilot     | `.github/skills/product/SKILL.md`   | Copilot         |
| agents      | `.agents/skills/product/SKILL.md`   | Copilot (and other agent frameworks adopting the convention) |

One canonical body (baked into the CLI binary) is written
byte-for-byte to one or more of these target paths. There is no
per-tool reformatting — the format is shared. The user-global
Claude Code target (`~/.claude/skills/product/SKILL.md`) is also
supported for ambient cross-repo availability.

The skill is the structural counterpart to `AGENTS.md`: where
`AGENTS.md` tells an agent *the current state of this repo*,
`SKILL.md` tells an agent *how to use Product in any repo*. All
artifacts get written by the same command (`product agent-init`)
so an operator who runs the one bootstrap step has every
supported agent fully wired up.

## Why now

Three things just landed in close succession:

- **ADR-051 / FT-072** — TCs must declare `observes:` and assert on
  the named surface. Agents that don't know this rule ship the
  FT-046 envelope-only failure mode.
- **PAT-001..PAT-003** (FT-075) — the structural patterns the
  codebase actually expects (slice + adapter, MCP write parity, TC
  causation). Without distribution, these patterns only help
  agents that already know to look in `docs/patterns/`.
- **FT-074** — `product implement` now surfaces patterns and
  `observes:` in the executor bundle, but only inside the implement
  loop. Authoring, manual review, and ad-hoc work still need an
  out-of-band channel for the same rules.

A baked-in skill closes the loop: any agent invoked in a
Product-managed repo discovers `SKILL.md` at one of the
well-known skill paths it already scans and learns the rules
without a system-prompt edit.

## Functional Specification

### Inputs

- `product agent-init` invocation (CLI args: optional `--watch`,
  optional `--skill-target user` to add the user-global Claude
  path to the install set).
- `.product/config.toml` (`[agent-context.skill]` block:
  `install` master switch; `targets` list naming which discovery
  paths to write to; optional `overlay` path for a repo-side body
  override).
- Repo root (resolved via existing `ProductConfig::discover`).
- The embedded skill body baked into the binary at compile time
  from `docs/skills/product-v1.md` via `include_str!`.

### Outputs

- One byte-identical `SKILL.md` file written to each enabled
  target path under the repo root (or the resolved user-global
  path for `target = "user"`). The body is the embedded default
  or the overlay if configured and present.
- One stdout `Generated: <path>` line per target that was
  written.
- When `install = false`, or a target name is absent from the
  `targets` list: that target is silently skipped (no `Generated:`
  line). When `install = false`: a `Skipped: skill install
  disabled in config` stdout line is emitted; no file is written
  at any target.

### State

- Skill installation is **stateless** with respect to graph
  state. The skill body is not derived from the loaded
  `KnowledgeGraph` and does not change when features, ADRs, or
  TCs change. (This is the key difference from `AGENTS.md` and
  the reason watch mode does not re-emit the skill.)
- Atomic write semantics per file via `fileops::write_file_atomic`
  (same guarantees as `AGENTS.md`). Each target is written
  independently; a write failure at one target does not block
  the others.
- No new on-disk state is introduced under `.product/`.

### Behaviour

- F1: the canonical skill source lives at
  `docs/skills/product-v1.md` in this repo and is embedded via
  `include_str!` at compile time. A future minor revision lifts
  the version filename (e.g. `product-v2.md`) and updates the
  `include_str!` site.
- F2: the adapter calls `agent_context::skill::plan_install_skill`
  once; the plan struct carries one body and a list of resolved
  target paths. `apply_install_skill` writes the same bytes to
  each path. The CLI flag `--skill-target user` adds the
  user-global Claude path to the install set.
- F3: `[agent-context.skill]` config defaults are `install =
  true`, `targets = ["claude"]`. Missing block is treated as
  defaults. The default keeps the install surface minimal; users
  who want Copilot or the cross-tool agents convention add
  `"copilot"` or `"agents"` to the list.
- F4: idempotence is guaranteed by the body being static; the
  apply step writes unconditionally so mtime changes but bytes do
  not — at every target.
- F5: slice + adapter shape per PAT-001 — see the F5 section
  below.

### Invariants

- Running `product agent-init` N times never produces a different
  `SKILL.md` body at any target unless the embedded source or
  the overlay changes between runs.
- All targets written in a single run are byte-identical — there
  is one body, copied to N paths.
- The installed file is always a valid `SKILL.md`: it has a YAML
  front-matter block with `name: product` and a non-empty
  `description:` key (the schema both Claude Code and Copilot
  enforce).
- `install = false` is a hard veto: no file is created at any
  target.

### Error handling

- Config parse errors are reported through the existing
  `ProductError::ConfigError` path (existing `agent-init`
  behaviour preserved).
- An overlay path that points at a missing file is a soft
  fallback to the embedded default and emits a warning to stderr
  (`warning: overlay <path> not found, using embedded default`).
- An unrecognised target name (anything outside `claude`,
  `copilot`, `agents`, `user`, or a relative path) is rejected
  via `ProductError::ConfigError` with a list of valid names —
  no silent skip.
- A target write failure (permission denied, ENOSPC) returns
  `ProductError::IoError` and propagates to the exit code via the
  existing `error.rs` mapping. A failure at one target does not
  prevent later targets from being attempted; the overall exit
  code reflects the first failure.

### Boundaries

- Skill distribution is the only LLM-tool-specific surface
  Product owns. The CLI does not attempt to discover, manage, or
  uninstall artifacts written by other tools.
- v1 ships three named targets (`claude`, `copilot`, `agents`)
  plus the user-global Claude path. Future tools that adopt the
  same `SKILL.md` shape are added by extending the name → path
  map in the slice; no rendering logic changes.
- The skill body is **never** rewritten by `--watch`. Graph
  edits during a watch session do not invalidate the skill.
- The user-global Claude target
  (`~/.claude/skills/product/SKILL.md`) is shared across repos.
  If two Product CLI versions write here, the later run wins;
  this is acceptable because the skill body changes are
  intentionally small and additive.

### F1 — Skill content shipped in the binary

The CLI binary embeds a canonical `SKILL.md` body via
`include_str!`. The embedded source-of-truth lives at
`docs/skills/product-v1.md` in this repo (analogous to
`docs/prompts/*.md`), so it is reviewable and the same artifact a
spec change touches.

`product prompts list` already lists authoring prompts; a sibling
listing path is introduced so `product agent-init` and an operator
can inspect what will be installed without running the install.

### F2 — `product agent-init` writes the skill to each enabled target

`product agent-init` already writes `AGENTS.md`. After F2 it also
writes the canonical `SKILL.md` body to each path named in the
`targets` list. v1 recognises four target names and any
caller-supplied relative path:

| Name      | Resolved path                                          |
|-----------|--------------------------------------------------------|
| `claude`  | `<repo>/.claude/skills/product/SKILL.md`               |
| `copilot` | `<repo>/.github/skills/product/SKILL.md`               |
| `agents`  | `<repo>/.agents/skills/product/SKILL.md`               |
| `user`    | `<home>/.claude/skills/product/SKILL.md`               |
| `<path>`  | `<repo>/<path>/SKILL.md` (literal relative path)       |

Claude Code's project-local skill discovery scans `.claude/skills/`;
Copilot's cloud agent scans all three of `.github/skills/`,
`.claude/skills/`, and `.agents/skills/` (per the GitHub Copilot
"Add skills" docs). So `targets = ["claude"]` alone is already
enough for both Claude Code and Copilot to pick up the same file;
adding `"copilot"` writes a second byte-identical copy at the
Copilot-native location for operators who prefer the explicit
mapping.

The handler creates each parent directory if missing
(`.claude/skills/product/`, `.github/skills/product/`, etc.) and
writes via `fileops::write_file_atomic` so re-runs are atomic
per target.

### F3 — Configurable opt-in/opt-out and overlay

A new `[agent-context.skill]` block in the Product config controls
distribution:

```toml
[agent-context.skill]
install = true                       # default
targets = ["claude"]                 # default — Copilot also reads this path
overlay = "docs/skills/product.md"   # optional repo-side body override
```

Behaviour:

- `install = false` — `agent-init` writes no skill file; prints
  `Skipped: skill install disabled in config`.
- `targets = ["claude", "copilot"]` — writes byte-identical
  `SKILL.md` files to both Claude Code and Copilot native paths.
- `targets = ["claude", "copilot", "agents"]` — also writes to
  the cross-tool `.agents/skills/product/SKILL.md`.
- `targets = ["claude", "user"]` — writes both project-local and
  user-global Claude paths. `--skill-target user` on the CLI is
  shorthand for adding `"user"` to the list for that invocation.
- `targets = ["some/custom/path"]` — writes
  `<repo>/some/custom/path/SKILL.md`. Useful for repos with
  bespoke skill discovery conventions.
- `overlay = <path>` — if the file exists, its contents are used
  instead of the embedded default. Lets a repo ship a customised
  skill body (e.g. with repo-specific MCP tool names) without
  forking the CLI.

### F4 — Idempotent regeneration

Re-running `agent-init` produces byte-identical files at every
enabled target when the embedded source (or overlay) has not
changed. Watch mode (`--watch`) does **not** rewrite the skill
on front-matter change — only on the first run of the session —
because the skill body is not derived from graph state.

### F5 — Slice + adapter shape

Per PAT-001, the install logic lives in a slice:
`src/agent_context/skill.rs` with:

```rust
pub struct SkillInstallPlan {
    pub body:    String,            // shared canonical body
    pub targets: Vec<TargetSpec>,   // one resolved path per enabled name
    pub source:  SkillSource,       // Embedded | Overlay(PathBuf)
}

pub struct TargetSpec {
    pub name: String,      // "claude" | "copilot" | "agents" | "user" | <path>
    pub path: PathBuf,     // fully resolved absolute path
}

pub fn plan_install_skill(config: &ProductConfig, root: &Path)
    -> Result<SkillInstallPlan, ProductError>;

pub fn apply_install_skill(plan: &SkillInstallPlan)
    -> Result<(), ProductError>;
```

The CLI adapter in `commands/agent_init.rs` calls the existing
`generate_agent_md` plus the new `plan_install_skill` +
`apply_install_skill`. The plan is fully unit-testable without
any tempdir; `apply_install_skill` is the only function that
touches the filesystem.

## Out of scope

- **Dynamic skill content from graph state.** The skill body is
  static for v1. A future feature can templatise sections (domain
  list, MCP tool count, observability surfaces) from the live
  graph, but that earns the same drift cost `AGENTS.md` already
  pays — keep skills stable until there's demand.
- **Tool-specific reformatting.** Claude Code, Copilot's cloud
  agent, and the cross-tool `.agents/` convention all consume
  the same `SKILL.md` shape (YAML front-matter with `name` and
  `description`, optional `license` and `allowed-tools`, markdown
  body). There is no per-tool renderer in v1, and the spec is
  explicit that adding one would require a new feature. Tools
  with incompatible shapes (e.g. Cursor's `.cursor/rules/*.mdc`,
  Windsurf's `.windsurfrules` plain-markdown convention) are
  deferred to a follow-up.
- **Updating `.gitignore`.** Operators decide whether to commit
  the skill files. `product agent-init` does not modify
  `.gitignore`.
- **Skill versioning negotiation.** Each installed file ships
  the body baked into the CLI version that wrote it. There is
  no protocol for "this file is older than the CLI"; if you want
  newer bodies, install a newer CLI.

## Exit criteria

- `product agent-init` writes `.claude/skills/product/SKILL.md`
  with the embedded body in a fresh repo with default config.
- With `targets = ["claude", "copilot"]`, `agent-init` also
  writes `.github/skills/product/SKILL.md` byte-identical to the
  Claude path.
- With `targets = ["claude", "copilot", "agents"]`, `agent-init`
  also writes `.agents/skills/product/SKILL.md` byte-identical to
  the others.
- Every installed file passes the shared `SKILL.md` schema
  (front-matter present with `name: product` and a non-empty
  `description:`; body non-empty).
- `[agent-context.skill].install = false` suppresses all writes.
- `[agent-context.skill].overlay = <path>` replaces the embedded
  body when the overlay file exists, at every enabled target.
- The install logic lives in `src/agent_context/skill.rs` as
  pure `plan_install_skill` + `apply_install_skill` per PAT-001,
  with unit tests against an in-memory `SkillInstallPlan`.
