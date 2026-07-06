---
name: feature-e2e
description: Run an end-to-end test of a Product CLI feature in a fresh tmp directory. Build the release binary, init a clean .product/ project, then exercise every CLI command the feature adds — including the existing commands the feature touches — against the new repo. Use when the user asks to "do an end-to-end test of FT-XXX", "verify FT-XXX works in a fresh repo", "test FT-XXX end to end", or "shake out FT-XXX from scratch".
---

# Feature End-to-End Test

This skill runs a real-world end-to-end shake-out of a Product CLI feature against a fresh, isolated repo, **driven entirely through the CLI**. The goal is to confirm that a feature works for a brand-new user — not just that the unit tests pass.

The CLI is the canonical user-facing surface. The MCP server is a transport over the same logic. If the feature has an MCP-only surface (e.g. it adds an MCP tool with no CLI equivalent), test that too — but only after the CLI surface is fully exercised.

## When to use

- The user asks to do a "thorough end-to-end test" of a specific feature ID
- A feature has just been implemented and needs human-style validation
- You suspect drift between PRD/ADR text and the actual binary behaviour

## What to do

### 1. Read the feature spec first

Open `docs/features/FT-XXX-*.md`. Extract:

- The **acceptance criteria** list — these become your test plan
- The **CLI surface** — every subcommand and flag the feature adds, plus existing commands whose behaviour the feature changes
- The **error codes** — each `EXX` has a path that should fire
- Any **config keys** the feature reads (`product.toml` sections)
- Any **resolution paths** the feature uses (e.g. repo → user → built-in)

### 2. Use the existing release binary

```bash
PRODUCT=/abs/path/to/repo/target/release/product
[ -x "$PRODUCT" ] || cargo build --release
```

Don't rebuild if a release binary already exists — the bottleneck is the test work, not compilation.

### 3. Create a clean tmp scratch directory

```bash
rm -rf /tmp/<feature-slug>-e2e-test
mkdir -p /tmp/<feature-slug>-e2e-test
cd /tmp/<feature-slug>-e2e-test
```

Always start clean — the resolution order for templates / config / `.product/` directory walks up the filesystem, so a stale parent state can leak into the test.

### 4. Initialise a fresh project via the CLI

```bash
$PRODUCT init -y --name ft-xxx-e2e-test
```

Defaults to the canonical `.product/` layout (ADR-048). Pass `--legacy-layout` only if the feature specifically needs the pre-FT-057 root-based layout.

Verify init landed correctly:

```bash
ls -la .product/                       # config.toml + products/<name>/ (the product home)
$PRODUCT --help | head -40             # confirms binary loads against the fresh repo
```

### 5. Seed the graph via the CLI

A typical e2e run needs at least one feature, one ADR, and one TC, linked together:

```bash
$PRODUCT feature new "Hello World"
$PRODUCT adr new "Use TOML for config"
$PRODUCT test new "feature renders"
$PRODUCT feature link FT-001 --adr ADR-001 --yes
$PRODUCT feature link FT-001 --test TC-001 --yes
```

After seeding, sanity-check via the CLI's read commands:

```bash
$PRODUCT feature list
$PRODUCT feature show FT-001
$PRODUCT adr list
$PRODUCT test show TC-001
$PRODUCT graph check
```

### 6. Exercise every CLI command the feature touches

This is the core of the skill. Walk every acceptance criterion **and** every CLI command listed in the feature spec. Capture stdout, stderr, and exit code separately for each invocation:

```bash
$PRODUCT <command> [args] > /tmp/out.txt 2> /tmp/err.txt
echo "exit=$? stdout=$(wc -c </tmp/out.txt) stderr=$(wc -c </tmp/err.txt)"
head /tmp/out.txt
[ -s /tmp/err.txt ] && cat /tmp/err.txt
```

Many features emit warnings on stderr while keeping stdout pipe-friendly — always capture stderr separately or you will miss the deprecation notes / drift signals.

For each command, exercise:

- **Happy path** — the canonical invocation from the spec
- **Every flag** — including JSON output (`--format json`), depth/filter flags, and any feature-specific knobs
- **Every subcommand variant** — e.g. `feature link --adr`, `feature link --test`, `feature link --dep`
- **Read-after-write** — when a command modifies state, run a read command to confirm the change
- **Idempotence** — if a write command claims to be idempotent, run it twice
- **JSON output structure** — when `--format json` is supported, parse the output with `python3 -c "import json,sys; json.load(open('/tmp/out.txt'))"` to confirm it's well-formed

