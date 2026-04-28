# Product Identification Specification

> Standalone reference for `product identify` — the cheap "what am I working on?" tool.
>
> Amendment to ADR-031 (AGENT.md and agent context tools).
> New CLI command: `product identify`
> New MCP tool: `product_identify`
> Updated author prompt: call identify as the first step.

---

## The Problem

When Claude on a phone connects to a Product MCP server, the server already
knows which repository it serves — it was started in that repo's directory.
But the agent inside the conversation does not know.

The agent connects to a URL, not to a labelled product. Two failure modes
follow:

1. **Wrong-Project mistakes.** Three Claude Projects (PiCloud, Signal, your
   C# project) connecting to three MCP servers. Open the wrong one, start
   describing what you want — the agent reads the graph and produces results
   that don't match your expectations because you're talking to the wrong
   server.

2. **Generic context until the agent investigates.** The author prompt is
   identical across all products. The first thing Claude must do is call
   tools to figure out where it is — graph_check, feature_list, dep_bom,
   etc. — before it can reason about your request. That's expensive both
   in tokens and in latency.

Both are solved by a single cheap call that returns the product's identity
in a few hundred bytes.

---

## `product identify` — CLI

```
product identify

  Name:           PiCloud
  Responsibility: Distributed private cloud platform for self-hosted
                  Raspberry Pi clusters
  Repository:     /home/emil/repos/picloud

  Current state:
    Phase:        2 — Products and IAM (LOCKED)
    Features:     47  (12 complete · 6 in-progress · 29 planned)
    ADRs:         31  (28 accepted · 3 proposed)
    Dependencies: 23
    Tests:        142 (88 passing · 5 failing · 49 unimplemented)

  Last apply:     2026-04-14T09:14:22Z (req-20260414-007)
```

### Field selectors

```bash
product identify --field name              # PiCloud
product identify --field responsibility    # full responsibility line
product identify --field repo              # absolute repo path
product identify --field current-phase     # 2
product identify --field json              # full output as JSON
```

The `--field` flag prints only the requested value to stdout, no labels,
no formatting. Useful in scripts.

```bash
PRODUCT=$(product identify --field name)
echo "Working on $PRODUCT"
```

### JSON output

```bash
product identify --format json
```

```json
{
  "name": "PiCloud",
  "responsibility": "Distributed private cloud platform for self-hosted Raspberry Pi clusters",
  "repo_root": "/home/emil/repos/picloud",
  "phases": [
    { "number": 1, "name": "Cluster Foundation", "status": "OPEN" },
    { "number": 2, "name": "Products and IAM",   "status": "LOCKED" },
    { "number": 3, "name": "RDF and Event Store", "status": "LOCKED" }
  ],
  "current_phase": 2,
  "counts": {
    "features": { "total": 47, "complete": 12, "in_progress": 6, "planned": 29 },
    "adrs":     { "total": 31, "accepted": 28, "proposed": 3 },
    "dependencies": 23,
    "tests":    { "total": 142, "passing": 88, "failing": 5, "unimplemented": 49 }
  },
  "last_apply": {
    "id": "req-20260414-007",
    "timestamp": "2026-04-14T09:14:22Z",
    "type": "create"
  }
}
```

The output is small — typically under 1KB. Computing it is fast (the data
is already in front-matter and the request log).

---

## `product_identify` — MCP Tool

```json
// Input — no parameters required
{}

// Output — same JSON as `product identify --format json`
{
  "name": "PiCloud",
  "responsibility": "...",
  "repo_root": "...",
  "phases": [...],
  "current_phase": 2,
  "counts": {...},
  "last_apply": {...}
}
```

Always available, no authentication scope required beyond what the connection
already grants.

The tool name is intentionally identify rather than `product_about` or
`product_self`: the agent is asking "identify yourself" of the server, in
the sense of "tell me which product you serve." This matches mental models
from network protocols (a server identifying itself in a handshake) rather
than reflective introspection.

---

## Author Prompt — Required First Step

The single `author-v1.md` prompt is updated to call `product_identify` as
the first action of every session:

```markdown
# Product Author Prompt v1

You are authoring specifications for a Product-managed repository.

## Mandatory first step

Call `product_identify` first. The response tells you which product you are
working on. Read its `name` and `responsibility` carefully — your output
must be consistent with this product's stated purpose.

If the user's request is not consistent with the product's responsibility,
flag this immediately. Do not proceed with authoring until the user confirms
they intended to work on this product.

## Then perform the read phase

After identification:
1. Call product_graph_check       — current graph health
2. Call product_graph_central     — top-5 foundational ADRs
3. Call product_feature_list      — what features exist
4. Call product_dep_bom           — what dependencies exist
5. Call product_agent_context     — full working protocol and schemas

## Output

Produce a Product request YAML. Validate with product_request_validate.
Apply with product_request_apply.
```

The change to the prompt is one section. The behavioural change is that
every session now opens with explicit confirmation of what it's working on,
and the agent has the chance to flag a mismatch before producing wrong-shaped
output.

---

## Wrong-Project Detection

When the user starts a conversation in the wrong Project, the flow becomes:

```
Phone (PiCloud Authoring Project, but accidentally connected to Signal MCP):

User:  Add a feature for raft leader timeout configuration

Claude: [calls product_identify]
        [reads: name=Signal, responsibility=Content domain registry]

Claude: Before I proceed — this server is serving Signal (Content domain
        registry), not PiCloud. The request you've made is about distributed
        consensus, which fits PiCloud's responsibility, not Signal's.

        Did you intend to connect to the PiCloud server? If so, please
        switch Projects and try again.

        Or did you intend to add a Signal feature? If so, please rephrase
        the request in terms of Signal's content domain.
```

The mismatch is caught in the first turn. No graph mutations happen. No
wasted token budget on producing the wrong-shaped output.

---

## Source of the Identification

The data in the response comes from three sources, all already present:

**`product.toml` `[product]` section** — name, responsibility, phases.
This information already exists.

**Graph state** — feature/ADR/dep/test counts. Computed from the in-memory
graph the MCP server already builds.

**Request log (`requests.jsonl`)** — last apply entry. Reads the last line
of the log file.

No new data, no new storage. `product identify` is a query over data that
all already exists. Implementation is a few dozen lines.

---

## When to Call

The author prompt mandates calling `product_identify` first in every session.
For other tools and harnesses:

**Always call at session start:** authoring sessions, implementation sessions,
review sessions. Cheap and confirms which product is in scope.

**Call again only on suspicion:** if a tool result seems inconsistent with
expectations, identify can confirm whether you're talking to the right server.

**Skip in scripted CI:** scripts that already know which repo they're in
don't need identify — they're not navigating uncertainty.

---

## Session Tests

```
ST-380  identify-returns-product-name-from-toml
ST-381  identify-returns-responsibility
ST-382  identify-counts-match-graph-state
ST-383  identify-current-phase-from-gate-state
ST-384  identify-last-apply-from-request-log
ST-385  identify-field-flag-prints-only-value
ST-386  identify-json-format-valid-schema
ST-387  identify-fast-under-100ms
ST-388  mcp-identify-tool-no-auth-required
ST-389  mcp-identify-output-matches-cli-json
ST-390  author-prompt-mentions-identify-as-first-step
```

---

## Invariants

- `product identify` is a pure read operation. No writes, no side effects.
- The output is bounded — typically under 1KB, never more than a few KB.
  Designed to be cheap enough to call as the first action of every session.
- Latency budget: under 100ms on a graph with 1000 features. The data is
  already in memory or trivially computable; identify is not allowed to be
  slow.
- The MCP tool requires no special auth scope beyond what the connection
  already provides. There is no `mcp.identify` gate — identify must be
  available wherever the MCP server is reachable.
- The CLI and MCP outputs are byte-identical when both are requested in
  JSON format. There is no divergence between the two surfaces.
