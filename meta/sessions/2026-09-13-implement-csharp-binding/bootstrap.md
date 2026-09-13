# Bootstrap — implement the C# stack binding

**Session:** `2026-09-13-implement-csharp-binding`
**Session kind:** implementation under gates. A stack binding is built beneath the domain state
change binding's schema; the schema, the act vocabulary and the fact vocabulary are used unchanged.
**Principal:** Emil. The session proposes; Emil ratifies. Gates hold until an explicit ratification
message.
**Commit identity:** `Claude <noreply@anthropic.com>` — session-neutral, per the invocation.

---

## Invocation

The chat message is filed verbatim as `invocation.md`. The instruction set is filed verbatim as
`prompt.md` (`session-implement-csharp-binding.md`, sha256
`8b6188b261fe594c7247a16591f9fc62b99ec233b5dfd44509382c123c4f53ab`). Where the two disagree, the
prompt governs and the disagreement is reported at Gate 0.

## Parameters

| Parameter | Value |
|---|---|
| Repository | `Hafeok/product-cli` |
| Branch | `claude/csharp-stack-binding-dbx7vr` |
| Base commit | `d0f429741fd06e6d09d25937efcb61f440b94472` (`main`, 2026-08-20, merge of #52) |
| Governance read at | `Hafeok/canon-governance` `c5383be06e5b181dc79307554a35cddeacbcd3e8` — see below |
| Gates | Gate 0 orientation · Gate 1 the delta measure alone · Gate 2 declaration, profiles, checks · Gate 3 one slice, instrumented · Gate 4 report |
| Register discipline | rulings in force · `[PROPOSED]` · `[OPEN]`, kept distinct in every gate output (`CG-rule-01`) |
| Inventory | measurement, re-derivable, **never committed and never stored as a determination** |

## The governance ref

The invocation reads "read `Hafeok/canon-governance` at `<REF>`". The placeholder arrived
unsubstituted. The head of `main` at read time was taken:
`c5383be06e5b181dc79307554a35cddeacbcd3e8`, 2026-09-02. Under `CG-R-10` a pinned citation records
what was read and complied with and does not advance on head movement, so this ref is the one this
session is bound by unless Emil names another at Gate 0. If another is named, the difference between
the two is stated in an appended note here; nothing above is amended.

## Rules in force, from the governance ref

Ten rules, `CG-rule-01` … `CG-rule-10`. Not restated; the four bearing hardest, as the prompt names
them, read in this session's words:

- **`CG-rule-01` three registers.** Every gate output separates what is in force, what this
  session proposes, and what is open.
- **`CG-rule-02` supersession never rewrites.** A wrong figure in an earlier gate stays visible with
  its correction appended beside it. `CG-rule-09` adds: the correction names what was wrong and
  what caught it.
- **`CG-rule-04` a named principal on every record.** Every determination this session files carries
  a principal, and (`CG-rule-05`) that principal is Emil, never this session — the session drafts
  the record and its acceptance is Emil's.
- **`CG-rule-10` prose describes what is in force and implemented.** Every README and every `--help`
  string written here describes what the binary does today; anything designed and not built carries
  a date and an explicit marker. This rule is *provisional*, evidenced only by its originating
  session; this session catching its own over-claim before commit would be an independent instance.

Also bearing: `CG-rule-06` (this record precedes the work), `CG-rule-07` (identifiers stable,
programme-scoped — the `DSC-` space belongs to the domain state change binding and is not this
session's to allocate from casually), `CG-rule-08` (pre-registration precedes execution; Emil's §12
predictions are committed before Gate 1's measurement, and there is no re-roll), `CG-R-7` (search
enforcement, not only prose — a rule can be in force while stated nowhere).

`CG-rule-03` (falsifiers on live claims) binds repositories that hold claims. This repository
holds `.ddd/claims/`; the artefacts this session produces are code, determinations and session
records, none of them a claim at a live status, and the PRD states at its head that nothing in it
carries a falsifier. Recorded so the scope question is visible, not settled here.

## Standing rules from the prompt

- Propose; do not ratify. Gates hold until an explicit ratification message.
- Report defects honestly, including in this session's own earlier gate output.
- Name the weakest point of each proposal.
- File arrived inputs with their sha256 (`arrived-inputs.md`).
- Search enforcement, not only prose.

## Prohibitions from the prompt

- No slices generated from code; the delta is a list of gaps (R-D).
- No inference from naming — class names, namespaces, folder layout, base types. Declared
  attributes only.
- No second determination schema. The binding's `determination.schema.json` is used unchanged.
- No judgement in the .NET tool. It emits facts; every rule and classification is Rust-side.
- No widening of an act to fit a class (R-B).
- No proceeding past a gate without explicit ratification.

## Environment facts at bootstrap

Recorded because they bear on what Gate 1 can do here.

- **No .NET SDK is installed** in this container. `builds.dotnet.microsoft.com`, `api.nuget.org`
  and the apt candidate `dotnet-sdk-8.0` are all reachable through the session proxy, so one can be
  installed for the session. The workspace's own CI has no .NET SDK by decision
  (`.ddd/decisions/fixtures-not-sdk.yaml`), which constrains how the inventory tool is tested here.
- **The workspace already carries a C# adapter** (`ddd-lsp/src/adapter/csharp*.rs`) that reads
  symbols through `roslyn-language-server` over LSP and slices attributes from source text, under
  `dec/ddd/lsp-as-seam` ("the core never touches Roslyn … APIs directly"). The PRD asks for a
  Roslyn-workspace .NET tool. Both facts are carried to Gate 0, where the reuse/duplication question
  is answered; nothing is decided here.
- **The brownfield solution is not named** in the invocation or the prompt. Gate 0 proposes one and
  confirms access; it is not cloned before that.

## What this record is not

Not a plan and not a design. The Gate 0 report is the first design artefact, and it is held for
ratification like every other.
