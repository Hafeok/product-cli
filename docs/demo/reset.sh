#!/usr/bin/env bash
# docs/demo/reset.sh — return the tree to the pre-demo state.
#
# Prerequisite, after every change to the demo material:   git tag -f demo-base
# This script resets to that tag, so anything under docs/demo/ or
# docs/demo-runbook.md that the tag does not carry is DESTROYED. The guard
# below refuses rather than letting that happen (F7).
#
# Run between the rehearsal and the live run, and again afterwards.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

DEMO_PATHS=(docs/demo docs/demo-runbook.md)

git rev-parse --verify -q demo-base >/dev/null || {
  echo "no 'demo-base' tag — run: git tag -f demo-base" >&2; exit 2; }

git ls-tree -r --name-only demo-base -- docs/demo | grep -q . || {
  echo "demo-base predates docs/demo/ — re-tag at the runbook commit" >&2; exit 2; }

# The tag must carry the CURRENT demo material, not merely some of it. The
# old guard only asked whether docs/demo existed at the tag, so a tag that
# predated a newly added script passed and the reset deleted that script.
stale=$(git diff --name-only demo-base HEAD -- "${DEMO_PATHS[@]}")
if [ -n "$stale" ]; then
  echo "demo-base is stale — resetting to it would revert demo material:" >&2
  printf '  %s\n' $stale >&2
  echo "  Fix: git tag -f demo-base   (with the demo files committed)" >&2
  exit 2
fi

# Same hazard, uncommitted: a reset --hard would discard live edits to the
# runbook or the scripts.
dirty=$(git status --porcelain -- "${DEMO_PATHS[@]}")
if [ -n "$dirty" ]; then
  echo "uncommitted changes to demo material — reset would discard them:" >&2
  printf '  %s\n' "$dirty" >&2
  echo "  Fix: commit them, then: git tag -f demo-base" >&2
  exit 2
fi

# 1. Drop the demo commits and any applied edit, back to the marked tip.
git reset --hard demo-base >/dev/null

# 2. Drop the declaration and the seam-event rows the interceptor wrote:
#    they are untracked after the reset, so git reset leaves them behind.
rm -f .ddd/seams/seam-htmlcss-escape-price.yaml
git clean -fdq .ddd/seams/events/

# 3. Drop the generated dashboard; the pre-flight regenerates it (0.3s).
rm -f .ddd/render.html

echo "reset: HEAD=$(git rev-parse --short HEAD)  dirty=$(git status --porcelain | wc -l)  seam-events=$(ls .ddd/seams/events/ | wc -l)"
