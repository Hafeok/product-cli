#!/usr/bin/env bash
# Walk the specification flow end to end across both runtimes, in a throwaway
# repo. Proves the seam the two halves meet at, which no single-runtime test
# can: the agent host opens records the Rust gate then refuses to let through
# until a principal closes them.
#
# Usage: scripts/checks/spec-flow-e2e.sh [workdir]
# Exit 0 conformant, 1 a step behaved unexpectedly, 2 could not run.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORK="${1:-$(mktemp -d)}"
SPEC="$ROOT/target/debug/spec"
HOST="$ROOT/spec-flow/src/SpecFlow.Cli"

step() { printf '\n── %s ──\n' "$1"; }
fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }

command -v dotnet >/dev/null || { echo "dotnet not on PATH" >&2; exit 2; }
[ -x "$SPEC" ] || { echo "build first: cargo build -p spec-cli" >&2; exit 2; }

mkdir -p "$WORK/src"
git -C "$WORK" init -q .
git -C "$WORK" config user.email emil@example.com
git -C "$WORK" config user.name Emil
cat > "$WORK/src/Api.cs" <<'CS'
namespace Shop.Api;
public class BasketController : ControllerBase
{
    [HttpPost("/baskets/{id}/settle")]
    public IActionResult Settle(string id) => Ok();
    [HttpGet("/health")]
    public IActionResult Health() => Ok();
}
CS
git -C "$WORK" add -A && git -C "$WORK" commit -q -m init

step "import (agent host, Roslyn)"
dotnet run --project "$HOST" -- import --root "$WORK" | sed -n '2p'

step "check — every entry point is unreviewed"
"$SPEC" --root "$WORK" check --ci && fail "drift must fail the gate"

step "accept — a principal names the act"
"$SPEC" --root "$WORK" accept cand/shop-api-basketcontroller-settle-httppost \
  --name "Settle a basket" \
  --settles "What the customer owes when the basket closes." \
  --principal emil@example.com | head -1

step "accept — a machine cannot"
"$SPEC" --root "$WORK" accept cand/shop-api-basketcontroller-health-httpget \
  --name "Health" --settles "Nothing." --principal ci@example.com \
  && fail "a machine must not ratify"

step "reject — a principal refuses the probe, with a reason"
"$SPEC" --root "$WORK" reject cand/shop-api-basketcontroller-health-httpget \
  --reason "A liveness probe settles nothing; it is infrastructure." \
  --principal emil@example.com | head -1

step "check — a reasoned refusal covers its entry point"
"$SPEC" --root "$WORK" check --ci || fail "the store should be clean here"

step "implement (agent host, MAF workflow) — exits 3, closure pending"
set +e
echo "-" | dotnet run --project "$HOST" -- implement \
  --slice settle-totals --act act/settle-a-basket \
  --root "$WORK" --spec "$SPEC" > "$WORK/implement.log"
IMPLEMENT_CODE=$?
set -e
tail -2 "$WORK/implement.log"
[ "$IMPLEMENT_CODE" -eq 3 ] || fail "implement must exit 3 (closure pending), got $IMPLEMENT_CODE"

step "check — the open record fails"
"$SPEC" --root "$WORK" check --ci && fail "an open record must fail the gate"

RECORD="$("$SPEC" --root "$WORK" records --open --json \
  | python3 -c 'import sys,json; print(json.load(sys.stdin)[0]["record"])')"

step "close — a machine cannot"
"$SPEC" --root "$WORK" close "$RECORD" --principal ci@example.com --nothing-arose \
  && fail "a machine must not close a record"

step "close — a principal declares that nothing arose"
"$SPEC" --root "$WORK" close "$RECORD" --principal emil@example.com --nothing-arose

step "check — green"
"$SPEC" --root "$WORK" check --ci || fail "the store should be conformant"

# ---------------------------------------------------------------------------
# Signing. Off until a key is filed; on for the whole repo once one is; and
# adopting it does not invalidate the history nobody could have signed.
# ---------------------------------------------------------------------------
KEYS="$(mktemp -d)"
trap 'rm -rf "$KEYS"' EXIT

step "trust list — signing is off"
"$SPEC" --root "$WORK" trust list | head -1

step "a secret key inside the repo is refused"
"$SPEC" --root "$WORK" trust generate --id inside --principal emil@example.com \
  --out "$WORK/secret.key" && fail "a key inside the repo must be refused"

step "adopt signing"
"$SPEC" --root "$WORK" trust generate --id emil-2026 --principal emil@example.com \
  --out "$KEYS/emil-2026.key" | head -1

step "check — the pre-adoption history is graced"
"$SPEC" --root "$WORK" check --ci || fail "adopting signing must not invalidate the past"

step "implement again, then close unsigned — refused"
set +e
echo "-" | dotnet run --project "$HOST" -- implement \
  --slice refund-totals --act act/settle-a-basket \
  --root "$WORK" --spec "$SPEC" > "$WORK/implement2.log"
set -e
RECORD2="$("$SPEC" --root "$WORK" records --open --json \
  | python3 -c 'import sys,json; print(json.load(sys.stdin)[0]["record"])')"
"$SPEC" --root "$WORK" close "$RECORD2" --principal emil@example.com --nothing-arose \
  && fail "an unsigned close must be refused once signing is on"

step "close with the key — signed"
"$SPEC" --root "$WORK" close "$RECORD2" --principal emil@example.com --nothing-arose \
  --key-file "$KEYS/emil-2026.key"

step "check — green, signed"
"$SPEC" --root "$WORK" check --ci || fail "the signed store should be conformant"

step "tampering with a signed record breaks its signature"
sed -i 's/slice: refund-totals/slice: tampered/' "$WORK/.spec/records/$RECORD2.yml"
# `check` exits 1 on findings, and `pipefail` would read that as the pipeline
# failing — so capture first, then look.
set +e
TAMPER_REPORT="$("$SPEC" --root "$WORK" check --ci)"
set -e
case "$TAMPER_REPORT" in
  *S015*) printf '%s\n' "$TAMPER_REPORT" ;;
  *) fail "S015 should catch the tamper, got: $TAMPER_REPORT" ;;
esac

printf '\nOK — the flow held end to end at %s\n' "$WORK"
