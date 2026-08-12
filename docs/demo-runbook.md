# Demo runbook — one hour, live terminal

**Rehearsed 2026-08-12** on branch `claude/demo-dryrun-rehearsal-wkkjuu`, release
binaries, this machine. Every output block below is pasted from a real run, not
reconstructed. Every timing is `date +%s.%N` around the command.

**The spine:** transcription got cheap; judgment didn't. Agents can write the
code; the question is which decisions were made, who answers for them, and how
anyone knows none escaped.

> **Read the failure list (§8) before the polished script.** Two things about
> this demo are not what the brief assumed, and one of them changes which file
> you edit on stage.

---

## 0. Pre-flight — do this before the room fills

```bash
cargo build --release                 # ~6 min cold. Do NOT do this live.
git tag -f demo-base                  # marks the tip the demo resets to
./target/release/ddd render           # 0.31s — the offline fallback (§7)
./docs/demo/reset.sh                  # confirm clean: dirty=0, seam-events=46
```

Then a **dry pass of beat 1** to confirm the tool answers, and reset again.

Terminal: **≥100 columns**, ≥26 rows. The widest line the demo prints is 88
chars (a `sha256:` in the rejection demand). Below 100 cols it wraps and the
binding block stops looking like a signature.

**Do not run `ledger verify` bare on stage** — it prints 79 ULIDs and scrolls
the screen. Use `ledger coverage` (4 lines) or `ledger verify | head -3`.

---

## 1. Beat 1 — the interceptor fires

*An agent edit that touches contract surface is rejected with a structured
demand.* **Wall clock: 0.07s.**

The staged change is one design token in the governed stylesheet:

```bash
python3 -c "
s=open('ddd-core/assets/render.css').read()
open('/tmp/after.css','w').write(s.replace('  --on-chip: #fff;','  --escape-price: #6b4fbb;\n  --on-chip: #fff;'))
"
```

That is a one-line diff:

```diff
   --status-warn: #8a5a00;
+  --escape-price: #6b4fbb;
   --on-chip: #fff;
```

Now push it through the interceptor — the path an agent uses, not `git`:

```bash
./docs/demo/mcp-call.sh ddd_apply_edit \
  "$(jq -n --arg f ddd-core/assets/render.css --rawfile t /tmp/after.css '{file:$f,new_text:$t}')" \
  | tee /tmp/demand.json | jq -f docs/demo/reject-view.jq
```

**Real output (17 lines, 88 cols, no scroll):**

```json
{
  "status": "rejected",
  "demand": {
    "file": "ddd-core/assets/render.css",
    "symbol": "--escape-price",
    "kind": "token",
    "change": "added",
    "rule": "web-token-membership"
  },
  "signs": {
    "after": "sha256:9751cdf4b2c8dadfd93335f079ed0a6ffffabe729360997bd23d8918ea29845e",
    "base_revision": "75b5472c844e272c588e605c36ea5893a75f693a",
    "before": "sha256:5a8eec55f9f578217dd37fbaa6bf7d014ee3c36f89777f66db71b5bdad1c6eab",
    "file": "ddd-core/assets/render.css",
    "symbol": "--escape-price"
  }
}
```

**Say:** the edit was not applied. The file on disk is unchanged. What came back
is not "denied" — it is a demand naming the exact transition that needs a
decision behind it, with the signature pre-computed.

**Note on `base_revision`:** it is the current `HEAD`, so it will differ from the
hash pasted above. The two `sha256:` content hashes are derived from file content
and *will* match, as long as `render.css` is at `demo-base`.

**Why a CSS token and not a `pub fn`:** see §8, finding F1. The Rust form works
and is documented in §1a, but it costs 17 seconds of silence and cannot use
`mcp-call.sh`.

### 1a. The Rust variant — documented, not recommended

```bash
./docs/demo/mcp-session.sh ddd_apply_edit \
  "$(jq -n --arg f ddd-core/src/configured.rs --rawfile t /tmp/after.rs '{file:$f,new_text:$t}')" \
  | jq -f docs/demo/reject-view.jq
```

after appending to `ddd-core/src/configured.rs`:

```rust
/// Whether this rule was disabled without a citation on record.
pub fn is_silent_waiver(rule: &ConfiguredRule) -> bool {
    rule.disabled && rule.scope.is_none()
}
```

