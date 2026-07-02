/* ============================================================
   The UI layer of the What (§3.2.1–3.2.4) + the render contract
   (preview) — data for the Pages / Steps / Screens / AIOs views.
   Loads after data.js; extends window.PF.
   ============================================================ */

/* -------- the render contract — a derived projection of the What.
   Lifted from the repo's preview/renderer.html worked example. -------- */
window.PF.contract = {
  contract_version: 'preview-0',
  title: 'Checkout',
  context: { form_factor: 'phone', modality: 'touch' },
  locale: 'en',
  content_store: {
    'checkout.review.heading':  { role: 'heading',       en: 'Review your order', es: 'Revisa tu pedido' },
    'cart.empty.message':       { role: 'empty-message', en: 'Nothing to check out yet — add something to get started.', es: 'Aún no hay nada para pagar — añade algo para empezar.' },
    'cart.failed.message':      { role: 'error-message', en: 'Couldn\u2019t load your cart. Check your connection and retry.', es: 'No se pudo cargar tu carrito. Revisa tu conexión e inténtalo de nuevo.' },
    'checkout.payment.heading': { role: 'heading',       en: 'How would you like to pay?', es: '¿Cómo quieres pagar?' },
    'browse.heading':           { role: 'heading',       en: 'Browse the shop', es: 'Explora la tienda' },
    'orders.heading':           { role: 'heading',       en: 'Your orders', es: 'Tus pedidos' },
    'confirm.heading':          { role: 'heading',       en: 'You\u2019re all set', es: 'Todo listo' },
  },
  start: 'ui-review-cart',
  root: {
    destinations: [
      { to: 'ui-browse', label: 'Browse' },
      { to: 'ui-orders', label: 'Orders' },
      { to: 'ui-review-cart', label: 'Cart' },
    ],
    global_actions: [{ issues: 'cmd-sign-out', label: 'Sign out' }],
  },
  flows: [
    { id: 'flow-checkout', entry: 'ui-review-cart', pages: ['ui-review-cart', 'ui-choose-payment', 'ui-confirmation'] },
    { id: 'flow-browse', entry: 'ui-browse', pages: ['ui-browse'] },
    { id: 'flow-orders', entry: 'ui-orders', pages: ['ui-orders'] },
  ],
  screens: [
    {
      id: 'ui-review-cart', name: 'Review cart',
      intent: 'Confirm what you\u2019re buying before paying',
      content: { heading: 'checkout.review.heading' },
      projection: 'rm-cart-summary',
      state_space: ['present', 'empty', 'loading', 'failed'],
      state_meanings: {
        empty: 'Nothing to check out yet — add something to get started.',
        loading: 'Fetching your cart\u2026',
        failed: 'Couldn\u2019t load your cart. Check your connection and retry.',
      },
      state_content: { empty: 'cart.empty.message', failed: 'cart.failed.message' },
      elements: [
        { aio: 'display-collection', role: 'the line items', binds: 'items',
          item_shape: [{ field: 'name', type: 'string' }, { field: 'qty', type: 'integer' }, { field: 'price', type: 'money' }],
          wcag: ['1.3.1'] },
        { aio: 'display-value', role: 'the total owed', binds: 'total', value_type: 'money',
          emphasis: 'primary', wcag: ['1.3.1'] },
        { aio: 'trigger-action', role: 'proceed to payment', issues: 'cmd-begin-payment',
          emphasis: 'primary', transitions_to: 'ui-choose-payment', wcag: ['2.5.8', '2.4.7'] },
      ],
    },
    {
      id: 'ui-choose-payment', name: 'Choose payment',
      intent: 'Pick how to pay',
      content: { heading: 'checkout.payment.heading' },
      projection: 'rm-payment-options',
      state_space: ['present', 'loading', 'failed'],
      state_meanings: {
        loading: 'Loading payment methods\u2026',
        failed: 'Couldn\u2019t load payment methods. Retry.',
      },
      elements: [
        { aio: 'single-select', role: 'a payment method', binds: 'methods',
          option_field: 'label', issues: 'cmd-select-method', wcag: ['1.3.1', '4.1.2'] },
        { aio: 'trigger-action', role: 'confirm and pay', issues: 'cmd-authorize-payment',
          emphasis: 'primary', transitions_to: 'ui-confirmation', wcag: ['2.5.8', '2.4.7'] },
      ],
    },
    {
      id: 'ui-confirmation', name: 'Order placed',
      intent: 'Reassure the order succeeded and what happens next',
      content: { heading: 'confirm.heading' },
      projection: 'rm-order-confirmation',
      state_space: ['present', 'loading'],
      state_meanings: { loading: 'Placing your order\u2026' },
      state_waivers: { loading: 'transition is sub-perceptual in the common path; meaning waived with reason' },
      elements: [
        { aio: 'display-value', role: 'the confirmation message', binds: 'message', value_type: 'string',
          emphasis: 'primary', wcag: ['1.3.1'] },
        { aio: 'display-value', role: 'the order number', binds: 'order_no', value_type: 'string', wcag: ['1.3.1'] },
        { aio: 'trigger-action', role: 'keep shopping', issues: 'cmd-continue-shopping',
          transitions_to: 'ui-browse', wcag: ['2.5.8'] },
      ],
    },
    {
      id: 'ui-browse', name: 'Browse',
      intent: 'Find something to buy',
      content: { heading: 'browse.heading' },
      projection: 'rm-catalog',
      state_space: ['present', 'loading', 'empty'],
      state_meanings: { loading: 'Loading products\u2026', empty: 'No products match.' },
      elements: [
        { aio: 'display-collection', role: 'the catalog', binds: 'products',
          item_shape: [{ field: 'name', type: 'string' }, { field: 'price', type: 'money' }], wcag: ['1.3.1'] },
        { aio: 'trigger-action', role: 'go to cart', issues: 'cmd-open-cart',
          emphasis: 'primary', transitions_to: 'ui-review-cart', wcag: ['2.5.8'] },
      ],
    },
    {
      id: 'ui-orders', name: 'Orders',
      intent: 'See past orders',
      content: { heading: 'orders.heading' },
      projection: 'rm-order-history',
      state_space: ['present', 'loading', 'empty'],
      state_meanings: { loading: 'Loading orders\u2026', empty: 'You haven\u2019t ordered yet.' },
      elements: [
        { aio: 'display-collection', role: 'past orders', binds: 'orders',
          item_shape: [{ field: 'order_no', type: 'string' }, { field: 'total', type: 'money' }], wcag: ['1.3.1'] },
      ],
    },
  ],
  scenario: {
    given: [
      { event: 'ev-item-added', data: { name: 'Coffee beans', qty: 2, price: 1800 } },
      { event: 'ev-item-added', data: { name: 'Filter papers', qty: 1, price: 600 } },
    ],
    projected: {
      'rm-cart-summary': {
        state: 'present',
        items: [{ name: 'Coffee beans', qty: 2, price: 1800 }, { name: 'Filter papers', qty: 1, price: 600 }],
        total: 4200,
      },
      'rm-payment-options': {
        state: 'present',
        methods: [{ label: 'Card ending 4242' }, { label: 'Apple Pay' }, { label: 'Pay on delivery' }],
      },
      'rm-order-confirmation': { state: 'present', message: 'Order placed — thanks!', order_no: '#A7F-3192' },
      'rm-catalog': {
        state: 'present',
        products: [{ name: 'Coffee beans', price: 1800 }, { name: 'Filter papers', price: 600 }, { name: 'Grinder', price: 4500 }],
      },
      'rm-order-history': {
        state: 'present',
        orders: [{ order_no: '#A7F-3192', total: 4200 }, { order_no: '#9C1-0088', total: 1800 }],
      },
    },
  },
};

