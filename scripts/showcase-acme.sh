#!/usr/bin/env bash
# Author the ACME Shop showcase/test product into the repo's .product graph via
# the real CLI. Nodes are created in strict dependency order because each create
# validates that its references already resolve. Run against a fresh acme graph:
#   rm -rf .product/products/acme/acme.ttl .product/products/acme/session.json && bash scripts/showcase-acme.sh
# Then view it in the explorer at  /?product=acme.
set -euo pipefail
BIN="${BIN:-./target/debug/product}"
d() {  # create one node; fail loudly if the graph rejects it
  local out
  if ! out="$("$BIN" domain new "$@" --product acme 2>&1)" || grep -q "Rejected" <<<"$out"; then
    echo "FAILED: domain new $*" >&2; echo "$out" >&2; exit 1
  fi
}

# ── §3.1 domains + structure ──────────────────────────────────────────────
d context ordering --label "Ordering" --purpose "Carts, orders, payments, refunds" --glossary "Cart,Line Item,Order,Payment Method"
d context catalog --label "Catalog" --purpose "Products and prices" --glossary "Product,Category,Price"
d entity cart --label "Cart" --context ordering --definition "A shopper's basket" --aggregate-root true --identity "cart_id"
d entity lineitem --label "LineItem" --context ordering --definition "One product line in a cart"
d entity order --label "Order" --context ordering --definition "A placed, paid order" --aggregate-root true --identity "order_no"
d entity product-item --label "Product" --context catalog --definition "A sellable item" --aggregate-root true --identity "sku"
d value-object money --context ordering --definition "An amount in minor units + currency"
d value-object orderno --context ordering --definition "A checksummed order number (Damm)"
d invariant cart-1 --context ordering --statement "a checking-out Cart has at least 1 LineItem" --applies-to cart
d invariant refund-1 --context ordering --statement "refund_total not greater than paid_total" --applies-to order
d relation r-cart-lineitem --from cart --to lineitem --cardinality "1 - *" --rationale "a cart holds its lines"

# ── §3.2 event model (events → read-models → commands → triggers) ─────────
d event ev-item-added --label "Item added" --context ordering --changes cart
d event ev-payment-begun --label "Payment begun" --context ordering --changes cart
d event ev-order-placed --label "Order placed" --context ordering --changes order
d event ev-refund-issued --label "Refund issued" --context ordering --changes order
d read-model rm-cart-summary --label "Cart summary" --projects ev-item-added --states "present,empty"
d read-model rm-order-confirmation --label "Order confirmation" --projects ev-order-placed --states "present"
d command cmd-add-item --label "Add item" --context ordering --targets cart --emits ev-item-added
d command cmd-begin-payment --label "Begin payment" --context ordering --targets cart --emits ev-payment-begun
d command cmd-authorize-payment --label "Authorize payment" --context ordering --targets order --emits ev-order-placed
d command cmd-issue-refund --label "Issue refund" --context ordering --targets order --emits ev-refund-issued
d trigger trg-open-cart --label "Shopper opens cart" --trigger-source user --issues cmd-add-item
d trigger trg-pay --label "Shopper pays" --trigger-source user --issues cmd-begin-payment
d trigger trg-authorize --label "Shopper confirms payment" --trigger-source user --issues cmd-authorize-payment

# ── §3.2.2 WCAG → AIOs → context of use → CIOs → reification → reference data ─
d wcag-criterion wc-131 --label "Info and Relationships" --level A --verification machine --satisfied true
d wcag-criterion wc-258 --label "Target Size (Minimum)" --level AA --verification machine --satisfied true
d wcag-criterion wc-247 --label "Focus Visible" --level AA --verification machine --satisfied true
d aio aio-display-collection --label "display-collection" --means "show many of a kind" --must-satisfy wc-131
d aio aio-display-value --label "display-value" --means "show a single datum" --must-satisfy wc-131
d aio aio-trigger-action --label "trigger-action" --means "invoke an operation" --must-satisfy wc-258 --must-satisfy wc-247
d aio aio-single-select --label "single-select" --means "choose one from a set" --must-satisfy wc-131
d context-of-use cou-phone --label "Phone" --dimension form_factor --value phone
d context-of-use cou-desktop --label "Desktop" --dimension form_factor --value desktop
d cio cio-list --label "List"
d cio cio-button --label "Primary Button"
d reification-rule rr-collection-phone --aio aio-display-collection --context cou-phone --cio cio-list --rationale "stacked list on a phone"
d reification-rule rr-action-phone --aio aio-trigger-action --context cou-phone --cio cio-button --rationale "full-width button"
d reference-set ref-payment-methods --label "Payment methods" --values "card,apple-pay,pay-on-delivery" --concept order
d reference-set ref-currencies --label "Supported currencies" --values "EUR" --concept money

# ── §3.2.1 UI steps (ws-confirm before ws-review-cart, which transitions to it) ─
d ui-step ws-confirm --label "Order placed" --intent "Reassure the order succeeded" \
  --surfaces "rm-order-confirmation:aio-display-value" --state-meaning "rm-order-confirmation:present:the placed order + its number"
d ui-step ws-review-cart --label "Review cart" --intent "Confirm the order before paying" \
  --surfaces "rm-cart-summary:aio-display-collection" --offers "cmd-begin-payment:aio-trigger-action" \
  --transitions-to ws-confirm --state-meaning "rm-cart-summary:present:the current cart contents" --state-meaning "rm-cart-summary:empty:Nothing to check out yet"
d application-root root-shop --label "Shop root" --navigates-from-root ws-review-cart

# ── §3.2.5 systems → flows → §3.0.1 journey ──────────────────────────────
d system acme-shop --label "Acme Shop" --system-kind application --purpose "Let customers buy coffee supplies" \
  --references-domain ordering --references-domain catalog --roots-at root-shop --target-classes gui --target-platforms ios --target-platforms web
d system acme-admin --label "Acme Admin" --system-kind website --purpose "Let staff manage orders and refunds" \
  --references-domain ordering --target-classes gui --target-platforms web
d flow flow-checkout --label "Checkout" --system acme-shop --entry-page ws-review-cart \
  --steps "trg-open-cart,cmd-add-item,ev-item-added,rm-cart-summary,ws-review-cart,cmd-begin-payment,ev-payment-begun,trg-authorize,cmd-authorize-payment,ev-order-placed,rm-order-confirmation,ws-confirm"
d flow flow-refunds --label "Refunds" --system acme-admin --steps "cmd-issue-refund,ev-refund-issued"
d trigger trg-fulfil --label "Translation in" --trigger-source automated --issues cmd-issue-refund --translates-from acme-shop --watches rm-order-confirmation
d journey journey-o2f --label "Order to fulfilment" --composes-flow flow-checkout --composes-flow flow-refunds --crosses-via trg-fulfil

# ── §3.6 quality demands ─────────────────────────────────────────────────
d quality-demand qd-latency --label "Checkout latency" --demand-kind runtime-bound --bound "checkout p99 not over 3s" --scopes acme-shop --measured-by "telemetry:checkout"
d quality-demand qd-residency --label "Data residency" --demand-kind architectural --bound "data_residency = EU" --scopes acme-shop --constrains pure-core

# ── §3.0 product last — it validates every owned domain/system exists ─────
d product acme --label "Acme" --purpose "Sell coffee supplies and run the business behind it" \
  --what-version 1.1 --owns-domain ordering --owns-domain catalog --owns-system acme-shop --owns-system acme-admin

echo "acme authored."
