# Arrived inputs — implement the C# stack binding

Every input is filed with its sha256 before use. Hashes are of the filed copy, which is
byte-identical to the file as it arrived in the bundle. Filing is provenance for what this session
was given; it is not canon and it makes nothing a determination.

## 1. The bundle

`f759a41f-csharp-binding-bundle.zip`, sha256
`466d66762b78ba2ecc978f2ef7096793c399070b5f290a537f14601ed4948b31`. Unpacked to
`csharp-binding-bundle/`; the tree below is filed relative to that root.

| # | Filed as | sha256 | Lines | Status |
|---|---|---|---|---|
| 1 | `prompt.md` (`session-implement-csharp-binding.md`) | `8b6188b261fe594c7247a16591f9fc62b99ec233b5dfd44509382c123c4f53ab` | 116 | **instruction set; governs** |
| 2 | `inputs/prd-csharp-stack-binding.md` | `f0049a22bcfc2c6aaaa4877e405b8cd741741c7abea3a274719700a827e80dbb` | 196 | the specification; `[PROPOSED]` by its own declaration, not ratified |
| 3 | `inputs/domain-state-change-binding/README.md` | `86079e9c64f4f40a77259d1c9b200e9cb79395d537217030f1cc6d1879ac63ae` | 117 | the binding's own README |
| 4 | `inputs/domain-state-change-binding/schema/determination.schema.json` | `4df4db09cf11d9a0cc02b4fee762fb9b71c3194dd1cb59691ccb6cba52fc67c6` | 255 | **the determination schema — used unchanged; no second schema is written** |
| 5 | `inputs/domain-state-change-binding/conformance/manifest.yaml` | `4bc78fb6e751735c50678dd8de00f074551867bc16e7ab55df3952af868b315a` | 165 | published conformance evidence, 11/11 self-certified |
| 6 | `inputs/domain-state-change-binding/examples/ordering.eventmodel.yaml` | `06d32bfa002ee0d69a22a15c09b767c80d085908bfd374936095c506c8bc87bb` | 74 | act vocabulary + fact vocabulary, the base |
| 7 | `inputs/domain-state-change-binding/examples/place-order.determinations.yaml` | `81ca01cbbdf0e7cd8dcac869467ba7a082cbf7deafbaee0bca333e7a490156dc` | 246 | determinations, build-time and act-time |
| 8 | `inputs/domain-state-change-binding/examples/fulfilment.eventmodel.yaml` | `f01036d2ffe3bca6e860c3d091e06cbbc44e781153d413681f15f61b14fc1216` | 27 | peer scope |
| 9 | `inputs/domain-state-change-binding/examples/fulfilment.determinations.yaml` | `e57e51f2f2b3db9aca36f10fad81c032d0dea6eebbf986933df96c934eb20081` | 69 | peer scope |
| 10 | `inputs/domain-state-change-binding/examples/broken.eventmodel.yaml` | `8acd9d7728cbfc5b0b50d674a2bdce5f51e9816ad384db74afd2447b5443a1ad` | 50 | negative fixture |
| 11 | `inputs/domain-state-change-binding/examples/seam-defect/fulfilment.eventmodel.yaml` | `ab9f2245b96c045688867f1169e583f2ec250a0784549220b948199861c0d209` | 30 | negative fixture |
| 12 | `inputs/domain-state-change-binding/examples/seam-defect/fulfilment.determinations.yaml` | `5b5255ca270a75e11537d09c0d6b462d47d2f32bdecd93b377e560f5841f6c41` | 76 | negative fixture |
| 13 | `inputs/domain-state-change-binding/tools/validate.py` | `829b37163797ae342488c2b0b18ccccee401254233c743f08dbec0cfef4563ef` | 29 | check |
| 14 | `inputs/domain-state-change-binding/tools/prove_prohibitions.py` | `22e59b0596b592110971fb2cc373637b205a606c338079b01a2131f9f59bef11` | 55 | check |
| 15 | `inputs/domain-state-change-binding/tools/check_resolution.py` | `47264f487c48e187b27bd74ca744afcf37bf8bd843e14112488a23f3e8e7254e` | 157 | check |
| 16 | `inputs/domain-state-change-binding/tools/check_composition.py` | `3726c9e3efdcd5e5ee634e752447d5207f14eb47e158df4c97f45bf8e441b376` | 166 | check |
| 17 | `inputs/domain-state-change-binding/tools/check_conformance.py` | `41d74d5a8457a2e4ce3da3d602ababdc7e86f1946b580088ea33143fcd4d9327` | 164 | check |
| 18 | `inputs/domain-state-change-binding/tools/run_all.sh` | `8ea152050f55a081f2c3039edb6760858de7ebdabdf5e021c58585b8e9c8ef9f` | 24 | check runner |

The PRD's hash prefix `f0049a22…` matches the one the invocation names.

## 2. Read, not filed

`Hafeok/canon-governance` at commit `c5383be06e5b181dc79307554a35cddeacbcd3e8` (head of `main`,
2026-09-02, "registry: append CG-R-10..16"). Read in full: `README.md`, `rules/README.md`,
`rules/CG-rule-01` … `CG-rule-10`, `registry/README.md`, `registry/rulings.yaml`,
`meta/sessions/README.md`, and the seeding session's `bootstrap.md` and `arrived-inputs.md` as the
model for these records. Not copied: the rules govern from their own repository, and copying them
here would be the thing `CG-rule-02`'s extraction discipline exists to prevent.

## 3. Did not arrive

| Input | Expected from | Blocks |
|---|---|---|
| The canon-governance ref (`<REF>` unsubstituted) | Emil | nothing — head taken, reported at Gate 0 for confirmation |
| The name of the brownfield solution | Emil | Gate 1 — a candidate is proposed at Gate 0 |
| Emil's §12 predictions (three, before Gate 1's measurement) | Emil | Gate 1's run — the tool may be built, not run, until they are committed |
