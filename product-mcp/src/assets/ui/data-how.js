/* ============================================================
   Behaviour (§3.3–3.5), the How (§4), the Build seam (§5) and
   verification (§6) — data for the Deciders / Scenarios /
   Decisions / Layout / Reification / Work units / Verifications
   views. Loads after data-ui.js; extends window.PF.
   ============================================================ */

/* -------- Deciders (§3.3) — signature derived, logic authored -------- */
window.PF.deciders = [
  {
    id: 'decider-cart', aggregate: 'Cart', conformance: 'verified',
    handles: [
      { cmd: 'cmd-add-item', emits: ['ev-item-added'], rejects: [] },
      { cmd: 'cmd-begin-payment', emits: ['ev-payment-begun'], rejects: ['CART-1'] },
    ],
    evolves: ['ev-item-added \u2192 items, subtotal'],
    stateRead: [
      { field: 'items', readBy: 'CART-1 \u2014 an empty cart cannot begin payment' },
      { field: 'subtotal', readBy: 'CART-2 \u2014 subtotal equals the sum of line totals' },
    ],
    rejections: [
      { id: 'CART-1', rule: 'items is non-empty at begin-payment', reachable: true },
      { id: 'CART-2', rule: 'subtotal = \u03a3 line totals', reachable: true },
    ],
    coverage: { commands: '2/2', foreign: 0, outputs: 'contained' },
  },
  {
    id: 'decider-order', aggregate: 'Order', conformance: 'verified',
    handles: [
      { cmd: 'cmd-authorize-payment', emits: ['ev-payment-authorized', 'ev-order-placed'], rejects: ['payment declined'] },
      { cmd: 'cmd-accept-fulfilment', emits: ['ev-fulfilment-accepted'], rejects: ['already accepted'] },
      { cmd: 'cmd-mark-shipped', emits: ['ev-order-shipped'], rejects: ['not yet accepted'] },
      { cmd: 'cmd-issue-refund', emits: ['ev-refund-issued'], rejects: ['ORDER-REFUND-1'] },
    ],
    evolves: ['ev-payment-authorized \u2192 paid_total', 'ev-refund-issued \u2192 refund_total', 'ev-fulfilment-accepted \u2192 status'],
    stateRead: [
      { field: 'paid_total', readBy: 'ORDER-REFUND-1 at refund time' },
      { field: 'refund_total', readBy: 'ORDER-REFUND-1 at refund time' },
      { field: 'status', readBy: 'ship-after-accept rejection' },
    ],
    rejections: [
      { id: 'ORDER-REFUND-1', rule: 'refund_total + amount \u2264 paid_total', reachable: true },
      { id: 'ship-after-accept', rule: 'shipped only from accepted', reachable: true },
    ],
    coverage: { commands: '4/4', foreign: 0, outputs: 'contained' },
  },
  {
    id: 'decider-loyalty', aggregate: 'LoyaltyAccount', conformance: 'described', planned: true,
    handles: [{ cmd: 'cmd-accrue-points', emits: ['ev-points-accrued'], rejects: ['balance never negative'] }],
    evolves: ['ev-points-accrued \u2192 balance'],
    stateRead: [{ field: 'balance', readBy: 'non-negative rejection (pending)' }],
    rejections: [{ id: 'LOYALTY-1', rule: 'balance \u2265 0', reachable: false, note: 'not yet modelled \u2014 What 2.0' }],
    coverage: { commands: '1/1', foreign: 0, outputs: 'contained' },
  },
];

/* -------- Projectors (§3.4) — the evolve half, symmetric -------- */
window.PF.projectors = [
  { id: 'proj-cart-summary', readModel: 'rm-cart-summary', conformance: 'verified',
    folds: [{ ev: 'ev-item-added', into: 'append item \u00b7 add to total' }],
    outputs: ['items[] {name, qty, price}', 'total: money'],
    consumers: ['ui-review-cart'], coverage: { events: '1/1', foreign: 0, outputs: 'contained' } },
  { id: 'proj-order-confirmation', readModel: 'rm-order-confirmation', conformance: 'verified',
    folds: [{ ev: 'ev-order-placed', into: 'set message \u00b7 set order_no' }],
    outputs: ['message: string', 'order_no: string'],
    consumers: ['ui-confirmation'], coverage: { events: '1/1', foreign: 0, outputs: 'contained' } },
  { id: 'proj-order-history', readModel: 'rm-order-history', conformance: 'verified',
    folds: [{ ev: 'ev-order-placed', into: 'append order row' }, { ev: 'ev-refund-issued', into: 'annotate refunded total' }],
    outputs: ['orders[] {order_no, total}'],
    consumers: ['ui-orders'], coverage: { events: '2/2', foreign: 0, outputs: 'contained' } },
];

