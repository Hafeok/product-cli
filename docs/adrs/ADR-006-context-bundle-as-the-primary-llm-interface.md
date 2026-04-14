---
id: ADR-006
title: Context Bundle as the Primary LLM Interface
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:19eb8d491a6f34877f83eb33b34755b47cf7a9c41fa58254458613c600ac3747
---

**Status:** Accepted

**Context:** The primary use case for Product is to give LLM agents precisely the context they need for implementation tasks. The question is: what is the right unit of context, and what format should it take?

A naive approach is to dump the entire repository into the LLM context. This fails at scale: a project with 40 features, 30 ADRs, and 80 test criteria produces a context document of 200,000+ tokens — past the practical window of most models and past the point where signal-to-noise is useful.

**Decision:** The context bundle — a feature, its linked ADRs, and its linked test criteria — is the primary output of Product and the primary input to LLM agents. Bundles are assembled deterministically and formatted as markdown. The context command is a first-class citizen of the CLI, not an afterthought.

**Rationale:**
- A single feature with its linked ADRs and test criteria typically produces 3,000–8,000 tokens — well within any current LLM's practical working window
- The relational structure means nothing relevant is omitted (every ADR that applies is included) and nothing irrelevant is included (ADRs for unrelated features are excluded)
- Deterministic assembly order means two invocations of `product context FT-001` produce identical output — cacheable, auditable, reproducible
- The header block (feature ID, phase, status, linked artifact IDs) is machine-parseable by the receiving agent without requiring it to read the full bundle
- Superseded ADRs are included with a `[SUPERSEDED by ADR-XXX]` annotation — the agent has the full decision history, not just the current state

**Rejected alternatives:**
- **Full repository dump** — complete context, no relevance filtering. Rejected because 200K tokens of mixed context produces demonstrably worse agent outputs than 5K tokens of targeted context. Empirically validated.
- **Feature file only** — minimal context. Rejected because the agent needs the rationale (ADRs) and the success criteria (tests) to implement correctly. A feature description without its decisions is ambiguous.
- **Streaming / agentic retrieval** — the agent calls Product as a tool to fetch context as needed. Possible, but requires the agent to be running in a tool-use environment. The bundle approach works in any context window — a terminal paste, a system prompt, a file attachment.
- **Token budget flag (`--max-tokens`)** — considered adding truncation logic to `product context` to fit a target context window. Rejected: token budget management is the agent's responsibility. Product's job is to assemble a complete and accurate bundle. Truncation decisions require knowledge of the model, the task, and the surrounding prompt — none of which Product has. An agent that needs to fit a window should request a narrower scope (single feature, ADRs-only) rather than rely on Product to guess what to drop.

**Supersession behaviour:** When a context bundle is assembled, superseded ADRs are replaced by their successors. The superseded ADR does not appear in the bundle. This keeps the bundle actionable — an agent receiving it sees only the current, accepted set of decisions. The supersession chain is recorded in the ADR's own front-matter and is queryable via `product adr show`, but it does not propagate into context bundles.