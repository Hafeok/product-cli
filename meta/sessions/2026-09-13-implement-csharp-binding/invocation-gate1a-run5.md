# Invocation — rulings CG-R-71 … CG-R-75, verbatim

Received 2026-09-14 with `rulings-cg-r-71-75.md`, filed beside it as `rulings-gate1a-run5.md`
(sha256 in the commit that files it).

---

## The message

Read the full report. Both figures measure the instrument, not the codebases — and there's a
pattern across five runs that needs breaking rather than another ruling.

The loop is the finding, not the four defects. Five runs, four changes to the denominator or
emission rules, each discovered after numbers existed, each voiding what came before. Another
ruling on another defect continues it.

The report names the cause exactly, about the marker proxy: a proxy whose test case is the one
shape the field never shows is not a proxy for the field. That generalises to every emission rule
and every proxy in the instrument — all tested against a fixture authored by the party that wrote
the rule. It's the category-list independence defect, in the reader, and a fixture structurally
cannot catch an emission gap.

So: hand-verified ground truth on one small real project before any further prediction.
Enumerate composition edges in A's Web project by reading source, before looking at the reader's
output, both directions. Recall and precision, reported with every coverage figure afterwards. A
resolver following 80% of the edges it was given means nothing if the reader gave it half.

Two things the headline hides. B's 79.5% is 504 of 1,450 composition edges — 35% — with 333
partial outside the fraction. And B's own table is inconsistent: in denominator reads 967, the
fraction uses 634.

On P-2 I've added a carve-out. UserManager<T> is registered by AddIdentity, which the reader
ignores. Calling it boundary would relabel a reader gap as a legitimate category and improve the
number for the wrong reason. Fifth state: registration-not-read, with the call recorded. The 58
and 1,554 ignored calls are the map of what the registration reader still has to learn.

And my own record. My prediction's premise was wrong — A uses source-generated Mediator, not
MediatR, and the labels showed it from run 2. That's the second time I've put an unchecked
assertion into a ruling; the 78% hand estimate was the first. Recorded rather than mentioned.

---

## Order the rulings set

1. CG-R-71's ground truth (A's `Web` project, by reading source, both directions, recall and
   precision).
2. P-4, then P-1 with the proxy re-examination, then P-2 (with the `registration-not-read`
   carve-out) and P-3 (test projects by project, mechanical).
3. Restate the emission rules in full.
4. Both predictions, committed against the restated rules. CG-R-70 is void.
5. Run 6. 6. §12.1 on run 6.

Recorded by the session; the rulings are Emil's.