**Real output** — identical shape, `"kind": "fn"`, `"rule": "rs-add-exposed"`,
`"symbol": "is_silent_waiver"`. **16.7s warm, ~50s cold.** Use `mcp-session.sh`,
never `mcp-call.sh` — see F1.

---

## 2. Beat 2 — the declaration binds

**Wall clock: 0.04 + 0.05 + 0.05 = 0.14s across three commands.**

### 2a. File the declaration, carrying the binding the demand pre-filled

```bash
./docs/demo/mcp-call.sh ddd_declare_seam "$(jq -n \
  --argjson b "$(jq -c '.demands[0].template.binding' /tmp/demand.json)" '{
  id: "seam/htmlcss/escape-price",
  boundary: "token --escape-price in ddd-core/assets/render.css",
  contract_location: "ddd-core/assets/render.css#--escape-price",
  symbol: "--escape-price",
  verdict_knowledge: "Undecided work is priced, not hidden: this token gives an escape a visible colour in the dashboard, so a reader sees the escape instead of reading past it.",
  binding: $b }')" | jq '{status, id, path, signs: .binding}'
```

```json
{
  "status": "filed",
  "id": "seam/htmlcss/escape-price",
  "path": ".ddd/seams/seam-htmlcss-escape-price.yaml",
  "signs": {
    "after": "sha256:9751cdf4b2c8dadfd93335f079ed0a6ffffabe729360997bd23d8918ea29845e",
    "base_revision": "75b5472c844e272c588e605c36ea5893a75f693a",
    "before": "sha256:5a8eec55f9f578217dd37fbaa6bf7d014ee3c36f89777f66db71b5bdad1c6eab",
    "file": "ddd-core/assets/render.css",
    "hash": "sha256:c1c360707204f0f66dcfaabbcdfac1c7fcb6d21b2d61e7e7c80f36bbe49d4534",
    "symbol": "--escape-price"
  }
}
```

**Say:** the facts were machine-filled. `verdict_knowledge` was not — that
sentence is the only thing in this record a human had to write, and the tool
files without it only under a warning (see Q&A, row 1).

### 2b. The sharper half — a binding signs a transition, not a state

**This is one command and it works. Do not cut it.** Same symbol, different
value — the declaration exists and still does not cover it:

```bash
# /tmp/variant.css: --escape-price: #c04fbb  (different value, same token)
./docs/demo/mcp-call.sh ddd_apply_edit \
  "$(jq -n --arg f ddd-core/assets/render.css --rawfile t /tmp/variant.css '{file:$f,new_text:$t}')" \
  | jq '{status, reason}'
```

```json
{
  "status": "rejected",
  "reason": "contract-surface change without a stored declaration signing this transition (PRD §8, M8)"
}
```

**Say:** the declaration for `--escape-price` is filed. This is still refused.
A declaration does not license a symbol; it signs one transition.

### 2c. The exact signed transition applies

```bash
./docs/demo/mcp-call.sh ddd_apply_edit \
  "$(jq -n --arg f ddd-core/assets/render.css --rawfile t /tmp/after.css '{file:$f,new_text:$t}')" \
  | jq '{status, linked, events_logged}'
```

```json
{
  "status": "applied",
  "linked": [ "seam/htmlcss/escape-price" ],
  "events_logged": [ "seam-event/49" ]
}
```

### 2d. The row on disk

The brief expected one row carrying "symbol, hashes, base revision". **It is two
files, not one** — worth knowing before you go looking on stage:

```bash
tail -8 .ddd/seams/seam-htmlcss-escape-price.yaml   # the signature
```

```yaml
bindings:
- symbol: --escape-price
  file: ddd-core/assets/render.css
  before: sha256:5a8eec55f9f578217dd37fbaa6bf7d014ee3c36f89777f66db71b5bdad1c6eab
  after: sha256:9751cdf4b2c8dadfd93335f079ed0a6ffffabe729360997bd23d8918ea29845e
  base_revision: 75b5472c844e272c588e605c36ea5893a75f693a
  hash: sha256:c1c360707204f0f66dcfaabbcdfac1c7fcb6d21b2d61e7e7c80f36bbe49d4534
```

```bash
grep -E '^(id|file|symbol|change|rule|outcome|linked_declaration):' \
  .ddd/seams/events/0049---escape-price.yaml       # the interception row
```

```yaml
id: seam-event/49
file: ddd-core/assets/render.css
symbol: --escape-price
change: added
rule: web-token-membership
outcome: applied-linked
linked_declaration: seam/htmlcss/escape-price
```

