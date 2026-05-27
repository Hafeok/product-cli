---
id: ADR-048
title: Canonical Repository Layout — All Product State Under `.product/`
status: accepted
features:
- FT-057
- FT-068
- FT-070
- FT-071
- FT-072
- FT-073
- FT-074
- FT-075
supersedes: []
superseded-by: []
domains: []
scope: cross-cutting
content-hash: sha256:140b545251c13f4f2e940bd3f7c53cf58fb1c271515f827c2eb1c1bb13412f94
---

**Status:** Proposed

---

**Context:**

Product CLI currently writes and reads files at six different
locations in a host repository:

| Location | Owner | Purpose |
|---|---|---|
| `product.toml` (root) | Product | Repo configuration |
| `docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/dependencies/`, `docs/graph/` | Product | Spec artifacts + generated graph cache |
| `docs/checklist.md` (or `CHECKLIST.md`) | Product | Generated checklist |
| `benchmarks/prompts/` | Product | System prompts (ADR-022) |
| `gaps.json` (root) | Product | Gap baseline (suppressions) |
| `requests.jsonl` (root) | Product | Tamper-evident request log |
| `.product/sessions/`, `.product/request-log.jsonl` | Product | Per-clone session logs and (legacy) request log |

Three problems follow from this scatter:

1. **Discoverability and noise.** A new contributor opening a repo
   sees seven product-managed paths intermixed with the project's
   own files. The repo root looks owned by the tool, not the
   project.
2. **Selective gitignore is fiddly.** Some product paths must be
   committed (`product.toml`, spec docs, request log), others must
   not (`docs/graph/`, `.product/sessions/`). The mix at the root
   makes `.gitignore` lines harder to reason about than a single
   `.product/cache/` line would be.
3. **Tooling friction.** Editors, link checkers, and search tools
   see product-managed markdown intermixed with project markdown.
   Linters that scan `docs/` for guides also pick up FT/ADR/TC
   front-matter and complain. A clear boundary between
   "product-owned content" and "project-owned content" would
   eliminate this.

The directory `.product/` already exists in the codebase as a home
for runtime state (sessions, optional log location). This ADR
promotes `.product/` from an ad-hoc holding pen to the canonical
home for **all** product CLI content.

---

**Decision:**

`.product/` is the canonical location for every file and directory
that Product reads or writes on behalf of a repository. The
default repository layout becomes:

```
<repo-root>/
  .product/
    config.toml              # was product.toml
    features/                # was docs/features/
    adrs/                    # was docs/adrs/
    tests/                   # was docs/tests/
    dependencies/            # was docs/dependencies/
    graph/                   # was docs/graph/ (gitignored, generated)
    prompts/                 # was benchmarks/prompts/
    checklist.md             # was docs/checklist.md
    gaps.json                # was gaps.json (root)
    requests.jsonl           # was requests.jsonl (root)
    request-log.jsonl        # already in .product/, unchanged
    sessions/                # already in .product/, unchanged (gitignored)
  AGENTS.md                  # external convention — stays at root
  .mcp.json                  # external convention — stays at root
  CLAUDE.md                  # external convention — stays at root
  docs/                      # project's own user-facing docs (not product-owned)
    guide/                   # Diátaxis guides — not part of the graph
```

Five rules govern this decision:

1. **Rule of containment.** Every default file path Product writes
   lives under `.product/`. A user listing `.product/` sees the
   complete set of product-managed content for the repo. Anything
   outside `.product/` is project-owned, even if Product reads it
   (e.g. source files referenced via `[adr.source-files]`).

