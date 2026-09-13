# Session: implement the C# stack binding in `product-cli`

**Commit this prompt and a bootstrap record to `meta/sessions/` as the first act, before any code. Session-neutral commit identity: `Claude <noreply@anthropic.com>`.**

---

## Standing rules

`canon-governance` holds the in-force rules; read them at the pinned commit and comply. **They are not restated here.** The ones bearing hardest on this session: three registers, supersession never rewrites, a named principal on every record, and **prose describes what is in force and implemented — anything designed but not built carries a date and an explicit marker**. That last one applies to every README and every `--help` string you write.

Beyond them:

- You propose; Emil ratifies. Gates hold until an explicit ratification message.
- Report defects honestly, including in your own earlier gate output.
- Name the weakest point of each proposal.
- File arrived inputs with their sha256.
- **Search enforcement, not only prose.** A rule can be in force while stated nowhere — held in a schema, a required field, an analyser.

## Prohibitions

- **Do not generate slices from code.** The delta is a list of gaps, never a queue of candidate slices for approval (R-D). Bulk approval is not ratification.
- **Do not infer from naming.** Not class names, not namespaces, not folder layout, not base types. The analyser reads declared attributes only.
- **Do not write a second determination schema.** The domain state change binding's schema is used unchanged.
- **Do not put judgement in the .NET tool.** It emits facts; every rule and classification is Rust-side against the store (§4).
- **Do not widen an act to fit a class** (R-B). Where they disagree, the code moves or the slice is declared not-attachable with the reason recorded.
- Do not proceed past a gate without explicit ratification.

---

## Arrived inputs

1. `prd-csharp-stack-binding.md` — the specification
2. the domain state change binding — schema, examples, checks

---

## Gate 0 — orientation

Without writing implementation code:

1. Read the PRD. List anything unsound, ambiguous, or unimplementable as written.
2. Report where this lands in `product-cli` as it stands: which crate, what it reuses, what it duplicates. **Duplication is the thing to find** — if determination loading, store access or validation already exist, they are used, not rewritten.
3. Confirm the four rules at §1 in your own words and flag any you read differently.
4. Propose the inventory JSON schema and its version field. Facts only; no classification.
5. Name the real brownfield solution the delta will run against, and confirm access.

**Hold.**

---

## Gate 1 — the delta measure, first and alone

Built and run **before** the declaration machinery, because it is the part that could be worthless and it needs no adoption to test.

Build: the .NET inventory tool, the Rust consumer, and the three-region classification with the two ratios.

**Before running it, Emil records a prediction** for each of the first three conditions at §12: expected reachability percentage, whether *declarable* and *unstructured* separate without a per-symbol judgement call, and what fraction of profile rules look Roslyn-enforceable. The prediction is committed before the measurement.

Then run it on the named solution and report:

- the three regions, by cluster — namespace, reachability, or change coupling — never a flat list of symbols
- reachable-undeclared and isolated-undeclared, separately
- **prediction against measurement**, with the gap stated

If a §12 condition fires, say so plainly. §12 exists so that a working tool cannot retroactively settle whether it was worth building.

**Hold.**

---

## Gate 2 — declaration, profiles, checks

The `Slice` and `RealisesFact` attributes; the profile schema; the five checks at §6.

Requirements:

- Profiles are filed as determinations addressed to the act type, not compiled in (R-C). Changing the architecture is a supersession, not a release.
- The handler role reuses the Decider rules — signature from the model, command coverage, output containment, rejection containment, event coverage, coverage declaration. **Do not author a second set**; they will drift within a release.
- Each profile rule is declared as **analyser-enforced** or **read-enforced**. A rule nobody can run is not enforcement, and saying which is which is the whole value.
- The five checks fail; the delta reports. A codebase with unspecified regions is unspecified, not non-conformant.

**Hold.**

---

## Gate 3 — implement one slice, instrumented

The exercise at §10. This is the part that tests something the checks cannot.

1. **Before the run**, record what you expect the actor to have to invent.
2. **During**, answer every question — and record each verbatim, with what prompted it. The question is the datum, and answering without recording destroys the measurement.
3. **After**, report per category: settled by the profile, settled by a determination, asked about, silently decided. Silent decisions are found by reading the produced code against the determination set, not by asking the actor what it decided.

Report the count of clarifications beside the count of categories settled.

**Hold.**

---

## Gate 4 — report

1. Every §12 condition, fired or not, with the evidence.
2. What the build establishes and what it does not. In particular: the profile, the schema and the determinations were authored by the party running the exercise, so a clean Gate 3 shows internal coherence and nothing about legibility to anyone else (§11.2).
3. The category list Gate 3 surfaced, in a form comparable against the list being enumerated independently for the notation experiment. **If the two disagree substantially, say so** — that bears on quantities not yet fixed there.
4. Weakest point of the whole build.

**Stop.**

---

## Carried risks

1. **Attribute adoption is the load-bearing assumption and this session cannot test it.** You will place the attributes, having designed the scheme. Record that §11.2 stays open however well Gate 3 goes.
2. **A working tool is weak evidence.** The difficulty has moved from *can this be built* to *was this worth building*, which is what §12 is for.
3. Reachability over-reports in DI-heavy codebases. If it fires, drop the ratio rather than defending it.
4. The inventory is measurement, not canon. It is re-derivable, never stored as a determination, and not committed.
