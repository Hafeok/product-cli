#!/usr/bin/env python3
"""Check determinations against an event model.

TWO CHECKS, and they answer different questions.

  ADDRESS VALIDITY
      Every determination addresses a slice the event model names, and every
      position it declares is one the slice actually has. Without this the
      address space is a convention: a determination can name a slice that
      does not exist and nothing notices.

  THE RESOLUTION CONDITION
      C-1  every fact read by an act resolves in the fact vocabulary, and is
           either written by some act in scope or declared external
      C-2  every fact written by an act resolves, and is either read by some
           act in scope or declared terminal
      C-3  every acceptance relation ranges over facts some act writes

      Incompleteness is a dangling reference. Nothing else is.

SCOPE. Evaluated at one scale, within one projection, over one declared scope.
Not composable: two specifications each internally complete may leave a
dangling reference between them, where one declares a fact terminal and the
other declares the same fact external. That check needs both specifications
and is not built.

WHAT PASSING DOES NOT ESTABLISH. Not that the act vocabulary is right, not
that any determination is true, not coverage of the act, not assurance. A
green run means no dangling references.

Usage: python3 tools/check_resolution.py <eventmodel.yaml> <determinations.yaml>
Exit:  0 resolved   1 unresolved   2 malformed input
"""

import sys
import yaml
from collections import defaultdict


def load(path):
    with open(path) as f:
        return yaml.safe_load(f)


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        return 2

    model = load(sys.argv[1])
    dets = load(sys.argv[2])

    facts = {f["id"]: f for f in model.get("facts", [])}
    slices = {(s["type"], s["name"]): s for s in model.get("slices", [])}

    written = set()
    read = set()
    for s in slices.values():
        written.update(s.get("writes", []))
        read.update(s.get("reads", []))

    # Boundary declarations are carried by determinations, aggregated per fact.
    declared = defaultdict(set)
    for d in dets:
        for pos in d.get("positions", []):
            b = pos.get("boundary", {})
            if b.get("kind") in ("external", "terminal"):
                declared[pos["fact_type"]].add(b["kind"])

    findings = []

    # --- Address validity -------------------------------------------------
    for d in dets:
        rid = d.get("id", "?")
        a = d["address"]
        key = (a["act_type"], a["act_instance"])
        if key not in slices:
            findings.append(("ADDRESS", rid,
                             f"addresses {a['act_type']}:{a['act_instance']}, "
                             f"which the event model does not name"))
            continue
        sl = slices[key]
        for pos in d.get("positions", []):
            ft, role = pos["fact_type"], pos["role"]
            declared_on_slice = sl.get("reads" if role == "read" else "writes", [])
            if ft not in facts:
                findings.append(("ADDRESS", rid,
                                 f"declares position on {ft}, not in the fact vocabulary"))
            elif ft not in declared_on_slice:
                findings.append(("ADDRESS", rid,
                                 f"declares {ft} in {role} position, but "
                                 f"{a['act_instance']} does not {role} it"))

    # --- C-1 --------------------------------------------------------------
    for ft in sorted(read):
        if ft not in facts:
            findings.append(("C-1", ft, "read but not in the fact vocabulary"))
        elif ft not in written and "external" not in declared[ft]:
            findings.append(("C-1", ft,
                             "read, written by no act in scope, and not declared external"))

    # --- C-2 --------------------------------------------------------------
    for ft in sorted(written):
        if ft not in facts:
            findings.append(("C-2", ft, "written but not in the fact vocabulary"))
        elif ft not in read and "terminal" not in declared[ft]:
            findings.append(("C-2", ft,
                             "written, read by no act in scope, and not declared terminal"))

    # --- C-3 --------------------------------------------------------------
    for d in dets:
        acc = d.get("allocation", {}).get("acceptance")
        if not acc:
            continue
        for ft in acc.get("ranges_over", []):
            if ft not in facts:
                findings.append(("C-3", d.get("id", "?"),
                                 f"acceptance ranges over {ft}, not in the fact vocabulary"))
            elif ft not in written:
                findings.append(("C-3", d.get("id", "?"),
                                 f"acceptance ranges over {ft}, which no act in scope writes"))

    # --- Report -----------------------------------------------------------
    print(f"\n  event model: {model.get('context','(unnamed)')}  "
          f"— {len(slices)} slices, {len(facts)} facts")
    print(f"  determinations: {len(dets)}\n")

    if findings:
        for kind, subj, msg in findings:
            print(f"  [FAIL] {kind:<8} {subj:<12} {msg}")
    else:
        print("  [ ok ] address validity — every determination addresses a named slice")
        print("  [ ok ] C-1 — every read fact resolves or is declared external")
        print("  [ ok ] C-2 — every written fact is read or declared terminal")
        print("  [ ok ] C-3 — every acceptance ranges over a produced fact")

    # Boundary ratio. Reported, no ceiling: a ceiling is arbitrary and turns a
    # measure into a target. A high ratio means a specification that resolves
    # little, which is what a reader needs to see.
    at_boundary = len([f for f in facts if declared[f]])
    total = len(facts)
    print(f"\n  boundary ratio: {at_boundary}/{total} facts declared at the boundary")
    if at_boundary and at_boundary / total > 0.5:
        print("  note: over half the fact vocabulary is at the boundary — this "
              "specification resolves little internally.")

    if findings:
        print(f"\n  UNRESOLVED. {len(findings)} dangling reference(s).\n")
        return 1
    print("\n  RESOLVED. No dangling references at this scope.")
    print("  Not composable across scopes: a fact terminal here may be external")
    print("  elsewhere, and nothing checks that the two agree.\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
