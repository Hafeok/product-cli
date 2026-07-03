#!/usr/bin/env bash
# Author ACME's per-product How + Delivery artifacts under
# .product/products/acme/ so the explorer's How / Delivery / Build views are
# acme-specific (the projection resolves this scoped base before the shared
# .product root). The What graph is authored separately by showcase-acme.sh.
set -euo pipefail
ROOT=".product/products/acme"
mkdir -p "$ROOT"/{blueprints/acme-storefront/cells,deployable-units,features,deliverables,targets,releases}

# ── §4 the How contract (why-cascade + contracts + interfaces) ─────────────
cat > "$ROOT/how-contract.yaml" <<'YAML'
blueprint: acme-storefront
version: 1.1.0
realises_version: "1.1"
top_decisions:
  - id: event-sourced-ordering
    decision: Event-source the Ordering context
    rationale: refunds and fulfilment need full history; the event model is already the spec
    licenses: [pure-decisions, projections-derived]
    enforced_by: [contract-conformance]
  - id: bind-wire-ds
    decision: Bind every GUI screen to the wire-ds design system
    rationale: a design system is the screen standard; inventing a UI language forfeits its components and a11y
    licenses: [closed-vocabulary, tokens-not-literals]
    enforced_by: [seam-verification]
principles:
  - id: pure-decisions
    statement: decision logic is a pure, isolable core — no I/O in decide()
    licensed_by: [event-sourced-ordering]
    enforced_by: [contract-conformance]
    realized_by: [pat-decider-module]
  - id: projections-derived
    statement: read models are derived by fold, never written directly
    licensed_by: [event-sourced-ordering]
    enforced_by: [behavioural-conformance]
    realized_by: [pat-projector]
  - id: closed-vocabulary
    statement: a screen composes only components the design system defines
    licensed_by: [bind-wire-ds]
    enforced_by: [seam-verification]
    realized_by: [pat-page]
  - id: tokens-not-literals
    statement: colour, spacing, type are token references — a literal is non-conformant
    licensed_by: [bind-wire-ds]
    enforced_by: [seam-verification]
    realized_by: [pat-theme]
patterns:
  - id: pat-decider-module
    shape: "one Decider module per aggregate: decide + evolve + scenarios"
    realizes: [pure-decisions]
    applied_by: [wu-refund-decider]
  - id: pat-projector
    shape: one Projector per read model, rebuildable from the log
    realizes: [projections-derived]
    applied_by: [wu-cart-projector]
  - id: pat-page
    shape: page = template + organisms from wire-ds, bound to its UI step
    realizes: [closed-vocabulary]
    applied_by: [wu-review-cart-page]
  - id: pat-theme
    shape: one theme file maps wire-ds tokens to CSS custom properties
    realizes: [tokens-not-literals]
    applied_by: [wu-review-cart-page]
application_contract:
  id: acme-app-contract
  language: TypeScript
  layering: [vertical slices, event store append-only]
  persistence_model: Postgres event store
  statements:
    - id: pure-decide
      statement: decision logic is pure and isolable (the Decider constraint, §3.3)
      enforced_by: contract-conformance
    - id: rebuildable-projections
      statement: projections are rebuildable from the log
      enforced_by: behavioural-conformance
infrastructure_contract:
  id: acme-runtime-contract
  satisfies: acme-app-contract
  frozen: true
  resources:
    - id: web-host
      kind: domain
      choice: shop.acme.com
    - id: runtime
      kind: runtime
      choice: Node 22 / eu-west
interface_contracts:
  - id: if-rest
    surface: REST interface
    standard: OpenAPI
    derived_from: [cmd-add-item, rm-cart-summary]
  - id: if-events
    surface: Event stream
    standard: AsyncAPI
    derived_from: [ev-order-placed]
YAML

# ── the blueprint (references the shared How; carries a layout model) ──────
printf 'ref: ../../how-contract.yaml\n' > "$ROOT/blueprints/acme-storefront/how-contract.yaml"
cat > "$ROOT/blueprints/acme-storefront/layout.yaml" <<'YAML'
version: "1"
blueprint: acme-storefront
allowlist: true
layout:
  - id: composition-root
    must_exist: "src/app/main.ts"
    cardinality: "exactly 1"
    rationale: the composition root; the app is not runnable without it
    enforces: [explicit-composition-root]
  - id: slice-files
    may_exist_here: "src/features/**"
    rationale: the slice interior stays free — constrain the skeleton, not the cytoplasm
    enforces: [closed-vocabulary]
  - id: no-secrets
    must_not_exist: "**/*.secrets.json"
    rationale: secrets never live in source
    enforces: [tokens-not-literals]
  - id: no-orphans
    no_orphans: "src/features/**"
    rationale: the unanticipated file is the failure case — allowlist, not denylist
    enforces: [closed-vocabulary]
YAML

# ── §4.2 DeployableUnits — the blueprint instantiated per system/environment ─
cat > "$ROOT/deployable-units/du-shop-ios.yaml" <<'YAML'
id: du-shop-ios
built_from: acme-storefront
deploys_system: [acme-shop]
environment: production
identity:
  bundle_id: com.acme.shop
  runtime: iOS 17
YAML
cat > "$ROOT/deployable-units/du-shop-web.yaml" <<'YAML'
id: du-shop-web
built_from: acme-storefront
deploys_system: [acme-shop]
environment: production
identity:
  domain_name: shop.acme.com
  runtime: Node 22 / eu-west
YAML
cat > "$ROOT/deployable-units/du-admin-web.yaml" <<'YAML'
id: du-admin-web
built_from: acme-storefront
deploys_system: [acme-admin]
environment: production
identity:
  domain_name: admin.acme.com
  runtime: Node 22 / eu-west
YAML

# ── §7 delivery — features, deliverables, a release, a target ──────────────
printf 'id: feat-checkout\nanchors: [flow-checkout]\n' > "$ROOT/features/feat-checkout.yaml"
printf 'id: feat-refunds\nanchors: [flow-refunds]\n' > "$ROOT/features/feat-refunds.yaml"
cat > "$ROOT/deliverables/del-checkout.yaml" <<'YAML'
id: del-checkout
feature: feat-checkout
acceptance:
  - id: ac-payment-first
    statement: payment is authorized before the order is placed
    status: passing
  - id: ac-cart-1
    statement: CART-1 holds at checkout
    status: passing
YAML
cat > "$ROOT/deliverables/del-refunds.yaml" <<'YAML'
id: del-refunds
feature: feat-refunds
acceptance:
  - id: ac-refund-1
    statement: "ORDER-REFUND-1: refund_total not greater than paid_total"
    status: passing
YAML
printf 'id: rel-storefront\nfeatures: [del-checkout]\n' > "$ROOT/releases/rel-storefront.yaml"
printf 'id: tgt-2\nversion: "2.0"\nin_target: [del-checkout, del-refunds]\n' > "$ROOT/targets/tgt-2.yaml"

echo "acme How + Delivery authored under $ROOT"
