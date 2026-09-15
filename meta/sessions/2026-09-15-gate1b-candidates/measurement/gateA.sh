#!/bin/bash
# Gate 1b, Gate A — the entry-point candidate set over A's run-8 inventory. CG-R-88: the script
# builds the binary it measures with, fails on any step that fails, and records the instrument;
# the working-tree count is captured before the record file exists. The reader is not rebuilt or
# re-run (CG-R-103): the inventory is the run-8 artefact, hash-checked. Before deriving anything
# the run-8 A reach output is regenerated and diffed — the existing measurement must not move.
set -euo pipefail
cd /home/user/product-cli
export CARGO_INCREMENTAL=0
cargo build --release -p product-cli
P=target/release/product
S=/tmp/claude-0/-home-user-product-cli/5816e30b-ba90-5d06-945b-75e8aa4a18b8/scratchpad/solutions
D=meta/sessions/2026-09-15-gate1b-candidates
M=$D/measurement
A=$S/eshop-inventory-v6.json
GT=$D/ground-truth-A-entry-points.yaml
RUN8=meta/sessions/2026-09-13-implement-csharp-binding
NAME=${1:-gateA}   # a re-run names its outputs (e.g. gateA-r2) so the earlier run's files stay in place
[ "$(sha256sum $A | cut -c1-16)" = "3197d7e07c5a0b46" ] || { echo "A inventory is not the run-8 artefact"; exit 1; }
TREE=$(git status --porcelain | wc -l)
{ echo "$NAME instrument record — $(date -u +%FT%TZ)"; echo "commit: $(git rev-parse HEAD) (working tree: $TREE uncommitted paths, captured before this record was created)"; echo "binary: target/release/product sha256 $(sha256sum $P | cut -c1-16)"; echo "inventory: A $(jq -r .inventory_version $A) sha256 $(sha256sum $A | cut -c1-16) (the run-8 artefact; reader not re-run)"; echo "ground truth: $(sha256sum $GT | cut -c1-16)"; } > $M/$NAME-instrument.txt
# Regression: the run-8 primary convention, regenerated, must be byte-identical.
AMAIN='member:M:Program.{Main}$(System.String[])@Web'
AFW='implements:T:FastEndpoints.Endpoint`2,implements:T:FastEndpoints.EndpointWithoutRequest`1,implements:T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel,implements:T:Microsoft.AspNetCore.Mvc.Controller,implements:T:Microsoft.AspNetCore.Mvc.ControllerBase,implements:T:Microsoft.AspNetCore.Components.ComponentBase,implements:T:Microsoft.AspNetCore.Mvc.ViewComponent'
$P csharp reach $A --roots "$AMAIN,$AFW" --track 'implements:T:Mediator.IRequestHandler`2' --track 'implements:T:Mediator.INotificationHandler`1' --ground-truth $RUN8/ground-truth-A-web.yaml > $M/$NAME-regression-run8-A-primary.txt
if diff -q $M/$NAME-regression-run8-A-primary.txt $RUN8/gate1a-measurement/run8-A-reach-PRIMARY-main-plus-framework-handlers.txt > /dev/null; then
  echo "regression: run-8 A primary reach output byte-identical" >> $M/$NAME-instrument.txt
else
  echo "regression: run-8 A primary reach output DIFFERS" >> $M/$NAME-instrument.txt; exit 1
fi
$P csharp candidates $A --ground-truth $GT > $M/$NAME-A-candidates.txt
$P --format json csharp candidates $A --ground-truth $GT > $M/$NAME-A-candidates.json
echo "$NAME complete"