/* -------- simulation scenarios (§3.3/§3.4) — the oracle, consumed twice -------- */
window.PF.scenarios = [
  { id: 'sc-checkout-empty', kind: 'decide', decider: 'decider-cart', flow: 'flow-checkout',
    given: [], when: 'cmd-begin-payment',
    then: { verdict: 'Rejected', reason: 'CART-1 \u2014 an empty cart cannot begin payment' },
    simulated: 'pass', realised: 'pass' },
  { id: 'sc-checkout-happy', kind: 'decide', decider: 'decider-cart', flow: 'flow-checkout',
    given: ['ev-item-added \u00d72 (coffee beans, filter papers)'], when: 'cmd-begin-payment',
    then: { verdict: 'Accepted', events: ['ev-payment-begun'] },
    simulated: 'pass', realised: 'pass' },
  { id: 'sc-refund-over', kind: 'decide', decider: 'decider-order', flow: 'flow-refunds',
    given: ['paid 100', 'refunded 60'], when: 'cmd-issue-refund (50)',
    then: { verdict: 'Rejected', reason: 'ORDER-REFUND-1 \u2014 60 + 50 > 100' },
    simulated: 'pass', realised: 'pass' },
  { id: 'sc-refund-ok', kind: 'decide', decider: 'decider-order', flow: 'flow-refunds',
    given: ['paid 100', 'refunded 60'], when: 'cmd-issue-refund (40)',
    then: { verdict: 'Accepted', events: ['ev-refund-issued'] },
    simulated: 'pass', realised: 'pass' },
  { id: 'sc-ship-early', kind: 'decide', decider: 'decider-order', flow: 'flow-fulfil',
    given: ['ev-order-placed'], when: 'cmd-mark-shipped',
    then: { verdict: 'Rejected', reason: 'ship-after-accept \u2014 not yet accepted' },
    simulated: 'pass', realised: 'pending', note: 'behavioural conformance still amber (rel-2)' },
  { id: 'sc-proj-cart', kind: 'project', projector: 'proj-cart-summary', flow: 'flow-checkout',
    given: ['ev-item-added (coffee beans, 2, 1800)', 'ev-item-added (filter papers, 1, 600)'],
    then: { state: 'rm-cart-summary = { items: 2 rows, total: 4200 }' },
    simulated: 'pass', realised: 'pass' },
  { id: 'sc-loyalty-accrue', kind: 'decide', decider: 'decider-loyalty', flow: 'flow-loyalty',
    given: ['ev-order-placed (4200)'], when: 'cmd-accrue-points',
    then: { verdict: 'Accepted', events: ['ev-points-accrued (42)'] },
    simulated: 'pending', realised: 'pending', note: 'slice not built \u2014 What 2.0' },
];

