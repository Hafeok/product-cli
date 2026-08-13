#!/usr/bin/env bash
# docs/demo/stage-beat1.sh — stage the beat 1/2 candidate files.
#
# Writes /tmp/after.css and /tmp/variant.css. It does NOT touch the repo file:
# beat 1's whole point is that the governed file stays unchanged until a
# declaration signs the transition. Fails loudly rather than silently no-op'ing.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

python3 - <<'PY'
src, anchor = 'ddd-core/assets/render.css', '  --on-chip: #fff;'
s = open(src).read()
if anchor not in s:
    raise SystemExit(f"ANCHOR NOT FOUND in {src}.\n"
                     f"  Expected the literal line: {anchor!r}\n"
                     f"  Fix: ./docs/demo/reset.sh")
if '--escape-price' in s:
    raise SystemExit(f"{src} already carries --escape-price — the tree was not reset.\n"
                     f"  Fix: ./docs/demo/reset.sh")
for out, tok in (('/tmp/after.css', '#6b4fbb'), ('/tmp/variant.css', '#c04fbb')):
    open(out, 'w').write(s.replace(anchor, f'  --escape-price: {tok};\n' + anchor, 1))
    print(f'wrote {out}  (+1 line)')
PY

# The interceptor refuses to bind uncommitted parent state (M8 ruling 2).
if ! git diff --quiet -- ddd-core/assets/render.css; then
  echo "WARNING: ddd-core/assets/render.css is dirty against HEAD." >&2
  echo "  Beat 1 will reject with a dirty-parent reason, not the demand." >&2
  echo "  Fix: git checkout -- ddd-core/assets/render.css   (or ./docs/demo/reset.sh)" >&2
  exit 1
fi
echo "staged — repo file untouched (that is the point); run beat 1 now"