/* -------- the page graph (§3.2.4) — one graph per system: root + pages +
   navigate edges. Top-level is DERIVED: inbound edge from the root. -------- */
window.PF.pageGraph = {
  systems: [
    {
      id: 'acme-shop', name: 'Acme Shop', root: 'root-shop',
      globalActions: ['cmd-sign-out'],
      pages: [
        { id: 'ui-browse', name: 'Browse the shop', flow: 'flow-browse', specced: true },
        { id: 'ui-product', name: 'Product detail', flow: 'flow-browse' },
        { id: 'ui-review-cart', name: 'Review cart', flow: 'flow-checkout', specced: true },
        { id: 'ui-choose-payment', name: 'Choose payment', flow: 'flow-checkout', specced: true },
        { id: 'ui-confirmation', name: 'Order placed', flow: 'flow-checkout', specced: true },
        { id: 'ui-orders', name: 'Your orders', flow: 'flow-orders', specced: true },
        { id: 'ui-order-detail', name: 'Order detail', flow: 'flow-orders' },
      ],
      edges: [
        { from: 'root-shop', to: 'ui-browse', label: 'Browse' },
        { from: 'root-shop', to: 'ui-orders', label: 'Orders' },
        { from: 'root-shop', to: 'ui-review-cart', label: 'Cart' },
        { from: 'ui-browse', to: 'ui-product' },
        { from: 'ui-product', to: 'ui-review-cart' },
        { from: 'ui-review-cart', to: 'ui-choose-payment' },
        { from: 'ui-choose-payment', to: 'ui-confirmation' },
        { from: 'ui-confirmation', to: 'ui-browse', label: 'keep shopping' },
        { from: 'ui-orders', to: 'ui-order-detail' },
      ],
      flows: [
        { id: 'flow-browse', name: 'Browse & add', entry: 'ui-browse' },
        { id: 'flow-checkout', name: 'Checkout', entry: 'ui-review-cart' },
        { id: 'flow-orders', name: 'Your orders', entry: 'ui-orders' },
      ],
    },
    {
      id: 'acme-admin', name: 'Acme Admin', root: 'root-admin',
      globalActions: ['cmd-sign-out'],
      pages: [
        { id: 'ui-queue', name: 'Fulfilment queue', flow: 'flow-fulfil' },
        { id: 'ui-ship', name: 'Ship order', flow: 'flow-fulfil' },
        { id: 'ui-order-admin', name: 'Order detail', flow: 'flow-refunds' },
        { id: 'ui-issue-refund', name: 'Issue refund', flow: 'flow-refunds' },
      ],
      edges: [
        { from: 'root-admin', to: 'ui-queue', label: 'Fulfilment' },
        { from: 'root-admin', to: 'ui-order-admin', label: 'Orders' },
        { from: 'ui-queue', to: 'ui-ship' },
        { from: 'ui-order-admin', to: 'ui-issue-refund' },
      ],
      flows: [
        { id: 'flow-fulfil', name: 'Fulfil order', entry: 'ui-queue' },
        { id: 'flow-refunds', name: 'Issue refund', entry: 'ui-order-admin' },
      ],
    },
  ],
};

