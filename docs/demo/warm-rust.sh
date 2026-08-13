#!/usr/bin/env bash
# docs/demo/warm-rust.sh — build rust-analyzer's on-disk index so the §1a
# Rust variant costs ~17-21s instead of ~31s if someone asks for it.
#
# What this does and does not buy you (F1):
#   - The language HOST dies with the `ddd serve` process. Nothing stays warm
#     in memory between calls, and no amount of warming makes the one-shot
#     `mcp-call.sh` work on a Rust file — it still returns {"status":"loading"}
#     in 0.1s. Always use mcp-session.sh for Rust. Verified with a warm index.
#   - The on-disk INDEX does persist, across processes and across reset.sh
#     (which never touches target/). That is the 31s -> 17-21s difference.
#
# Run this BEFORE reset.sh: the warm-up performs a real rejection, and every
# rejection writes a seam-event row. Warm, then reset, and the count is clean
# while the index stays warm.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

SRC=ddd-core/src/configured.rs

git diff --quiet -- "$SRC" || {
  echo "$SRC is dirty against HEAD — the interceptor would reject on a dirty" >&2
  echo "  parent rather than exercising the classifier. Fix: git checkout -- $SRC" >&2
  exit 1; }

python3 - <<PY
src = "$SRC"
s = open(src).read()
if 'is_silent_waiver' in s:
    raise SystemExit(f"{src} already carries is_silent_waiver — tree not reset.\n"
                     f"  Fix: ./docs/demo/reset.sh")
open('/tmp/after.rs', 'w').write(s + '''
/// Whether this rule was disabled without a citation on record.
pub fn is_silent_waiver(rule: &ConfiguredRule) -> bool {
    rule.disabled && rule.scope.is_none()
}
''')
print('staged /tmp/after.rs (+5 lines) — repo file untouched')
PY

echo "warming rust-analyzer (first run pays the index build; ~31s cold)…"
start=$(date +%s)
status=$(./docs/demo/mcp-session.sh ddd_apply_edit \
  "$(jq -n --arg f "$SRC" --rawfile t /tmp/after.rs '{file:$f,new_text:$t}')" \
  | jq -r '.status')
elapsed=$(( $(date +%s) - start ))

[ "$status" = "rejected" ] || {
  echo "expected 'rejected', got '$status' after ${elapsed}s — §1a is NOT ready" >&2
  exit 1; }

echo "warm: rejected in ${elapsed}s. Expect ~17-21s if you run §1a on stage."
echo "Now run ./docs/demo/reset.sh — this warm-up logged a seam-event row."
