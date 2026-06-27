#!/usr/bin/env bash
# session-e2e.sh — drive a fresh `init --demo` repo through each phase skill's
# worked example end-to-end via the CLI, asserting each phase's gate. Keeps the
# product-what / product-how / product-build skills honest as the tools evolve.
#
# Phases are not gated here (the CLI has no session lock), so this exercises the
# underlying capability each skill relies on. Run it after changing the tool
# surface: `bash scripts/checks/session-e2e.sh`.
set -uo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
PRODUCT="${PRODUCT:-$ROOT/target/debug/product}"
[ -x "$PRODUCT" ] || { echo "building product…"; (cd "$ROOT" && cargo build -q) || exit 1; }

T="$(mktemp -d "${TMPDIR:-/tmp}/product-session-e2e.XXXXXX")"
trap 'rm -rf "$T"' EXIT
cd "$T"

fails=0
run() { # run "label" cmd...
  local label="$1"; shift
  if "$@" >/tmp/e2e.out 2>&1; then
    printf '  \033[32mPASS\033[0m %s\n' "$label"
  else
    printf '  \033[31mFAIL\033[0m %s\n' "$label"; sed 's/^/        /' /tmp/e2e.out | tail -4; fails=$((fails + 1))
  fi
}
assert_file() { # assert_file "label" file pattern
  if grep -q "$3" "$2"; then printf '  \033[32mPASS\033[0m %s\n' "$1"
  else printf '  \033[31mFAIL\033[0m %s\n' "$1"; fails=$((fails + 1)); fi
}
phase() { printf '\n\033[1m== %s ==\033[0m\n' "$1"; }

phase "init --demo — seed the What"
run "init --demo"             "$PRODUCT" init --demo --yes --name bookstore

phase "product-what — domain + event model (§3)"
run "domain validate"         "$PRODUCT" domain validate
run "domain new value-object" "$PRODUCT" domain new value-object vo-isbn --label ISBN --context Catalog --definition "a book identifier"
run "domain validate (after)" "$PRODUCT" domain validate

phase "product-how — the §4 architecture"
run "how init"                "$PRODUCT" how init
run "how add decision"        "$PRODUCT" how add decision d-lang --decision "Use Rust" --rationale "safety + zero-cost" --licenses zero-unwrap
run "how add principle"       "$PRODUCT" how add principle zero-unwrap --statement "no unwrap in non-test code" --licensed-by d-lang --enforced-by clippy-gate
run "how add pattern"         "$PRODUCT" how add pattern slice-adapter --shape "pure slice + thin adapter" --realizes zero-unwrap
run "how set app-contract"    "$PRODUCT" how set app-contract --id bookstore-app --language Rust
run "how validate"            "$PRODUCT" how validate
run "archetype init"          "$PRODUCT" archetype init bookstore
run "cell init"               "$PRODUCT" cell init order-impl
run "cell dispatch → units"   "$PRODUCT" cell dispatch --bind entity=Order

phase "product-build — slices, deliverables, build (§5–7)"
run "slice new"               "$PRODUCT" slice new place-order --anchor PlaceOrder --anchor OrderPlaced --anchor Order
run "deliverable new"         "$PRODUCT" deliverable new place-order --slice place-order --accept "handler-exists: a PlaceOrder handler writes OrderPlaced for Order"
run "deliverable runner"      "$PRODUCT" deliverable runner place-order handler-exists --runner cargo-test --args "tc_place_order_handler"
run "build --dry-run"         "$PRODUCT" build place-order --dry-run

phase "assertions"
"$PRODUCT" build place-order --dry-run >"$T/build.out" 2>&1 || true
assert_file "build run-plan uses the dispatched work unit (handler-order)" "$T/build.out" "handler-order"
assert_file "build gate reports domain conformance" "$T/build.out" "domain Order: conformant"
assert_file "acceptance criterion carries its bound runner (cargo test)" "$T/build.out" "handler-exists: cargo test"
grep -q "handler-order" "$T/build.out" || { echo "  --- run plan was: ---"; grep -A4 "run plan" "$T/build.out" | sed 's/^/        /'; echo "  --- work-units on disk: ---"; ls .product/work-units/ 2>&1 | sed 's/^/        /'; }

printf '\n'
if [ "$fails" -eq 0 ]; then printf '\033[32mALL PASS\033[0m — session e2e green\n'; exit 0
else printf '\033[31m%d step(s) FAILED\033[0m\n' "$fails"; exit 1; fi