/* -------- UI-step spec sheets (§3.2.1) — modelled meaning on top of the
   contract's buildable core, for the fully-specced checkout screens. -------- */
window.PF.stepSpecs = {
  'ui-review-cart': {
    emphasis: 'The total owed is the decisive figure at this step.',
    transitions: [{ on: 'cmd-begin-payment', to: 'ui-choose-payment' }],
    inheritedWcag: { '1.3.1': 'display-collection, display-value', '2.5.8': 'trigger-action', '2.4.7': 'trigger-action' },
    stepWcag: {},
    intentReliance: 0,
  },
  'ui-choose-payment': {
    emphasis: 'The available methods are peers; confirm is the single decisive act.',
    transitions: [{ on: 'cmd-authorize-payment', to: 'ui-confirmation' }, { on: 'ev-payment-declined', to: 'ui-choose-payment', note: 'stay, surface failed meaning' }],
    inheritedWcag: { '1.3.1': 'single-select', '4.1.2': 'single-select', '2.5.8': 'trigger-action', '2.4.7': 'trigger-action' },
    stepWcag: {},
    intentReliance: 1,
    intentNote: 'grouping of methods consulted intent once — candidate for a context-of-use rule',
  },
  'ui-confirmation': {
    emphasis: 'Reassurance first; the order number is the artifact to keep.',
    transitions: [{ on: 'cmd-continue-shopping', to: 'ui-browse' }],
    inheritedWcag: { '1.3.1': 'display-value \u00d72', '2.5.8': 'trigger-action' },
    stepWcag: {},
    intentReliance: 0,
  },
  'ui-browse': {
    emphasis: 'The catalog is the content; the cart action is ambient.',
    transitions: [{ on: 'cmd-open-cart', to: 'ui-review-cart' }],
    inheritedWcag: { '1.3.1': 'display-collection', '2.5.8': 'trigger-action' },
    stepWcag: {},
    intentReliance: 0,
  },
  'ui-orders': {
    emphasis: 'Recency matters; the most recent order comes first.',
    transitions: [],
    inheritedWcag: { '1.3.1': 'display-collection' },
    stepWcag: {},
    intentReliance: 0,
  },
};