For each rendering target the feature exposes, validate the output is structurally well-formed:

```bash
python3 -c "import json; json.load(open('/tmp/out.txt')); print('JSON ok')"
python3 -c "import yaml; yaml.safe_load(open('/tmp/out.txt')); print('YAML ok')"
python3 -c "import xml.etree.ElementTree as ET; ET.parse('/tmp/out.txt'); print('XML ok')"
```

For every error code the spec promises, intentionally trigger it and confirm the exit code, stderr message, and hint text:

```bash
$PRODUCT <bad invocation>; echo "exit=$?"; cat /tmp/err.txt
```

Confirm exit codes follow ADR-013 (1 = error, 2 = warning).

### 7. Test repo-local / user-level resolution (when applicable)

Many features resolve files in repo → user → built-in order (templates, prompts, config overlays). Test all three by:

- Drop a file under `.product/<thing>/` and confirm it shadows the built-in
- Drop the same name under `~/.product/<thing>/` and confirm the repo file wins
- Remove the repo override and confirm the user file then wins

### 8. Test the MCP surface only when the feature is MCP-specific

The MCP server is a transport over the same logic — it's exercised by every CLI handler under the hood. Only test it directly when:

- The feature **adds** an MCP tool (e.g. FT-021, FT-046, FT-050, FT-059, FT-062)
- The feature **changes** an MCP tool's input schema or output envelope
- The feature is about MCP parity, write-tool gating, or transport behaviour

When you do, the JSON-RPC sequence is `initialize` → `notifications/initialized` → `tools/call`. The response is double-wrapped — parse twice:

```bash
{
  printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"e2e","version":"0"}}}'
  printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  printf '%s\n' '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"<tool>","arguments":{...}}}'
  sleep 0.3
} | $PRODUCT mcp 2>/tmp/mcp.err
```

Init with `--write-tools` if you need to call a write tool, and use `python3 -c "import json,sys; ..."` to walk both wrapper layers.

### 9. Run the health gates against the seeded repo

After exercising the feature's commands, run the standard health gates one more time to confirm no command corrupted the graph:

```bash
$PRODUCT graph check
$PRODUCT gap check
$PRODUCT drift check
$PRODUCT preflight        # if defined for this repo
```

### 10. Report back

For every acceptance criterion in the spec, mark pass/fail. Call out any drift between spec and behaviour — these are the highest-value findings of an e2e run.

## Gotchas captured from prior runs

- **Subcommand naming.** The CLI is `product test new`, not `product tc new`. The internal IDs use `TC-XXX` but the subcommand family is `test`.
- **`feature link` in non-TTY needs `--yes`.** Without it, the inferred-transitive-TC prompt blocks waiting on stdin.
- **`product init` is not idempotent without `--force`.** Re-running over an existing config errors out — clean the tmp dir first.
- **Stderr deprecation notes are easy to miss.** Several flags (`--for-llm`) emit deprecation notes to stderr while still producing valid stdout. Always redirect stderr to a separate file.
- **Exit code semantics.** Per ADR-013: 0 = success, 1 = hard error, 2 = warning-only state. Some commands (`gap check`, `dep check`, `preflight`) deliberately exit 2 — that is not a test failure.
- **`--format json` is supported widely but not universally.** Check the help output before parsing JSON; some commands only emit text.
- **No-flag default vs `--target human`.** Per FT-063 these should produce byte-identical Markdown. If they don't, that is a drift to flag.
- **Built-in templates ship embedded.** They live at `src/context/template/builtin/*.toml` but are pulled in via `include_str!` — there is no `templates/` directory at the repo root or anywhere on disk in a fresh install.
- **Resolution paths in `templates --where`.** Built-ins show as `(built-in)` (no path); repo/user templates show their absolute path.
- **MCP tool input arg name is `id`** (not `feature_id`). PRD examples may show `feature_id` — that is doc drift; the binary reads `id`.
- **MCP envelope is double-wrapped.** Tool results come as `{"result":{"content":[{"text":"<inner json>","type":"text"}]}}`. You must `json.loads()` twice when validating tool output.

## Skill output

End each run with:

1. A pass/fail table covering every acceptance criterion in the spec
2. A per-command results table — every CLI command exercised, exit code, brief notes
3. A drift list — every place spec says X but the binary does Y (cite file paths, exact strings)
4. The path to the scratch directory so the user can poke around (e.g. `/tmp/ft063-e2e-test/`)
