---
id: ADR-032
title: Content Hash Immutability Enforcement
status: proposed
features: [FT-034]
supersedes: []
superseded-by: []
domains: []
scope: domain
source-files:
  - src/graph.rs
  - src/parser.rs
  - src/types.rs
  - src/main.rs
---

**Status:** Proposed

**Context:** Product manages long-lived specification artifacts — ADRs, test criteria, features, and dependencies — whose content forms the authoritative basis for agent-driven implementation. An implementing agent that silently modifies an accepted ADR's rationale while working on a feature can invalidate reasoning that other agents and humans depend on. Similarly, if a TC's formal specification blocks or ADR linkage change after the TC is established, the test no longer validates what it claims to validate. There is currently no mechanism to detect these mutations.

Not all fields have the same mutability profile. ADR status, links, and domains change throughout the lifecycle. But the body text and title of an accepted ADR represent a frozen decision — the correct action for changing them is a superseding ADR, not an edit. The same applies to a TC's body, type, and `validates.adrs` — if what a TC validates changes fundamentally, it should be a new TC.

The system needs a mechanism that (1) detects unauthorized mutations to protected content, (2) provides a legitimate amendment path for genuine corrections like typos, and (3) creates an auditable trail of all amendments.

**Decision:** Introduce a `content-hash` field in ADR and TC front-matter. The hash is computed over protected content at the moment of acceptance (ADRs) or first hash set (TCs). `product graph check` verifies the hash on every run and emits hard errors (E014, E015) on mismatch. A `product adr amend` command provides the legitimate amendment path with a mandatory reason and audit trail.

---

### Mutability Matrix

Not all fields on all artifacts have the same mutability profile.

**ADR — mutable always:**
- `status` — lifecycle transitions (proposed -> accepted -> superseded)
- `superseded-by`, `features`, `domains`, `scope`, `source-files` — links can always be added or removed

**ADR — immutable once `status: accepted`:**
- Body text (Context, Decision, Rationale, Rejected alternatives sections)
- `title`

If someone rewrites the rationale of an accepted ADR, they are potentially invalidating reasoning that agents and humans are relying on. The correct action is a superseding ADR, not an edit.

**TC — mutable always:**
- `status`, `last-run`, `failure-message`, `last-run-duration` — `product verify` writes these
- `validates.features` — `migrate link-tests` writes this
- `runner`, `runner-args`, `runner-timeout`, `requires` — infrastructure details can change

**TC — immutable once hash is set:**
- Body text (description, formal specification blocks)
- `type`
- `validates.adrs`

If what a TC validates changes fundamentally, it is a new TC.

**Feature — mostly mutable:**
- `status`, links, `domains-acknowledged` change constantly
- `id` and `title` — immutable once set (permanent ID, stable title)

**Dependency — mostly mutable:**
- `version`, `interface`, `availability-check`, `status` — mutable
- `id` and `title` — immutable once set

Feature and dependency immutability is enforced by convention, not by content-hash. The hash mechanism targets ADRs and TCs where unauthorized mutation has the highest impact.

---

### Content Hash Mechanism

#### Hash Computation

The hash is computed over:
- **Body text**: everything after the closing `---` of the YAML front-matter, normalized (LF line endings, leading/trailing whitespace trimmed)
- **Protected front-matter fields**: `title` for ADRs; `title`, `type`, `validates.adrs` for TCs

The following fields are explicitly **excluded** from the hash input: `content-hash`, `amendments`, `status`, `features`, `supersedes`, `superseded-by`, `domains`, `scope`, `source-files`, `last-run`, `failure-message`, `last-run-duration`, `runner`, `runner-args`, `runner-timeout`, `requires`, `validates.features`, `phase`.

Hash algorithm: SHA-256, hex-encoded, prefixed with `sha256:`.

```
sha256:a3f9b2c1d4e5f67890abcdef1234567890abcdef1234567890abcdef12345678
```

#### When the Hash is Written