/* -------- the How (§4.1–4.4) — blueprint, decisions, principles, patterns, contracts -------- */
window.PF.how = {
  /* §4 intro: wherever a system shape recurs, the How is captured once and reused —
     a BLUEPRINT (a reusable How; formerly termed archetype). Instantiating it
     produces DeployableUnits — the concrete deployable artifacts (v1.7.0). */
  blueprint: {
    id: 'bp-event-sourced-storefront', name: 'Event-sourced storefront',
    packages: ['application contract', 'repository layout model', '7 principles', '8 patterns', 'wire-ds binding', 'wire-content binding'],
    instances: [{ sys: 'acme-shop', conformance: 'verified' }, { sys: 'acme-admin', conformance: 'realised' }],
    note: 'two systems, one How — the blueprint is the reusable answer to a recurring shape',
  },
  /* DeployableUnits — concrete instances of the blueprint (§4). The DORA unit:
     deployment frequency, lead time, change-fail rate all count per unit.
     Mapping is 1:1:1 in the common case; fan-out must be declared. */
  deployableUnits: [
    { id: 'du-shop-prod', system: 'acme-shop', env: 'production',
      identity: 'shop.acme.com · com.acme.shop · Node 22 / eu-west', frozen: true },
    { id: 'du-shop-staging', system: 'acme-shop', env: 'staging',
      identity: 'staging.shop.acme.com · com.acme.shop.beta', frozen: true },
    { id: 'du-admin-prod', system: 'acme-admin', env: 'production',
      identity: 'admin.acme.com · web only · eu-west', frozen: true },
    { id: 'du-admin-staging', system: 'acme-admin', env: 'staging',
      identity: 'staging.admin.acme.com', frozen: true },
  ],
  decisions: [
    { id: 'dec-event-sourced', title: 'Event-sourced Ordering context', why: 'refunds and fulfilment need full history; the event model is already the spec',
      applies: 'Ordering domain', not: 'Catalog — CRUD suffices', licenses: ['prin-pure-decisions', 'prin-projections-derived', 'prin-generated-surfaces'] },
    { id: 'dec-vertical-slices', title: 'Vertical slice layout', why: 'a slice carries everything from trigger to view — the unit of work should be the unit of code',
      applies: 'acme-shop, acme-admin repos', not: 'shared kernel packages', licenses: ['prin-slice-complete'] },
    { id: 'dec-bind-wire-ds', title: 'Bind screens to wire-ds', why: 'a design system is the screen standard — inventing a UI description language forfeits its components, tokens, and a11y (§4.5)',
      applies: 'every GUI screen', not: 'the TUI admin console — separate reification', licenses: ['prin-closed-vocabulary', 'prin-tokens-not-literals'] },
    { id: 'dec-postgres-eventstore', title: 'Postgres as the event store', why: 'append-only table + notify beats a new infra dependency at this scale',
      applies: 'all Ordering events', not: 'catalog reads — plain tables', licenses: ['prin-projections-derived'] },
    { id: 'dec-observability', title: 'Telemetry-first runtime bounds', why: 'a quality demand without a measurement is prose — every budget gets its probe at birth (§3.6)',
      applies: 'flows with latency budgets', not: 'build-time architectural constraints', licenses: ['prin-budgets-measured'] },
  ],
  principles: [
    { id: 'prin-pure-decisions', text: 'decision logic is kept in a pure, isolable core — no I/O in decide()', enforcedBy: 'contract conformance', appliedBy: 'wu-checkout-refund-decider-0007' },
    { id: 'prin-projections-derived', text: 'read models are derived by fold, never written directly', enforcedBy: 'behavioural conformance', appliedBy: 'wu-cart-projector-0004' },
    { id: 'prin-slice-complete', text: 'every slice is structurally complete — code, tests, contract in one place', enforcedBy: 'layout conformance', appliedBy: 'every scaffolding unit' },
    { id: 'prin-closed-vocabulary', text: 'a screen composes only components the design system defines', enforcedBy: 'seam verification', appliedBy: 'wu-review-cart-page-0009' },
    { id: 'prin-tokens-not-literals', text: 'colour, spacing, type are token references — a literal style value is non-conformant', enforcedBy: 'seam verification', appliedBy: 'wu-review-cart-page-0009' },
    { id: 'prin-generated-surfaces', text: 'interface contracts are generated from the domain model, never hand-written', enforcedBy: 'contract conformance', appliedBy: 'every scaffolding unit' },
    { id: 'prin-budgets-measured', text: 'every declared budget is measured — a probe ships with the flow it bounds', enforcedBy: 'runtime-bound conformance', appliedBy: 'every scaffolding unit' },
  ],
  patterns: [
    { id: 'pat-decider-module', text: 'one Decider module per aggregate: decide + evolve + scenarios', implements: 'prin-pure-decisions',
      files: [
        { path: 'src/features/{slice}/{agg}.decide.ts', verb: 'lays down', note: 'pure decide() — rejections from the invariants' },
        { path: 'src/features/{slice}/{agg}.evolve.ts', verb: 'lays down', note: 'the state fold — no I/O' },
        { path: 'scenarios/{agg}.scenarios.json', verb: 'freezes', note: 'the simulation oracle, consumed twice' },
      ], rules: ['slice-files'], units: ['wu-checkout-refund-decider-0007'] },
    { id: 'pat-projector-per-view', text: 'one Projector per read model, rebuildable from the log', implements: 'prin-projections-derived',
      files: [
        { path: 'src/features/{slice}/{view}.projector.ts', verb: 'lays down', note: 'the fold — project(state, event)' },
        { path: 'src/app/main.ts', verb: 'modifies', note: 'registers the fold on the event stream' },
      ], rules: ['slice-files', 'composition-root'], units: ['wu-cart-projector-0004', 'wu-fulfilment-projector-0012'] },
    { id: 'pat-feature-folder', text: 'src/features/{slice}/ — handler, projector, tests as siblings', implements: 'prin-slice-complete',
      files: [
        { path: 'src/features/{slice}/', verb: 'creates', note: 'the slice — the unit of work is the unit of code' },
        { path: 'src/features/{slice}/{slice}.handler.ts', verb: 'lays down', note: 'the command endpoint' },
        { path: 'src/features/{slice}/{slice}.test.ts', verb: 'requires', note: 'a slice without tests is incomplete' },
        { path: 'src/app/main.ts', verb: 'modifies', note: 'mounts the slice — the only shared file it touches' },
      ], rules: ['slice-has-tests', 'slice-files', 'composition-root'], units: ['wu-scaffold'] },
    { id: 'pat-translation-adapter', text: 'cross-system reads go through one Translation adapter per source', implements: 'prin-slice-complete',
      files: [
        { path: 'src/features/{slice}/{source}.translation.ts', verb: 'lays down', note: 'the only legal cross-system channel (§3.0.1)' },
        { path: 'scenarios/{source}.translation.scenarios.json', verb: 'freezes', note: 'given published events → then translated commands' },
      ], rules: ['slice-files'], units: ['wu-fulfilment-projector-0012'] },
    { id: 'pat-page-composition', text: 'page = template + organisms from wire-ds, bound to its UI step', implements: 'prin-closed-vocabulary',
      files: [
        { path: 'src/features/{slice}/{page}.page.ts', verb: 'lays down', note: 'composes only wire-ds CIOs — checked at the seam' },
        { path: 'src/features/{slice}/{page}.content.ts', verb: 'declares', note: 'the content keys + roles the page references (§4.6)' },
      ], rules: ['slice-files'], units: ['wu-review-cart-page-0009'] },
    { id: 'pat-token-theme', text: 'one theme file maps wire-ds tokens → CSS custom properties', implements: 'prin-tokens-not-literals',
      files: [
        { path: 'src/app/theme.ts', verb: 'lays down', note: 'the single token→value mapping — no literals anywhere else' },
      ], rules: ['app-shell'], units: ['wu-review-cart-page-0009'] },
    { id: 'pat-interface-generation', text: 'interface surfaces are generated from the domain model (§4.4)', implements: 'prin-generated-surfaces',
      files: [
        { path: 'src/contracts/{surface}.openapi.yaml', verb: 'generates', note: 'from cmd-* payloads & rm-* shapes' },
        { path: 'src/contracts/{stream}.asyncapi.yaml', verb: 'generates', note: 'from ev-* declarations' },
      ], rules: ['contracts-isolation'], units: ['wu-scaffold'] },
    { id: 'pat-latency-probe', text: 'every flow with a budget ships its probe — the demand is measured where it is located (§3.6)', implements: 'prin-budgets-measured',
      files: [
        { path: 'src/app/telemetry.ts', verb: 'lays down', note: 'budget registry + p99 emitters' },
        { path: 'src/features/{slice}/{slice}.handler.ts', verb: 'modifies', note: 'wraps every bounded flow’s handler in a probe' },
        { path: 'src/features/checkout/checkout.handler.ts', verb: 'modifies', note: 'checkout ≤ 3s — the declared budget (§3.6)' },
      ], rules: ['app-shell', 'slice-files'], units: ['wu-scaffold'] },
  ],
  contracts: [
    { id: 'contract-application', kind: 'application', items: ['TypeScript \u00b7 vertical slices \u00b7 event store append-only', 'decision logic pure & isolable (the Decider constraint, \u00a73.3)', 'projections rebuildable from the log'], frozen: true,
      scope: 'stable across all DeployableUnits instantiated from the blueprint' },
    { id: 'contract-runtime', kind: 'infrastructure / runtime', items: ['web: shop.acme.com \u00b7 iOS bundle com.acme.shop', 'Node 22 runtime \u00b7 Postgres event store', 'regions: eu-west \u2014 satisfies the residency demand (\u00a73.6 Kind 2)'], frozen: true, satisfies: 'contract-application',
      scope: 'one per DeployableUnit \u2014 each unit is one such contract; staging and production are two units' },
  ],
  standards: [
    { surface: 'REST interface', standard: 'OpenAPI', derived: 'from cmd-* payloads & rm-* shapes' },
    { surface: 'Event stream', standard: 'AsyncAPI', derived: 'from ev-* declarations' },
    { surface: 'Auth', standard: 'OIDC', derived: 'ingested, not invented' },
  ],
  layout: [
    { id: 'composition-root', kind: 'must-exist', glob: 'src/app/main.ts', cardinality: 'exactly 1',
      rationale: 'the composition root; the app is not runnable without it', enforces: 'explicit-composition-root', verdict: 'pass' },
    { id: 'app-shell', kind: 'may-exist-here', glob: 'src/app/**',
      rationale: 'the shell: composition root + the token theme — nothing else lives here', enforces: 'prin-tokens-not-literals', verdict: 'pass' },
    { id: 'slice-has-tests', kind: 'must-exist', glob: 'src/features/*/ \u2192 {dir}/*.test.ts', cardinality: '1 per slice',
      rationale: 'a slice is structurally incomplete without its tests', enforces: 'every-slice-tested', verdict: 'pass' },
    { id: 'slice-files', kind: 'may-exist-here', glob: 'src/features/*/**',
      rationale: 'the slice interior stays free \u2014 constrain the skeleton, not the cytoplasm', enforces: 'prin-slice-complete', verdict: 'pass' },
    { id: 'contracts-isolation', kind: 'may-exist-here', glob: 'src/contracts/**',
      rationale: 'consumers depend on shape, not implementation; generated, never hand-written (\u00a74.4)', enforces: 'prin-generated-surfaces', verdict: 'pass' },
    { id: 'no-secrets-in-source', kind: 'must-not-exist', glob: '**/*.secrets.json',
      rationale: 'secrets never live in source', enforces: 'secrets-out-of-repo', verdict: 'pass' },
    { id: 'no-orphans', kind: 'no-orphans', glob: 'every file under src/ matches an allow rule',
      rationale: 'the unanticipated file is the failure case \u2014 allowlist, not denylist', enforces: 'provable shape', verdict: 'fail',
      finding: 'src/util/helpers.ts matches no allow rule' },
  ],
};