The **declaration** carries the hashes and the base revision. The **event row**
carries what the interceptor saw and which declaration discharged it.

---

## 3. Beat 3 — the bypass catch (the centrepiece)

**Wall clock: 0.05s local. Do it locally. Do not wait for CI on stage.**

### 3a. Commit the governed change and show it green

```bash
git add ddd-core/assets/render.css .ddd/seams/
git commit -q -m "Add --escape-price token, declared through the governed path"
./target/release/ddd diff-contracts HEAD~1..HEAD
```

```
== contract surface: 75b5472c844e..8ba327bbc1d7 ==
ddd-core/assets/render.css (htmlcss)
  discharged  contract/htmlcss/ddd-core/assets/render.css#--escape-price@added  [web-token-membership]
1 contract-surface event(s), 0 undischarged
```

Exit 0.

### 3b. Now bypass the tool entirely

Edit in the editor. Commit with plain `git`. The tool is never invoked.

```bash
# add "  --escape-overdue: #b3261e;" to render.css in your editor
git add -A && git commit -q -m "Add --escape-overdue token"
./target/release/ddd diff-contracts HEAD~1..HEAD
```

```
== contract surface: 8ba327bbc1d7..dc4fd389c9dd ==
ddd-core/assets/render.css (htmlcss)
  UNDISCHARGED  contract/htmlcss/ddd-core/assets/render.css#--escape-overdue@added  [web-token-membership]
undischarged: contract/htmlcss/ddd-core/assets/render.css#--escape-overdue@added — no signed binding chain composes 9751cdf4b2c8 -> 55e4c19aae3e for this file
1 contract-surface event(s), 1 undischarged
error: 1 undischarged contract-surface change(s) — file the covering declarations (signed bindings) or the change stands as a governance escape
```

Exit 1.

**Say:** same command, same classifier, two commits apart. One green, one red.
The catch does not depend on the author having used the tool — which is the only
version of this claim worth making.

### ⚠ Trap — do not run the aggregate range

`ddd diff-contracts demo-base..HEAD` reports **both** tokens UNDISCHARGED,
including the one you correctly declared:

```
  UNDISCHARGED  ...#--escape-overdue@added
  UNDISCHARGED  ...#--escape-price@added
```

This is correct behaviour — discharge is judged by **chain composition**, and the
unsigned second hop breaks the chain from the range's before-hash to its
after-hash. It is also a story-wrecker on stage. **Always `HEAD~1..HEAD`.**

### 3c. The CI path — measured, and my recommendation

Real numbers from the last green PR run (`31589596834`, 2026-08-12):

| Step | Wall clock |
|---|---|
| Job start → contract-surface gate starts | **3m 39s** |
| Contract-surface gate itself | 34s |
| Verdict available | **~4m 13s** after job start |
| Whole job | 5m 34s |

Plus queue time before the job starts.

**Recommendation: run the local command live; show CI as pre-baked evidence.**
Push the bypass commit to a PR *before* the session, so the red check is already
on screen when you want it. Four minutes of a spinning GitHub tab is the single
most likely way this hour goes wrong, and the local command is the same
classifier over the same range — you lose nothing but the logo.

---

## 4. Beat 4 — acceptance is a human act

**Wall clock: 0.02 + 0.02 + 0.16 = 0.20s.**

### 4a. Who filed, and who signed

```bash
./docs/demo/who-did-what.sh
```

```
identity                             FILED  SIGNED
noreply@anthropic.com                   88       0
claude-opus-5@anthropic.invalid          3       0
emk@delegate.dk                          0      12
```

**Say this slowly.** Ninety-one decisions filed by models. Zero signed by them.
Twelve signed by a person, who filed none of them. That split is not a
convention — the next command is why it holds.

### 4b. One acceptance, in full

```bash
./target/release/ledger blame dec:hafeok.ledger/01KZR5P5MM1WW16RXT5ERWSCWJ
```

```
dec:hafeok.ledger/01KZR5P5MM1WW16RXT5ERWSCWJ
  emk@delegate.dk signed c937a631dce1 at 2026-08-11 (live, committed by emk@delegate.dk)
```

Three things in one line: **who** (identity from git config, no `--as` flag
exists), **what** (`c937a631dce1` — a content hash of the decision version, not
its id, so revising it invalidates the signature), and **corroboration** — the
acceptance was committed by the same person who claimed it. That last clause is
gate class `L009`; it is what makes the claimed identity cost something.

