/* ============================================================
   The Acme "What" — the worked checkout example from the repo,
   shaped for three connected graph views in the product-cli tool.
     • PF.product / PF.systems / PF.domains / PF.journeys  → Systems map
     • PF.domain (Ordering bounded context)               → Domain ER graph
     • PF.flows (per system, event-model slices)           → Flows timeline
   Every id and label is lifted from examples/checkout.
   ============================================================ */
window.PF = {
  /* -------- product / systems / domains / journeys (§3.0, §3.2.5) -------- */
  product: {
    id: 'acme', name: 'Acme',
    purpose: 'Sell coffee supplies and run the business behind it',
    direction: 'target What 2.0',
    quality: 'data_residency = EU',
    ownsDomains: ['ordering', 'catalog'],
    ownsSystems: ['acme-shop', 'acme-admin'],
    conformance: 'verified',
  },

  domains: [
    { id: 'ordering', name: 'Ordering', sub: 'bounded context · §3.1',
      language: ['Cart', 'Line Item', 'Order', 'Payment Method'], conformance: 'verified' },
    { id: 'catalog', name: 'Catalog', sub: 'bounded context · §3.1',
      language: ['Product', 'Category', 'Price'], conformance: 'realised' },
  ],

  systems: [
    {
      id: 'acme-shop', name: 'Acme Shop', kind: 'application',
      purpose: 'Let customers buy coffee supplies',
      cls: 'gui', platforms: ['iOS', 'Android', 'web'],
      references: ['ordering', 'catalog'],
      demands: ['concurrent_users ≤ 10000', 'uptime ≥ 99.9%'],
      flows: ['flow-checkout', 'flow-browse', 'flow-orders'],
      conformance: 'verified',
    },
    {
      id: 'acme-admin', name: 'Acme Admin', kind: 'website',
      purpose: 'Let staff manage orders and refunds',
      cls: 'gui', platforms: ['web', 'desktop'],
      references: ['ordering', 'catalog'],
      demands: [],
      flows: ['flow-fulfil', 'flow-refunds'],
      conformance: 'realised',
    },
  ],

  // a product-level journey composes one flow from each system, linked by a Translation
  journeys: [
    {
      id: 'order-to-fulfilment', name: 'Order to fulfilment',
      from: { system: 'acme-shop', flow: 'flow-checkout', label: 'place-order' },
      to: { system: 'acme-admin', flow: 'flow-fulfil', label: 'fulfil-order' },
      translation: 'ev-order-placed (acme-shop) → cmd-accept-fulfilment (acme-admin)',
    },
  ],

  /* -------- the Ordering domain — structure & data (§3.1) -------- */
  // ER graph: entities / value-objects / invariants colour-coded, edges are relations
  domain: {
    contextId: 'ordering',
    nodes: [
      { id: 'cart', kind: 'aggregate', label: 'Cart', sub: 'aggregate',
        fields: ['items: LineItem[]', 'state: checking-out | open'] },
      { id: 'lineitem', kind: 'entity', label: 'LineItem', sub: 'entity',
        fields: ['product: Product', 'qty: Integer ≥ 1', 'unit_price: Money'] },
      { id: 'order', kind: 'aggregate', label: 'Order', sub: 'aggregate · entity',
        fields: ['order_no: OrderNo', 'placed_at: Instant', 'paid_total: Money', 'refund_total: Money'] },
      { id: 'money', kind: 'value-object', label: 'Money', sub: 'value object',
        fields: ['amount: Integer (minor)', 'currency: EUR'] },
      { id: 'orderno', kind: 'value-object', label: 'OrderNo', sub: 'value object',
        fields: ['digits: String', 'check_digit (Damm §3.5)'] },
      { id: 'product', kind: 'external', label: 'Product', sub: 'Catalog context',
        fields: ['referenced across context'] },
      // invariants — executable shapes
      { id: 'cart-1', kind: 'invariant', label: 'CART-1', sub: 'invariant',
        fields: ['a checking-out Cart has ≥ 1 LineItem'] },
      { id: 'cart-2', kind: 'invariant', label: 'CART-2', sub: 'invariant',
        fields: ['qty ≥ 1 for every LineItem'] },
      { id: 'refund-1', kind: 'invariant', label: 'ORDER-REFUND-1', sub: 'invariant',
        fields: ['refund_total ≤ paid_total'] },
      // reference data — constitutive (§3.1)
      { id: 'refdata', kind: 'reference', label: 'Reference data', sub: 'closed sets',
        fields: ['shipping-methods', 'tax-categories', 'currencies: [EUR]'] },
    ],
    edges: [
      { from: 'cart', to: 'lineitem', label: 'has many', card: '1 — *' },
      { from: 'lineitem', to: 'product', label: 'references', kind: 'cross' },
      { from: 'lineitem', to: 'money', label: 'unit_price' },
      { from: 'order', to: 'cart', label: 'created-from' },
      { from: 'order', to: 'orderno', label: 'identity' },
      { from: 'order', to: 'money', label: 'paid / refund total' },
      { from: 'lineitem', to: 'refdata', label: 'tax-category ∈', kind: 'ref' },
      // invariant governance (dashed)
      { from: 'cart-1', to: 'cart', label: 'governs', kind: 'inv' },
      { from: 'cart-2', to: 'lineitem', label: 'governs', kind: 'inv' },
      { from: 'refund-1', to: 'order', label: 'governs', kind: 'inv' },
    ],
  },

  /* -------- event-model flows, per system (§3.2) -------- */
  // each flow is an Event-Modeling slice: triggers/ui · commands·views · event streams
  flows: {
    'flow-checkout': {
      system: 'acme-shop', name: 'Checkout', pattern: 'Command + View',
      conformance: 'verified',
      lanes: [
        { id: 'ui', label: 'Triggers / UI', kind: 'rail' },
        { id: 'cmdview', label: 'Commands · Views', kind: 'rail' },
        { id: 'cart', label: 'Cart', kind: 'stream' },
        { id: 'order', label: 'Order', kind: 'stream' },
      ],
      cols: 4,
      nodes: [
        { id: 'trg-open', kind: 'trigger', label: 'Shopper opens cart', col: 0, lane: 'ui', sub: 'user trigger' },
        { id: 'ui-review', kind: 'ui-step', label: 'Review cart', col: 0, lane: 'ui', sub: 'ui-review-cart' },
        { id: 'ui-choose', kind: 'ui-step', label: 'Choose payment', col: 1, lane: 'ui', sub: 'ui-choose-payment' },
        { id: 'ui-confirm', kind: 'ui-step', label: 'Order placed', col: 3, lane: 'ui', sub: 'ui-confirmation' },

        { id: 'cmd-additem', kind: 'command', label: 'Add item', col: 0, lane: 'cmdview', sub: 'cmd-add-item' },
        { id: 'cmd-begin', kind: 'command', label: 'Begin payment', col: 1, lane: 'cmdview', sub: 'cmd-begin-payment' },
        { id: 'cmd-auth', kind: 'command', label: 'Authorize payment', col: 2, lane: 'cmdview', sub: 'cmd-authorize-pay' },
        { id: 'rm-cart', kind: 'view', label: 'Cart summary', col: 0, lane: 'cmdview', sub: 'rm-cart-summary' },
        { id: 'rm-confirm', kind: 'view', label: 'Order confirmation', col: 3, lane: 'cmdview', sub: 'rm-order-confirmation' },

        { id: 'ev-added', kind: 'event', label: 'Item added', col: 0, lane: 'cart', sub: 'ev-item-added' },
        { id: 'ev-begun', kind: 'event', label: 'Payment begun', col: 1, lane: 'cart', sub: 'ev-payment-begun' },
        { id: 'ev-auth', kind: 'event', label: 'Payment authorized', col: 2, lane: 'cart', sub: 'ev-payment-authorized' },
        { id: 'ev-placed', kind: 'event', label: 'Order placed', col: 2, lane: 'order', sub: 'ev-order-placed' },
      ],
      edges: [
        { from: 'trg-open', to: 'cmd-additem', type: 'spine' },
        { from: 'cmd-additem', to: 'ev-added', type: 'spine' },
        { from: 'ev-added', to: 'rm-cart', type: 'cross' },
        { from: 'rm-cart', to: 'ui-review', type: 'cross' },
        { from: 'ui-review', to: 'cmd-begin', type: 'spine' },
        { from: 'cmd-begin', to: 'ev-begun', type: 'spine' },
        { from: 'ui-choose', to: 'cmd-auth', type: 'spine' },
        { from: 'cmd-auth', to: 'ev-auth', type: 'spine' },
        { from: 'cmd-auth', to: 'ev-placed', type: 'spine' },
        { from: 'ev-placed', to: 'rm-confirm', type: 'cross' },
        { from: 'rm-confirm', to: 'ui-confirm', type: 'cross' },
      ],
      meta: {
        'cmd-begin': { context: 'Ordering', guards: 'CART-1', out: ['→ ev-payment-begun (emits)'], in: ['← ui-review-cart (triggers)'] },
        'cmd-auth': { context: 'Ordering', out: ['→ ev-payment-authorized (emits)', '→ ev-order-placed (emits)', '| ev-payment-declined'], in: ['← ui-choose-payment (triggers)'] },
        'rm-cart': { context: 'Ordering', note: 'folds events — has a real Projector (§4)', out: ['→ ui-review-cart (displays)'], in: ['← ev-item-added (projects)'] },
        'ev-placed': { context: 'Ordering', out: ['→ rm-order-confirmation (projects)', '→ Translation: fulfilment'], in: ['← cmd-authorize-pay (emits)'] },
        'ui-review': { context: 'acme-shop', aio: 'display-collection + trigger-action', out: ['→ cmd-begin-payment (triggers)'], in: ['← rm-cart-summary (surfaces)'] },
      },
    },

    'flow-browse': {
      system: 'acme-shop', name: 'Browse', pattern: 'View', conformance: 'realised',
      lanes: [
        { id: 'ui', label: 'Triggers / UI', kind: 'rail' },
        { id: 'cmdview', label: 'Commands · Views', kind: 'rail' },
        { id: 'catalog', label: 'Catalog', kind: 'stream' },
      ],
      cols: 2,
      nodes: [
        { id: 'trg-browse', kind: 'trigger', label: 'Open shop', col: 0, lane: 'ui', sub: 'user trigger' },
        { id: 'ui-browse', kind: 'ui-step', label: 'Browse the shop', col: 0, lane: 'ui', sub: 'ui-browse' },
        { id: 'ui-product', kind: 'ui-step', label: 'Product detail', col: 1, lane: 'ui', sub: 'ui-product' },
        { id: 'rm-catalog', kind: 'view', label: 'Catalog listing', col: 0, lane: 'cmdview', sub: 'rm-catalog-listing' },
        { id: 'cmd-add', kind: 'command', label: 'Add item', col: 1, lane: 'cmdview', sub: 'cmd-add-item' },
        { id: 'ev-added2', kind: 'event', label: 'Item added', col: 1, lane: 'catalog', sub: 'ev-item-added' },
      ],
      edges: [
        { from: 'trg-browse', to: 'rm-catalog', type: 'spine' },
        { from: 'rm-catalog', to: 'ui-browse', type: 'cross' },
        { from: 'ui-browse', to: 'ui-product', type: 'spine' },
        { from: 'ui-product', to: 'cmd-add', type: 'spine' },
        { from: 'cmd-add', to: 'ev-added2', type: 'spine' },
      ],
      meta: {},
    },

    'flow-orders': {
      system: 'acme-shop', name: 'Orders', pattern: 'View', conformance: 'realised',
      lanes: [
        { id: 'ui', label: 'Triggers / UI', kind: 'rail' },
        { id: 'cmdview', label: 'Commands · Views', kind: 'rail' },
        { id: 'order', label: 'Order', kind: 'stream' },
      ],
      cols: 2,
      nodes: [
        { id: 'trg-orders', kind: 'trigger', label: 'Open orders', col: 0, lane: 'ui', sub: 'user trigger' },
        { id: 'ui-orders', kind: 'ui-step', label: 'Your orders', col: 0, lane: 'ui', sub: 'ui-orders' },
        { id: 'ui-order', kind: 'ui-step', label: 'Order detail', col: 1, lane: 'ui', sub: 'ui-order-detail' },
        { id: 'rm-orders', kind: 'view', label: 'Order history', col: 0, lane: 'cmdview', sub: 'rm-order-history' },
        { id: 'rm-orderd', kind: 'view', label: 'Order detail', col: 1, lane: 'cmdview', sub: 'rm-order-detail' },
        { id: 'ev-placed2', kind: 'event', label: 'Order placed', col: 0, lane: 'order', sub: 'ev-order-placed' },
      ],
      edges: [
        { from: 'ev-placed2', to: 'rm-orders', type: 'cross' },
        { from: 'trg-orders', to: 'rm-orders', type: 'spine' },
        { from: 'rm-orders', to: 'ui-orders', type: 'cross' },
        { from: 'ui-orders', to: 'ui-order', type: 'spine' },
        { from: 'ui-order', to: 'rm-orderd', type: 'cross' },
      ],
      meta: {},
    },

    'flow-fulfil': {
      system: 'acme-admin', name: 'Fulfil order', pattern: 'Command (via Translation)', conformance: 'realised',
      lanes: [
        { id: 'ui', label: 'Triggers / UI', kind: 'rail' },
        { id: 'cmdview', label: 'Commands · Views', kind: 'rail' },
        { id: 'fulfil', label: 'Fulfilment', kind: 'stream' },
      ],
      cols: 3,
      nodes: [
        { id: 'trg-trans', kind: 'trigger', label: 'Translation in', col: 0, lane: 'ui', sub: 'automated · ev-order-placed' },
        { id: 'cmd-accept', kind: 'command', label: 'Accept fulfilment', col: 0, lane: 'cmdview', sub: 'cmd-accept-fulfilment' },
        { id: 'ev-accepted', kind: 'event', label: 'Fulfilment accepted', col: 0, lane: 'fulfil', sub: 'ev-fulfilment-accepted' },
        { id: 'ui-queue', kind: 'ui-step', label: 'Fulfilment queue', col: 1, lane: 'ui', sub: 'ui-queue' },
        { id: 'rm-queue', kind: 'view', label: 'Pending fulfilments', col: 1, lane: 'cmdview', sub: 'rm-pending-fulfilments' },
        { id: 'cmd-ship', kind: 'command', label: 'Mark shipped', col: 2, lane: 'cmdview', sub: 'cmd-mark-shipped' },
        { id: 'ui-ship', kind: 'ui-step', label: 'Ship order', col: 2, lane: 'ui', sub: 'ui-ship' },
        { id: 'ev-shipped', kind: 'event', label: 'Order shipped', col: 2, lane: 'fulfil', sub: 'ev-order-shipped' },
      ],
      edges: [
        { from: 'trg-trans', to: 'cmd-accept', type: 'spine' },
        { from: 'cmd-accept', to: 'ev-accepted', type: 'spine' },
        { from: 'ev-accepted', to: 'rm-queue', type: 'cross' },
        { from: 'rm-queue', to: 'ui-queue', type: 'cross' },
        { from: 'ui-queue', to: 'ui-ship', type: 'spine' },
        { from: 'ui-ship', to: 'cmd-ship', type: 'spine' },
        { from: 'cmd-ship', to: 'ev-shipped', type: 'spine' },
      ],
      meta: {
        'trg-trans': { context: 'acme-admin', note: 'the only sanctioned cross-system channel (§3.2.5)', in: ['← ev-order-placed (acme-shop)'] },
      },
    },

    'flow-refunds': {
      system: 'acme-admin', name: 'Refunds', pattern: 'Command', conformance: 'verified',
      lanes: [
        { id: 'ui', label: 'Triggers / UI', kind: 'rail' },
        { id: 'cmdview', label: 'Commands · Views', kind: 'rail' },
        { id: 'order', label: 'Order', kind: 'stream' },
      ],
      cols: 2,
      nodes: [
        { id: 'trg-refund', kind: 'trigger', label: 'Staff opens order', col: 0, lane: 'ui', sub: 'user trigger' },
        { id: 'ui-order2', kind: 'ui-step', label: 'Order detail', col: 0, lane: 'ui', sub: 'ui-order-detail' },
        { id: 'ui-refund', kind: 'ui-step', label: 'Issue refund', col: 1, lane: 'ui', sub: 'ui-issue-refund' },
        { id: 'rm-order2', kind: 'view', label: 'Order detail', col: 0, lane: 'cmdview', sub: 'rm-order-detail' },
        { id: 'cmd-refund', kind: 'command', label: 'Issue refund', col: 1, lane: 'cmdview', sub: 'cmd-issue-refund' },
        { id: 'ev-refund', kind: 'event', label: 'Refund issued', col: 1, lane: 'order', sub: 'ev-refund-issued' },
      ],
      edges: [
        { from: 'trg-refund', to: 'rm-order2', type: 'spine' },
        { from: 'rm-order2', to: 'ui-order2', type: 'cross' },
        { from: 'ui-order2', to: 'ui-refund', type: 'spine' },
        { from: 'ui-refund', to: 'cmd-refund', type: 'spine' },
        { from: 'cmd-refund', to: 'ev-refund', type: 'spine' },
      ],
      meta: {
        'cmd-refund': { context: 'Ordering', guards: 'ORDER-REFUND-1', out: ['→ ev-refund-issued (emits)'], in: ['← ui-issue-refund (triggers)'] },
      },
    },
  },

  // semantic colours — never reassigned (load-bearing EM palette)
  kindColor: {
    trigger: 'var(--em-trigger)', 'ui-step': 'var(--slate-400)',
    command: 'var(--em-command)', view: 'var(--em-view)', event: 'var(--em-event)',
    aggregate: 'var(--kind-entity)', entity: 'var(--kind-value-object)',
    'value-object': 'var(--blue-400)', invariant: 'var(--kind-invariant)',
    external: 'var(--slate-400)', reference: 'var(--em-event)',
  },
};

