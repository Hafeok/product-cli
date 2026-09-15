# Arrived inputs — Gate 1b, entry-point candidates and the delta

Every input is filed with its sha256 before use. Hashes are of the filed copy, byte-identical to
the file as it arrived. Filing is provenance; it is not canon and makes nothing a determination.

## 1. The bundle

`58754e03-gate1b-bundle.zip`, sha256
`1f5198bbffe4d42d37ae9dea1c6dfbbeec502c38f0feb39f3359481c1abf7959`. Two files inside, filed
relative to this directory:

| # | Filed as | sha256 | Lines | Status |
|---|---|---|---|---|
| 1 | `prompt.md` (`session-gate1b-candidates.md`) | `f067201ded580355b3afa8f012965d233b17acb9414db29d3def6af85f8fcd84` | 87 | **instruction set; governs** |
| 2 | `inputs/rulings-cg-r-105-110.md` | `565cc7cc7fac37e26732452715646958ffb2a83e5c7fe85075a3031c914006df` | 127 | **rulings in force** — CG-R-105 … CG-R-109; CG-R-110 `[PROPOSED]` by its own marking |

## 2. The same rulings file, attached separately

`460cfe40-rulings-cg-r-105-110.md`, sha256 `565cc7cc…` — byte-identical to the bundle's copy.
One file, two arrival paths; filed once.

## 3. Hash on arrival, as instructed

The invocation cites the rulings file as `6e0992b1…` and says it has since been extended to
CG-R-110, to be re-hashed on arrival. **Received: `565cc7cc7fac37e26732452715646958ffb2a83e5c7fe85075a3031c914006df`.**
The `6e0992b1…` version (CG-R-105 … CG-R-108 only, by the file's own title) was never received
here; only the extended file arrived, and the hash above is the one this session is bound by.

## 4. Discrepancies in what arrived, recorded not corrected

- The file's title reads "CG-R-105 … CG-R-108" and its closing register line reads
  "CG-R-17 … CG-R-108"; the body carries CG-R-109 and CG-R-110. The extension left the title
  and the register line behind. Filed as received.
- The prompt cites **CG-R-101** (the table ground truth) and **CG-R-103** (the instrument
  closed). **Rulings CG-R-99 … CG-R-104 did not arrive** in this session and are not in the
  prior session's records (`grep` over `meta/` finds no CG-R-99 … CG-R-104). This session
  complies with CG-R-103 as the prompt states it — the instrument is closed, a defect is
  reported not repaired — without having read the ruling's own text. Reported at Gate A.

## 5. Read, not filed

- `meta/sessions/2026-09-13-implement-csharp-binding/` — the prior session's records, in
  particular `reader-emission-rules-v5.md`, `gate1a-run8.md`, the rulings CG-R-57 … CG-R-98.
- The run-8 A inventory, `eshop-inventory-v6.json`, sha256
  `3197d7e07c5a0b46c7c48c4f71fe3912a6493c091184e1219fc0546f9f078451` — the same artefact run 8
  measured (`gate1a-measurement/run8-instrument.txt`); measurement, not committed.
- eShopOnWeb at `03d8cff` (the run-8 checkout) — read only for the hand enumeration of entry
  points that checks the reader's recall (CG-R-71's form).

## 6. Did not arrive

| Input | Expected from | Blocks |
|---|---|---|
| Rulings CG-R-99 … CG-R-104 | Emil | nothing — complied with as the prompt restates them |
| Ratification of the candidate set | Emil | Gate B |
| Slice declarations against accepted candidates | Emil (naming is ratification, CG-R-106) | Gate B |