### 4c. The refusal — the line that lands the thesis

```bash
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=user.email GIT_CONFIG_VALUE_0=claude@anthropic.com \
  ./target/release/ledger accept dec:hafeok.ddd/01KZTGGGS14S17F2P5K6EZEZXH
```

```
refused — the write would fail verify with:
  - [L006] acc:01KZVQ3H481P6Q690X5GQ5ZEPQ: acceptance actor `claude` carries the model-actor token `claude`
```

Exit 1. Nothing was written — verified: `.decisions/log/` stayed at 101 files
across every rehearsal run.

The `GIT_CONFIG_*` prefix borrows a model identity for exactly one command; your
real git config is untouched. Rehearsed — it works.

**Honest note on this message: it is good, not great.** See F4. It states the
fact and not the principle, and `[L006]` means nothing to the room. Supply the
sentence yourself, immediately:

> *"A model may author a decision. It may not sign one. The tool cannot tell
> whether the judgment was any good — it can only make sure a person is on the
> hook for it."*

---

## 5. Beat 5 — the swamp (fallback beat, keep under two minutes)

**Wall clock: 0.03s.** Frame: *"does this help with the code we already have?"*

Start from a lint anyone in the room has seen in a CI log:

```bash
./target/release/ddd why clippy::unwrap_used
```

```
diagnostic clippy/clippy::unwrap_used — severity deny
  governed by:
  decision dec/rust/no-unwrap — Deny clippy::unwrap_used workspace-wide
    principal: Emil (Context&)
    date: 2026-08-06
    rationale: Every unwrap is an undeclared panic path; the workspace routes fallibility through ProductError instead. Enforced twice by arrangement, not exhortation: [workspace.lints.clippy] in the root manifest and the CI clippy gate (cargo clippy -- -D warnings -D clippy::unwrap_used). ...
    basedOn:
      basis (constraint) — The workspace error model routes every failure through ProductError to a defined exit code; an unwrap is a panic path that structurally bypasses that contract, so denying the lint is what keeps the error model total.
      claim DDD-gates-01 [reported] — Denying clippy::unwrap_used workspace-wide converts every potential panic site into an explicit Result path or a reviewed expect, at negligible authoring cost in this codebase.
        falsifier: A recurring class of code where satisfying the lint produces worse error handling than the unwrap it replaced, or waiver pressure in review.
        evidence: The zero-unwrap policy has held across the whole workspace ... without a single waiver request; error paths route through ProductError.
```

**Say:** you did not start from the governance store. You started from a lint id
in a build log and arrived at a named person, a dated rationale, and the
observation that would kill the rule. Nobody wrote a wiki page.

Optional second card, also instant — the whole store's disposition:

```bash
./target/release/ledger coverage
```

```
coverage — 91 decision(s)
by set:
  ddd-governance: awaiting-acceptance 78 · escaped-priced 1
  ledger-design: decided 12
by namespace:
  hafeok.ddd: awaiting-acceptance 78 · escaped-priced 1
  hafeok.ledger: decided 12
the honest limit: coverage is measured against the enumerated set; nothing verifies the set itself — enumeration completeness has no mechanical check (PRD §8)
```

That last line is the tool disclosing its own blind spot on screen. If someone
in the room is looking for the catch, hand it to them.

---

## 6. The two numbers — verified tonight, from the record

### 6a. The M8 friction reading

From [`docs/ddd-m8-report.md`](ddd-m8-report.md) §4:

- **154** undischarged contract-surface events raised by the M8 implementation
  diff at the phase-4 mark;
- by the end, the full range carried **153 real events across 30 files** (a facts
  fix removed phantom demands — see 6b);
- discharged by **30 per-file declarations carrying 149 signed bindings**, each
  with authored verdict knowledge.

**Verified live from the store tonight**, not read off the report:

```
excluding the demo's own declaration: 30 declarations, 149 signed bindings
```

**The cost, in one sentence:** one copy-back per governed edit (the demand
pre-fills the binding), and a second governed edit to the same file must wait for
the first to be committed. That serialization is the largest real behavioural
change for an agent on the governed path.

Two corrections to how this is easy to say wrong:

- **154 and 153 are both true and are not the same number.** 154 was the
  mid-milestone reading; 153 is the final count after phantom demands were fixed.
  Quote 153, or quote both with the reason.
