/* global React, ReactDOM, PF, PFUI, SystemsMap, DomainGraph, FlowsView, FeaturesView, VersionsView,
   PageGraphView, UIStepsView, ScreenPreview, AIOCatalogView, DataView, ContentView,
   DecidersView, ScenariosView, DecisionsView, LayoutView, ReificationView, CompositionView, PatternsView, HowProcessView, WorkUnitsView, VerificationsView, GraphView,
   useTweaks, TweaksPanel, TweakSection, TweakToggle, TweakRadio */
const { useState } = React;
const NS = window.ProductFrameworkDesignSystem_52ecf1;
const { PhaseStepper } = NS;
const { DetailPanel, ConfDot } = window.PFUI;

const KIND_FILTERS = ['trigger', 'ui-step', 'command', 'view', 'event'];
const FILTER_COLOR = {
  trigger: 'var(--em-trigger)', 'ui-step': 'var(--slate-400)',
  command: 'var(--em-command)', view: 'var(--em-view)', event: 'var(--em-event)',
};

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "surface": "graph",
  "showFilters": true,
  "showConf": true,
  "density": "comfortable",
  "featuresLayout": "board",
  "versionsLayout": "ladder",
  "screenContext": "phone",
  "locale": "en"
}/*EDITMODE-END*/;

// ---- a small coloured kind tag ----
function KindTag({ label, color }) {
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', fontFamily: 'var(--font-mono)', fontWeight: 600,
      fontSize: 9.5, letterSpacing: '.12em', textTransform: 'uppercase', color: '#0b1120',
      background: color, borderRadius: 3, padding: '2px 8px',
    }}>{label}</span>
  );
}