/* the repository, as the layout model sees it — each row cites the rule that admits it */
window.PF.repoTree = [
  { line: 'acme-shop/', dir: true },
  { line: '├─ src/', dir: true },
  { line: '│  ├─ app/', dir: true },
  { line: '│  │  ├─ main.ts', rule: 'composition-root', verdict: 'ok', note: 'exactly 1 ✓' },
  { line: '│  │  └─ theme.ts', rule: 'app-shell', verdict: 'ok', note: 'tokens → CSS vars' },
  { line: '│  ├─ features/', dir: true },
  { line: '│  │  ├─ checkout/', dir: true },
  { line: '│  │  │  ├─ begin-payment.ts', rule: 'slice-files', verdict: 'ok' },
  { line: '│  │  │  ├─ cart.decider.ts', rule: 'slice-files', verdict: 'ok' },
  { line: '│  │  │  ├─ cart.projector.ts', rule: 'slice-files', verdict: 'ok' },
  { line: '│  │  │  └─ checkout.test.ts', rule: 'slice-has-tests', verdict: 'ok', note: '1 per slice ✓' },
  { line: '│  │  ├─ orders/', dir: true },
  { line: '│  │  │  ├─ order-history.projector.ts', rule: 'slice-files', verdict: 'ok' },
  { line: '│  │  │  └─ orders.test.ts', rule: 'slice-has-tests', verdict: 'ok', note: '1 per slice ✓' },
  { line: '│  │  └─ refunds/', dir: true },
  { line: '│  │     ├─ refund.decider.ts', rule: 'slice-files', verdict: 'ok' },
  { line: '│  │     └─ refunds.test.ts', rule: 'slice-has-tests', verdict: 'ok', note: '1 per slice ✓' },
  { line: '│  ├─ contracts/', dir: true },
  { line: '│  │  ├─ orders.openapi.yaml', rule: 'contracts-isolation', verdict: 'ok', note: 'generated' },
  { line: '│  │  └─ events.asyncapi.yaml', rule: 'contracts-isolation', verdict: 'ok', note: 'generated' },
  { line: '│  └─ util/', dir: true },
  { line: '│     └─ helpers.ts', rule: 'no-orphans', verdict: 'fail', note: 'matches no allow rule' },
  { line: '├─ scenarios/', dir: true },
  { line: '│  └─ refund.scenarios.json', verdict: 'ok', note: 'the frozen simulation oracle · outside src/ scope' },
  { line: '└─ package.json', verdict: 'ok' },
];

