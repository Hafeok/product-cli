#!/bin/bash
# Gate 1b, Gate B — the three-region delta over the ratified set, on A's run-8 inventory.
# CG-R-102: the full gate set runs BEFORE the first measurement; a binary that has not passed
# its gates does not measure. CG-R-88: the script builds what it measures with, fails on any
# step that fails, and records the instrument with the working-tree count captured before the
# record exists. The reader is not rebuilt or re-run (CG-R-103). The run-8 A reach output is
# regenerated and diffed first — the existing measurement must not move.
set -euo pipefail
cd /home/user/product-cli
export CARGO_INCREMENTAL=0
NAME=${1:-gateB}
S=/tmp/claude-0/-home-user-product-cli/5816e30b-ba90-5d06-945b-75e8aa4a18b8/scratchpad/solutions
D=meta/sessions/2026-09-15-gate1b-candidates
M=$D/measurement
A=$S/eshop-inventory-v6.json
RAT=$D/ratification-A-filled.yaml
RUN8=meta/sessions/2026-09-13-implement-csharp-binding
[ "$(sha256sum $A | cut -c1-16)" = "3197d7e07c5a0b46" ] || { echo "A inventory is not the run-8 artefact"; exit 1; }
[ "$(sha256sum $RAT | cut -c1-16)" = "dff974baac758904" ] || { echo "ratification is not the filed artefact"; exit 1; }
# CG-R-102: gates first.
cargo t > $M/$NAME-gates-t.log 2>&1
cargo clippy -- -D warnings -D clippy::unwrap_used > $M/$NAME-gates-clippy.log 2>&1
cargo xtask check > $M/$NAME-gates-xtask.log 2>&1
GATES="cargo t $(grep -c 'test result: ok' $M/$NAME-gates-t.log) binaries ok, $(grep -c 'test result: FAILED' $M/$NAME-gates-t.log || true) failed; clippy ok; xtask ok"
rm -f $M/$NAME-gates-*.log
cargo build --release -p product-cli
P=target/release/product
TREE=$(git status --porcelain | wc -l)
{ echo "$NAME instrument record — $(date -u +%FT%TZ)"; echo "gates before measurement (CG-R-102): $GATES"; echo "commit: $(git rev-parse HEAD) (working tree: $TREE uncommitted paths, captured before this record was created)"; echo "binary: target/release/product sha256 $(sha256sum $P | cut -c1-16)"; echo "inventory: A $(jq -r .inventory_version $A) sha256 $(sha256sum $A | cut -c1-16) (the run-8 artefact; reader not re-run)"; echo "ratification: $(sha256sum $RAT | cut -c1-16) (Emil's, stand-in graded)"; } > $M/$NAME-instrument.txt
AMAIN='member:M:Program.{Main}$(System.String[])@Web'
AFW='implements:T:FastEndpoints.Endpoint`2,implements:T:FastEndpoints.EndpointWithoutRequest`1,implements:T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel,implements:T:Microsoft.AspNetCore.Mvc.Controller,implements:T:Microsoft.AspNetCore.Mvc.ControllerBase,implements:T:Microsoft.AspNetCore.Components.ComponentBase,implements:T:Microsoft.AspNetCore.Mvc.ViewComponent'
$P csharp reach $A --roots "$AMAIN,$AFW" --track 'implements:T:Mediator.IRequestHandler`2' --track 'implements:T:Mediator.INotificationHandler`1' --ground-truth $RUN8/ground-truth-A-web.yaml > $M/$NAME-regression-run8-A-primary.txt
if diff -q $M/$NAME-regression-run8-A-primary.txt $RUN8/gate1a-measurement/run8-A-reach-PRIMARY-main-plus-framework-handlers.txt > /dev/null; then
  echo "regression: run-8 A primary reach output byte-identical" >> $M/$NAME-instrument.txt
else
  echo "regression: run-8 A primary reach output DIFFERS" >> $M/$NAME-instrument.txt; exit 1
fi
$P csharp regions $A --ratification $RAT > $M/$NAME-A-regions.txt
$P --format json csharp regions $A --ratification $RAT > $M/$NAME-A-regions.json
echo "$NAME complete"
