#!/usr/bin/env bash
# docs/demo/reset.sh — return the tree to the pre-demo state.
#
# Prerequisite, ONCE, after this runbook is committed and before the first
# rehearsal:      git tag -f demo-base
# The tag must sit at or after the commit that adds docs/demo/, or this
# script deletes its own scripts.
#
# Run between the rehearsal and the live run, and again afterwards.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

git rev-parse --verify -q demo-base >/dev/null || {
  echo "no 'demo-base' tag — run: git tag -f demo-base" >&2; exit 2; }
git merge-base --is-ancestor demo-base HEAD 2>/dev/null || true
git ls-tree -r --name-only demo-base -- docs/demo | grep -q . || {
  echo "demo-base predates docs/demo/ — re-tag at the runbook commit" >&2; exit 2; }

# 1. Drop the demo commits and any applied edit, back to the marked tip.
git reset --hard demo-base >/dev/null

# 2. Drop the declaration and the seam-event rows the interceptor wrote:
#    they are untracked after the reset, so git reset leaves them behind.
rm -f .ddd/seams/seam-htmlcss-escape-price.yaml
git clean -fdq .ddd/seams/events/

# 3. Drop the generated dashboard; the pre-flight regenerates it (0.3s).
rm -f .ddd/render.html

echo "reset: HEAD=$(git rev-parse --short HEAD)  dirty=$(git status --porcelain | wc -l)  seam-events=$(ls .ddd/seams/events/ | wc -l)"