/* -------- §4.5 — the screen-composition contract: Atomic Design, normative -------- */
window.PF.composition = {
  levels: [
    { level: 'Atoms', is: 'indivisible primitives', role: 'the design system’s leaf vocabulary — a screen may not introduce an atom the system does not define', examples: ['text-label', 'money-value', 'button-core', 'radio-dot'] },
    { level: 'Molecules', is: 'small functional groups of atoms', role: 'the smallest reusable unit a UI step references', examples: ['value-block', 'list-row', 'primary-button', 'option-row'] },
    { level: 'Organisms', is: 'composite sections', role: 'where a projection or a command’s controls are bound', examples: ['line-item-list', 'method-picker', 'app-header'] },
    { level: 'Templates', is: 'a page’s layout skeleton, content-agnostic', role: 'the placement contract a page conforms to', examples: ['stack-page', 'sidebar-page'] },
    { level: 'Pages', is: 'a template filled with a flow’s data and controls', role: 'the realised UI step — one page per UI step', examples: ['pg-review-cart', 'pg-choose-payment', 'pg-confirmation'] },
  ],
  /* the worked page: pg-review-cart realizes ui-review-cart */
  page: {
    id: 'pg-review-cart', realizes: 'ui-review-cart', template: 'stack-page',
    tree: [
      { line: 'pg-review-cart', kind: 'page', bind: 'realizes_step → ui-review-cart' },
      { line: '└─ conforms_to → stack-page', kind: 'template' },
      { line: '   ├─ app-header', kind: 'organism', bind: 'root navigate edges, reified' },
      { line: '   ├─ line-item-list', kind: 'organism', bind: 'binds → rm-cart-summary.items' },
      { line: '   │  └─ list-row ×2', kind: 'molecule' },
      { line: '   │     ├─ text-label', kind: 'atom' },
      { line: '   │     └─ money-value', kind: 'atom' },
      { line: '   ├─ value-block', kind: 'molecule', bind: 'binds → rm-cart-summary.total · emphasis: primary' },
      { line: '   │  ├─ text-label', kind: 'atom' },
      { line: '   │  └─ money-value', kind: 'atom' },
      { line: '   └─ primary-button', kind: 'molecule', bind: 'issues → cmd-begin-payment · wcag 2.5.8 ✓' },
      { line: '      ├─ button-core', kind: 'atom' },
      { line: '      └─ text-label', kind: 'atom' },
    ],
    checks: [
      { t: 'every datum shown is projected by rm-cart-summary — no view needs a field no projection supplies', ok: true },
      { t: 'every control maps to a command valid at this step (cmd-begin-payment)', ok: true },
      { t: 'all components drawn from wire-ds — the closed vocabulary', ok: true },
      { t: 'styles are token references only (color.accent, space.inset, type.display)', ok: true },
    ],
  },
};