2. **Rule of external conventions.** Files whose location is
   dictated by tools other than Product remain at the repo root.
   These are: `AGENTS.md` (multi-tool agent context convention),
   `.mcp.json` (Claude Code's MCP discovery file), and `CLAUDE.md`
   (project-level Claude Code instructions). Their location is
   configurable in `product.toml` if a team wants to relocate them
   (e.g. `[agent-context].output-file`), but the default stays at
   root because that is where the consuming tool looks.

3. **Rule of discoverability fallback.** `ProductConfig::discover`
   walks up from cwd checking, in order:
   (a) `.product/config.toml`
   (b) `.product/product.toml`  (legacy alias)
   (c) `product.toml`           (pre-`.product/` layout)
   The first match wins. Existing repositories continue to work
   without any change. New repositories use (a).

4. **Rule of explicit migration.** Pre-existing repositories are
   **not** auto-migrated. `product migrate consolidate` is the
   one-shot command that physically moves files, rewrites
   `[paths]` in the config, updates `.gitignore`, and records the
   migration as a `migrate`-type entry in the request log
   (preserving hash-chain integrity per ADR-039 decision 4).
   Running other commands on a legacy layout never silently
   relocates files.

5. **Rule of overrides.** Every default path remains overridable
   via `[paths]` in `config.toml`. A team that prefers spec docs
   under `docs/features/` (e.g. for GitHub Pages publishing or
   for keeping FT-XXX URLs short) sets the override in their
   config and the migration command honours it.

The ADR governs the default layout. Individual teams retain full
control over their actual layout via `[paths]`.

---

**Configurable boundaries:**

`[paths]` defaults updated:

```toml
[paths]
features      = ".product/features"
adrs          = ".product/adrs"
tests         = ".product/tests"
dependencies  = ".product/dependencies"
graph         = ".product/graph"
checklist     = ".product/checklist.md"
requests      = ".product/requests.jsonl"
prompts       = ".product/prompts"      # NEW key — was hardcoded `benchmarks/prompts`
gaps          = ".product/gaps.json"    # NEW key — was hardcoded `gaps.json` at root
```

`[agent-context].output-file` default unchanged (`AGENTS.md` at
root) — see Rule 2.

---

**Rationale:**

- **Rule of containment.** A single root for all product state is
  the same idea as `.git/`, `.cargo/`, `target/`, or
  `node_modules/`. The convention is universally understood:
  a `.foo/` directory at the repo root means "tool foo's working
  area." Promoting `.product/` to that role makes the boundary
  obvious without documentation.
- **Rule of external conventions.** `AGENTS.md`, `.mcp.json`, and
  `CLAUDE.md` aren't really product files — Product writes
  AGENTS.md but the file is consumed by any agent that reads
  AGENTS.md, and the format is shared across tools. Forcing them
  into `.product/` would break their consumers without giving
  Product anything in return. A clean boundary respects external
  conventions.
- **Rule of discoverability fallback.** Discovery walking up the
  directory tree is the central UX feature that lets `product`
  commands work from anywhere in the repo. The fallback chain
  preserves that for legacy repos for as long as they keep
  `product.toml` at root, with no perceptible cost (one or two
  extra `Path::exists` calls per discovery — both far below disk
  I/O for the rest of `discover`).
- **Rule of explicit migration.** Auto-migration is dangerous —
  moving files mid-command can produce a half-migrated state if
  the process is interrupted, and existing CI pipelines may have
  hard-coded paths. A dedicated `product migrate consolidate`
  command makes the migration intentional, scriptable, and
  auditable (it appears as a `migrate` entry in the request log
  with its own hash-chained provenance).
- **Rule of overrides.** Spec docs under `docs/features/` is a
  legitimate preference — public ADR URLs, GitHub Pages
  publishing, integration with documentation generators, or
  simply a team's existing muscle memory. The default changes,
  but `[paths]` lets any team keep the legacy layout indefinitely
  with one config edit.

Container-level boundaries (`.product/` for tool state, `docs/`
for project content) match how the rest of the ecosystem already
works (Cargo, npm, Git, Hugo, mdBook). Spec docs fall on the
"tool-managed" side of that line because Product owns their
schema, lifecycle, and validation — they are not free-form
project documentation, even though they happen to be markdown.

---

**Rejected alternatives:**

- **Move only "internal" state to `.product/`, keep spec docs at
  `docs/features/`.** This is the conservative choice and
  preserves the established URL shape for FT/ADR pages. Rejected
  as the **default** because it splits product-owned content
  across two locations and re-creates the original problem in
  miniature. Teams who want this layout get it with a one-line
  `[paths]` override; the **default** should reflect the
  containment principle.
- **Rename `product.toml` to `.product/product.toml` (no shorter
  filename).** Reads awkwardly (`.product/product.toml` repeats
  "product"). Rejected in favour of `.product/config.toml`. The
  `product.toml` filename is kept as a legacy alias inside
  `.product/` and at the repo root for back-compat.
- **Auto-migrate on first run after upgrade.** Tempting but
  unsafe — silent file moves can break CI, hooks, and
  contributors' working trees. The opt-in `product migrate
  consolidate` command is safer and more transparent.
- **Symlink-based migration** (`docs/features/` becomes a
  symlink to `.product/features/`). Rejected — symlinks are
  fragile across platforms (Windows file-share semantics, git
  handling, archive tooling) and the team's PR review tooling
  often follows the symlink, defeating the consolidation goal.
- **Top-level rename to `.product-cli/` instead of `.product/`.**
  Rejected — `.product/` is already in use, and the binary is
  named `product` (single word). Renaming would break existing
  installations that already have `.product/sessions/` in their
  repos.
- **Use `.config/product/` (XDG-style).** Rejected — XDG is for
  per-user state in `$HOME`, not per-repo state. Putting repo
  state in `~/.config/product/` would lose the per-repo isolation
  that is the whole point of having `product.toml` at the repo
  root.

---

**Test coverage:**

- TC tagged `tc-migrate-consolidate` (scenario):
  `product migrate consolidate --dry-run` and apply, on a repo
  using the legacy layout, produces the canonical `.product/`
  layout, rewrites `[paths]` accordingly, appends
  `.product/graph/` and `.product/sessions/` to `.gitignore`,
  and emits a `migrate` entry in the request log.
- TC tagged `tc-discovery-fallback` (scenario):
  `ProductConfig::discover` resolves a legacy `product.toml` at
  root when `.product/config.toml` is absent, and prefers
  `.product/config.toml` when both exist.
- TC tagged `tc-consolidate-exit-criteria` (exit-criteria):
  consolidates the build/test/lint and regression invariants for
  FT-057.