**ADR:** `product adr status ADR-XXX accepted` computes and writes the hash at the moment of acceptance. Before acceptance, the ADR is a draft and can be freely edited. The hash field is not present in draft ADRs.

**TC:** `product hash seal TC-XXX` computes and writes the hash. This is a manual step — TCs may exist in draft form for some time before their specification is finalized. Alternatively, `product hash seal --all-unsealed` seals all TCs that have a body and no existing hash.

#### When the Hash is Checked

`product graph check` verifies all content-hashes on every run:
- If an ADR has `status: accepted` and a `content-hash` field, recompute and compare. Mismatch emits **E014**.
- If an ADR has `status: accepted` and **no** `content-hash` field, emit **W016** (accepted ADR without content-hash — run `product adr rehash ADR-XXX` to seal it).
- If a TC has a `content-hash` field, recompute and compare. Mismatch emits **E015**.
- TCs without a `content-hash` field are not checked (they are unsealed drafts).

---

### Error Codes

| Code | Tier | Description |
|---|---|---|
| E014 | Integrity | ADR body or title changed after acceptance — content-hash mismatch |
| E015 | Integrity | TC protected fields changed (type, validates.adrs, or body) — content-hash mismatch |
| W016 | Warning | Accepted ADR has no content-hash — seal with `product adr rehash` |

Both E014 and E015 are exit code 1 hard errors. The error message names the file, shows the expected vs actual hash, and tells the developer to either revert the change or run `product adr amend` (for ADRs) or create a new TC (for TCs).

```
error[E014]: content-hash mismatch — accepted ADR body or title was modified
  --> docs/adrs/ADR-002-openraft-for-cluster-consensus.md
   | content-hash: sha256:a3f9b2c1... (expected)
   | recomputed:   sha256:7d8e9f0a... (actual)
   = hint: revert the change, or run `product adr amend ADR-002 --reason "..."` to record a legitimate amendment
```

```
error[E015]: content-hash mismatch — sealed TC body or protected fields were modified
  --> docs/tests/TC-002-raft-leader-election.md
   | content-hash: sha256:b4c5d6e7... (expected)
   | recomputed:   sha256:1a2b3c4d... (actual)
   = hint: revert the change, or create a new TC if the specification has fundamentally changed
```

---

### Amendment Path

Sometimes a typo genuinely needs fixing in an accepted ADR. The correct path:

```bash
product adr amend ADR-002 --reason "Fix typo: 'openraft' misspelled as 'openrat' in rationale"
```

This command:
1. Verifies ADR-002 has `status: accepted` and a `content-hash` field
2. Recomputes the hash from the current file content
3. If the hash matches (no change), exits with a message: "nothing to amend"
4. If the hash differs, records the amendment in the `amendments` array
5. Updates `content-hash` to the new value
6. Writes the file atomically (ADR-015)

The amendment record:

```yaml
amendments:
  - date: 2026-04-14T09:00:00Z
    reason: "Fix typo: 'openraft' misspelled as 'openrat' in rationale"
    previous-hash: sha256:a3f9b2c1...
```

The `--reason` flag is mandatory. Amendment without a reason is rejected. This creates an immutable audit trail — anyone can see exactly what changed and why.

For TCs, there is no amend command. If a sealed TC's specification changes, the correct action is to create a new TC. The old TC should be marked with `status: abandoned` or left as-is if the change is a supersession.

---

### New CLI Commands

```
product adr amend ADR-XXX --reason "..."   # record amendment, recompute hash
product hash seal TC-XXX                    # compute and write content-hash for a TC
product hash seal --all-unsealed            # seal all TCs without a content-hash
product hash verify [ARTIFACT-ID]           # verify one or all content-hashes (subset of graph check)
product adr rehash ADR-XXX                  # seal an accepted ADR that predates this feature
product adr rehash --all                    # seal all accepted ADRs without content-hash
```

`product hash verify` is a focused subset of `product graph check` — it only checks content-hashes, useful in CI pipelines that want a fast integrity check without the full graph validation.