/* -------- the How process — How is binding resolution (companion doc).
   The What declares placeholders; the How resolves them in dependency order,
   each binding gated. The worklist is EXTRACTED from the What, never authored. -------- */
window.PF.howProcess = {
  steps: [
    { id: 'H1', name: 'Application contract', view: 'decisions', spec: '§4.1–4.2',
      question: 'What invariant decisions shape all the code — and which one makes behaviour verifiable?',
      gate: 'locatability — every principle placeable by H2', dep: null },
    { id: 'H2', name: 'Repository layout model', view: 'layout', spec: '§4.3',
      question: 'Where does each kind of file legally live, and what does each principle forbid?',
      gate: 'layout conformance runs — every principle enforced by ≥1 rule', dep: 'H1' },
    { id: 'H3', name: 'Bind reification', view: 'reification', spec: '§4.5', parallel: true,
      question: 'For every abstract interaction the What used, what concrete component realises it, per context?',
      gate: 'every (AIO, context) pair covered or explicitly waived', dep: 'H2' },
    { id: 'H4', name: 'Bind content', view: 'content', spec: '§4.6', parallel: true,
      question: 'For every content key, what words resolve it, in each locale?',
      gate: 'every (key, locale) pair resolves · roles honoured', dep: 'H2' },
    { id: 'H5', name: 'Generate interfaces', view: 'decisions', spec: '§4.4', parallel: true,
      question: 'What published surface does the domain model imply — generated, never hand-written?',
      gate: 'generated, not authored — hand-edits are drift', dep: 'the settled domain model' },
    { id: 'H6', name: 'Deployment identity', view: 'decisions', spec: '§4.2',
      question: 'Which DeployableUnits instantiate the blueprint, and what concrete address does each carry, per environment?',
      gate: 'every What-side system reconciled to ≥1 DeployableUnit per environment · fan-out declared', dep: 'H1/H2' },
  ],
  /* H5 surfaces — generation state */
  surfaces: [
    { id: 'orders.openapi.yaml', from: 'cmd-* payloads & rm-* shapes', generated: 'from What 1.1', drift: false },
    { id: 'events.asyncapi.yaml', from: 'ev-* declarations', generated: 'from What 1.1', drift: false },
  ],
  /* H6 — the runtime contract, per environment. The What column is fixed; only the How column changes. */
  deployments: [
    { env: 'production', rows: [
      { sys: 'acme-shop', addr: 'shop.acme.com · com.acme.shop · Node 22 / eu-west' },
      { sys: 'acme-admin', addr: 'admin.acme.com · web only · eu-west' },
    ] },
    { env: 'staging', rows: [
      { sys: 'acme-shop', addr: 'staging.shop.acme.com · com.acme.shop.beta' },
      { sys: 'acme-admin', addr: 'staging.admin.acme.com' },
    ] },
  ],
};
/* a principle is LOCATED iff a layout rule enforces it directly, or a pattern
   implementing it lands its files on rules (H1 gate: locatability) */
