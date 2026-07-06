# The Developer Guide — working through the framework day to day

> **Non-normative.** This guide describes *a* way to run the Product Framework day
> to day. It is **not** part of the specification and adds no conformance rules —
> the spec is deliberately silent on practice (cadence, ceremonies, who, when;
> [spec §7.3](docs/product-framework.md#73-model-not-practice)). Everything here
> traces to something the spec already defines; this is the spec's **authoring
> order** ([§2.1](docs/product-framework.md#21-authoring-order--the-funnel-as-a-process))
> turned into the activities a person actually performs. Your team may run it
> differently and still be fully conformant.

## The shape of the day

A feature lands on your desk — usually as an informal description, a UI sketch, or
a sentence from a stakeholder. You move it through four stages:

```
  (a request arrives)
        │
   ┌────▼─────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
   │  INTAKE  │ ──▶ │   WHAT   │ ──▶ │   HOW     │ ──▶ │  BUILD   │
   │ before   │     │ meaning  │     │ realise  │     │ make it  │
   │ the What │     │          │     │          │     │  real    │
   └──────────┘     └────┬─────┘     └──────────┘     └────┬─────┘
                         ▲                                  │
                         └──────────────────────────────────┘
                    feedback: intent-reliance, data-divergence,
                    state/Decider justification send you upstream
```

Each stage has **one question** you are answering, a few **concrete steps**, and a
clear **exit criterion** — you do not move on until it is met. The constraint that
fixes this order is not invented here: each stage's artifacts *derive from* the
previous stage's, so you cannot run them in another order without authoring
something before the thing it depends on exists (spec §2.1).

The single most important habit: **hard thinking moves upstream.** Every problem
is cheaper to fix the further left you catch it. A behaviour bug found in the What
(by simulating a Decider) costs a sentence; the same bug found in Build costs a
day. The whole point of the funnel is to make the left stages carry the weight so
the right stages are determined.

---

## Stage 0 — Intake (before the What)

> **The question: *Where on the model does this live, and what is the one-sentence
> intent?***

A feature description is **not** a What. It is raw and ambiguous; the What is
precise, typed, and derived. Intake is the short translation step that turns a
request into a tracked piece of work located on the existing graph. Keep it
lightweight — this is triage, not modelling.

1. **Locate it on the graph.** A feature is *a slice of the event model*
   (spec §7.1), so the first question is always "where does this slice live?"
   Which **system** ([§3.2.5](docs/product-framework.md#325-the-system--what-is-being-described))?
   Which **bounded context** ([§3.1](docs/product-framework.md#31-the-domain-model--structure-and-data))?
   Which existing **flows** does it touch or extend?
2. **Triage new-vs-change.** Is this genuinely new behaviour, or a change to
   behaviour that already exists? A change means you start from the existing
   Deciders/flows and amend; new behaviour means a fresh slice.
3. **Extract the intent — one sentence.** State what the user is trying to
   accomplish, in the ubiquitous language. Write it down knowing it is **debt**:
   intent is the marked residue of what the model does not yet determine (§3.2.1),
   and the rest of the process is the act of discharging it into modelled elements.
   If you cannot state the intent in one sentence, the request is not yet ready —
   send it back, do not model around the ambiguity.

**Exit criterion:** you can name the system, the context, the flow(s) involved, and
a one-sentence intent. If a request is too vague to place on the graph, it is not
ready for the What.

---

## Stage 1 — What (the meaning)

> **The question: *What happens, what decides it, what does the user see — and does
> it simulate?***

This is where the weight is. You are building a precise, typed description that
every role can read and agree on, **before any code.** Run the spec's authoring
order (§2.1, steps 1–5) as activities:

1. **Confirm the system and the domain slice.** Name the entities, relations, and
   **invariants** the feature touches (§3.1). If the feature needs constitutive
   data (valid options, categories), declare it as **reference data** now — events
   and invariants will reference it.
2. **Model the events and flows — the Event Modeling step.** Lay the timeline of
   what happens, and identify which **pattern** each slice is
   ([§3.2.0](docs/product-framework.md#320-building-blocks-and-patterns)): a
   **Command** (`Trigger→Command→Event(s)`), a **View** (`Event(s)→View`), an
   **Automation** (the system acting on itself), or a **Translation** (one system
   telling another). Name the Trigger's source: user, external, or automated.
3. **Write the Deciders and Projectors for the interesting behaviour** (§3.3,
   §3.4). The Decider is where the business rules live — its rejections *are* the
   invariants. Carry on the aggregate exactly the **invariant-bearing state** a
   Decider must read at decision time, and nothing else (§3.4). Trivial CRUD needs
   no Decider; do not invent ceremony.
4. **Simulate — the cheapest gate.** Before writing any code, run the Decider and
   Projector against scenarios drawn from the flow (a *given* of prior events, a
   *when* command, a *then* of expected events/state). Prove they are **sound and
   complete**: invalid commands rejected for the right reason, valid ones producing
   the right events, no View needing a field no event supplies. This is the single
   highest-leverage habit in the whole process — a behaviour defect caught here
   costs a sentence.
5. **Specify the UI steps and page-graph position** (§3.2.1, §3.2.4). Type each
   interaction against an **Abstract Interaction Object** (`single-select`,
   `trigger-action`, …), never a concrete control. Declare the projection it
   surfaces, the commands it offers, its transitions, and its modelled meaning
   (emphasis, state meanings, accessibility, content references). Drive the intent
   down: anything you used the intent to decide must become a modelled element.

**Run the self-audit as you go.** The model tells you where it is incomplete:

- **State justification** (§3.4): every field on an aggregate must be read by some
  Decider. An unread field is a finding — dead state, or (more often) an
  *unmodelled invariant* you have not written yet.
- **Decider justification** (§3.4): every Decider must have at least one reachable
  rejection. A Decider that never rejects is a finding — trivial behaviour that
  needs no Decider, or a missing invariant.
- **Intent-reliance** (§3.2.1): if you leaned on the one-sentence intent to settle
  a realisation choice, that choice was a missing model element. Model it, retire
  that part of the intent.

**Exit criterion:** the What is a typed slice of the graph; every Decider and
Projector **simulates sound and complete**; the self-audit checks pass (no
unjustified state, no toothless Decider, intent driven down). You have written no
code, and you are confident the behaviour is right.

---

## Stage 2 — How (the realisation)

> **The question: *What realises this, on what substrate, against which
> contracts?***

The What is settled; now bind it to realisation. You are writing *against* a fixed
What, never alongside an unsettled one (§2.1, step 6).

1. **Bind to the contracts** (§4.2). Confirm the application contract holds —
   decision logic in a pure, isolable core, so it can be checked against the
   Decider. Record the runtime/deployment identity (domain, bundle, runtime) for
   this instance.
2. **Place it in the repository layout model** (§4.3). Know where each file the
   feature needs is legal, and what spine the slice requires (its tests, its
   contracts). The layout model both scaffolds and verifies.
3. **Reify the UI** (§4.5). For each AIO, choose the concrete component per
   **context of use** — and remember the **interaction class** (GUI/TUI) gates
   which contexts apply (§3.2.2): a GUI brings form factor and pointer/touch; a TUI
   brings terminal capabilities. Bind to the design system; style via tokens, never
   literals. Resolve **content** keys against the content store per locale (§4.6).
   Where an AIO cannot honestly cross to a target class, declare it *unreifiable* —
   a recorded gap, not a silent one.

   On the CLI this is the `product design-system` family: `add <manifest>` vendors
   a §11.3 manifest (declaration half **plus** its implementation bundle — per-target
   component sources, token values per theme, templates) under
   `.product/design-systems/<id>/`; `validate` runs the wholeness + bundle checks;
   `couple` is the §11.2 coverage check; `bind <id>` records the choice on the How
   contract. Once bound, every `product codegen` backend resolves it: the coupling
   becomes a *plan-time gate* (a coverage gap fails the emit), the reified tree
   carries `design-system.g.json` (the reify(AIO, context) → CIO map by value) +
   `tokens.g.css`, and `product codegen web` renders one on-system page per UI step,
   styled exclusively through tokens. `product codegen check` catches design-system
   drift exactly as it catches graph drift.
4. **Generate the interface contracts** from the domain model (§4.4) — OpenAPI,
   AsyncAPI, and so on — never hand-written, so the published surface cannot drift
   from the meaning.

**Exit criterion:** every AIO has a reifying component for each declared context;
every content key resolves for each locale; the interface contracts are generated;
the layout model knows where the feature's files belong. The realisation is fully
specified and nothing about *how* is left to chance at build time.

---

## Stage 3 — Build (make it real)

> **The question: *Does it pass, and is 'done' computed?***

Now the work units run — each a single bounded transformation from a frozen input
to one artifact (§5), referencing the What and How by pointer, never re-deciding
them.

> **Build runs *across* a seam.** The work unit is where the framework's
> specification ends and execution begins (§5.1). The specification side **emits**
> a frozen work unit — by value, with a hash identity — and an **executor** returns
> a **verdict event**. The framework fixes that contract and nothing on the far
> side: scheduling, batching, and *how* the realisation is produced are the
> executor's concern. You may run the framework's own executor or any other that
> honours the seam — they are interchangeable. The steps below are what happens
> on the execution side of that seam.

1. **Scaffold from the layout model** — it tells the generator where to place what.
2. **Implement against the oracles.** The Decider and Projector you simulated in
   Stage 1 are now the **oracles** the realised code is checked against — the
   realised behaviour must produce *identical* outputs to them (§6.3).
3. **Run the verifications, cheapest first** (§6.3): **layout conformance** (tree
   shape) → **behavioural conformance** (matches the Decider/Projector) →
   **seam** (screen agrees with its UI step) → **domain & data conformance**
   (structure matches; real data validates). Each is a deterministic gate; a
   failure stops the build, with no override-by-assertion.
4. **Let "done" be computed, not judged** (§7.2). A feature is done when every flow
   passes behavioural conformance, every concept in its *derived* footprint passes
   domain conformance, every verification citing one of its elements is green, and
   its acceptance criteria pass. You do not *decide* it is done; the predicate
   says so.

**Exit criterion:** all required verifications are green, and `feature_done` computes
true. Ship it.

---

## The loop back — why this is a cycle, not a line

The arrows go forward, but three signals send you upstream, and a healthy team
treats them as routine, not failure:

- **Intent-reliance rate** (§3.2.1) — you are still leaning on prose intent to
  decide UI. Go back to the What and model what was missing.
- **State / Decider justification** (§3.4) — orphaned state or a never-rejecting
  Decider has surfaced an *unmodelled invariant*. Go back and write the rule.
- **Data-divergence rate** (§3.1) — and this one can fire *after you ship*:
  production data no longer satisfies the declared structure. Either the data is
  wrong, **or the spec has gone stale** — go back to the domain model and
  reconcile. This is the framework's one continuous, in-production signal.

Feedback re-enters the funnel as a fresh forward pass — you do not run the stages
backwards, you run them forward again over the gap that was found (§2.1). Over
time the model gets more complete and the right-hand stages get more determined:
that is the *reproducibility → measurement → improvement* chain
([§1](docs/product-framework.md#1-purpose-and-scope)) made into a daily habit.

---

## The one-page cheat sheet

| Stage | The question | You produce | Exit when |
|---|---|---|---|
| **Intake** | Where on the model does this live, and what's the one-sentence intent? | a located request + intent | you can name system, context, flow(s), intent |
| **What** | What happens, what decides it, what does the user see — and does it simulate? | a typed event-model slice + Deciders/Projectors + UI steps | simulations pass; self-audit clean; no code yet |
| **How** | What realises this, on what substrate, against which contracts? | reification, content, contracts, layout placement | every AIO reified, every key resolved, contracts generated |
| **Build** | Does it pass, and is 'done' computed? | realised code + green verifications | all verifications green; `feature_done` = true |

> **If you remember one thing:** simulate the Decider before you write code. Most of
> what the framework buys you is paid out at that single moment — the behaviour is
> proven right while a fix still costs a sentence.
