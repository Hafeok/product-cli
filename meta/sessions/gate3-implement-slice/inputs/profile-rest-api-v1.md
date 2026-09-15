# [PROPOSED] Profile `rest-api-v1`

**What this is.** How a command slice is realised in a C#/ASP.NET Core stack. Filed as a determination addressed to the act type, per R-C — not compiled into the tool. Changing the architecture is a supersession, not a release.

**Status:** `[PROPOSED]`. Authored, not derived. There is no existing codebase it was fitted to, which is what makes authoring it legitimate here.

---

## The determination record

```yaml
- id: DSC-0100
  address:
    act_type: command
    act_instance: PlaceOrder      # anchor instance; see the note below
  statement: >-
    A command slice in this context is realised by a controller, a handler and,
    where external data is required, a provider. Profile rest-api-v1, bound by
    content hash.
  extent:
    axes:
      slice-type:
        state: travels-to
        region: all-command-slices
        reason: >-
          A pattern-level determination. Every command slice ever written in
          this context collects it.
      context:
        state: bound-here
      sector:
        state: silent
  allocation:
    class: pinned
    settled_by: "profile:rest-api-v1@<content-hash>"
  provenance:
    made_at: build-time
    made_by: "emil"
    recorded: "2026-09-14T00:00:00Z"
```

**Anchor note.** `act_instance` is required, so this pattern-level determination anchors on one instance and travels. That anchor is arbitrary and privileged — the schema gap recorded at CG-R-53. It is not repaired here.

---

## The profile body

```yaml
profile: rest-api-v1
act_type: command
stack: "C# / ASP.NET Core"

roles:
  - name: controller
    required: true
    enforcement: analyser
    must:
      - "declares [Slice(<instance>, \"controller\")]"
      - "calls exactly one type declaring the handler role for the same act instance"
      - "returns a transport result derived from the handler's Accepted or Rejected"
    must_not:
      - "contains a conditional on domain state"
      - "references a provider role directly"
      - "references a persistence type"

  - name: handler
    required: true
    enforcement: analyser
    must:
      - "declares [Slice(<instance>, \"handler\")]"
      - "exposes a single entry point taking the command and returning Accepted or Rejected"
      - "emits only events the act declares it writes"
      - "rejects only for invariants the fact vocabulary declares"
    must_not:
      - "references a transport type"
      - "performs I/O directly"

  - name: provider
    required: false
    enforcement: analyser
    must:
      - "declares [Slice(<instance>, \"provider\")]"
      - "is reached only from a handler role for the same act instance"
      - "every fact it supplies is declared in a read position on the act"
    must_not:
      - "contains a decision"
      - "references a transport type"

read_enforced:
  - "the handler's logic is the behavioural specification, not a realisation of one stated elsewhere"
  - "a rejection reason corresponds to the invariant it cites, not merely to a declared one"
  - "the provider's supplied fact means what the fact vocabulary says it means"
```

**Every rule is declared `analyser` or `read_enforced`.** A rule nobody can run is not enforcement, and which is which is the point. The three read-enforced rules are the residual — checked by a person, or not at all.

---

## What is deliberately absent

- **No Decider/Projector conformance rules.** The handler is a Decider by another name and its rules exist already. They are reused where the act vocabulary carries the edge they need, and the gap is stated where it does not (CG-R-56). A second set would drift within a release.
- **No coverage declaration.** That is D-7 against the product-framework and it is not in scope for this run.
- **No profile for read-model, automation or translation slices.** One act type, one profile, one slice.

---

## Open

1. **Not validated.** No slice has been built against it. Whether it is specific enough is the question Gate 3 exists to answer, and the answer may be that it is not.
2. **The `must` / `must_not` strings are prose.** Their analyser-enforceability is asserted, not demonstrated. PRD §11.4 stands: which subset Roslyn can actually check is undecided, and the Gate 3 run is the first evidence.
3. **Authored, not derived.** Its warrant is one person's engineering judgement about one stack. It carries no incidence, because it has no field yet.
