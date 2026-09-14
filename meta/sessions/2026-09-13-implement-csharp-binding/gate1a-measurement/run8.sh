#!/bin/bash
# Gate 1a run 8 — instrument v6 with the table grown by run-7 frequency (CG-R-95), parsed/known/opaque
# split (CG-R-94), coverage below the fold (CG-R-93). CG-R-88: the script builds the binary it
# measures with, fails on any step that fails, and records the instrument — the working-tree
# state is captured before the record file exists (CG-R-94).
set -euo pipefail
cd /home/user/product-cli
export CARGO_INCREMENTAL=0
cargo build --release -p product-cli
P=target/release/product
S=/tmp/claude-0/-home-user-product-cli/5816e30b-ba90-5d06-945b-75e8aa4a18b8/scratchpad/solutions
M=meta/sessions/2026-09-13-implement-csharp-binding/gate1a-measurement
A=$S/eshop-inventory-v6.json; B=$S/orchard-inventory-v6.json
GT=meta/sessions/2026-09-13-implement-csharp-binding/ground-truth-A-web.yaml
TREE=$(git status --porcelain | wc -l)
{ echo "run 8 instrument record — $(date -u +%FT%TZ)"; echo "commit: $(git rev-parse HEAD) (working tree: $TREE uncommitted paths, captured before this record was created)"; echo "binary: target/release/product sha256 $(sha256sum $P | cut -c1-16)"; echo "inventories: A $(jq -r .inventory_version $A) sha256 $(sha256sum $A | cut -c1-16); B $(jq -r .inventory_version $B) sha256 $(sha256sum $B | cut -c1-16)"; } > $M/run8-instrument.txt
AMAIN='member:M:Program.{Main}$(System.String[])@Web'
AFW='implements:T:FastEndpoints.Endpoint`2,implements:T:FastEndpoints.EndpointWithoutRequest`1,implements:T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel,implements:T:Microsoft.AspNetCore.Mvc.Controller,implements:T:Microsoft.AspNetCore.Mvc.ControllerBase,implements:T:Microsoft.AspNetCore.Components.ComponentBase,implements:T:Microsoft.AspNetCore.Mvc.ViewComponent'
ATR=(--track 'implements:T:Mediator.IRequestHandler`2' --track 'implements:T:Mediator.INotificationHandler`1')
BMAIN='member:M:Program.{Main}$(System.String[])@OrchardCore.Cms.Web'
BCTRL='implements:T:Microsoft.AspNetCore.Mvc.Controller'
BSTART='implements:T:OrchardCore.Modules.StartupBase'
$P csharp inventory $A > $M/run8-A-inventory.txt
$P csharp reach $A --roots "$AMAIN" "${ATR[@]}" > $M/run8-A-reach-main-only.txt
$P csharp reach $A --roots "$AMAIN,$AFW" "${ATR[@]}" --ground-truth $GT > $M/run8-A-reach-PRIMARY-main-plus-framework-handlers.txt
$P --format json csharp reach $A --roots "$AMAIN,$AFW" "${ATR[@]}" --ground-truth $GT > $M/run8-A-primary.json
$P csharp reach $A --roots entry-point "${ATR[@]}" > $M/run8-A-reach-every-entry-point.txt
$P csharp reach $A --roots public "${ATR[@]}" > $M/run8-A-reach-public.txt
$P csharp inventory $B > $M/run8-B-inventory.txt
$P csharp reach $B --roots "$BMAIN" > $M/run8-B-reach-main-only.txt
$P csharp reach $B --roots "$BMAIN,$BCTRL" > $M/run8-B-reach-PRIMARY-main-plus-controllers.txt
$P csharp reach $B --roots "$BMAIN,$BCTRL,$BSTART" > $M/run8-B-reach-main-controllers-startups.txt
$P --format json csharp reach $B --roots "$BMAIN,$BCTRL,$BSTART" > /tmp/claude-0/-home-user-product-cli/5816e30b-ba90-5d06-945b-75e8aa4a18b8/scratchpad/run8-B-primary.json
$P csharp reach $B --roots entry-point > $M/run8-B-reach-every-entry-point.txt
$P csharp reach $B --roots public > $M/run8-B-reach-public.txt
echo "run 7 complete"