/* ============================================================
   DELIVERY (§7) — features, releases, versions & direction.
   Everything here is a *partition of the What above*, never a
   free-floating ticket. 'done' is a computed predicate; a feature
   is its flows and its concepts are the derived footprint.
   ============================================================ */
window.PF.delivery = {
  // virtual concepts a target slice pulls in that don't exist in the graph yet
  concepts: {
    'loyalty-account': { label: 'LoyaltyAccount', kind: 'aggregate', sub: 'aggregate · not built' },
    'points-ledger':   { label: 'PointsLedger',   kind: 'entity',    sub: 'entity · not built' },
    'fx-rate':         { label: 'FxRate',         kind: 'reference', sub: 'reference data · not built' },
  },

  /* -------- features — each references a slice of the event model (§7.1) --------
     done: the four clauses of feature_done(f) (§7.2), each pass | partial | pending.
     footprint: the concepts *derived* from the flow slice (domain node ids). */
  features: [
    {
      id: 'browse', name: 'Browse the shop', sub: 'feat-browse', whatVersion: '1.0',
      flows: ['flow-browse'], footprint: ['product', 'lineitem', 'refdata'],
      conformance: 'verified', valueAction: 'shopper reaches product detail',
      done: { flows: 'pass', footprint: 'pass', verifications: 'pass', acceptance: 'pass' },
      acceptance: ['catalog listing projects Product', 'add-item reachable from product detail'],
    },
    {
      id: 'checkout', name: 'Checkout', sub: 'feat-checkout', whatVersion: '1.0',
      flows: ['flow-checkout'], footprint: ['cart', 'lineitem', 'order', 'money', 'orderno', 'cart-1', 'cart-2'],
      conformance: 'delivered', valueAction: 'order placed & paid',
      done: { flows: 'pass', footprint: 'pass', verifications: 'pass', acceptance: 'pass' },
      acceptance: ['payment authorized before order placed', 'CART-1 holds at checkout', 'confirmation shows order number'],
    },
    {
      id: 'orders', name: 'Your orders', sub: 'feat-orders', whatVersion: '1.0',
      flows: ['flow-orders'], footprint: ['order', 'orderno', 'money'],
      conformance: 'verified', valueAction: 'shopper inspects a past order',
      done: { flows: 'pass', footprint: 'pass', verifications: 'pass', acceptance: 'pass' },
      acceptance: ['order history projects ev-order-placed', 'detail reachable from history'],
    },
    {
      id: 'fulfilment', name: 'Fulfil order', sub: 'feat-fulfilment', whatVersion: '1.1',
      flows: ['flow-fulfil'], footprint: ['order'],
      conformance: 'realised', valueAction: 'order shipped',
      done: { flows: 'pass', footprint: 'pass', verifications: 'partial', acceptance: 'pending' },
      acceptance: ['Translation-in is the only cross-system channel', 'shipped only after accepted'],
    },
    {
      id: 'refunds', name: 'Issue refund', sub: 'feat-refunds', whatVersion: '1.1',
      flows: ['flow-refunds'], footprint: ['order', 'money', 'refund-1'],
      conformance: 'verified', valueAction: 'refund issued to customer',
      done: { flows: 'pass', footprint: 'pass', verifications: 'pass', acceptance: 'pass' },
      acceptance: ['ORDER-REFUND-1: refund_total ≤ paid_total', 'refund emits ev-refund-issued', 'staff-only trigger'],
    },
    {
      id: 'loyalty', name: 'Loyalty points', sub: 'feat-loyalty · not built', whatVersion: '2.0',
      flows: ['flow-loyalty'], footprint: ['order', 'loyalty-account', 'points-ledger'],
      conformance: 'described', valueAction: 'points accrue on purchase',
      done: { flows: 'pending', footprint: 'pending', verifications: 'pending', acceptance: 'pending' },
      acceptance: ['points accrue on ev-order-placed', 'balance never negative (Decider pending)'],
    },
    {
      id: 'multicurrency', name: 'Multi-currency', sub: 'feat-multicurrency · not built', whatVersion: '2.0',
      flows: ['flow-multicurrency'], footprint: ['money', 'order', 'fx-rate'],
      conformance: 'described', valueAction: 'checkout in a shopper\u2019s currency',
      done: { flows: 'pending', footprint: 'pending', verifications: 'pending', acceptance: 'pending' },
      acceptance: ['Money.currency \u2208 supported set', 'fx-rate reference data present'],
    },
  ],

  /* -------- releases — a coherent set of features that ship together (§7.1) -------- */
  releases: [
    { id: 'rel-1', name: 'Storefront MVP', whatVersion: '1.0', status: 'delivered',
      features: ['browse', 'checkout', 'orders'], closed: true,
      note: 'the cut is closed — nothing included depends on anything excluded.' },
    { id: 'rel-2', name: 'Back office', whatVersion: '1.1', status: 'in-progress',
      features: ['fulfilment', 'refunds'], closed: true,
      note: 'refunds is done; fulfilment\u2019s behavioural verifications are still amber.' },
    { id: 'rel-planned', name: 'Unreleased', whatVersion: '2.0 target', status: 'planned',
      features: ['loyalty', 'multicurrency'], closed: false,
      note: 'described slices targeted for What 2.0 — not yet a release cut.' },
  ],

  /* -------- versions — two independent semantic axes that reference each other (§7.3) -------- */
  versions: {
    what: [
      { v: '2.0', name: 'Loyalty & currency', bump: 'major', target: true, status: 'described',
        diff: 'new points Decider + a multi-currency invariant — a conformant realisation must now do more',
        adds: ['loyalty', 'multicurrency'] },
      { v: '1.1', name: 'Back office', bump: 'minor', current: true, status: 'realised',
        diff: 'additive: new admin flows, breaks nothing already realised',
        adds: ['fulfilment', 'refunds'] },
      { v: '1.0', name: 'Storefront', bump: 'major', status: 'delivered',
        diff: 'initial behaviour: the checkout Deciders and CART-1 / CART-2 invariants',
        adds: ['browse', 'checkout', 'orders'] },
    ],
    how: [
      { v: '1.1.2', name: 'dependency bump', bump: 'patch', realises: '1.1', current: true,
        diff: 'same What, refreshed substrate' },
      { v: '1.1.1', name: 'projection cache', bump: 'patch', realises: '1.1',
        diff: 'perf fix — the same What realised faster' },
      { v: '1.1.0', name: 'admin console', bump: 'minor', realises: '1.1',
        diff: 'realises the back-office What' },
      { v: '1.0.0', name: 'React storefront', bump: 'major', realises: '1.0',
        diff: 'the first realisation of What 1.0' },
    ],
  },

  /* -------- target versions — a declared future partition; direction is the computed gap (§7.3) -------- */
  targets: [
    { id: 'what-2', name: 'What 2.0', whatVersion: '2.0',
      partition: ['refunds', 'loyalty', 'multicurrency'],
      note: 'a declared future partition of the graph — membership may include slices that do not exist yet.' },
  ],
};

// helpers — resolve a footprint concept id from the domain graph or the virtual set
window.PF.concept = function (id) {
  const d = window.PF.domain.nodes.find(n => n.id === id);
  if (d) return { id, label: d.label, kind: d.kind, sub: d.sub };
  const v = window.PF.delivery.concepts[id];
  if (v) return { id, label: v.label, kind: v.kind, sub: v.sub };
  return { id, label: id, kind: 'external', sub: '' };
};
window.PF.feature = function (id) { return window.PF.delivery.features.find(f => f.id === id); };
window.PF.featureDone = function (f) {
  const ft = typeof f === 'string' ? window.PF.feature(f) : f;
  return ft && (ft.conformance === 'verified' || ft.conformance === 'delivered');
};
