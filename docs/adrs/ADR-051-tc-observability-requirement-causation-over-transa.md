---
id: ADR-051
title: TC Observability Requirement — Causation Over Transaction
status: accepted
features:
- FT-070
- FT-071
- FT-072
- FT-073
- FT-074
- FT-075
supersedes: []
superseded-by: []
domains:
- data-model
- testing
scope: cross-cutting
content-hash: sha256:6fc8ce7c8ee24050183706e614b1825c03ada00b54aa5b312bf8948b7df530f9
---

**Status:** Proposed

**Context:** The FT-046 → FT-066 series is the canonical case study.
FT-046 introduced MCP parity for ADR lifecycle operations. The handler
was a no-op stub that returned `{ id, status, note: "Use CLI for status
updates with full side-effects" }` without ever touching disk. Every
scenario TC for FT-046 asserted on the success envelope (`status: ok`,
`status: "complete"`, `note: ...`) and every one of them passed,
because the envelope was correct. The TCs verified that the *response
transaction* was well-formed. They did not verify that the *underlying
causation* — a status field on disk changing from one value to another
— actually happened.

FT-066 had to revisit the same surface a year later. The fix path was
straightforward (route the MCP handler through the existing slice).
The diagnosis was the hard part: how did a feature ship with every TC
green and the contract fundamentally broken? The TC-778 family
(authored as part of FT-066) shows the answer. Each TC composed a
temp repo, invoked the MCP tool, and **asserted on the file**:
"after this call, `docs/features/FT-X.md` contains `status: complete`
in its front-matter". That is the shape the original FT-046 TCs
should have had. The shift is from "the response said OK" to "the
world changed in the way the response claimed".

This is not a one-off lesson. The shape repeats wherever a TC verifies
a side-effectful operation by inspecting only the response. It is
also under-served by prompt-only guidance — implementing agents
genuinely do not know which surface to assert on unless the
specification names one. A free-form "make the test good" instruction
in the implement prompt does not survive contact with a feature spec
that hands the agent a return-type and an example assertion in the
same paragraph.

The fix is to make the observed surface a structural field on the TC
itself, validated by `product graph check`, with the failure mode
being the obvious one: a TC of the relevant type that omits or empties
the field is rejected before it ever runs.

Three alternatives were considered.

1. **Prompt-only guidance.** Add a "TCs must observe causation, not
   transaction" paragraph to the authoring prompt and the implement
   prompt. Rejected because the FT-046 episode shows that guidance
   without structure drifts. The same paragraph existed informally in
   reviewer feedback for months; it did not prevent FT-066.

2. **Mutation testing as a verify-time backstop.** Run `cargo-mutants`
   or similar on each TC; require the TC fails when the target code is
   mutated to a no-op. This is the strongest possible check —
   mathematically equivalent to "the TC verifies causation". Rejected
   for Phase 1 on grounds of cost (mutation testing is minutes per
   feature, not milliseconds), tooling maturity (cargo-mutants needs
   tuning per project), and false-positive surface (mutations that
   leave behaviour observably equivalent generate noise the TC
   reviewer must triage). Worth doing as a separate, deferred
   backstop; not the right shape for the front-line gate.

3. **Whole-system property tests as the primary check.** Define
   invariants like "every successful MCP write touches at least one
   file" and rely on property tests to catch the FT-046 class of bug.
   Rejected because property tests cover the *system* contract, not
   the *per-feature* contract. A property test cannot tell you that a
   specific feature's MCP handler is a stub; it can only tell you that
   *some* MCP handler somewhere is misbehaving when the test surfaces
   a failure. The diagnostic signal is too coarse for the use case.

---

**Decision:** Every TC of a relevant type carries a non-empty
`observes:` front-matter field that names the surface(s) the test
asserts against. The field is validated structurally by
`product graph check`. A TC body that contains no reference (free-text
or assertion-shape) to its declared surfaces emits a warning. The
validation is intentionally cheap and local — it does not run the
test, does not mutate code, does not invoke an LLM. It is the
"presence of structural intent" check that prevents the FT-046 shape
of failure.

### Front-matter schema addition

```yaml
observes: [graph, file]            # non-empty list of surface names
```

Allowed surface values (extensible via `product.toml` per
`[tc-observability].custom`, mirroring `[tc-types].custom` from
ADR-042):

