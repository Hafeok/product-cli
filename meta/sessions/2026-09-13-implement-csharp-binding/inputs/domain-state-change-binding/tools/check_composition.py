#!/usr/bin/env python3
"""Check that two internally-resolved specifications agree at their seam.

WHY THIS EXISTS. The resolution condition is scope-relative. Two specifications
can each pass it and still leave a dangling reference between them: A declares
a fact terminal, B declares the same fact external, and nobody checked that
what A produces is what B expects. That gap is exactly the seam where
divergence concentrates — recurrent, and touched by two actor populations —
which is why this is the highest-value check available and why the boundary
declaration fields were designed not to foreclose it.

WHAT IT CHECKS

  CS-1  every fact A declares terminal, naming B as consumer, is declared
        external in B naming A as source, and is read by some act in B
  CS-2  every fact B declares external, naming A as source, is written by
        some act in A and declared terminal there
  CS-3  the fact kind agrees across both vocabularies
  CS-4  a fact A declares as having unobservable consumption, while B reads it
        within the composition, is an inconsistent pair — B's read is the
        observation A says does not exist

WHAT IT DOES NOT CHECK. That the fact means the same thing on both sides. Two
specifications can agree on a name, a kind, and a boundary and still hold
incompatible notions of what the object is. That residual is checked by
reading, by a named principal, and no instrument replaces it.

Usage:
  python3 tools/check_composition.py A.eventmodel.yaml A.determinations.yaml \\
                                     B.eventmodel.yaml B.determinations.yaml
Exit: 0 composed   1 seam defect   2 malformed input
"""

import sys
import yaml
from collections import defaultdict


def load_spec(model_path, det_path):
    model = yaml.safe_load(open(model_path))
    dets = yaml.safe_load(open(det_path))
    facts = {f["id"]: f for f in model.get("facts", [])}
    reads, writes = set(), set()
    for s in model.get("slices", []):
        reads.update(s.get("reads", []))
        writes.update(s.get("writes", []))
    boundaries = defaultdict(dict)
    for d in dets:
        for pos in d.get("positions", []):
            b = pos.get("boundary", {})
            if b.get("kind") in ("external", "terminal"):
                boundaries[pos["fact_type"]][b["kind"]] = b
    return {
        "name": model.get("context", "(unnamed)"),
        "facts": facts, "reads": reads, "writes": writes,
        "boundaries": boundaries,
    }


def check(a, b):
    findings = []

    def seam_name(decl, other):
        """A boundary declaration names the other scope if its source or
        consumer field mentions that scope's name."""
        txt = (decl.get("source") or "") + " " + (decl.get("consumer") or "")
        return other["name"].lower() in txt.lower()

    # CS-1 — A terminal -> B external
    for ft, kinds in a["boundaries"].items():
        term = kinds.get("terminal")
        if not term or not seam_name(term, b):
            continue
        ext = b["boundaries"].get(ft, {}).get("external")
        if ft not in b["facts"]:
            findings.append(("CS-1", ft,
                             f"{a['name']} declares it terminal to {b['name']}, "
                             f"but {b['name']} has no such fact"))
        elif not ext:
            findings.append(("CS-1", ft,
                             f"{a['name']} declares it terminal to {b['name']}, "
                             f"but {b['name']} does not declare it external"))
        elif not seam_name(ext, a):
            findings.append(("CS-1", ft,
                             f"{b['name']} declares it external but names "
                             f"'{ext.get('source')}' rather than {a['name']}"))
        elif ft not in b["reads"]:
            findings.append(("CS-1", ft,
                             f"{b['name']} declares it external but no act "
                             f"in {b['name']} reads it"))
        # CS-4
        if term.get("consumption_observable") is False and ft in b["reads"]:
            findings.append(("CS-4", ft,
                             f"{a['name']} declares consumption unobservable, "
                             f"but {b['name']} reads it inside this composition"))

    # CS-2 — B external -> A terminal
    for ft, kinds in b["boundaries"].items():
        ext = kinds.get("external")
        if not ext or not seam_name(ext, a):
            continue
        if ft not in a["facts"]:
            findings.append(("CS-2", ft,
                             f"{b['name']} declares it external from {a['name']}, "
                             f"but {a['name']} has no such fact"))
        elif ft not in a["writes"]:
            findings.append(("CS-2", ft,
                             f"{b['name']} expects it from {a['name']}, but no "
                             f"act in {a['name']} writes it"))
        elif "terminal" not in a["boundaries"].get(ft, {}):
            findings.append(("CS-2", ft,
                             f"{b['name']} expects it from {a['name']}, but "
                             f"{a['name']} does not declare it terminal"))

    return findings


def check_kinds(a, b):
    """CS-3 is symmetric, so it runs once rather than in both directions."""
    out = []
    for ft in sorted(set(a["facts"]) & set(b["facts"])):
        ka, kb = a["facts"][ft].get("kind"), b["facts"][ft].get("kind")
        if ka != kb:
            out.append(("CS-3", ft,
                        f"kind disagrees: {a['name']} says {ka}, "
                        f"{b['name']} says {kb}"))
    return out


def main():
    if len(sys.argv) != 5:
        print(__doc__)
        return 2
    a = load_spec(sys.argv[1], sys.argv[2])
    b = load_spec(sys.argv[3], sys.argv[4])

    findings = check(a, b) + check(b, a) + check_kinds(a, b)
    # dedupe, preserving order
    seen, uniq = set(), []
    for f in findings:
        if f not in seen:
            seen.add(f)
            uniq.append(f)

    shared = sorted(set(a["facts"]) & set(b["facts"]))
    print(f"\n  composing: {a['name']} × {b['name']}")
    print(f"  shared facts: {', '.join(shared) if shared else '(none)'}\n")

    if uniq:
        for kind, ft, msg in uniq:
            print(f"  [FAIL] {kind:<6} {ft:<18} {msg}")
        print(f"\n  SEAM DEFECT. {len(uniq)} finding(s).\n")
        return 1

    print("  [ ok ] CS-1 — every terminal fact is received where it is sent")
    print("  [ ok ] CS-2 — every external fact is produced where it is expected")
    print("  [ ok ] CS-3 — fact kinds agree across both vocabularies")
    print("  [ ok ] CS-4 — no fact is declared unobserved while being read here")
    print("\n  COMPOSED. No dangling references across the seam.")
    print("  Agreement on name, kind and boundary is not agreement on meaning;")
    print("  that residual is checked by reading.\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