/* WCAG criteria referenced anywhere — ingested entities (§3.2.3) */
window.PF.wcag = {
  '1.1.1': { name: 'Non-text Content', level: 'A', vtype: 'assisted' },
  '1.3.1': { name: 'Info and Relationships', level: 'A', vtype: 'assisted' },
  '1.4.3': { name: 'Contrast (Minimum)', level: 'AA', vtype: 'machine' },
  '2.4.7': { name: 'Focus Visible', level: 'AA', vtype: 'machine' },
  '2.5.8': { name: 'Target Size (Minimum)', level: 'AA', vtype: 'machine' },
  '3.3.2': { name: 'Labels or Instructions', level: 'A', vtype: 'assisted' },
  '4.1.2': { name: 'Name, Role, Value', level: 'A', vtype: 'machine' },
};

/* -------- the AIO catalog (§3.2.2) — the core vocabulary. Meaning, typing,
   inherited obligations, and reification per context of use. -------- */
window.PF.aios = [
  { id: 'trigger-action', means: 'invoke an operation', typedOver: 'a command',
    wcag: ['2.5.8', '2.4.7'],
    reify: { phone: 'full-width button', desktop: 'inline button', tui: 'labelled key / reverse-video button' } },
  { id: 'single-select', means: 'choose one from a set', typedOver: 'a command parameter / a domain enumeration',
    wcag: ['1.3.1', '4.1.2'],
    reify: { phone: 'option list (\u226440) / searchable list', desktop: 'segmented control (\u22645) / select', tui: 'highlighted list, arrow keys' } },
  { id: 'multi-select', means: 'choose any number from a set', typedOver: 'a command parameter / a collection',
    wcag: ['1.3.1', '4.1.2'],
    reify: { phone: 'checkbox list', desktop: 'checkbox group / token field', tui: 'toggle list, space to mark' } },
  { id: 'text-entry', means: 'supply a typed value', typedOver: 'a command payload field (type from the domain model)',
    wcag: ['1.3.1', '3.3.2', '4.1.2'],
    reify: { phone: 'field + matched keyboard', desktop: 'field', tui: 'inline edit line' } },
  { id: 'numeric-entry', means: 'supply a number', typedOver: 'a command payload field (numeric domain type)',
    wcag: ['1.3.1', '3.3.2', '4.1.2'],
    reify: { phone: 'field + numeric keypad', desktop: 'field + stepper', tui: 'inline edit, digits only' } },
  { id: 'date-entry', means: 'supply a date', typedOver: 'a command payload field (date domain type)',
    wcag: ['1.3.1', '3.3.2', '4.1.2'],
    reify: { phone: 'native date wheel', desktop: 'calendar popover', tui: 'masked field YYYY-MM-DD' } },
  { id: 'display-value', means: 'show a single datum', typedOver: 'a projected field',
    wcag: ['1.3.1'],
    reify: { phone: 'value block', desktop: 'value block / stat', tui: 'label: value line' } },
  { id: 'display-collection', means: 'show many of a kind', typedOver: 'a projected collection',
    wcag: ['1.3.1'],
    reify: { phone: 'stacked list', desktop: 'table with columns', tui: 'box-drawn table' } },
  { id: 'navigate', means: 'move between interaction spaces', typedOver: 'a transition (§3.2.1)',
    wcag: ['2.4.7'],
    reify: { phone: 'tab bar / drawer', desktop: 'persistent sidebar', tui: 'function keys / menu bar' } },
  { id: 'edit', means: 'revise an existing value in place', typedOver: 'a projected field + the command that updates it',
    wcag: ['1.3.1', '3.3.2', '4.1.2'],
    reify: { phone: 'tap-to-edit sheet', desktop: 'inline edit on click', tui: 'edit mode on the row' } },
];