`product adr rehash` is a migration tool for accepted ADRs that predate the content-hash feature. It computes the hash from the current content and writes it. This is distinct from `adr amend` because there is no previous hash to record — it is the initial sealing.

---

### MCP Implications

**Read tools:** No changes. `product_graph_check` already runs `graph check` — it will now include E014/E015.

**Write tools:** Add `product_adr_amend` as a write tool. There must be no MCP tool that writes to an accepted ADR's body directly. `product_adr_status` is fine — it only touches `status` in front-matter (and writes `content-hash` when the new status is `accepted`). `product_feature_link` is fine — it only touches `features` in front-matter, which is excluded from the hash.

If an agent modifies the body of an accepted ADR while implementing a feature, E014 fires on the next `product graph check`. This is the key protection against agent drift.

---

### Front-Matter Schema Changes

**ADR (new optional fields):**

```yaml
content-hash: sha256:...        # computed on acceptance, verified by graph check
amendments:                     # audit trail for legitimate post-acceptance edits
  - date: 2026-04-14T09:00:00Z
    reason: "description of change"
    previous-hash: sha256:...
```

**TC (new optional field):**

```yaml
content-hash: sha256:...        # computed by `product hash seal`, verified by graph check
```

Both fields are optional — their absence means the artifact is unsealed. For accepted ADRs, absence triggers W016.

---

**Rationale:**
- Content-hash verification is the standard pattern for detecting unauthorized modification. Git uses SHA-1/SHA-256 for exactly this purpose at the commit level; this extends it to the artifact level within the repository.
- The amendment path with mandatory reason preserves the ability to fix genuine errors (typos, factual corrections) while creating accountability. A rewritten rationale without an amendment record is a red flag; a rewritten rationale with "Fix typo in section heading" is a transparent correction.
- Separating E014 (ADRs) and E015 (TCs) allows different remediation paths: ADRs can be amended, TCs should be replaced. The error messages encode this guidance directly.
- W016 for accepted ADRs without hashes provides a migration path — existing repos can adopt this feature incrementally without breaking their `graph check`.
- SHA-256 is chosen for collision resistance and consistency with the `sha2` crate already in the dependency tree (used for gap ID generation per ADR-019).

**Rejected alternatives:**
- **Git hooks only** — a pre-commit hook could diff accepted ADRs and reject changes. Rejected because it is bypassable (`--no-verify`), does not create an audit trail, and does not protect against agents that modify files programmatically without going through git.
- **Immutable files (chmod, chattr)** — filesystem-level protection. Rejected because it breaks normal editor workflows, does not work across all platforms, and is not portable to CI environments.
- **Hash over entire file including front-matter** — simpler but wrong. Status changes, link additions, and measurement updates would all trigger false positives. The protected-fields-only approach allows the mutable parts to change freely.
- **Amend command for TCs** — considered but rejected. TC amendments create version confusion: which version of the specification does a passing test validate? If the specification changes, it is a new test. ADRs are different because the decision itself is stable; only presentation errors (typos) need fixing.
- **Automatic hash on first write** — compute the hash when the file is first created. Rejected because ADRs go through a draft phase where content should change freely. The hash should be set at the moment of acceptance, not creation.

**Test coverage:**
- TC-420: Hash computed and written when `product adr status ADR-XXX accepted` is run
- TC-421: E014 emitted when accepted ADR body is modified
- TC-422: E015 emitted when sealed TC body or protected fields are modified
- TC-423: `product adr amend` records amendment and recomputes hash
- TC-424: W016 emitted for accepted ADR without content-hash
- TC-425: MCP write tools cannot modify accepted ADR body
- TC-426: `product hash seal` computes and writes TC content-hash
- TC-427: `product hash verify` checks content-hashes independently of full graph check
- TC-428: `product adr rehash` seals accepted ADRs that predate this feature
- TC-429: Mutable front-matter fields (status, features, domains) do not affect content-hash
- TC-430: Exit criteria — full system passes on sealed repository