// ---- detail-panel content per view ----
function buildDetail(view, sel, openSystem, openDomain, openFlow, openStep) {
  if (!sel) return null;
  if (view === 'systems') {
    if (sel === 'acme') {
      const p = PF.product;
      return { tag: <KindTag label="product" color="var(--blue-400)" />, title: p.name, rows: [
        { k: 'purpose', v: p.purpose }, { k: 'owns domains', v: p.ownsDomains.join(', ') },
        { k: 'owns systems', v: p.ownsSystems.join(', ') }, { k: 'direction', v: p.direction },
        { k: 'quality (product-wide)', v: p.quality, color: 'var(--blue-400)' },
      ] };
    }
    const dom = PF.domains.find(d => d.id === sel);
    if (dom) {
      const refBy = PF.systems.filter(s => s.references.includes(dom.id)).map(s => s.name).join(', ');
      return { tag: <KindTag label="domain" color="var(--kind-entity)" />, title: dom.name, rows: [
        { k: 'ubiquitous language', v: dom.language.join(' · ') },
        { k: 'referenced by', v: refBy }, { k: 'conformance', v: dom.conformance, color: 'var(--conf-' + dom.conformance + ')' },
      ], action: dom.id === 'ordering' ? { label: 'open domain graph →', fn: () => openDomain('ordering') } : null };
    }
    const sys = PF.systems.find(s => s.id === sel);
    if (sys) {
      return { tag: <KindTag label={'system · ' + sys.kind} color="var(--slate-300)" />, title: sys.name, rows: [
        { k: 'purpose', v: sys.purpose }, { k: 'interaction class', v: sys.cls },
        { k: 'platforms', v: sys.platforms.join(', ') }, { k: 'references domains', v: sys.references.join(', ') },
        sys.demands.length ? { k: 'quality demands', v: sys.demands } : null,
        { k: 'flows', v: sys.flows.map(f => PF.flows[f].name).join(', ') },
        { k: 'conformance', v: sys.conformance, color: 'var(--conf-' + sys.conformance + ')' },
      ], action: { label: 'open event-model flows →', fn: () => openSystem(sys.id) } };
    }
  }
  if (view === 'domain') {
    const n = PF.domain.nodes.find(x => x.id === sel);
    if (!n) return null;
    const out = PF.domain.edges.filter(e => e.from === n.id).map(e => `→ ${e.label} ${e.to}`);
    const gov = PF.domain.edges.filter(e => e.to === n.id && e.kind === 'inv').map(e => `⟵ ${e.from}`);
    const COLORS = { aggregate: 'var(--kind-entity)', entity: 'var(--blue-500)', 'value-object': 'var(--blue-400)',
      invariant: 'var(--kind-invariant)', external: 'var(--slate-400)', reference: 'var(--em-event)' };
    return { tag: <KindTag label={n.kind} color={COLORS[n.kind]} />, title: n.label, rows: [
      { k: 'fields', v: n.fields }, out.length ? { k: 'relations', v: out } : null,
      gov.length ? { k: 'governed by', v: gov, color: 'var(--em-trigger-soft)' } : null,
    ] };
  }
  if (view === 'flows') {
    const flow = PF.flows[sel.flowId];
    const n = flow.nodes.find(x => x.id === sel.nodeId);
    if (!n) return null;
    const m = flow.meta[n.id] || {};
    const TONE = { trigger: 'var(--em-trigger)', 'ui-step': 'var(--slate-400)', command: 'var(--em-command)',
      view: 'var(--em-view)', event: 'var(--em-event)' };
    const specced = n.kind === 'ui-step' && PF.screen(n.sub);
    return { tag: <KindTag label={n.kind} color={TONE[n.kind]} />, title: n.label, rows: [
      { k: 'id', v: n.sub || n.id }, m.context ? { k: 'context', v: m.context } : null,
      m.aio ? { k: 'aio typing', v: m.aio } : null,
      m.guards ? { k: 'guards invariant', v: m.guards, color: 'var(--em-trigger-soft)' } : null,
      m.out ? { k: 'out', v: m.out } : null, m.in ? { k: 'in', v: m.in } : null,
      m.note ? { k: 'note', v: m.note, color: 'var(--slate-400)' } : null,
    ], action: specced ? { label: 'open spec sheet →', fn: () => openStep(n.sub) } : null };
  }
  if (view === 'features') {
    return featureDetail(sel, openFlow);
  }
  if (view === 'versions') {
    if (typeof sel === 'string' && (sel.slice(0, 2) === 'w:' || sel.slice(0, 2) === 'h:')) {
      const axis = sel.slice(0, 2) === 'w:' ? 'what' : 'how';
      const v = PF.delivery.versions[axis].find(x => x.v === sel.slice(2));
      if (!v) return null;
      const rows = [
        { k: 'bump', v: v.bump + (v.target ? ' \u00b7 target' : v.current ? ' \u00b7 current' : ''),
          color: v.current ? (axis === 'what' ? 'var(--blue-400)' : 'var(--em-event)') : 'var(--slate-200)' },
        { k: 'what the diff touched', v: v.diff },
      ];
      if (v.adds) rows.push({ k: 'adds slices', v: v.adds.map(id => PF.feature(id).name).join(', ') });
      if (v.realises) rows.push({ k: 'realises', v: 'What ' + v.realises, color: 'var(--em-bridge)' });
      return {
        tag: <KindTag label={axis === 'what' ? 'what-version' : 'how-version'}
          color={axis === 'what' ? 'var(--blue-500)' : 'var(--em-event)'} />,
        title: (axis === 'what' ? 'What ' : 'How ') + v.v + ' \u2014 ' + v.name, rows,
      };
    }
    return featureDetail(sel, openFlow); // a partition member
  }
  if (view === 'pages') {
    const d = window.pageGraphDetail(sel);
    if (!d) return null;
    return {
      tag: <KindTag label={d.kind === 'root' ? 'system root' : 'page · ui step'} color={d.kind === 'root' ? 'var(--em-trigger)' : 'var(--slate-400)'} />,
      title: d.title, rows: d.rows,
      action: d.specced ? { label: 'open spec sheet →', fn: () => openStep(d.id) } : null,
    };
  }
  if (view === 'aios') {
    const a = PF.aios.find(x => x.id === sel);
    if (!a) return null;
    const usage = PF.aioUsage(a.id);
    return {
      tag: <KindTag label="aio" color={a.id.startsWith('display') ? 'var(--em-view)' : a.id === 'navigate' ? 'var(--em-trigger)' : 'var(--em-command)'} />,
      title: a.id, rows: [
        { k: 'means', v: a.means },
        { k: 'typed over', v: a.typedOver },
        { k: 'inherited a11y criteria', v: a.wcag.map(c => c + ' ' + (PF.wcag[c] || {}).name) },
        { k: 'reifies', v: ['phone → ' + a.reify.phone, 'desktop → ' + a.reify.desktop, 'tui → ' + a.reify.tui] },
        usage.length ? { k: 'used by', v: usage } : null,
      ],
      action: usage.length ? { label: 'open spec sheet →', fn: () => openStep(usage[0]) } : null,
    };
  }
  if (view === 'data') {
    const r = PF.refData.find(x => x.id === sel);
    if (r) {
      return { tag: <KindTag label="reference data" color="var(--kind-entity)" />, title: r.name, rows: [
        { k: 'declared set', v: r.values.join(' · ') },
        { k: 'reference_data_for', v: r.referenceFor },
        { k: 'why constitutive', v: r.note },
        { k: 'conformance', v: r.conformance, color: 'var(--conf-' + r.conformance + ')' },
      ] };
    }
    const d = PF.oracle.datasets.find(x => x.id === sel);
    if (d) {
      return { tag: <KindTag label="oracle · populated" color="var(--em-event)" />, title: d.name, rows: [
        { k: 'rows', v: d.rows }, { k: 'conforms_to_shape', v: d.shape },
        { k: 'assertion', v: d.assertion + ' — continuous, not one-time' },
        { k: 'data-divergence rate', v: d.rate + '%' + (d.rising ? ' and rising — the model is falling behind reality' : ' — steady'),
          color: d.rising ? 'var(--em-event)' : 'var(--conf-verified)' },
      ] };
    }
    const viol = PF.oracle.violations.find(x => x.id === sel);
    if (viol) {
      return { tag: <KindTag label={viol.triage} color={viol.triage === 'spec drift' ? 'var(--em-event)' : 'var(--blue-400)'} />,
        title: 'Shape violation', rows: [
          { k: 'shape', v: viol.shape }, { k: 'dataset', v: viol.dataset }, { k: 'failing rows', v: String(viol.count) },
          { k: 'triage', v: viol.triage + ' — ' + viol.note, color: viol.triage === 'spec drift' ? 'var(--em-event)' : 'var(--blue-400)' },
        ] };
    }
    return null;
  }
  if (view === 'decisions') {
    const d = PF.how.decisions.find(x => x.id === sel);
    if (d) {
      return { tag: <KindTag label="decision" color="var(--em-event)" />, title: d.title, rows: [
        { k: 'why', v: d.why }, { k: 'applies', v: d.applies }, { k: 'does not apply', v: d.not },
        { k: 'licenses principles', v: d.licenses },
      ] };
    }
    const p = PF.how.principles.find(x => x.id === sel);
    if (p) {
      const loc = PF.principleLocated(p.id);
      return { tag: <KindTag label="principle" color="var(--blue-400)" />, title: p.id, rows: [
        { k: 'states', v: p.text },
        { k: 'enforced by', v: p.enforcedBy, color: 'var(--conf-verified)' },
        { k: 'applied by', v: p.appliedBy },
        { k: 'locatable · the H1 gate', v: loc ? 'located — ' + loc.via : 'no rule locates it — unenforceable prose',
          color: loc ? 'var(--conf-verified)' : 'var(--error, #dc2626)' },
      ] };
    }
    const pat = PF.how.patterns.find(x => x.id === sel);
    if (pat) {
      return { tag: <KindTag label="pattern" color="var(--slate-400)" />, title: pat.id, rows: [
        { k: 'shape', v: pat.text }, { k: 'implements', v: pat.implements },
      ] };
    }
    const wu = PF.workUnits.find(x => x.id === sel);
    if (wu) {
      return { tag: <KindTag label="work unit" color="var(--em-view)" />, title: wu.id, rows: [
        { k: 'transformation', v: wu.bundle.prompt },
        { k: 'verdict', v: wu.status, color: wu.status === 'accepted' ? 'var(--conf-verified)' : 'var(--em-event)' },
        { k: 'references by pointer', v: wu.bundle.context },
      ] };
    }
    if (sel === 'wu-scaffold') {
      return { tag: <KindTag label="work units" color="var(--em-view)" />, title: 'Scaffolding units', rows: [
        { k: 'what', v: 'every scaffolding unit reads the layout model to place what it generates — the dual-read property (§4.3)' },
      ] };
    }
    return null;
  }
  if (view === 'content') {
    const e = PF.contract.content_store[sel];
    if (!e) return null;
    const usage = PF.contentUsage(sel);
    return { tag: <KindTag label={'content · ' + e.role} color="var(--blue-400)" />, title: sel, rows: [
      { k: 'role', v: e.role },
      { k: 'en', v: '“' + e.en + '”' }, { k: 'es', v: '“' + e.es + '”' },
      usage.length ? { k: 'referenced by (references_content)', v: usage } : { k: 'referenced by', v: 'shell only' },
    ], action: usage.length ? { label: 'open spec sheet →', fn: () => openStep(usage[0]) } : null };
  }
  return null;
}