/* -------- reference data (§3.1) — constitutive instance data: part of the What.
   The test: if changing a datum changes what the system MEANS, it belongs here. -------- */
window.PF.refData = [
  { id: 'ref-payment-methods', name: 'Payment methods', values: ['card', 'apple-pay', 'pay-on-delivery'],
    referenceFor: 'cmd-select-method · PaymentMethod', conformance: 'verified',
    note: 'remove one and choose-payment behaviour becomes impossible — constitutive' },
  { id: 'ref-shipping-methods', name: 'Shipping methods', values: ['standard', 'express'],
    referenceFor: 'Order.shipping_method', conformance: 'verified',
    note: 'events and invariants reference the set, not the strings' },
  { id: 'ref-tax-categories', name: 'Tax categories', values: ['standard-20', 'reduced-7', 'zero'],
    referenceFor: 'LineItem.tax_category · Money maths', conformance: 'realised',
    note: 'behaviour is undefined without it — totals cannot be computed' },
  { id: 'ref-currencies', name: 'Supported currencies', values: ['EUR'],
    referenceFor: 'Money.currency', conformance: 'verified',
    note: 'What 2.0 multi-currency widens this set — a reference-data change IS a What change (§7.3)' },
];

/* -------- production data (§3.1 / §13) — the oracle, not the What.
   Shapes asserted continuously; failures triaged bidirectionally. -------- */
window.PF.oracle = {
  datasets: [
    { id: 'prod-orders', name: 'Production orders', shape: 'Order shape (SHACL)', rows: '48,112',
      assertion: 'on write', rate: 2.1, trend: [0.8, 0.7, 0.9, 1.4, 1.1, 1.6, 2.1], rising: true },
    { id: 'prod-catalog', name: 'Production catalog', shape: 'Product shape (SHACL)', rows: '4,200',
      assertion: 'nightly', rate: 0.2, trend: [0.3, 0.2, 0.2, 0.1, 0.2, 0.2, 0.2], rising: false },
  ],
  violations: [
    { id: 'v-1', shape: 'ORDER-REFUND-1 · refund_total ≤ paid_total', dataset: 'prod-orders', count: 14,
      triage: 'data defect', note: 'legacy refunds imported before the invariant — fix the data' },
    { id: 'v-2', shape: 'Order.shipping_method ∈ ref-shipping-methods', dataset: 'prod-orders', count: 37,
      triage: 'spec drift', note: 'production shows “pickup-point” — the model fell behind; promote it into the reference set' },
  ],
};

/* screens that use an AIO — derived by scanning the contract */
window.PF.aioUsage = function (aioId) {
  return window.PF.contract.screens
    .filter(s => s.elements.some(e => e.aio === aioId))
    .map(s => s.id);
};
window.PF.screen = function (id) { return window.PF.contract.screens.find(s => s.id === id); };
window.PF.resolveContent = function (key, locale) {
  const e = window.PF.contract.content_store[key];
  const loc = locale || window.PF.contract.locale;
  return (e && e[loc] != null) ? e[loc] : '\u27e8missing: ' + key + '@' + loc + '\u27e9';
};
/* which screens reference a content key (references_content, §9) */
window.PF.contentUsage = function (key) {
  return window.PF.contract.screens.filter(s =>
    (s.content && Object.values(s.content).includes(key)) ||
    (s.state_content && Object.values(s.state_content).includes(key))
  ).map(s => s.id);
};
