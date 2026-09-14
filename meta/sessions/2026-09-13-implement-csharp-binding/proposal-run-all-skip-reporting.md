# Proposal — `tools/run_all.sh` reports a check that could not run (CG-R-67)

**Target:** the domain state change binding, `tools/run_all.sh` (filed here at sha256
`8ea15205…`). **Status: `[PROPOSED]`.** The binding's repository is not attached to this
session; the patch below is against the filed copy and is not applied. Filed at Emil's direction
(CG-R-67: the same fix is owed to `run_all.sh`).

## The defect, as it stands in the filed copy

Two shapes, one class — a check that could not run reads as a check that passed:

1. **Positive lines** are `python3 tools/x.py … >/dev/null && echo "    … ok"`. Under `set -e`
   a failure inside an `&&` list does not abort the script (POSIX: `-e` is ignored for every
   command in an AND-list but the last, and here the last is `echo`). A check that crashes —
   PyYAML missing, a malformed fixture, exit 2 — prints nothing and the run continues to a
   green exit.
2. **Negative lines** are `if python3 … ; then UNEXPECTED PASS; else "caught"; fi`. Any
   non-zero exit, including exit 2 *could not run*, reads as the adverse result the fixture
   exists to produce. A crashed check is reported as discriminating power.

Both are the pattern CG-R-67 names: silence indistinguishable from green.

## The patch

Each check's exit code is read: `0` passed, `1` adverse (what a negative fixture must produce),
anything else *could not run*. A run with any *could not run* result ends non-zero and says so,
and a negative fixture "caught" only on exit `1`.

```diff
--- a/tools/run_all.sh
+++ b/tools/run_all.sh
@@
 #!/bin/sh
 # Every check this binding ships. Positive fixtures must pass; negative
 # fixtures must fail — an instrument that has never returned an adverse result
-# has untested discriminating power.
-set -e
+# has untested discriminating power. A check that could not run (exit 2, a
+# crash, a missing dependency) is reported as such, never as passed and never
+# as an adverse result: silence is not green.
 cd "$(dirname "$0")/.."
+not_run=0
+# positive <label> <message> <cmd…>: exit 0 passes; anything else fails the run.
+positive() {
+  label=$1; msg=$2; shift 2
+  "$@" >/dev/null 2>&1; rc=$?
+  case $rc in
+    0) echo "    $label $msg" ;;
+    1) echo "    $label FAILED (exit 1)"; not_run=$((not_run+1)) ;;
+    *) echo "    $label COULD NOT RUN (exit $rc)"; not_run=$((not_run+1)) ;;
+  esac
+}
+# negative <label> <message> <cmd…>: exit 1 is the adverse result the fixture
+# must produce; exit 0 is an unexpected pass; anything else could not run.
+negative() {
+  label=$1; msg=$2; shift 2
+  "$@" >/dev/null 2>&1; rc=$?
+  case $rc in
+    1) echo "    $label $msg" ;;
+    0) echo "    $label UNEXPECTED PASS"; not_run=$((not_run+1)) ;;
+    *) echo "    $label COULD NOT RUN (exit $rc) — not evidence of discriminating power"; not_run=$((not_run+1)) ;;
+  esac
+}
 echo
 echo "  POSITIVE"
-python3 tools/validate.py examples/place-order.determinations.yaml >/dev/null && echo "    schema        5/5 ordering records valid"
-python3 tools/validate.py examples/fulfilment.determinations.yaml >/dev/null && echo "    schema        2/2 fulfilment records valid"
-python3 tools/prove_prohibitions.py >/dev/null && echo "    prohibitions  7/7 forbidden shapes rejected"
-python3 tools/check_resolution.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml >/dev/null && echo "    resolution    ordering resolved"
-python3 tools/check_resolution.py examples/fulfilment.eventmodel.yaml examples/fulfilment.determinations.yaml >/dev/null && echo "    resolution    fulfilment resolved"
-python3 tools/check_composition.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml examples/fulfilment.eventmodel.yaml examples/fulfilment.determinations.yaml >/dev/null && echo "    composition   seam agrees"
-python3 tools/check_conformance.py conformance/manifest.yaml >/dev/null && echo "    criterion     11/11"
+positive "schema       " "5/5 ordering records valid"  python3 tools/validate.py examples/place-order.determinations.yaml
+positive "schema       " "2/2 fulfilment records valid" python3 tools/validate.py examples/fulfilment.determinations.yaml
+positive "prohibitions " "7/7 forbidden shapes rejected" python3 tools/prove_prohibitions.py
+positive "resolution   " "ordering resolved"   python3 tools/check_resolution.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml
+positive "resolution   " "fulfilment resolved" python3 tools/check_resolution.py examples/fulfilment.eventmodel.yaml examples/fulfilment.determinations.yaml
+positive "composition  " "seam agrees"         python3 tools/check_composition.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml examples/fulfilment.eventmodel.yaml examples/fulfilment.determinations.yaml
+positive "criterion    " "11/11"               python3 tools/check_conformance.py conformance/manifest.yaml
 echo
 echo "  NEGATIVE (each must fail)"
-if python3 tools/check_resolution.py examples/broken.eventmodel.yaml examples/place-order.determinations.yaml >/dev/null 2>&1; then
-  echo "    resolution    UNEXPECTED PASS"; exit 1
-else echo "    resolution    dangling references caught"; fi
-if python3 tools/check_composition.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml examples/seam-defect/fulfilment.eventmodel.yaml examples/seam-defect/fulfilment.determinations.yaml >/dev/null 2>&1; then
-  echo "    composition   UNEXPECTED PASS"; exit 1
-else echo "    composition   seam defect caught"; fi
+negative "resolution   " "dangling references caught" python3 tools/check_resolution.py examples/broken.eventmodel.yaml examples/place-order.determinations.yaml
+negative "composition  " "seam defect caught"         python3 tools/check_composition.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml examples/seam-defect/fulfilment.eventmodel.yaml examples/seam-defect/fulfilment.determinations.yaml
 echo
+if [ "$not_run" -gt 0 ]; then
+  echo "  NOT GREEN: $not_run check(s) failed or could not run"; echo; exit 1
+fi
+echo "  GREEN: every check ran and returned the expected result"; echo
```

The checks' own exit conventions make this work: `check_resolution.py` and
`check_composition.py` document `0 resolved / 1 unresolved / 2 malformed input`, and
`validate.py`, `prove_prohibitions.py` and `check_conformance.py` follow the same shape. The
patch relies on no other change to the binding.

## Evidence

This session's own gate runs grepped a failing check's output down to the line they were
looking for and reported green (`gate1a-run.md`, appended note 2026-09-14), and the ignore
globs in `.ddd/config.yaml` made the same silence one level up. The three instances are one
class; this patch is the binding's share of the fix.