window.PF.principleLocated = function (pid) {
  const direct = window.PF.how.layout.find(r => r.enforces === pid);
  if (direct) return { via: 'rule ' + direct.id, direct: true };
  const pat = window.PF.how.patterns.find(p => p.implements === pid && (p.rules || []).length);
  if (pat) return { via: pat.id + ' \u2192 ' + pat.rules.join(', '), direct: false };
  return null;
};

/* -------- §11 design-system manifest (wire-ds) — the other half of the render contract -------- */
window.PF.manifest = {
  id: 'wire-ds', version: '0.1.0', wcagTarget: '2.2-AA',
  contexts: { form_factor: ['phone', 'tablet'], modality: ['touch', 'pointer'] },
  components: [
    { id: 'searchable-list', tokens: ['color.fg', 'color.bg', 'space.inset', 'type.body'],
      satisfies: [['1.3.1', 'machine'], ['4.1.2', 'machine'], ['2.4.7', 'assisted']] },
    { id: 'segmented-control', tokens: ['color.fg', 'color.bg', 'space.inset'],
      satisfies: [['1.3.1', 'machine'], ['4.1.2', 'machine']] },
    { id: 'list', tokens: ['color.fg', 'space.inset', 'type.body'], satisfies: [['1.3.1', 'machine']] },
    { id: 'value-block', tokens: ['color.fg', 'type.display'], satisfies: [['1.3.1', 'machine']] },
    { id: 'primary-button', tokens: ['color.accent', 'color.on-accent', 'space.inset', 'type.label'],
      satisfies: [['2.5.8', 'machine'], ['2.4.7', 'assisted']] },
    { id: 'drawer', tokens: ['color.bg', 'color.fg', 'space.inset'],
      satisfies: [['2.4.7', 'assisted'], ['2.1.1', 'manual']] },
    { id: 'rail', tokens: ['color.bg', 'color.fg'], satisfies: [['2.4.7', 'assisted']] },
  ],
  reification: [
    { aio: 'single-select', when: 'phone \u00b7 options: many', cio: 'searchable-list', rationale: 'no room for many side-by-side options on a phone' },
    { aio: 'single-select', when: 'tablet \u00b7 options: few', cio: 'segmented-control', rationale: 'few options, ample width \u2014 direct choice beats a menu' },
    { aio: 'display-collection', when: 'any', cio: 'list', rationale: 'the canonical collection surface' },
    { aio: 'display-value', when: 'any', cio: 'value-block', rationale: 'a single datum reads as a stat' },
    { aio: 'trigger-action', when: 'emphasis: primary', cio: 'primary-button', rationale: 'the decisive act gets the accent' },
    { aio: 'navigate', when: 'root \u00b7 phone', cio: 'drawer', rationale: 'global destinations behind a drawer on a phone' },
    { aio: 'navigate', when: 'root \u00b7 tablet', cio: 'rail', rationale: 'persistent rail \u2014 width allows it' },
  ],
  unreifiable: [
    { aio: 'display-collection (of images)', cls: 'TUI', rationale: 'no faithful character-grid form \u2014 a recorded, deliberate gap, not an omission' },
  ],
  tokens: ['color.fg', 'color.bg', 'color.accent', 'color.on-accent', 'space.inset', 'type.body', 'type.label', 'type.display'],
};

