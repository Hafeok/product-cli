# Bootstrap — 2026-08-28-closure-falsifier-prereg

The invocation message and the session's parameters, committed before any other act of this
session, per `DDD-dec-20` (`decision-driven-design`, `core/decisions/DDD-dec-20.yaml`).

## Invocation

The session arrived as a single message: the prompt in `prompt.md`, verbatim, with one attached
document — the pre-registration design, filed beside it as `prereg-closure-falsifier.md`.

## Parameters

| Field | Value |
|---|---|
| Principal | Emil |
| Session kind | Proposal only — no executable code, no experiment run |
| Repository | `Hafeok/product-cli` |
| Branch | `claude/closure-falsifier-prereg-w1mm7d` |
| Base commit | `d0f429741fd06e6d09d25937efcb61f440b94472` (`main`, "Merge pull request #52 from Hafeok/claude/prd-tick-rate-validity-re7ofs") |
| Gates | 0–5 (orientation; act set selection rule; consequential-property category list; quantities and thresholds; classification rubric; assembly) |
| Gate discipline | Hold at every gate. No gate is passed on the session's own assessment; each requires an explicit ratification message from the principal. |
| Commit identity | `Claude <noreply@anthropic.com>` — session-neutral, per the prompt's standing rule |
| Output register | Every artefact marked `[PROPOSED]`; Rulings / `[PROPOSED]` / `[OPEN]` kept distinct |

## Arrived inputs, with identity

| Artefact | Lines | sha256 |
|---|---|---|
| `prompt.md` (this session's charter, verbatim) | 145 | `4b5296634f648957b56563922d37b5103610795a8c17797e4b0e884ac40a3be0` |
| `prereg-closure-falsifier.md` (the design this session works from) | 113 | `145ed0a2204d1a698cb0d4d6ff2b33ba431d20a152d52b86cc91986df3fd632b` |

The identity check is recorded because `DDD-dec-20`'s own notes make it load-bearing: an artefact
accepted without its identity checked can record an arrival that did not happen.

## Arrival gap, recorded not resolved

Gate 0 instructs the session to read two documents: the pre-registration and **the specification
language foundation**. Only the first arrived.

Searched, without finding it: this repository (`Hafeok/product-cli`, full tree);
`Hafeok/decision-driven-design` at `main` (`core/`, `meta/`, `spec/`, `papers/`, recent commit
history); `Hafeok/ai-development-foundations` at `main`. There is no `Hafeok/specification-language`
repository. The document is either unpublished, held on a branch not searched, or was intended as a
second attachment that did not travel.

Per the second instance recorded in `DDD-dec-20`'s notes, a non-arrival is reported as a
non-arrival and not closed by substituting the nearest plausible artefact. The session therefore
proceeds at Gate 0 with the pre-registration alone, and states at each point where the missing
foundation would have been load-bearing.

## Convention note

`meta/sessions/` is a convention of `Hafeok/decision-driven-design`, filed there as `DDD-dec-20`.
This session runs on a `product-cli` branch, so the directory is created here to hold this
session's arrival record under the same convention. That is an application of the convention to a
sibling repository, not an extension of it; no filing is made upstream.

## Corrections

Recorded by appending, not by rewriting the line above. The erroneous figure stays visible.

1. **2026-08-28, before Gate 0 output.** The arrived-inputs table states `prompt.md` at **145 lines**.
   It is **134 lines**. The sha256 in that row (`4b529663…`) is correct and is the identity that
   binds; the line count was a secondary descriptor, stated without being measured. The error is
   mine and is reported here rather than edited away.