// feature detail — shared by Features view and Versions partition
function featureDetail(id, openFlow) {
  const f = PF.feature(id);
  if (!f) return null;
  const done = PF.featureDone(f);
  const concepts = f.footprint.map(cid => PF.concept(cid).label).join(', ');
  const inSystem = PF.systems.find(s => s.flows.includes(f.flows[0]));
  return {
    tag: <KindTag label="feature" color="var(--em-view)" />, title: f.name,
    rows: [
      { k: 'references flows', v: f.flows.join(', ') },
      { k: 'derived footprint', v: concepts },
      { k: 'value action', v: f.valueAction, color: 'var(--em-view)' },
      { k: 'conformance', v: f.conformance, color: 'var(--conf-' + f.conformance + ')' },
      { k: 'feature_done', v: done ? 'true' : 'false', color: done ? 'var(--conf-verified)' : 'var(--em-event)' },
      { k: 'acceptance criteria', v: f.acceptance },
    ],
    action: inSystem ? { label: 'open event-model flow \u2192', fn: () => openFlow(f.flows[0]) } : null,
  };
}

function App() {
  const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);
  const [view, setView] = useState('systems');
  const [systemId, setSystemId] = useState('acme-shop');
  const [flowId, setFlowId] = useState('flow-checkout');
  const [sel, setSel] = useState({ systems: null, domain: null, flows: null, features: null, versions: null, pages: null, aios: null, data: null, content: null });
  const [stepId, setStepId] = useState('ui-review-cart');
  const [screenId, setScreenId] = useState(null);
  const [layoutSel, setLayoutSel] = useState(null);
  const [hidden, setHidden] = useState(new Set());

  const dense = t.density === 'compact';
  const toggleKind = (k) => setHidden(h => { const n = new Set(h); n.has(k) ? n.delete(k) : n.add(k); return n; });
  const select = (v, id) => setSel(s => ({ ...s, [v]: id }));

  const openSystem = (sid) => { setSystemId(sid); setFlowId(PF.systems.find(s => s.id === sid).flows[0]); setView('flows'); };
  const openDomain = () => setView('domain');
  const openFlow = (fid) => {
    const sys = PF.systems.find(s => s.flows.includes(fid));
    if (!sys) return;
    setSystemId(sys.id); setFlowId(fid); select('flows', null); setView('flows');
  };
  const openStep = (id) => {
    if (PF.screen(id)) { setStepId(id); select('pages', null); select('aios', null); setView('steps'); }
  };
  const openPreview = (id) => { setScreenId(id); setView('screens'); };

  const sysName = PF.systems.find(s => s.id === systemId).name;
  const flowName = PF.flows[flowId] ? PF.flows[flowId].name : '';
  const [navOpen, setNavOpen] = useState(() => { try { return localStorage.getItem('pf-nav-open') !== '0'; } catch (e) { return true; } });
  const toggleNav = () => setNavOpen(o => { try { localStorage.setItem('pf-nav-open', o ? '0' : '1'); } catch (e) {} return !o; });

  // detail
  const selValue = view === 'flows' ? (sel.flows ? { flowId, nodeId: sel.flows } : null) : sel[view];
  const detail = view === 'screens' ? null : buildDetail(view, selValue, openSystem, openDomain, openFlow, openStep);

  const NAV = [
    { id: 'graph', label: 'The Graph', ref: '§2·§9', color: 'var(--em-bridge)', items: [
      ['graph', 'Everything']] },
    { id: 'what', label: 'The What', ref: '§3', color: 'var(--blue-500)', items: [
      ['systems', 'Product'], ['domain', 'Domain'], ['data', 'Data'], ['flows', 'Flows'],
      ['deciders', 'Deciders'], ['scenarios', 'Scenarios']] },
    { id: 'ui', label: 'UI', ref: '§3.2', color: 'var(--em-trigger)', items: [
      ['pages', 'Pages'], ['steps', 'Steps'], ['screens', 'Screens'], ['content', 'Content'], ['aios', 'AIOs']] },
    { id: 'how', label: 'The How', ref: '§4', color: 'var(--em-event)', items: [
      ['howprocess', 'Process'], ['decisions', 'Systems'], ['patterns', 'Patterns'], ['layout', 'Layout'], ['composition', 'Composition'], ['reification', 'Reification']] },
    { id: 'build', label: 'Build', ref: '§5–6', color: 'var(--em-view)', items: [
      ['workunits', 'Work units'], ['verifications', 'Verifications']] },
    { id: 'delivery', label: 'Delivery', ref: '§7', color: 'var(--em-view)', items: [
      ['features', 'Features'], ['versions', 'Versions']] },
  ];
  const activeGroup = NAV.find(g => g.items.some(([k]) => k === view));

  return (
    <div data-surface={t.surface} style={{ minHeight: '100vh', height: '100vh', display: 'flex', flexDirection: 'column',
      background: 'var(--slate-900)', color: 'var(--slate-200)' }}>
      {/* header */}
      <header style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '10px 18px',
        background: 'var(--slate-800)', borderBottom: '1px solid var(--slate-600)', flexWrap: 'wrap', flex: 'none' }}>
        <img src="assets/logo-wordmark-light.svg" height="22" alt="Product" style={{ display: 'block' }} />
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--slate-400)',
          border: '1px solid var(--slate-600)', borderRadius: 4, padding: '2px 8px' }}>product mcp --http</span>
        <PhaseStepper current={
          view === 'features' || view === 'versions' || view === 'workunits' || view === 'verifications' ? 'build'
          : view === 'screens' || view === 'decisions' || view === 'layout' || view === 'reification' || view === 'composition' || view === 'patterns' || view === 'howprocess' ? 'how'
          : 'what'} until="build" />
        <span style={{ display: 'flex', alignItems: 'center', gap: 6, color: 'var(--slate-400)', fontSize: 12, fontFamily: 'var(--font-mono)' }}>
          <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#22c55e' }} />live
        </span>
        <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--slate-500)' }}>
          {activeGroup ? activeGroup.label + ' · ' + activeGroup.ref : ''}
        </span>
      </header>

      {/* nav pane + main column */}
      <div style={{ flex: 1, minHeight: 0, display: 'flex' }}>
        <nav aria-label="Views" style={{ width: navOpen ? 188 : 44, flex: 'none', background: 'var(--slate-800)',
          borderRight: '1px solid var(--slate-600)', display: 'flex', flexDirection: 'column',
          transition: 'width .2s var(--ease)', overflow: 'hidden' }}>
          <style>{`
            .pf-nav-item{display:flex;align-items:center;gap:8px;width:100%;text-align:left;background:transparent;
              border:0;border-left:2px solid transparent;color:var(--slate-400);font-family:var(--font-mono);
              font-size:12px;padding:6px 12px 6px 22px;cursor:pointer;white-space:nowrap}
            .pf-nav-item:hover{color:var(--slate-200);background:var(--slate-700)}
            .pf-nav-item[aria-current="true"]{background:var(--slate-700);color:var(--slate-100);font-weight:600}
            .pf-nav-grp{display:flex;align-items:baseline;gap:7px;padding:12px 12px 5px;white-space:nowrap}
            .pf-nav-rail{display:flex;flex-direction:column;align-items:center;gap:2px;padding:10px 0;background:transparent;
              border:0;cursor:pointer;width:100%}
            .pf-nav-rail:hover{background:var(--slate-700)}
            .pf-nav-toggle{background:transparent;border:0;color:var(--slate-500);font-family:var(--font-mono);
              font-size:13px;cursor:pointer;padding:7px;text-align:right}
            .pf-nav-toggle:hover{color:var(--slate-200)}
          `}</style>
          <button className="pf-nav-toggle" onClick={toggleNav} title={navOpen ? 'collapse' : 'expand'}
            style={{ textAlign: navOpen ? 'right' : 'center' }}>{navOpen ? '‹' : '›'}</button>
          {navOpen ? (
            <div style={{ overflowY: 'auto' }}>
              {NAV.map(g => (
                <div key={g.id}>
                  <div className="pf-nav-grp">
                    <span style={{ width: 8, height: 8, borderRadius: 2, background: g.color, flex: 'none', alignSelf: 'center' }} />
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, fontWeight: 700, letterSpacing: '.15em',
                      textTransform: 'uppercase', color: 'var(--slate-300)' }}>{g.label}</span>
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)' }}>{g.ref}</span>
                  </div>
                  {g.items.map(([k, label]) => (
                    <button key={k} className="pf-nav-item" aria-current={view === k ? 'true' : 'false'}
                      style={view === k ? { borderLeftColor: g.color } : null} onClick={() => setView(k)}>{label}</button>
                  ))}
                </div>
              ))}
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4, marginTop: 6 }}>
              {NAV.map(g => (
                <button key={g.id} className="pf-nav-rail" title={g.label + ' · ' + g.ref} onClick={toggleNav}>
                  <span style={{ width: 9, height: 9, borderRadius: 2, background: g.color,
                    boxShadow: activeGroup && activeGroup.id === g.id ? '0 0 0 2px var(--slate-800), 0 0 0 3.5px ' + g.color : 'none' }} />
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, letterSpacing: '.08em',
                    color: activeGroup && activeGroup.id === g.id ? 'var(--slate-200)' : 'var(--slate-500)',
                    writingMode: 'vertical-rl', padding: '3px 0' }}>{g.label}</span>
                </button>
              ))}
            </div>
          )}
        </nav>

        {/* main column */}
        <div style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column' }}>

      {/* breadcrumb + context bar */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '7px 18px', flex: 'none',
        background: 'var(--slate-900)', borderBottom: '1px solid var(--slate-800)', fontFamily: 'var(--font-mono)', fontSize: 12 }}>
        <Crumb onClick={() => setView('systems')} active={view === 'systems'}>acme</Crumb>
        {view === 'domain' && <><Sep /><Crumb active>Ordering</Crumb></>}
        {view === 'graph' && <><Sep /><Crumb active>One graph · everything connected</Crumb></>}
        {view === 'flows' && <><Sep /><Crumb onClick={() => setView('systems')}>{sysName}</Crumb><Sep /><Crumb active>{flowName}</Crumb></>}
        {view === 'features' && <><Sep /><Crumb active>Delivery · Features</Crumb></>}
        {view === 'versions' && <><Sep /><Crumb active>Delivery · Versions</Crumb></>}
        {view === 'pages' && <><Sep /><Crumb active>UI · Page graph</Crumb></>}
        {view === 'steps' && <><Sep /><Crumb active>UI · {PF.screen(stepId) ? PF.screen(stepId).name : ''}</Crumb></>}
        {view === 'screens' && <><Sep /><Crumb active>UI · What → Preview</Crumb></>}
        {view === 'aios' && <><Sep /><Crumb active>UI · AIO catalog</Crumb></>}
        {view === 'data' && <><Sep /><Crumb active>Ordering · Data</Crumb></>}
        {view === 'content' && <><Sep /><Crumb active>UI · Content store</Crumb></>}
        {view === 'deciders' && <><Sep /><Crumb active>Ordering · Deciders & Projectors</Crumb></>}
        {view === 'scenarios' && <><Sep /><Crumb active>Ordering · Simulation</Crumb></>}
        {view === 'decisions' && <><Sep /><Crumb active>How · Systems</Crumb></>}
        {view === 'howprocess' && <><Sep /><Crumb active>How · Process</Crumb></>}
        {view === 'layout' && <><Sep /><Crumb active>How · Repository layout</Crumb></>}
        {view === 'patterns' && <><Sep /><Crumb active>How · Patterns</Crumb></>}
        {view === 'composition' && <><Sep /><Crumb active>How · Screen composition</Crumb></>}
        {view === 'reification' && <><Sep /><Crumb active>How · wire-ds manifest</Crumb></>}
        {view === 'workunits' && <><Sep /><Crumb active>Build · Work units</Crumb></>}
        {view === 'verifications' && <><Sep /><Crumb active>Build · Verifications</Crumb></>}
        <span style={{ marginLeft: 'auto', color: 'var(--slate-600)' }}>
          {view === 'graph' && '“describe this system” is a query · §2 / §9'}
          {view === 'systems' && 'product → systems & journeys · §3.0'}
          {view === 'domain' && 'domain entities & relations · §3.1'}
          {view === 'flows' && 'event-model flows · §3.2'}
          {view === 'features' && 'delivery · features & releases · §7.1'}
          {view === 'versions' && 'delivery · versions & direction · §7.3'}
          {view === 'pages' && 'navigation as one graph · §3.2.4'}
          {view === 'steps' && 'the What of a screen · §3.2.1'}
          {view === 'screens' && 'generic AIO renderer · render contract (preview)'}
          {view === 'aios' && 'abstract interaction objects · §3.2.2'}
          {view === 'data' && 'reference data & the oracle · §3.1 / §13'}
          {view === 'content' && 'words live in a content store · §4.6 / §12'}
          {view === 'deciders' && 'the executable form of behaviour · §3.3–3.4'}
          {view === 'scenarios' && 'simulated before any code exists · §3.3 / §6.3'}
          {view === 'decisions' && 'the why, made traceable · §4.1–4.4'}
          {view === 'layout' && 'what files are legal where · §4.3'}
          {view === 'patterns' && 'the building blocks — shapes, files, rules · §4.1'}
          {view === 'howprocess' && 'binding resolution in dependency order · H1–H6'}
          {view === 'composition' && 'Atomic Design, normative · §4.5'}
          {view === 'reification' && 'reify(AIO, context) → CIO · §4.5 / §11'}
          {view === 'workunits' && 'the work-unit emission contract · §5.1'}
          {view === 'verifications' && 'the conformance bar · §6'}
        </span>
      </div>

      {/* kind filters (flows view only) */}
      {view === 'flows' && t.showFilters && (
        <div style={{ display: 'flex', gap: 16, padding: '8px 18px', flex: 'none', flexWrap: 'wrap',
          background: 'var(--slate-900)', borderBottom: '1px solid var(--slate-800)',
          fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--slate-400)' }}>
          {KIND_FILTERS.map(k => (
            <label key={k} style={{ display: 'flex', alignItems: 'center', gap: 6, cursor: 'pointer' }}>
              <input type="checkbox" checked={!hidden.has(k)} onChange={() => toggleKind(k)} style={{ accentColor: 'var(--blue-500)' }} />
              <span style={{ width: 9, height: 9, borderRadius: 2, background: FILTER_COLOR[k] }} />{k}
            </label>
          ))}
        </div>
      )}

      {/* canvas */}
      <div style={{ position: 'relative', flex: 1, minHeight: 0, overflow: 'hidden' }}>
        {view === 'graph' && <GraphView onOpen={(v) => setView(v)} />}
        {view === 'systems' && (
          <SystemsMap onOpenSystem={openSystem} selected={sel.systems} onSelect={(id) => select('systems', id)}
            showConf={t.showConf} showLabels={true} dense={dense} />
        )}
        {view === 'domain' && (
          <DomainGraph selected={sel.domain} onSelect={(id) => select('domain', id)} showConf={t.showConf} showLabels={true} />
        )}
        {view === 'flows' && (
          <FlowsView systemId={systemId} flowId={flowId} setFlowId={(f) => { setFlowId(f); select('flows', null); }}
            hidden={hidden} selected={sel.flows} onSelect={(id) => select('flows', id)}
            showConf={t.showConf} showLabels={true} dense={dense} />
        )}
        {view === 'features' && (
          <FeaturesView layout={t.featuresLayout} selected={sel.features} onSelect={(id) => select('features', id)}
            showConf={t.showConf} dense={dense} />
        )}
        {view === 'versions' && (
          <VersionsView layout={t.versionsLayout} selected={sel.versions} onSelect={(id) => select('versions', id)}
            showConf={t.showConf} />
        )}
        {view === 'pages' && (
          <PageGraphView selected={sel.pages} onSelect={(id) => select('pages', id)} onOpenStep={openStep} />
        )}
        {view === 'steps' && (
          <UIStepsView stepId={stepId} setStepId={setStepId} onPreview={openPreview} />
        )}
        {view === 'screens' && (
          <ScreenPreview context={t.screenContext} locale={t.locale} screenId={screenId} setScreenId={setScreenId} />
        )}
        {view === 'aios' && (
          <AIOCatalogView selected={sel.aios} onSelect={(id) => select('aios', id)} />
        )}
        {view === 'data' && (
          <DataView selected={sel.data} onSelect={(id) => select('data', id)} showConf={t.showConf} />
        )}
        {view === 'content' && (
          <ContentView selected={sel.content} onSelect={(id) => select('content', id)} />
        )}
        {view === 'deciders' && <DecidersView showConf={t.showConf} />}
        {view === 'scenarios' && <ScenariosView />}
        {view === 'decisions' && <DecisionsView selected={sel.decisions} onSelect={(id) => select('decisions', id)} />}
        {view === 'layout' && <LayoutView selected={layoutSel} onSelect={setLayoutSel} />}
        {view === 'patterns' && <PatternsView onOpenRule={(id) => { setLayoutSel(id); setView('layout'); }} />}
        {view === 'howprocess' && <HowProcessView onOpen={(v) => setView(v)} />}
        {view === 'composition' && <CompositionView />}
        {view === 'reification' && <ReificationView />}
        {view === 'workunits' && <WorkUnitsView />}
        {view === 'verifications' && <VerificationsView />}

        {detail && (
          <DetailPanel title={detail.title} kindTag={detail.tag} rows={detail.rows} onClose={() => select(view, null)}>
            {detail.action && (
              <button onClick={detail.action.fn} style={{
                marginTop: 14, width: '100%', background: 'var(--blue-600)', border: '1px solid var(--blue-500)',
                color: '#fff', borderRadius: 5, padding: '8px 12px', cursor: 'pointer',
                fontFamily: 'var(--font-mono)', fontSize: 11.5,
              }}>{detail.action.label}</button>
            )}
          </DetailPanel>
        )}
      </div>
        </div>
      </div>

      <TweaksPanel>
        <TweakSection label="Surface" />
        <TweakRadio label="Theme" value={t.surface} options={[{ value: 'graph', label: 'dark' }, { value: 'blueprint', label: 'light' }]}
          onChange={(v) => setTweak('surface', v)} />
        <TweakRadio label="Density" value={t.density} options={['compact', 'comfortable']}
          onChange={(v) => setTweak('density', v)} />
        <TweakSection label="Overlays" />
        <TweakToggle label="Kind filters (flows)" value={t.showFilters} onChange={(v) => setTweak('showFilters', v)} />
        <TweakToggle label="Conformance overlay" value={t.showConf} onChange={(v) => setTweak('showConf', v)} />
        <TweakSection label="UI" />
        <TweakRadio label="Screen context of use" value={t.screenContext === 'desktop' ? 'web' : t.screenContext}
          options={[{ value: 'phone', label: 'phone' }, { value: 'web', label: 'web' }]}
          onChange={(v) => setTweak('screenContext', v)} />
        <TweakRadio label="Locale (content store)" value={t.locale}
          options={[{ value: 'en', label: 'en' }, { value: 'es', label: 'es' }]}
          onChange={(v) => setTweak('locale', v)} />
        <TweakSection label="Delivery" />
        <TweakRadio label="Features view" value={t.featuresLayout}
          options={[{ value: 'board', label: 'board' }, { value: 'footprint', label: 'footprint' }, { value: 'ledger', label: 'ledger' }]}
          onChange={(v) => setTweak('featuresLayout', v)} />
        <TweakRadio label="Versions view" value={t.versionsLayout}
          options={[{ value: 'ladder', label: 'ladder' }, { value: 'partition', label: 'partition' }]}
          onChange={(v) => setTweak('versionsLayout', v)} />
      </TweaksPanel>
    </div>
  );
}

function Crumb({ children, active, onClick }) {
  return (
    <span onClick={onClick} style={{
      color: active ? 'var(--slate-100)' : 'var(--blue-400)', cursor: onClick ? 'pointer' : 'default',
      fontWeight: active ? 600 : 400,
    }}>{children}</span>
  );
}
function Sep() { return <span style={{ color: 'var(--slate-600)' }}>›</span>; }

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