| Value | Meaning |
|---|---|
| `file` | Asserts on the contents of a file on disk after the action. |
| `graph` | Asserts on the loaded `KnowledgeGraph` after the action (e.g. a TC's `validates.features` membership). |
| `exit-code` | Asserts on the process exit code from a CLI invocation. |
| `tag` | Asserts on the presence/absence of a git tag (ADR-036 family). |
| `stdout` | Asserts on captured stdout text (CLI scenarios). |
| `stderr` | Asserts on captured stderr text. |
| `disk-state` | Asserts on broader disk state than a single file (directory contents, byte-equality across files). |
| `mcp-response` | Asserts on the MCP JSON-RPC response envelope. |

`mcp-response` is allowed but **never alone** for tests of MCP write
tools — the FT-046 lesson is that the response is necessary but not
sufficient. A TC validating an MCP write whose `observes:` list is
exactly `[mcp-response]` is the structural anti-pattern this ADR
exists to prevent; a follow-on validator should flag it once the
field-level checks are in place (tracked as a fitness function in the
implementing feature, not as part of this ADR).

### Required-for table

| TC type | `observes:` required | Rationale |
|---|---|---|
| `scenario` | yes | Scenarios verify a single behaviour; the surface must be named. |
| `session` | yes | Session tests cover MCP / CLI end-to-end; the surface is exactly the kind of thing that goes missing. |
| `smoke` | yes | Smoke tests are the broadest user-visible check; ambiguity here defeats the type. |
| `contract` | yes | Contract tests assert on shape — the shape is the surface. |
| `invariant` | optional | The quantified property *is* the observation. |
| `property` | optional | As above. |
| `chaos` | optional | Failure-injection tests assert on system-wide behaviour, not a named surface. |
| `exit-criteria` | not applicable | Aggregators only reference other TCs; they do not directly observe. |

### Validation rules in `product graph check`

A new diagnostic code (allocated by the implementing feature, F3 in
the brief) covers:

1. **Hard error.** A TC of a required-for type with missing or empty
   `observes:` field. The error references this ADR. No exit
   ambiguity.

2. **Warning.** A TC body whose text contains no reference to any
   declared surface (regex check against the surface name and a small
   synonym set). Intentionally low-confidence — the goal is to nudge
   the author toward an explicit assertion, not to police prose. The
   warning is suppressible only by adding the missing reference.

3. **Warning (deferred to the follow-on fitness check noted above).**
   A scenario / session TC of an MCP write tool with `observes:` =
   `[mcp-response]` and nothing else.

The grammar of `observes:` is intentionally flat (strings, not
objects). A richer structured form (`[{kind: file, path: "..."}]`)
would enable stronger validation but cannot be retrofitted onto the
existing TC corpus without migration work. Start flat; promote to
structured later only if the soft warnings prove insufficient.

### Migration

Existing TCs are grandfathered. The check applies from a configurable
phase (default phase 5, matching the FT-066 era when the lesson was
learned), set via `[tc-observability].required-from-phase` in
`product.toml`. Earlier phases continue to validate but do not require
the field. New TCs authored after this ADR lands carry the field from
day one.

---

⟦Γ:Invariants⟧{
  every_scenario_session_smoke_or_contract_tc_authored_after_this_decision_carries_a_non_empty_observes_list
  every_value_in_a_tc_observes_list_is_one_of_the_eight_allowed_surfaces_or_is_declared_in_tc_observability_custom
  no_tc_body_passes_the_check_while_failing_to_reference_any_declared_surface_warning_emitted
  the_observes_field_is_not_required_for_invariant_property_chaos_or_exit_criteria_tcs
  the_validation_runs_in_product_graph_check_without_executing_the_tc_or_invoking_an_llm
  a_tc_omitting_observes_when_required_blocks_request_apply_for_that_tc
  the_field_grammar_is_a_flat_list_of_strings_in_phase_one_no_object_form
  existing_tcs_authored_before_the_required_from_phase_threshold_are_grandfathered
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

---

**Rationale:**

- **The lesson must be structural to survive.** FT-046 had the right
  intent in oral tradition: the TC author knew the response wasn't
  enough. The reviewer knew. The agent did not. Putting the intent in
  prompts is one revision away from being ignored; putting it in
  front-matter and validating it on every `graph check` makes it part
  of the spec the agent must satisfy to ship.

- **Cheap to validate is the right severity level for a structural
  check.** Mutation testing is the perfect check for this class of
  bug; it is also expensive, slow, and noisy. Splitting the work —
  structural presence here, deeper verification deferred to a separate
  feature — keeps every layer at the right granularity. The structural
  check catches the FT-046 shape (empty observation surface); the
  deferred mutation check would catch the residual cases the
  structural check cannot see (declared surface, but the assertion is
  vacuous).

- **Per-feature signal beats whole-system signal.** Property tests
  catch families of bugs; per-TC structural fields catch the specific
  bug in this specific feature. The diagnostic burden of a property
  failure is "find the affected feature in a corpus of 70"; the
  diagnostic burden of an `observes:` failure is "the TC you just
  authored is missing one line".

- **`mcp-response`-alone is the named anti-pattern.** Listing it as an
  allowed value but flagging the alone-case in a follow-on fitness
  function captures the exact shape of FT-046 / FT-066 without
  outlawing the response as evidence (which is sometimes legitimate
  — e.g. error-envelope-shape tests).

- **Flat strings now, objects later.** The cost of a flat list is
  zero migration. The cost of a structured list is per-TC author
  attention forever, on the bet that the structured form catches
  enough additional cases to justify the bookkeeping. Start where the
  cost is zero; promote only if the warnings prove too soft.

**Rejected alternatives:**

- **Prompt-only guidance** — rejected above (drift).
- **Mutation testing as the primary check** — rejected above (cost,
  tooling, false positives); deferred to a separate feature as a
  backstop.
- **Whole-system property tests as the primary check** — rejected
  above (diagnostic granularity is wrong).
- **Make `observes:` mandatory on every TC type including
  exit-criteria.** Rejected because exit-criteria aggregators do not
  themselves observe anything; the assertion lives in the constituent
  TCs. Forcing the field there would be ceremonial.
- **Structured `observes:` from day one (`[{kind: file, path:
  "...glob..."}]`).** Rejected above as premature. Promote later if
  the soft warning proves insufficient.
- **Hard-fail on `observes: [mcp-response]` alone immediately.**
  Rejected because some TCs legitimately observe only the response
  (error-envelope-shape, ping/echo). Delegating the alone-detection
  to a follow-on fitness function lets the rule land without breaking
  legitimate uses, while still committing to the heuristic.

**Test coverage:** Validated by F3 in the implementing feature
cluster. Scenario TCs cover: rejection of an `observes:`-missing TC by
`graph check`; warning emission for a TC body lacking any declared
surface reference; correct grandfathering of pre-required-from-phase
TCs. The TCs validating this ADR themselves declare their own
`observes:` surfaces — the implementing feature dogfoods its own
contract.
