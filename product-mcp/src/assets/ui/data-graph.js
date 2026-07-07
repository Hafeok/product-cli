/* ============================================================
   THE GRAPH (§2 / §9) — the whole instance as one connected,
   queryable graph. Nothing here is authored: every node and
   edge below is DERIVED by scanning the PF data the other
   views already render. Loads after data-how.js.
   ============================================================ */
window.PF.buildGraph = function () {
  const PF = window.PF;
  const nodes = [], edges = [], seen = {};
  const N = (id, label, kind, sector, ring, view, dashed) => {
    if (seen[id]) return;
    seen[id] = true;
    nodes.push({ id, label, kind, sector, ring, view: view || null, dashed: !!dashed });
  };
  const E = (from, to, rel) => { edges.push({ from, to, rel }); };

  /* ---------- WHAT ---------- */
  // The centre node is the selected product — driven by the live PF.product
  // projection (falls back to the bundled demo when /api/pf is unreachable).
  const P = PF.product || { id: 'acme', name: 'Acme Commerce' };
  const prodId = 'product:' + P.id;
  N(prodId, P.name, 'product', 'what', 0, 'systems');
  N('domain:ordering', 'Ordering', 'domain', 'what', 1, 'domain');
  E(prodId, 'domain:ordering', 'owns');

  PF.systems.forEach(s => {
    N('sys:' + s.id, s.name, 'system', 'what', 1, 'systems');
    E(prodId, 'sys:' + s.id, 'owns');
    E('domain:ordering', 'sys:' + s.id, 'references');
  });

  // domain concepts (incl. virtual, not-yet-built ones referenced by footprints)
  const conceptIds = new Set(PF.domain.nodes.map(n => n.id));
  PF.delivery.features.forEach(f => f.footprint.forEach(c => conceptIds.add(c)));
  conceptIds.forEach(id => {
    const c = PF.concept(id);
    N('c:' + id, c.label, 'concept-' + c.kind, 'what', 2, 'domain', (c.sub || '').includes('not built'));
    E('domain:ordering', 'c:' + id, 'declares');
  });
  PF.domain.edges.forEach(e => E('c:' + e.from, 'c:' + e.to, 'relation'));

  // flows
  Object.keys(PF.flows).forEach(fid => {
    N('f:' + fid, PF.flows[fid].name, 'flow', 'what', 3, 'flows');
  });
  PF.systems.forEach(s => s.flows.forEach(fid => { if (PF.flows[fid]) E('sys:' + s.id, 'f:' + fid, 'owns'); }));
  // virtual 2.0 flows referenced by features but not modelled yet
  PF.delivery.features.forEach(f => f.flows.forEach(fid => {
    if (!PF.flows[fid]) { N('f:' + fid, fid.replace('flow-', ''), 'flow', 'what', 3, 'flows', true); }
  }));
  // concept → flow (derived from feature footprints: the flows that pull each concept in)
  PF.delivery.features.forEach(f => f.footprint.forEach(c => f.flows.forEach(fid => E('c:' + c, 'f:' + fid, 'derives'))));

  // deciders & projectors
  const AGG = { Cart: 'cart', Order: 'order', LoyaltyAccount: 'loyalty-account' };
  PF.deciders.forEach(d => {
    N('dec:' + d.id, d.aggregate + ' Decider', 'decider', 'what', 4, 'deciders', d.planned);
    if (AGG[d.aggregate]) E('c:' + AGG[d.aggregate], 'dec:' + d.id, 'derived-from');
  });
  PF.projectors.forEach(p => {
    N('proj:' + p.id, p.readModel, 'projector', 'what', 4, 'deciders');
    p.consumers.forEach(ui => E('proj:' + p.id, 'ui:' + ui, 'projects-into'));
  });
  // flows invoke deciders (via the commands they carry)
  [['flow-checkout', 'decider-cart'], ['flow-checkout', 'decider-order'], ['flow-fulfil', 'decider-order'],
   ['flow-refunds', 'decider-order'], ['flow-loyalty', 'decider-loyalty']].forEach(([f, d]) => {
    if (seen['f:' + f] && seen['dec:' + d]) E('dec:' + d, 'f:' + f, 'decides-in');
  });

  /* ---------- UI ---------- */
  PF.aios.forEach(a => N('aio:' + a.id, a.id, 'aio', 'ui', 2, 'aios'));
  PF.contract.screens.forEach(s => {
    N('ui:' + s.id, s.name, 'screen', 'ui', 3, 'steps');
    s.elements.forEach(el => E('aio:' + el.aio, 'ui:' + s.id, 'typed-against'));
    if (s.content) Object.values(s.content).forEach(k => E('ck:' + k, 'ui:' + s.id, 'references-content'));
    if (s.state_content) Object.values(s.state_content).forEach(k => E('ck:' + k, 'ui:' + s.id, 'references-content'));
    s.elements.forEach(el => { if (el.transitions_to) E('ui:' + s.id, 'ui:' + el.transitions_to, 'navigate'); });
  });
  Object.keys(PF.contract.content_store).forEach(k => N('ck:' + k, k.split('.').slice(-2).join('.'), 'content', 'ui', 4, 'content'));
  // flows own their ui steps
  [['flow-checkout', ['ui-review-cart', 'ui-choose-payment', 'ui-confirmation']],
   ['flow-browse', ['ui-browse']], ['flow-orders', ['ui-orders']]].forEach(([f, uis]) =>
    uis.forEach(u => { if (seen['ui:' + u]) E('f:' + f, 'ui:' + u, 'contains'); }));

  /* ---------- HOW ---------- */
  PF.how.decisions.forEach(d => { N('d:' + d.id, d.title, 'decision', 'how', 2, 'decisions'); });
  // a live How may not carry a blueprint yet
  if (PF.how.blueprint) N('bp:' + PF.how.blueprint.id, PF.how.blueprint.name, 'blueprint', 'how', 1, 'decisions');
  PF.how.principles.forEach(p => { N('p:' + p.id, p.id.replace('prin-', ''), 'principle', 'how', 3, 'decisions'); });
  PF.how.decisions.forEach(d => d.licenses.forEach(p => E('d:' + d.id, 'p:' + p, 'licenses')));
  (PF.how.deployableUnits || []).forEach(du => {
    N('du:' + du.id, du.id.replace('du-', ''), 'deployable', 'how', 2, 'decisions');
    if (PF.how.blueprint) E('bp:' + PF.how.blueprint.id, 'du:' + du.id, 'instantiates');
    E('sys:' + du.system, 'du:' + du.id, 'realised-by'); // bridge
  });
  PF.how.patterns.forEach(p => {
    N('pat:' + p.id, p.id.replace('pat-', ''), 'pattern', 'how', 4, 'patterns');
    E('p:' + p.implements, 'pat:' + p.id, 'realized-by');
    (p.rules || []).forEach(() => {}); // layout rules stay in their view
  });
  PF.manifest.components.forEach(c => N('cio:' + c.id, c.id, 'cio', 'how', 4, 'reification'));
  PF.manifest.reification.forEach(r => { if (seen['aio:' + r.aio] && seen['cio:' + r.cio]) E('aio:' + r.aio, 'cio:' + r.cio, 'reifies'); });

  /* ---------- BUILD & DELIVERY ---------- */
  const WUF = { 'wu-checkout-refund-decider-0007': 'refunds', 'wu-cart-projector-0004': 'checkout',
    'wu-fulfilment-projector-0012': 'fulfilment', 'wu-review-cart-page-0009': 'checkout', 'wu-scaffold': null };
  const wuIds = new Set();
  PF.how.patterns.forEach(p => (p.units || []).forEach(u => wuIds.add(u)));
  wuIds.forEach(id => {
    N('wu:' + id, id.replace(/^wu-/, '').replace(/-\d+$/, ''), 'workunit', 'build', 1, 'workunits', id === 'wu-scaffold');
  });
  PF.how.patterns.forEach(p => (p.units || []).forEach(u => E('pat:' + p.id, 'wu:' + u, 'applied-by')));
  // What artifacts feed work units (the SPMC context) — bridge edges
  [['dec:decider-order', 'wu:wu-checkout-refund-decider-0007'], ['proj:proj-cart-summary', 'wu:wu-cart-projector-0004'],
   ['f:flow-fulfil', 'wu:wu-fulfilment-projector-0012'], ['ui:ui-review-cart', 'wu:wu-review-cart-page-0009']].forEach(([a, b]) => {
    if (seen[a] && seen[b]) E(a, b, 'frozen-into');
  });

  PF.delivery.features.forEach(f => {
    N('feat:' + f.id, f.name, 'feature', 'build', 2, 'features', f.conformance === 'described');
    f.flows.forEach(fid => { if (seen['f:' + fid]) E('f:' + fid, 'feat:' + f.id, 'referenced-by'); });
  });
  Object.entries(WUF).forEach(([wu, feat]) => { if (feat && seen['wu:' + wu]) E('wu:' + wu, 'feat:' + feat, 'realises'); });
  PF.delivery.releases.forEach(r => {
    N('rel:' + r.id, r.name, 'release', 'build', 3, 'features', r.status === 'planned');
    r.features.forEach(f => E('feat:' + f, 'rel:' + r.id, 'cut-into'));
  });
  PF.delivery.versions.what.forEach(v => {
    N('vw:' + v.v, 'What ' + v.v, 'what-version', 'build', 3, 'versions', v.target);
    (v.adds || []).forEach(f => E('feat:' + f, 'vw:' + v.v, 'adds'));
  });
  PF.delivery.versions.how.forEach(v => {
    N('vh:' + v.v, 'How ' + v.v, 'how-version', 'build', 4, 'versions');
    E('vw:' + v.realises, 'vh:' + v.v, 'realises');
  });
  PF.delivery.targets.forEach(t => {
    N('tgt:' + t.id, t.name + ' target', 'target', 'build', 4, 'versions', true);
    t.partition.forEach(f => E('feat:' + f, 'tgt:' + t.id, 'partitions'));
  });

  // drop any edge whose endpoint doesn't exist
  const ok = edges.filter(e => seen[e.from] && seen[e.to] && e.from !== e.to);
  return { nodes, edges: ok };
};