- **The M8 report says `ddd validate` covered 192 entries. It is 222 today** —
  the store has grown since. Run the command rather than quoting the report.

### 6b. The classifier reading

From [`docs/audits/classifier-corpus-2026-08.md`](audits/classifier-corpus-2026-08.md),
measured over the diff path:

| Language | Cases | Recall | False-demand rate |
|---|---|---|---|
| rust | 12 | 9/9 = 100% | 0/19 = 0% |
| htmlcss | 9 | 10/10 = 100% | 0/3 = 0% |

**Your summary was right: 21 hand-labelled cases, rust 12, htmlcss 9, C# and
Bicep scoped as follow-up.** Nothing to correct. Two things to attach when you
say it:

- **Cases ≠ labels.** 21 cases carry 19 surface labels and 22 non-surface
  labels. If someone asks "100% of what?", that is the answer.
- **Single-labeller provenance.** The labels were written by the session that
  maintains the classifier, before running it. That discipline is what let the
  corpus catch a real defect on first contact — phantom `signature-changed`
  events on enum members and struct fields in 6 of 12 rust cases, from a
  declaration slice that read past a terminator-less declaration. No label was
  bent to fit. But independent labelling is future work, and **say so before
  someone asks**. C#'s facts layer carries the same unbounded slicer and is named
  in the reading as the follow-up's first check.

### 6c. One falsifier, out loud

From `DDD-web-01`, phrased for a developer:

> **"If a team writes agent-authored HTML and CSS for a few months with this
> switched off, and the orphan classes and dead selectors just don't show up —
> then I built a seatbelt for a crash that doesn't happen, and this claim is
> dead."**

Two alternates if the room is more architectural:

- `DDD-adapter-01`: *a boundary defect class that no policy-table row could have
  named* — i.e. contract-surface knowledge that cannot live in a language
  adapter. Four languages in, not yet observed.
- `DDD-gates-01` (the one already on screen in beat 5): *a recurring class of
  code where satisfying the lint produces worse error handling than the unwrap it
  replaced.*

---

## 7. The fallback — if anything stalls

```bash
./target/release/ddd render          # 0.31s
open .ddd/render.html                # or: xdg-open
```

**Path:** `/home/user/product-cli/.ddd/render.html` (151 KB).

**Verified offline:** zero `<script src>`, zero `<link href>`, zero external
image references, all CSS inline. Rendered headlessly with networking disabled
and screenshotted — it draws. Sections: Claims (78) · Decisions (45) · Manifest
coverage (12 rules) · **Escapes dashboard** · **Decision ledger** · Seam map (64
declarations, 49 interception rows).

**Scroll straight past the first screen.** It opens on the claims table — dense
paragraph text, poor on a projector. The Escapes dashboard and Decision ledger
sections are the ones that read.

Generate it in pre-flight, before the room fills. `reset.sh` deletes it; the
pre-flight regenerates it.

---

## 8. The failure list — what broke, what was slow, what I cut

This section matters more than the script above.

### F1 — Rust is 400× slower than CSS, and the obvious invocation silently fails

**Severity: changes which file you edit on stage.**

The brief asked for a small `pub fn`. It works, and it costs:

| Path | Beat 1 (interceptor) | Beat 3 (diff-contracts) |
|---|---|---|
| **html-css** (hostless) | **0.07s** | **0.05s** |
| rust (rust-analyzer) | 16.7s warm · ~50s cold | 16.5–20.1s warm · **54.5s cold** |

Cold vs warm is rust-analyzer's on-disk index. First run of the evening pays 54s.

Worse: the natural one-shot invocation **does not fail loudly, it returns
nothing useful**:

```json
{ "status": "loading",
  "readiness": { "state": "loading",
    "detail": "rust is loading the workspace (waiting for experimental/serverStatus); retry shortly" } }
```

Not "rejected", not "applied" — a third status, in 0.07s, that looks like the
tool shrugged. The language host dies with the `ddd serve` process, so a
one-call-per-process helper can *never* warm it. That is why there are two
scripts: `mcp-call.sh` (hostless classes) and `mcp-session.sh` (warmup + call in
one process).

**Cut/kept:** kept the beat, **swapped the artifact to a CSS design token**. The
diff is one line and arguably more legible to a mixed room than a Rust
signature; the output shape, the rule, the demand and the binding are identical.
Rust stays documented in §1a if someone asks for code.

### F2 — the aggregate-range trap

