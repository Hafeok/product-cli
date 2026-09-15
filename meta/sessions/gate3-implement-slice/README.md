# Gate 3 — builder session

**Party:** builder (CG-R-128). Blind to the design: the four arrived inputs, the session
prompt, and the two rulings that scope the run. No README of the binding, no conformance
manifest, no repository, no web search.

**Result: 29 clarifications against 3 of 13 frame categories settled without invention.**
All 29 questions are open — no principal answered. Read `gate-c-report.md` first.

| File | What it is |
|---|---|
| `prompt.md` | the session prompt, verbatim |
| `inputs/` | the four arrived inputs, verbatim, hashed in `bootstrap.md` before use |
| `bootstrap.md` | first act: hashes, what was read, the reading order, the one standing rule that could not be complied with |
| `gate-a.md` | the expectation list, written with the determinations unopened; the reading of the slice; six contradictions |
| `frame-categories.md` | the 13-category scheme — **invented**, because the bundle never defines "frame category" |
| `questions.md` | 29 questions, verbatim, with what prompted each and the provisional reading the build proceeded under |
| `decisions.md` | 37 points the specification does not settle, each tagged `D-nn` at its site in the source |
| `gate-c-report.md` | the Gate 3 report |
| `solution/` | the slice: `dotnet test` → 15 passing, clean under `TreatWarningsAsErrors` |

## The three findings that do not depend on any unanswered question

1. **The write position has no realisation in the profile.** `PlaceOrder` declares
   `writes: [OrderPlaced]`. The handler may not perform I/O, the controller may not name
   a persistence type, the provider only supplies reads, and there is no fourth role.
2. **"Returns a transport result derived from Accepted or Rejected" names a dependency,
   not a function.** No status code, no `Location`, no body is stated anywhere. Two
   readers build two incompatible APIs and both conform.
3. **Every `must_not` in the profile is satisfied as written while the forbidden thing
   happens.** `Unroled/` holds three types that declare no role, so no rule reaches them;
   they do the persistence, the transport read and the handler's I/O.
   `ProfileConformanceTests.EvadesEveryMustNot_ByIndirection` asserts it, and passes.

## Running it

```bash
cd solution && dotnet test        # 15 tests, .NET 8
```