/* -------- the Build seam (§5.1) — one worked message pair -------- */
window.PF.workUnits = [
  {
    id: 'wu-checkout-refund-decider-0007', lineage: 'deliverable-refunds',
    hash: 'b3d1f2a9c7e8\u2026a09f8e7', status: 'accepted',
    bundle: {
      schema: { artifact: 'module \u00b7 typescript', criteria: ['exports a pure decide(order, cmd)', 'rejects ORDER-REFUND-1 when refund_total + amount > paid_total', 'passes the frozen simulation scenarios'] },
      prompt: 'Realise the Order refund Decider from its specification \u2014 decide() only; no I/O, no persistence.',
      model: 'code-implementation (abstract capability \u2014 the executor maps it)',
      context: ['Order#decide (inlined)', 'ORDER-REFUND-1', 'domain: Order, Money', '2 frozen simulation scenarios'],
    },
    verdict: { event: 'ev-9f81c0a4\u2026', at: '2026-06-26T02:14:08Z', verdict: 'accepted', consequence: 'advance',
      findings: ['pure-decide: pass', 'ORDER-REFUND-1 rejection: pass', 'scenario parity 2/2: pass'] },
  },
  {
    id: 'wu-fulfilment-projector-0012', lineage: 'deliverable-fulfilment',
    hash: '7c2e0b91d4aa\u2026e11c202', status: 'escalate',
    bundle: {
      schema: { artifact: 'module \u00b7 typescript', criteria: ['folds ev-fulfilment-accepted & ev-order-shipped', 'output contains only declared fields'] },
      prompt: 'Realise the pending-fulfilments Projector from its specification.',
      model: 'code-implementation',
      context: ['rm-pending-fulfilments (inlined)', 'events: ev-fulfilment-accepted, ev-order-shipped', '1 frozen scenario'],
    },
    verdict: { event: 'ev-2a67d0f1\u2026', at: '2026-06-30T11:42:51Z', verdict: 'escalate', consequence: 'human review',
      findings: ['fold coverage: pass', 'scenario parity 0/1: fail \u2014 shipped orders linger in the queue'] },
  },
];

/* -------- verification kinds (§6.3) — the required set, with Acme's standing -------- */
window.PF.verificationKinds = [
  { kind: 'Layout conformance', oracle: 'repository layout model (\u00a74.3)', when: 'build \u00b7 first (cheapest)', verdict: 'fail', finding: 'no-orphans: 1 file matches no allow rule' },
  { kind: 'Behavioural simulation', oracle: 'flow-derived scenarios (\u00a73.3)', when: 'before realisation', verdict: 'pass' },
  { kind: 'Pattern conformance', oracle: 'the four patterns (\u00a73.2.0)', when: 'build', verdict: 'pass' },
  { kind: 'Journey conformance', oracle: 'journeys & Translations (\u00a73.0.1)', when: 'build', verdict: 'pass' },
  { kind: 'Internal coherence', oracle: 'one unit\u2019s own output', when: 'per work unit', verdict: 'pass' },
  { kind: 'Contract conformance', oracle: 'the How\u2019s contracts (\u00a74.2)', when: 'build', verdict: 'pass' },
  { kind: 'Seam', oracle: 'contracts \u00b7 UI steps vs screens (\u00a74.5)', when: 'build', verdict: 'pass' },
  { kind: 'Domain conformance', oracle: 'domain model (\u00a73.1)', when: 'build', verdict: 'pass' },
  { kind: 'Data conformance', oracle: 'declared shapes vs production (\u00a73.1)', when: 'continuous', verdict: 'fail', finding: 'divergence 2.1% and rising \u2014 51 rows, 2 triaged' },
  { kind: 'State justification', oracle: 'every field has a reader (\u00a73.4)', when: 'build', verdict: 'fail', finding: 'Customer.credit_limit read by no Decider \u2014 unmodelled invariant?' },
  { kind: 'Runtime-bound conformance', oracle: 'quality demands vs telemetry (\u00a73.6)', when: 'continuous', verdict: 'pass', finding: 'checkout p99 1.9s \u2264 3s budget' },
  { kind: 'Behavioural conformance', oracle: 'Decider/Projector scenarios (\u00a73.3\u20134)', when: 'after realisation', verdict: 'partial', finding: 'fulfilment projector: scenario parity 0/1' },
  { kind: 'Oracle conformance', oracle: 'named-algorithm reference pairs (\u00a73.5)', when: 'build', verdict: 'n/a', finding: 'no named-algorithm primitives declared' },
];