`ddd diff-contracts demo-base..HEAD` marks your *correctly declared* change
UNDISCHARGED once an unsigned commit lands after it (chain composition, working
as designed — M8 report §4). On stage this reads as "the tool doesn't work."
**Mitigation: `HEAD~1..HEAD` only.** Flagged inline in §3.

### F3 — CI is four minutes of dead air, and it is the *late* four minutes

The contract-surface gate is step 12 of 16. It does not start until build, the
full test suite, clippy, the ledger gate and `ddd validate` have passed —
**3m 39s** on a warm-cache green run, before the gate you care about even
begins. **Cut from the live path.** Pre-push the bypass commit so the red check
is already there.

### F4 — the L006 refusal states the fact, not the principle

The single best sentence in the demo, and the tool does not say it:

```
refused — the write would fail verify with:
  - [L006] acc:01KZVQ3H481P6Q690X5GQ5ZEPQ: acceptance actor `claude` carries the model-actor token `claude`
```

It reads clean and it is not cryptic — but `[L006]` is noise to the room, the
generated `acc:` ULID is noise, and nothing on screen says *why* a model may not
sign. Someone could hear it as email validation. **Not fixed** — it is a real
message and changing it to suit a demo is exactly the wrong move. Say the
principle yourself (§4c).

Two smaller notes on the same beat: prefer `claude@anthropic.com` over
`noreply@anthropic.com` — both are refused, but `carries the model-actor token
'claude'` names the model, where `is a vendor no-reply address` sounds
administrative. And the identity floor is a floor: it catches what an agent
harness produces by default and **cannot catch a model configured with a
human-looking address**. The format spec says so; say it before someone finds it.

### F5 — the seam "row" is two files

The brief expected one seam-log row carrying symbol, hashes and base revision.
The hashes live on the **declaration**; the event row carries the symbol,
outcome and the declaration it linked to. Not a defect — but don't go hunting for
one file on stage. Corrected in §2d.

### F6 — `ledger verify` scrolls the screen

Prints 79 ULIDs, one per line. Use `ledger coverage` (4 lines, 0.01s) or
`| head -3`. Noted in §0.

### F7 — the first reset ate its own scripts

`reset.sh` originally targeted `origin/main`, which deleted `docs/demo/` once
those scripts were committed. Fixed: it resets to the `demo-base` tag and
**refuses to run** if the tag predates `docs/demo/`:

```
demo-base predates docs/demo/ — re-tag at the runbook commit
```

Verified: the guard fires.

### F8 — rehearsal pollutes `ddd report escapes`

After a rehearsal, `ddd report escapes` lists the demo tokens as UNGOVERNED
(they are detected token sources with no manifest entry). Harmless, and
`reset.sh` clears it — but if you improvise a `report escapes` mid-demo without
resetting first, your own demo tokens appear in it.

### What I did **not** do

- **No fixes.** Nothing above was patched to make a beat look better. F1 and F4
  are reported as found.
- **No acceptances filed.** Verified across every rehearsal: `.decisions/log/`
  held at 101 files throughout. Both refusal attempts wrote nothing.
- The only new files are `docs/demo-runbook.md` and `docs/demo/` (five helper
  scripts, ~100 lines total). No source, no test, no config was changed.
- **Could not verify visually through Playwright** — the MCP server looks for
  Chrome at `/opt/google/chrome/chrome`, which is not installed. Worked around it
  with the bundled Chromium headless screenshot; the dashboard is confirmed to
  render (§7).

---

## 9. Timing against the hour, and the cut order

Machine time for all five beats is **under one second total**. The hour is
narration — plan it that way.

| Beat | Machine | Suggested wall clock |
|---|---|---|
| 1 — interceptor fires | 0.07s | 4 min |
| 2 — declaration binds (a–d) | 0.14s | 8 min |
| 3 — bypass catch (a–b) | 0.10s | 10 min |
| 4 — acceptance is human (a–c) | 0.20s | 10 min |
| 5 — the swamp | 0.03s | 5 min |
| Numbers + falsifier (§6) | — | 8 min |
| Q&A (§10) | — | 12 min |
| **Total** | **~0.5s** | **~57 min** |

**Cut order, first to go:**

1. **Beat 5** — it is the fallback beat by construction. Drop it whole.
2. **Beat 2d** (the two files on disk) — the payoff already landed at 2b/2c.
3. **§6b's provenance caveat** — say the headline number, drop the labelling
   discussion. *(Keep the number itself; do not quote it without the n.)*
