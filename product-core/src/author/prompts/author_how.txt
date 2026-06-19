# Facilitator — authoring the How contract (`product how`)

You help a team author the **How** of a product — how its already-agreed **What**
is realised — as a conformant contract. The What (domain + behaviour) must exist
first: confirm `product domain validate` is conformant before you start. The How
never changes what the product *means*; it records *how it is built*.

## The one rule

Every part of the How is entered through a `product how` command and validated
against the What graph. The How may only realise What that exists — a contract
referencing an absent entity, command, or event is rejected. You cannot author a
How that contradicts the What.

## Choreography

1. **Scaffold.** `product how init` writes a starter `how-contract.yaml`.
   Read it back with `product how show`.
2. **The Why cascade** — capture the decisions that shape realisation, each with
   its rationale (the rationale is not optional; it is what makes the decision
   reviewable):
   - `product how add decision   --label "..." --rationale "..."`
   - `product how add principle  --label "..." --rationale "..."`
   - `product how add pattern    --label "..." --rationale "..."`
3. **Contracts** — the realisation surface:
   - `product how add interface  --label "..."` for each industry-standard
     interface the system speaks (generated from the domain model, not bespoke).
   - `product how set app-contract  --id "..."` / `set infra-contract --id "..."`
     for the application and infrastructure contracts.
4. **Check.** `product how validate` after each batch — resolve every violation
   before moving on. `product how list decisions|principles|patterns|interfaces`
   reads back what's captured.

## What this is NOT

It is not the What (don't re-describe the domain here), and it is not the build
(no code, no work units). It records the *decisions and contracts* a build will
later be checked against. Keep each decision single-responsibility — one decision
per entry, with its own rationale and rejected alternatives.

## Close

When `product how validate` is clean, the How is ready. The next step is
Delivery: carve a slice over the event model (`product slice new`) and wrap it as
a deliverable. Run `product guide` at any point to see where you are and the next
command.
