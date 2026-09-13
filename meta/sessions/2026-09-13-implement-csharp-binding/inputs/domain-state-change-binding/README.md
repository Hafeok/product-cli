# Domain state change — a conforming specification language

**Event modelling gives you the addresses. This gives you something to put at them.**

Event modelling produces a genuine act vocabulary — command, read-model, automation and translation slices, each named — and its information-completeness rule marks read and write position structurally. Assessed against the [conformance criterion](https://github.com/Hafeok/specification-foundation), event modelling plus a domain model scores **2 of 11**.

This binding scores **11 of 11**. It adds the determination layer: extent, allocation, coverage, boundary declarations, and an act-time write form. It does not replace event modelling and does not make it heavier — the additions live in a store addressed by the slice vocabulary, not in the diagram.

```bash
./tools/run_all.sh     # every check, positive and negative fixtures
```

Individually: `validate.py` (records against the schema), `prove_prohibitions.py` (forbidden shapes are unrepresentable), `check_resolution.py` (addresses and the resolution condition), `check_composition.py` (two scopes agree at their seam), `check_conformance.py` (the criterion).

---

## What the schema enforces, and why that matters

The criterion's prohibitions are not satisfiable by advice. A notation that merely *recommends* addressing every determination is one where an unaddressed determination is expressible, and expressible means it will be written. So the prohibitions are enforced by the schema:

| | Forbidden | Enforcement |
|---|---|---|
| PR-1 | a determination with no address | `address` is required; it needs act type and instance |
| PR-2 | a determination with no allocation | closed union over pinned/checked/residual; residual requires a carrying actor kind **and** a principal, and `principal.kind` omits machine by construction |
| PR-3 | a coverage claim with no uncovered set | `does_not_cover` is required; the empty case is asserted with a sentinel, never by omission |

`tools/prove_prohibitions.py` constructs each forbidden shape and shows the schema rejects it — seven cases, all rejected. That run is the evidence the conformance claim cites.

## The four states of extent

The distinction that makes revisiting computable:

| State | Means |
|---|---|
| `bound-here` | applies at this address only |
| `travels-to` | applies across a named region |
| `does-not-travel` | a stated boundary — someone decided |
| `silent` | the axis was not considered |

A silent axis **may not carry a region**, enforced by the schema. That prevents the one failure that cannot be repaired after the fact: silence being quietly upgraded to a stated boundary, so that when a new axis appears nobody can tell which determinations were bounded deliberately and which were merely filed before the axis existed.

Default is the narrowest. Travel is a deliberate write, which inverts the incentive of a prose rules file — there, generality is free and specificity is expensive, which is how those files rot.

## One schema, both times

The property no prior specification language has:

```yaml
provenance:
  made_at: build-time    # or act-time
```

That enum is the only difference between a determination taken in advance and one produced while implementing. Not two shapes, not two stores, not a design-time artefact plus a changelog. `DSC-0005` in the example is a currency decision made mid-implementation, filed at the same address the four build-time determinations were read from.

This is what stops the store going stale from first use. Decomposition never completes; acting produces determinations the specification did not contain; if those cannot return to the address they were read from, everything above them decays.

## Addresses and the resolution condition

`check_resolution.py` runs two checks against the event model.

**Address validity.** Every determination addresses a slice the model names, and every position it declares is one the slice actually has. Without this the address space is a convention — a determination can name a slice that does not exist and nothing notices.

**The resolution condition.** Incompleteness is a dangling reference, and nothing else is:

- **C-1** — every fact read by an act resolves in the fact vocabulary, and is either written by some act in scope or declared external
- **C-2** — every fact written resolves, and is either read by some act in scope or declared terminal
- **C-3** — every acceptance relation ranges over facts some act writes

The check found two dangling references in this repository's own event model the first time it ran: `Cart` was read by `PlaceOrder` and written by nothing, and `CartEmptied` was written by nothing that read it. Both were real modelling defects — the fix was to add the `Cart` projection that produces the state `PlaceOrder` reads. `examples/broken.eventmodel.yaml` is that model preserved, so the check's discriminating power can be seen rather than asserted.

The boundary ratio is reported, with no ceiling. A ceiling is arbitrary and turns a measure into a target; a high ratio means a specification that resolves little internally, which is what a reader needs to see.

## Composition — the seam check

The resolution condition is scope-relative. Two specifications can each pass it and still leave a dangling reference between them: `ordering` declares a fact terminal, `fulfilment` declares the same fact external, and nothing verifies that what one produces is what the other expects.

That seam is where divergence is predicted to concentrate — recurrent, and touched by two actor populations — so it is the highest-value check here. `check_composition.py` runs it:

- **CS-1** — every fact A declares terminal, naming B, is declared external in B naming A, and is read by some act in B
- **CS-2** — every fact B declares external from A is written by some act in A and declared terminal there
- **CS-3** — the fact kind agrees across both vocabularies
- **CS-4** — a fact A declares as having unobservable consumption, while B reads it inside the composition, is an inconsistent pair: B's read is the observation A says does not exist

`examples/seam-defect/` is the negative fixture — fulfilment expecting a `CustomerAddress` ordering never produces, and the two disagreeing on whether `OrderConfirmed` is an event or a read model. Both are caught.

## What is not built, and why each stays that way

Three items remain, and they close differently.

**Meaning across the seam is not checked, and will not be.** Two specifications can agree on a fact's name, kind and boundary and still hold incompatible notions of what the object is. Every candidate instrument would check that a declaration has some property; the property that matters is whether the declaration is *right*, which is residual. An instrument here would pass a wrong agreement as readily as a correct one. This is checked by a named principal reading both sides, and saying so is stronger than proposing a check nobody built.

**Seven pairs are not proven sufficient, and cannot be from inside.** Passing all seven establishes that a notation failing any is non-conforming. Adequacy would need a notation that passes all seven and still fails in practice — which is evidence from use, not from analysis.

**No falsifier.** The claim this notation makes is about **inference load**, not about escaped decisions — those are the accrual claim and need a corpus and time. What a format can affect is how much an actor must supply that the artefact did not determine, on a single act.

Stated: *holding the determinations constant, a conforming notation leaves less to inference than prose carrying the same content.*

The comparison is not specification versus none, which is trivially true. It is the same determinations in two notations. Per decision point in a fixed frame, the actor's resolution is **determined** (retrieved from the artefact), **inferred** (supplied from elsewhere), or **asked**; the measure is `inferred / frame size`. Correctness is irrelevant — an actor can infer correctly, and a right answer arrived at by invention is still an inference and still fails on the next act where the convention does not hold.

The prediction has a floor: inference rate cannot fall below the declared residual. A run where it goes lower means the actor is inventing over declared gaps rather than surfacing them, which is a failure disguised as a good result.

This is the product framework's **intent-reliance rate** generalised from screens to any act. It is untested, and nothing in this repository substitutes for testing it.

## Layout

```
schema/determination.schema.json     the language — and the prohibition enforcement
examples/ordering.eventmodel.yaml    the base: act vocabulary + fact vocabulary
examples/place-order.*.yaml          determinations, build-time and act-time
examples/fulfilment.*.yaml            the peer scope, for the composition check
examples/broken.eventmodel.yaml      negative fixture — two dangling references
examples/seam-defect/                negative fixture — a seam that does not agree
conformance/manifest.yaml            published evidence, 11/11
tools/run_all.sh                     every check, positive and negative
```

Conforms to `specification-foundation`. Self-certified with published evidence: re-run the three commands above.