4. **Beat 3a** (the green half) — go straight to the bypass. You lose the
   green/red contrast, which is a real loss; cut this before beat 2b, never
   after.
5. **Beat 1a / the Rust variant** — already cut. Only run it on request, and
   only if you pre-warmed.

**Never cut:** beat 2b (the reuse refusal — one command, 0.05s, and it is the
whole idea) and beat 4c (the L006 refusal — the thesis).

---

## 10. Q&A — the true answer and the command that shows it

| Question | The true answer | Command |
|---|---|---|
| **"What stops the agent just declaring things itself?"** | **Nothing does, and that is the design.** The agent files declarations — beat 2 *is* an agent doing exactly that. It proposes; it cannot accept. Filing with empty judgment is permitted and flagged, so it lands in the PR diff for a human to read. | Beat 4c (`L006` refusal). For the warning: `ddd_declare_seam` with `verdict_knowledge: ""` → `"verdict_knowledge is empty — this boundary declares seam cost with no demand absorbed"` |
| **"What if I just don't use it?"** | Then the change is caught in the range, by the same classifier, with no cooperation from you. | Beat 3b — `ddd diff-contracts HEAD~1..HEAD`, exit 1 |
| **"What does this cost me per change?"** | One copy-back per governed edit; facts are pre-filled, only the judgment sentence is yours. Real cost on the last milestone: 153 events across 30 files → 30 declarations, 149 signed bindings. The sharp edge: two governed edits to one file serialize through a commit. | §6a; `ddd why` shows what a filed one looks like |
| **"Does this stop bad code?"** | **No.** Say it plainly. It stops *undecided* code being merged. A terrible decision, signed by a person who will answer for it, passes every gate here. | `ledger coverage` — and read its last line aloud |
| **"Can't a model just use a human's email?"** | Yes. The identity check is a floor that catches what agent harnesses produce by default; `L009` adds corroboration by requiring the acceptance's committer to match the acceptor. Neither is a cryptographic signature — the `signature` field is reserved and empty. | `ledger blame …` — the `(live, committed by …)` clause |
| **"How do you know the classifier isn't just noisy?"** | 0 false demands over 22 non-surface labels across 21 hand-labelled cases — and the corpus caught a real false-demand bug on first run, in 6 of 12 rust cases. Single-labeller provenance is the standing caveat. | `cargo test -p ddd-cli --test corpus` (do not run live) |
| **"Is anything unresolved?"** | 79 entries awaiting acceptance, one priced escape with a review date, and two questions explicitly awaiting a ruling. The queue is visible rather than assumed. | `ledger coverage`, `ledger verify \| head -3` |

---

## 11. Reset — verified

**Between the rehearsal and the live run, and again after.**

```bash
./docs/demo/reset.sh
```

```
reset: HEAD=25d9f91  dirty=0  seam-events=46
```

What it does, and why each step is needed:

1. `git reset --hard demo-base` — drops the demo commits and any applied edit.
2. `rm -f .ddd/seams/seam-htmlcss-escape-price.yaml` + `git clean -fdq
   .ddd/seams/events/` — the declaration and the interception rows are
   **untracked** after step 1, so `git reset` leaves them behind. Without this,
   beat 1 applies instead of rejecting on the second run — the demo silently
   fails.
3. `rm -f .ddd/render.html` — regenerate in pre-flight.

**Verified end to end**, against the real tag, twice: the tree was polluted
exactly as a rehearsal leaves it (rejection + declaration + applied edit +
generated dashboard, all committed), then reset, then beat 1 re-run from clean →
`"status": "rejected"` in **0.052s**. Post-reset: declaration gone, seam-events
back to 46, `render.css` unmodified, `render.html` gone, all five scripts in
`docs/demo/` intact.

Useful side-effect of that check: the two `sha256:` content hashes came back
**byte-identical** to the first rehearsal — only `base_revision` moved, tracking
`HEAD`. The hashes in §1 and §2a will match what you see on stage.

**Requires the tag**, once, at the commit that adds this runbook:

```bash
git tag -f demo-base
```

The script refuses to run if the tag predates `docs/demo/` (F7).

**Between rehearsal and live you do *not* need:** a rebuild, a cargo clean, or
any cache clear. rust-analyzer's warm index is worth keeping — it is the
difference between 54s and 17s if you run the Rust variant.
