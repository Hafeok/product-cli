# Workshop — Specifikationsdrevet design (Clever)

Materials for the Danish "Specifikationsdrevet design" workshop (devs + POs),
built around this `product` CLI. Teams model the **What** of one of their own
systems by *describing* it in plain language to Copilot CLI, which translates to
`product` commands; the binary validates every node.

## What's here

| File | Purpose |
|---|---|
| [facilitator-spec-da.md](facilitator-spec-da.md) | The *What* of the session — agenda, per-slide intentions, run-of-show. A facilitator can run the session from this alone. |
| [exercise-1-handout-da.md](exercise-1-handout-da.md) | Participant handout — model your own system (domain + event model). |
| [exercise-2-handout-da.md](exercise-2-handout-da.md) | Participant handout — break a component into one SPMC bundle. |
| [`../../.github/copilot-instructions.md`](../../.github/copilot-instructions.md) | Primes Copilot CLI with the `product` grammar. **This is what makes the prepared repo work** — participants clone the repo, start `copilot`, and describe their system. |
| [`../workshop-runbook.md`](../workshop-runbook.md) | The general 90-min framework-capture runbook (manual / `claude`-facilitated path). |

The slide deck (`spec-drevet-design-workshop-da.pptx`, the *How*) lives with the
facilitator — the per-slide intentions in `facilitator-spec-da.md` are the
source of record extracted from its talenotes.

## Facilitator prep checklist

1. **This repo is the prepared repo.** It ships `.github/copilot-instructions.md`,
   so a cloned copy + Copilot CLI is all a participant needs.
2. Confirm the binary builds and runs: `cargo build --release` then
   `./target/release/product --version`. (For workshops, install `product` on the
   PATH so handout commands work verbatim.)
3. Confirm an agent CLI is available: `copilot` (primary) — or `claude` for the
   `product author domain` path.
4. Dry-run the flow once: `getting-started-framework.md` end to end. If it runs
   clean for you, it runs clean in the room.
5. Print the two handouts (`exercise-1-handout-da.md`, `exercise-2-handout-da.md`)
   and skim `../guide/framework-concepts.md` for the vocabulary.

## Run-of-show

| Tid | Segment | Slides |
|---|---|---|
| 12:00 | What-talk: fra LLM til specifikation | 1–21 |
| 13:00 | **Øvelse 1:** modellér jeres eget system | 22 |
| 13:55 | How-talk: fem måder at bruge LLM'er | 23–28 |
| 14:45 | SPMC: fra spec til eksekvering | 29–31 |
| 15:15 | **Øvelse 2:** byg et SPMC-bundle | 32 |
| 15:40 | Afrunding & adoption | 33–34 |
