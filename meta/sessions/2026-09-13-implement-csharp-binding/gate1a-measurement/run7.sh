#!/bin/bash
# Gate 1a run 7 — instrument v6 (CG-R-85..88: O-18, host-builder provider, scored fraction
# headline, registration-not-read inside the denominator from this run). CG-R-88: the script
# builds the binary it measures with, fails on any step that fails, and records the instrument.
set -euo pipefail
cd /home/user/product-cli
export CARGO_INCREMENTAL=0
cargo build --release -p product-cli
P=target/release/product
S=/tmp/claude-0/-home-user-product-cli/5816e30b-ba90-5d06-945b-75e8aa4a18b8/scratchpad/solutions
M=meta/sessions/2026-09-13-implement-csharp-binding/gate1a-measurement
A=$S/eshop-inventory-v6.json; B=$S/orchard-inventory-v6.json
GT=meta/sessions/2026-09-13-implement-csharp-binding/ground-truth-A-web.yaml
{ echo "run 7 instrument record — $(date -u +%FT%TZ)"; echo "commit: $(git rev-parse HEAD) (working tree: $(git status --porcelain | wc -l) uncommitted paths)"; echo "binary: target/release/product sha256 $(sha256sum $P | cut -c1-16)"; echo "inventories: A $(jq -r .inventory_version $A) sha256 $(sha256sum $A | cut -c1-16); B $(jq -r .inventory_version $B) sha256 $(sha256sum $B | cut -c1-16)"; } > $M/run7-instrument.txt
AMAIN='member:M:Program.{Main}$(System.String[])@Web'
AFW='implements:T:FastEndpoints.Endpoint`2,implements:T:FastEndpoints.EndpointWithoutRequest`1,implements:T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel,implements:T:Microsoft.AspNetCore.Mvc.Controller,implements:T:Microsoft.AspNetCore.Mvc.ControllerBase,implements:T:Microsoft.AspNetCore.Components.ComponentBase,implements:T:Microsoft.AspNetCore.Mvc.ViewComponent'
ATR=(--track 'implements:T:Mediator.IRequestHandler`2' --track 'implements:T:Mediator.INotificationHandler`1')
BMAIN='member:M:Program.{Main}$(System.String[])@OrchardCore.Cms.Web'
BCTRL='implements:T:Microsoft.AspNetCore.Mvc.Controller'
BSTART='implements:T:OrchardCore.Modules.StartupBase'
$P csharp inventory $A > $M/run7-A-inventory.txt
$P csharp reach $A --roots "$AMAIN" "${ATR[@]}" > $M/run7-A-reach-main-only.txt
$P csharp reach $A --roots "$AMAIN,$AFW" "${ATR[@]}" --ground-truth $GT > $M/run7-A-reach-PRIMARY-main-plus-framework-handlers.txt
$P --format json csharp reach $A --roots "$AMAIN,$AFW" "${ATR[@]}" --ground-truth $GT > $M/run7-A-primary.json
$P csharp reach $A --roots entry-point "${ATR[@]}" > $M/run7-A-reach-every-entry-point.txt
$P csharp reach $A --roots public "${ATR[@]}" > $M/run7-A-reach-public.txt
$P csharp inventory $B > $M/run7-B-inventory.txt
$P csharp reach $B --roots "$BMAIN" > $M/run7-B-reach-main-only.txt
$P csharp reach $B --roots "$BMAIN,$BCTRL" > $M/run7-B-reach-PRIMARY-main-plus-controllers.txt
$P csharp reach $B --roots "$BMAIN,$BCTRL,$BSTART" > $M/run7-B-reach-main-controllers-startups.txt
$P --format json csharp reach $B --roots "$BMAIN,$BCTRL,$BSTART" > /tmp/claude-0/-home-user-product-cli/5816e30b-ba90-5d06-945b-75e8aa4a18b8/scratchpad/run7-B-primary.json
$P csharp reach $B --roots entry-point > $M/run7-B-reach-every-entry-point.txt
$P csharp reach $B --roots public > $M/run7-B-reach-public.txt
echo "run 7 complete"
