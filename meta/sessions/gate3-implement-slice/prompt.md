# Session: Gate 3 — implement a slice from the specification

**Commit this prompt and a bootstrap record to `meta/sessions/` as the first act. Session-neutral commit identity: `Claude <noreply@anthropic.com>`.**

---

## What this tests

Not whether declarations resolve. **Whether act, fact and position are sufficient for an actor to build a conforming slice without asking what was meant.**

Everything in Gates 1a and 1b measured existing code. This builds new code from a specification, and it is the question the whole binding exists to answer.

**Greenfield.** No eShopOnWeb, no Orchard Core, no delta, no reachability. A fresh ASP.NET Core solution, empty, into which one command slice is implemented.

---

## Arrived inputs

1. `ordering.eventmodel.yaml` — the act vocabulary and the fact vocabulary
2. `place-order.determinations.yaml` — the determinations at those addresses
3. `profile-rest-api-v1.md` — how a command slice is realised in this stack
4. `determination.schema.json` — the schema the determinations validate against

File each with its sha256 before use.

---

## Standing rules

`canon-governance` holds the in-force rules; read them at the pinned commit and comply. They are not restated. Beyond them: you propose, Emil ratifies; report defects honestly including in your own output; name the weakest point.

## Prohibitions

- **Do not read the binding's README or its conformance manifest.** They describe what the specification is *for*, and this run measures whether the specification alone is sufficient. Read the four arrived inputs and nothing else about the scheme.
- **Do not author determinations.** If the specification does not settle something, that is the finding, not a gap to fill.
- **Do not silently decide.** See the instrumentation below; this is the whole measurement.
- Do not proceed past a gate without ratification.

---

## Gate A — before writing any code

Produce, and commit, **before reading further into the determinations than you already have**:

1. **What you expect to have to invent.** A list, written first. Recorded so the result cannot be rationalised afterwards.
2. Your reading of what the slice is: the act, the facts it reads and writes, the positions, and what the profile requires.
3. Anything in the four inputs you believe is contradictory or unimplementable as written.

**Hold.**

---

## Gate B — implement

Implement `PlaceOrder` as a command slice under `rest-api-v1`: controller, handler, and a provider if the act's read positions require one.

**The instrumentation is the measurement and it is not optional.**

**Every question you would ask, you ask — and each is recorded verbatim**, with what prompted it and which frame category it concerns. Emil answers; the question is the datum and the answer does not erase it. A slice built with six clarifications is a different result from one built with none.

**Every decision the specification does not settle, you state in the output** — in the code, as a marked comment, or in the run record. Silent resolution is the thing being measured and it is the thing that must not happen unrecorded.

Where the specification *does* settle something, implement it as settled. Do not improve on it, do not apply a convention you prefer, and where you think a determination is wrong, say so and implement it anyway.

**Hold.**

---

## Gate C — report

Per frame category, four columns:

| | |
|---|---|
| **settled by a determination** | the specification decided it and you read it there |
| **settled by the profile** | the profile decided it |
| **asked** | you raised it rather than resolving it |
| **decided** | you resolved it and the specification did not settle it |

Plus:

- **clarification count beside the count of categories settled.** That ratio is the headline.
- the Gate A expectation list against what actually happened, with the gap stated
- which profile rules you could have checked mechanically and which you could not — first evidence for PRD §11.4
- **the categories this surfaced**, in a form comparable against the category list being enumerated independently for the notation experiment. If they disagree substantially, say so.
- weakest point

**Stop.**

---

## Carried risks

1. **The specification, the schema, the profile and the act vocabulary were all authored by the party running this exercise.** A clean run shows the design is internally coherent. It says nothing about legibility to anyone else, and that limit stands however well this goes.
2. **The profile is unvalidated.** No slice has been built against it and it may not be specific enough. That is a legitimate outcome and the finding, not a failure to work around.
3. `place-order.determinations.yaml` was written as an illustration of the schema, not as a complete specification of a slice. Expect it to be insufficient, and report *where* rather than filling it.
4. This is one slice, one act type, one stack. It establishes nothing about generalisation.
