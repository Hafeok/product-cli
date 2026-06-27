---
name: product-session-e2e
description: >
  Shake out a full What→How→Build session end-to-end in a fresh `init --demo`
  repo, driving each phase skill's worked example through the CLI and asserting
  every phase gate. Use when the user asks to "run the session e2e", "verify the
  phase skills", "shake out a session end to end", or after changing the
  domain/how/build tool surface to confirm the skills still hold.
---

# Product Session — end-to-end shake-out

Confirms the **product-what / product-how / product-build** skills still work as a
set against a brand-new repo, driven entirely through the CLI (the canonical
surface; the MCP tools are a transport over the same logic). Run it whenever the
tool surface changes.

## Run it

```bash
bash scripts/checks/session-e2e.sh        # uses target/debug/product
PRODUCT=/abs/path/to/product bash scripts/checks/session-e2e.sh   # pin a binary
```

The script is self-contained: it makes a fresh tmp repo, runs the sequence below,
prints `PASS`/`FAIL` per step, and exits non-zero if any step or assertion fails.

## What it drives (each phase skill's worked example)

1. **init** — `product init --demo --yes --name bookstore` (seeds the What).
2. **product-what** — `domain validate` → `domain new value-object` → re-validate.
3. **product-how** — `how init` → `how add decision|principle|pattern`
   (the principle is `--enforced-by` so it earns its place, §4.1) →
   `how set app-contract` → `how validate` → `archetype init` → `cell init` →
   `cell dispatch --bind entity=Order` (produces a real work unit).
4. **product-build** — `slice new` → `deliverable new` (acceptance as
   `id: statement`, §7.2) → `build --dry-run`.
5. **assertions** — the build run-plan uses the *dispatched* work unit
   (`handler-order`, not a fallback) and the gate reports domain conformance.

## When it fails

Each `FAIL` prints the offending command's last lines. Common causes:
- A renamed/removed tool or flag → update the matching phase skill **and** this
  script together (they must stay in lockstep).
- A new conformance rule (e.g. §4.1 earn-their-place) the worked example trips →
  fix the example to satisfy it, then mirror the fix in the phase skill.

## Note on session gating

This runs the CLI, which has **no phase lock**, so it exercises the underlying
capability rather than the gated ordering. Inside a real `product mcp --workflow`
session the phases gate the tools — in particular **cell dispatch / work-unit
authoring are How-phase** and freeze in Build, so a session must dispatch work
units before advancing (see **product-how** / **product-build**).
