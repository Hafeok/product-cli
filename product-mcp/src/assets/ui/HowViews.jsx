/* global React, PF */
/* The How (§4) — three views in one file:
   DecisionsView  — decisions → principles → patterns chain + contracts + standards (§4.1–4.4)
   LayoutView     — the repository layout model, allowlist semantics (§4.3)
   ReificationView— the §11 design-system manifest: CIOs, reify rules, unreifiable (§4.5) */
(function () {
  const HOW = 'var(--em-event)';
  const CMD = 'var(--em-command)', VIEW = 'var(--em-view)', NAV = 'var(--em-trigger)';
  const { EdgeLayer, FitCanvas } = window.PFUI;

  function Mono({ children, color, size }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: size || 10.5, color: color || 'var(--slate-300)' }}>{children}</span>;
  }
  function SecLabel({ children, color }) {
    return <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
      textTransform: 'uppercase', color: color || 'var(--slate-500)' }}>{children}</div>;
  }
  function Chip({ children, color }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: color || 'var(--slate-300)',
      border: `1px solid ${color || 'var(--slate-600)'}`, borderRadius: 3, padding: '1px 7px' }}>{children}</span>;
  }
  function Card({ children, dashed }) {
    return <div style={{ background: 'var(--slate-800)', borderRadius: 8, padding: '12px 15px',
      border: `1.5px ${dashed ? 'dashed' : 'solid'} var(--slate-700)`, boxShadow: dashed ? 'none' : 'var(--shadow-graph)' }}>{children}</div>;
  }
  function Eyebrow({ children, color }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.12em',
      textTransform: 'uppercase', color }}>{children}</span>;
  }
  function Page({ children, max = 1060 }) {
    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto', background: 'var(--slate-900)' }}>
        <div style={{ maxWidth: max, margin: '0 auto', padding: '20px 26px 40px' }}>{children}</div>
      </div>
    );
  }
  function Foot({ children }) {
    return <div style={{ marginTop: 18, fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
      borderTop: '1px dashed var(--slate-700)', paddingTop: 11, lineHeight: 1.55, maxWidth: 840 }}>{children}</div>;
  }

  /* ================= Decisions (§4.1–4.2, §4.4) ================= */
  /* the chain as a layered dependency graph: decisions ← principles ← patterns ← work units */
  const CG_W = 1280, CG_H = 640;
  function chainData() {
    const H = PF.how;
    const wus = [
      { id: 'wu-checkout-refund-decider-0007', label: 'refund decider' },
      { id: 'wu-cart-projector-0004', label: 'cart projector' },
      { id: 'wu-fulfilment-projector-0012', label: 'fulfilment projector' },
      { id: 'wu-review-cart-page-0009', label: 'review-cart page' },
      { id: 'wu-scaffold', label: 'scaffolding units', dashed: true },
    ];
    const pos = {}, nodes = [], edges = [];
    const col = (items, x, w, h, layer) => {
      const gap = (CG_H - 90 - items.length * h) / Math.max(items.length - 1, 1);
      items.forEach((n, i) => {
        pos[n.id] = { x, y: 78 + h / 2 + i * (h + gap), w, h };
        nodes.push({ ...n, layer });
      });
    };
    col(H.decisions.map(d => ({ id: d.id, label: d.title })), 168, 268, 68, 'decision');
    col(H.principles.map(p => ({ id: p.id, label: p.text.split(' \u2014 ')[0] })), 508, 262, 60, 'principle');
    col(H.patterns.map(p => ({ id: p.id, label: p.text.split(':')[0].split(' \u2014 ')[0].split(' (')[0] })), 848, 258, 50, 'pattern');
    col(wus, 1148, 204, 48, 'workunit');
    H.decisions.forEach(d => d.licenses.forEach(p => edges.push([d.id, p])));
    H.patterns.forEach(p => edges.push([p.implements, p.id]));
    H.patterns.forEach(p => (p.units || []).forEach(w => edges.push([p.id, w])));
    return { nodes, edges, pos };
  }
  const LAYER_STYLE = {
    decision: { c: HOW, tag: 'decision' },
    principle: { c: 'var(--blue-400)', tag: 'principle' },
    pattern: { c: 'var(--slate-400)', tag: 'pattern' },
    workunit: { c: VIEW, tag: 'work unit' },
  };
  function ChainGraph({ selected, onSelect }) {
    const { nodes, edges, pos } = React.useMemo(chainData, []);
    // trace: full cone up + down from the selected node
    const inTrace = React.useMemo(() => {
      if (!selected) return null;
      const down = {}, up = {};
      edges.forEach(([a, b]) => { (down[a] = down[a] || []).push(b); (up[b] = up[b] || []).push(a); });
      const s = new Set([selected]);
      const walk = (id, adj) => (adj[id] || []).forEach(n => { if (!s.has(n)) { s.add(n); walk(n, adj); } });
      walk(selected, down); walk(selected, up);
      return s;
    }, [selected, edges]);

    const edgeDefs = edges.map(([a, b]) => {
      const hot = inTrace && inTrace.has(a) && inTrace.has(b);
      return {
        from: a, to: b,
        stroke: hot ? HOW : 'var(--slate-600)',
        width: hot ? 1.8 : 1.1,
        dash: hot ? undefined : '4 4',
        opacity: inTrace ? (hot ? 1 : 0.16) : 0.75,
      };
    });

    return (
      <FitCanvas width={CG_W} height={CG_H}>
        {/* column captions + edge-kind labels */}
        {[[168, 'decisions', 'the foundation'], [508, 'principles', ''], [848, 'patterns', ''], [1148, 'work units', 'the volatile edge']].map(([x, t, sub]) => (
          <div key={t} style={{ position: 'absolute', left: x - 140, top: 14, width: 280, textAlign: 'center', zIndex: 3 }}>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
              textTransform: 'uppercase', color: 'var(--slate-300)' }}>{t}</div>
            {sub && <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: 'var(--slate-500)', marginTop: 1 }}>{sub}</div>}
          </div>
        ))}
        {[[338, 'license \u2192'], [678, 'realized by \u2192'], [1000, 'applied by \u2192']].map(([x, t]) => (
          <div key={t} style={{ position: 'absolute', left: x - 60, top: 40, width: 120, textAlign: 'center',
            fontFamily: 'var(--font-mono)', fontSize: 9, letterSpacing: '.08em', color: HOW, zIndex: 3 }}>{t}</div>
        ))}

        <EdgeLayer edges={edgeDefs} pos={pos} width={CG_W} height={CG_H} showLabels={false} />

        {nodes.map(n => {
          const p = pos[n.id];
          const st = LAYER_STYLE[n.layer];
          const on = selected === n.id;
          const dim = inTrace && !inTrace.has(n.id);
          return (
            <div key={n.id} onClick={() => onSelect(on ? null : n.id)} style={{
              position: 'absolute', left: p.x - p.w / 2, top: p.y - p.h / 2, width: p.w, height: p.h, zIndex: 2,
              cursor: 'pointer', boxSizing: 'border-box', background: 'var(--slate-800)',
              border: `1.5px ${n.dashed ? 'dashed' : 'solid'} ${on ? 'var(--blue-400)' : (n.layer === 'decision' ? HOW : 'var(--slate-600)')}`,
              borderRadius: 7, padding: '7px 11px', overflow: 'hidden',
              opacity: dim ? 0.28 : 1, transition: 'opacity .18s var(--ease)',
              boxShadow: on ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)' : 'var(--shadow-graph)',
              display: 'flex', flexDirection: 'column', justifyContent: 'center', gap: 2,
            }}>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: 7.5, fontWeight: 600, letterSpacing: '.11em',
                textTransform: 'uppercase', color: st.c }}>{st.tag}</span>
              <span style={{ fontFamily: n.layer === 'decision' ? 'var(--font-sans)' : 'var(--font-mono)',
                fontWeight: n.layer === 'decision' ? 700 : 500,
                fontSize: n.layer === 'decision' ? 12.5 : 10.5, lineHeight: 1.25, color: 'var(--slate-100)' }}>{n.label}</span>
            </div>
          );
        })}
      </FitCanvas>
    );
  }

  function ChainArrow({ label }) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '2px 0 2px 18px' }}>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--slate-500)' }}>↓</span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, letterSpacing: '.1em', color: HOW }}>{label}</span>
      </div>
    );
  }
  function DecisionsView({ selected, onSelect }) {
    const H = PF.how;
    const A = H.blueprint;
    const dus = H.deployableUnits;
    return (
      <Page>
        {/* the blueprint — a reusable How, instantiated as DeployableUnits */}
        <div style={{ background: 'var(--slate-800)', border: `1.5px solid ${HOW}`, borderRadius: 9,
          padding: '13px 16px', boxShadow: 'var(--shadow-graph)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap' }}>
            <Eyebrow color={HOW}>blueprint · a reusable How</Eyebrow>
            <span style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 16, color: 'var(--slate-100)' }}>{A.name}</span>
            <Mono color="var(--slate-500)" size={9.5}>{A.id}</Mono>
            <span style={{ marginLeft: 'auto', display: 'flex', gap: 8 }}>
              {A.instances.map(i => (
                <span key={i.sys} style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-200)',
                  border: '1px solid var(--slate-600)', borderRadius: 3, padding: '1px 8px' }}>
                  {i.sys} <span style={{ color: 'var(--conf-' + i.conformance + ')' }}>●</span></span>
              ))}
            </span>
          </div>
          <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap', marginTop: 9 }}>
            {A.packages.map(p => (
              <span key={p} style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-300)',
                background: 'var(--slate-900)', border: '1px solid var(--slate-700)', borderRadius: 3, padding: '1px 7px' }}>{p}</span>
            ))}
          </div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 8 }}>{A.note}</div>

          {/* blueprint → DeployableUnits — the instantiation, per system per environment */}
          <div style={{ marginTop: 11, borderTop: '1px dashed var(--slate-700)', paddingTop: 10 }}>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, fontWeight: 600, letterSpacing: '.13em',
              textTransform: 'uppercase', color: 'var(--slate-500)', marginBottom: 8 }}>
              instantiates → DeployableUnits · the DORA unit · 1:1:1 here, fan-out must be declared</div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))', gap: 8 }}>
              {dus.map(du => (
                <div key={du.id} style={{ background: 'var(--slate-900)', border: '1.5px solid var(--slate-700)',
                  borderRadius: 6, padding: '8px 11px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 7, flexWrap: 'wrap' }}>
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.1em',
                      textTransform: 'uppercase', color: VIEW }}>deployable unit</span>
                    <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 8.5,
                      color: du.env === 'production' ? 'var(--conf-delivered)' : 'var(--slate-400)',
                      border: '1px solid var(--slate-600)', borderRadius: 3, padding: '0 6px' }}>{du.env}</span>
                  </div>
                  <div style={{ fontFamily: 'var(--font-mono)', fontWeight: 700, fontSize: 11.5, color: 'var(--slate-100)', marginTop: 3 }}>{du.id}</div>
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-400)', marginTop: 2 }}>
                    realises <span style={{ color: 'var(--slate-200)' }}>{du.system}</span></div>
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: 'var(--slate-500)', marginTop: 3 }}>
                    {du.identity}{du.frozen ? ' · frozen ✓' : ''}</div>
                </div>
              ))}
            </div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)', marginTop: 8, lineHeight: 1.5 }}>
              system identity is What; a DeployableUnit is How — staging and production are two units of the
              same system on the same blueprint, which is why deployment identity varies per unit (§4.2).</div>
          </div>
        </div>

        {/* the chain as a layered dependency graph — stable at the left, volatile at the right */}
        <div style={{ position: 'relative', height: 640, marginTop: 14, background: 'var(--slate-900)',
          border: '1px solid var(--slate-800)', borderRadius: 8, overflow: 'hidden' }}>
          <ChainGraph selected={selected} onSelect={onSelect} />
        </div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 8, lineHeight: 1.5 }}>
          click a node to trace its dependency cone — few stable decisions found many principles; principles are
          realized by many patterns; patterns are applied by many work units. dependencies point at what is stable.
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20, alignItems: 'start', marginTop: 16 }}>

          <div style={{ display: 'grid', gap: 12 }}>
            <SecLabel color={HOW}>contracts — the realisation surface (§4.2)</SecLabel>
            {H.contracts.map(c => (
              <Card key={c.id}>
                <div style={{ display: 'flex', gap: 9, alignItems: 'center' }}>
                  <Eyebrow color={HOW}>{c.kind}</Eyebrow>
                  {c.frozen && <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--blue-400)' }}>frozen once chosen</span>}
                </div>
                <div style={{ display: 'grid', gap: 5, marginTop: 8 }}>
                  {c.items.map((it, i) => <Mono key={i} size={10.5}>· {it}</Mono>)}
                </div>
                {c.scope && (
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 7 }}>
                    scope — {c.scope}</div>
                )}
                {c.satisfies && (
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--em-bridge)', marginTop: 8,
                    borderTop: '1px dashed var(--slate-700)', paddingTop: 7 }}>
                    satisfies → {c.satisfies} · the seam between them is verified (§6.3)</div>
                )}
              </Card>
            ))}

          </div>

          <div style={{ display: 'grid', gap: 12 }}>
            <SecLabel color={HOW}>interface contracts — use the standards (§4.4)</SecLabel>
            <Card dashed>
              <div style={{ display: 'grid', gap: 7 }}>
                {H.standards.map(s => (
                  <div key={s.surface} style={{ display: 'flex', gap: 9, alignItems: 'baseline', flexWrap: 'wrap' }}>
                    <Mono size={10.5} color="var(--slate-100)">{s.surface}</Mono>
                    <Chip color="var(--blue-400)">{s.standard}</Chip>
                    <Mono size={9.5} color="var(--slate-500)">{s.derived}</Mono>
                  </div>
                ))}
              </div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 9,
                borderTop: '1px dashed var(--slate-700)', paddingTop: 8, lineHeight: 1.5 }}>
                generated from the domain model, never hand-written — the surface cannot drift from the meaning.</div>
            </Card>
          </div>
        </div>
        <Foot>declared once, referenced by pointer — work units apply principles; they never re-decide them.
          a principle no unit applies and no verification enforces is documentation, not model.</Foot>
      </Page>
    );
  }

  /* ================= Layout (§4.3) ================= */
  const KIND_COLOR = { 'must-exist': 'var(--conf-verified)', 'may-exist-here': 'var(--blue-400)',
    'must-co-exist': 'var(--blue-400)', 'must-not-exist': 'var(--error, #dc2626)', 'no-orphans': HOW };
  function LayoutView({ selected, onSelect }) {
    const sel = selected;
    const pick = (id) => onSelect(sel === id ? null : id);
    return (
      <Page max={1180}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
          <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>Repository layout model</h2>
          <Mono color="var(--slate-500)">what files are legal where — allowlist by default, the first gate to run</Mono>
        </div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 6 }}>
          click a rule to light up the files it admits — or a file to find its rule.
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1.05fr', gap: 20, marginTop: 14, alignItems: 'start' }}>
          {/* rules */}
          <div style={{ display: 'grid', gap: 9 }}>
            {PF.how.layout.map(r => {
              const on = sel === r.id;
              return (
                <div key={r.id} onClick={() => pick(r.id)} style={{ cursor: 'pointer',
                  background: 'var(--slate-800)', borderRadius: 8, padding: '10px 13px',
                  border: `1.5px solid ${on ? 'var(--blue-400)' : (r.verdict === 'fail' ? 'var(--error, #dc2626)' : 'var(--slate-700)')}`,
                  boxShadow: on ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)' : 'var(--shadow-graph)' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 9, flexWrap: 'wrap' }}>
                    <Chip color={KIND_COLOR[r.kind]}>{r.kind}</Chip>
                    <Mono size={10.5} color="var(--slate-100)">{r.glob}</Mono>
                    {r.cardinality && <Mono size={9} color="var(--slate-500)">· {r.cardinality}</Mono>}
                    <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9.5,
                      color: r.verdict === 'pass' ? 'var(--conf-verified)' : 'var(--error, #dc2626)' }}>
                      {r.verdict === 'pass' ? '● pass' : '● fail'}</span>
                  </div>
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-400)', marginTop: 5, lineHeight: 1.5 }}>
                    {r.rationale}
                    <span style={{ color: HOW }}> · enforces → {r.enforces}</span>
                  </div>
                  {r.finding && (
                    <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--error, #dc2626)', marginTop: 4 }}>
                      finding: {r.finding}</div>
                  )}
                </div>
              );
            })}
          </div>

          {/* the tree */}
          <div style={{ background: 'var(--slate-950)', border: '1px solid var(--slate-700)', borderRadius: 8,
            padding: '13px 8px 13px 15px', overflow: 'auto' }}>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, fontWeight: 600, letterSpacing: '.14em',
              textTransform: 'uppercase', color: 'var(--slate-500)', marginBottom: 9 }}>the tree, as the model sees it</div>
            {PF.repoTree.map((row, i) => {
              const hot = sel && row.rule === sel;
              const dim = sel && !hot;
              const ign = row.verdict === 'ignored';
              const fail = row.verdict === 'fail';
              const lineColor = ign ? 'var(--slate-600)' : fail ? 'var(--error, #dc2626)' : row.dir ? 'var(--slate-300)' : 'var(--slate-100)';
              return (
                <div key={i} onClick={() => row.rule && pick(row.rule)} style={{
                  display: 'flex', alignItems: 'baseline', gap: 10, padding: '1.5px 7px 1.5px 0',
                  cursor: row.rule ? 'pointer' : 'default', borderRadius: 4,
                  background: hot ? 'color-mix(in srgb, var(--blue-400) 14%, transparent)'
                    : (fail && !sel ? 'color-mix(in srgb, var(--error, #dc2626) 10%, transparent)' : 'transparent'),
                  opacity: dim ? 0.38 : ign ? 0.62 : 1, transition: 'opacity .15s var(--ease), background .15s var(--ease)',
                }}>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11.5, whiteSpace: 'pre',
                    color: lineColor, fontStyle: ign ? 'italic' : 'normal',
                    fontWeight: row.dir ? 600 : 400 }}>{row.line}</span>
                  <span style={{ marginLeft: 'auto', display: 'flex', gap: 7, alignItems: 'baseline', flex: 'none' }}>
                    {row.note && <Mono size={8.5} color={ign ? 'var(--slate-500)' : fail ? 'var(--error, #dc2626)' : 'var(--slate-500)'}>{row.note}</Mono>}
                    {row.rule && (
                      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, borderRadius: 3, padding: '0 6px',
                        color: ign ? 'var(--slate-600)' : fail ? 'var(--error, #dc2626)' : KIND_COLOR[(PF.how.layout.find(r => r.id === row.rule) || {}).kind] || 'var(--slate-400)',
                        border: `1px solid ${ign ? 'var(--slate-700)' : fail ? 'var(--error, #dc2626)' : 'var(--slate-700)'}` }}>{ign ? 'git-ignored' : row.rule}</span>
                    )}
                  </span>
                </div>
              );
            })}
          </div>
        </div>

        <Foot>every rule cites the principle it protects — a prohibition with no principle behind it is a superstition.
          globs match paths, not meaning: this is the cheap structural gate, layered below the content audits.
          the same declaration scaffolds (a work unit reads it to place files) and verifies (the checker reads it to reject).</Foot>
      </Page>
    );
  }

  /* ================= Reification (§4.5 / §11) ================= */
  const VT_COLOR = { machine: 'var(--conf-verified)', assisted: 'var(--em-event)', manual: 'var(--em-trigger)' };
  function ReificationView() {
    const M = PF.manifest;
    return (
      <Page max={1120}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
          <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>Design-system manifest</h2>
          <Mono color="var(--slate-500)">{M.id} · v{M.version} · wcag {M.wcagTarget} · reify(AIO, context) → CIO</Mono>
        </div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 6, lineHeight: 1.55, maxWidth: 780 }}>
          the other half of the render contract: §11 says what a design system can render; the contract says what
          there is to render. couple them and the Screens wireframes become real components — without touching the What.
        </div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, marginTop: 7 }}>
          <span style={{ color: 'var(--conf-verified)' }}>H3 gate green</span>
          <span style={{ color: 'var(--slate-500)' }}> — {M.reification.length} (AIO × context) pairs bound · {M.unreifiable.length} waived with reason · 0 unresolved</span>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1.1fr', gap: 20, marginTop: 16, alignItems: 'start' }}>
          {/* CIO catalog */}
          <div>
            <SecLabel color={VIEW}>the CIO catalog — a closed vocabulary</SecLabel>
            <div style={{ display: 'grid', gap: 8, marginTop: 9 }}>
              {M.components.map(c => (
                <div key={c.id} style={{ background: 'var(--slate-800)', border: '1.5px solid var(--slate-700)',
                  borderRadius: 7, padding: '9px 12px' }}>
                  <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
                    <Mono size={11.5} color="var(--slate-100)">{c.id}</Mono>
                    <span style={{ marginLeft: 'auto', display: 'flex', gap: 4 }}>
                      {c.satisfies.map(([cr, vt]) => (
                        <span key={cr} title={vt} style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5,
                          color: VT_COLOR[vt], border: '1px solid var(--slate-700)', borderRadius: 3, padding: '0 5px' }}>{cr}</span>
                      ))}
                    </span>
                  </div>
                  <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginTop: 6 }}>
                    {c.tokens.map(t => (
                      <span key={t} style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: 'var(--slate-500)',
                        background: 'var(--slate-900)', borderRadius: 3, padding: '1px 5px' }}>{t}</span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* reification rules */}
          <div>
            <SecLabel color={NAV}>reification rules — with rationale, coverage-checked</SecLabel>
            <div style={{ display: 'grid', gap: 8, marginTop: 9 }}>
              {M.reification.map((r, i) => (
                <div key={i} style={{ background: 'var(--slate-800)', border: '1.5px solid var(--slate-700)',
                  borderRadius: 7, padding: '9px 12px' }}>
                  <div style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                    <Chip color={NAV}>{r.aio}</Chip>
                    <Mono size={9.5} color="var(--slate-500)">{r.when}</Mono>
                    <Mono size={10} color="var(--slate-400)">→</Mono>
                    <Chip color={VIEW}>{r.cio}</Chip>
                  </div>
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 5, lineHeight: 1.5 }}>
                    rationale — {r.rationale}</div>
                </div>
              ))}
            </div>

            <div style={{ marginTop: 14 }}>
              <SecLabel>declared unreifiable — a recorded gap, never a silent pass</SecLabel>
              {M.unreifiable.map((u, i) => (
                <div key={i} style={{ marginTop: 8, border: '1.5px dashed var(--em-event)', borderRadius: 7,
                  padding: '9px 12px', background: 'color-mix(in srgb, var(--em-event) 6%, transparent)' }}>
                  <div style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                    <Chip color={NAV}>{u.aio}</Chip>
                    <Mono size={9.5} color="var(--slate-400)">in {u.cls}</Mono>
                    <Mono size={9.5} color="var(--em-event)">unreifiable</Mono>
                  </div>
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 5, lineHeight: 1.5 }}>{u.rationale}</div>
                </div>
              ))}
            </div>
          </div>
        </div>
        <Foot>one AIO, many CIOs, by context — the What is unchanged, which is the entire reason the AIO layer exists.
          a rule may only target a component the system defines: reification chooses among the vocabulary, never escapes it.</Foot>
      </Page>
    );
  }

  /* ================= Composition (§4.5 · Atomic Design) ================= */
  const LEVEL_COLOR = { Atoms: 'var(--slate-400)', Molecules: 'var(--blue-400)', Organisms: VIEW,
    Templates: HOW, Pages: NAV };
  const KIND_LC = { atom: 'var(--slate-400)', molecule: 'var(--blue-400)', organism: VIEW, template: HOW, page: NAV };
  function CompositionView() {
    const C = PF.composition;
    return (
      <Page max={1120}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
          <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>Screen composition</h2>
          <Mono color="var(--slate-500)">Atomic Design is the normative structure — nothing in a screen exists outside the five levels</Mono>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1.15fr', gap: 20, marginTop: 16, alignItems: 'start' }}>
          {/* the five levels */}
          <div style={{ display: 'grid', gap: 9 }}>
            <SecLabel color={HOW}>the five levels — conformance roles</SecLabel>
            {C.levels.map(l => (
              <div key={l.level} style={{ background: 'var(--slate-800)', borderRadius: 8, padding: '10px 13px',
                border: '1.5px solid var(--slate-700)', borderLeft: `3px solid ${LEVEL_COLOR[l.level]}` }}>
                <div style={{ display: 'flex', gap: 9, alignItems: 'baseline' }}>
                  <span style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 13.5, color: LEVEL_COLOR[l.level] }}>{l.level}</span>
                  <Mono size={9.5} color="var(--slate-400)">{l.is}</Mono>
                </div>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-400)', marginTop: 5, lineHeight: 1.5 }}>{l.role}</div>
                <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginTop: 7 }}>
                  {l.examples.map(e => (
                    <span key={e} style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: 'var(--slate-300)',
                      background: 'var(--slate-900)', borderRadius: 3, padding: '1px 6px' }}>{e}</span>
                  ))}
                </div>
              </div>
            ))}
          </div>

          {/* the worked page */}
          <div>
            <SecLabel color={NAV}>the worked page — {C.page.id} realizes {C.page.realizes}</SecLabel>
            <div style={{ marginTop: 9, background: 'var(--slate-950)', border: '1px solid var(--slate-700)',
              borderRadius: 8, padding: '13px 15px', overflow: 'auto' }}>
              {C.page.tree.map((row, i) => (
                <div key={i} style={{ display: 'flex', alignItems: 'baseline', gap: 10, padding: '1.5px 0' }}>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11.5, whiteSpace: 'pre', color: 'var(--slate-100)' }}>{row.line}</span>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: KIND_LC[row.kind],
                    border: '1px solid var(--slate-700)', borderRadius: 3, padding: '0 6px', flex: 'none' }}>{row.kind}</span>
                  {row.bind && <Mono size={8.5} color="var(--slate-500)">{row.bind}</Mono>}
                </div>
              ))}
            </div>

            <div style={{ marginTop: 14 }}>
              <SecLabel>the seam checks (§6.3) — what makes the screen and the flow agree</SecLabel>
              <div style={{ display: 'grid', gap: 5, marginTop: 8 }}>
                {C.page.checks.map((c, i) => (
                  <div key={i} style={{ display: 'flex', gap: 8, alignItems: 'baseline' }}>
                    <span style={{ color: 'var(--conf-verified)', fontSize: 11, width: 12, textAlign: 'center', flex: 'none' }}>●</span>
                    <Mono size={10} color="var(--slate-400)">{c.t}</Mono>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
        <Foot>a page is the realised form of a UI step — its data and controls are derived from the step, never authored
          on the screen. the component set is closed: a screen composed of a component the design system does not define
          fails, which is what makes a UI provably on-system rather than merely styled to look like it.</Foot>
      </Page>
    );
  }

  /* ================= Patterns (§4.1) — the building blocks ================= */
  const VERB_COLOR = { 'lays down': VIEW, creates: VIEW, requires: 'var(--blue-400)', freezes: HOW,
    modifies: HOW, generates: 'var(--blue-400)', declares: 'var(--blue-400)' };
  function PatternsView({ onOpenRule }) {
    const H = PF.how;
    return (
      <Page max={1150}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
          <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>Patterns — the building blocks</h2>
          <Mono color="var(--slate-500)">each pattern is a concrete shape: the files it lays down, and the layout rules that make it checkable</Mono>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(480px, 1fr))', gap: 14, marginTop: 16 }}>
          {H.patterns.map(p => {
            const prin = H.principles.find(x => x.id === p.implements) || {};
            return (
              <div key={p.id} style={{ background: 'var(--slate-800)', border: '1.5px solid var(--slate-700)',
                borderRadius: 9, padding: '13px 16px', boxShadow: 'var(--shadow-graph)',
                display: 'flex', flexDirection: 'column', gap: 10 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 9 }}>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.12em',
                    textTransform: 'uppercase', color: 'var(--slate-400)' }}>pattern</span>
                  <Mono size={11} color="var(--slate-100)">{p.id}</Mono>
                </div>
                <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 600, fontSize: 13.5, color: 'var(--slate-100)', lineHeight: 1.35 }}>{p.text}</div>

                {/* the file shape — what changes, how */}
                <div style={{ background: 'var(--slate-950)', border: '1px solid var(--slate-700)', borderRadius: 6, padding: '9px 12px' }}>
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.13em',
                    textTransform: 'uppercase', color: 'var(--slate-500)', marginBottom: 7 }}>file shape</div>
                  <div style={{ display: 'grid', gap: 5 }}>
                    {p.files.map((f, i) => (
                      <div key={i} style={{ display: 'flex', gap: 9, alignItems: 'baseline', flexWrap: 'wrap' }}>
                        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: VERB_COLOR[f.verb] || 'var(--slate-400)',
                          border: `1px solid ${VERB_COLOR[f.verb] || 'var(--slate-600)'}`, borderRadius: 3, padding: '0 6px', flex: 'none' }}>{f.verb}</span>
                        <Mono size={10.5} color="var(--slate-100)">{f.path}</Mono>
                        <Mono size={9} color="var(--slate-500)">{f.note}</Mono>
                      </div>
                    ))}
                  </div>
                </div>

                {/* the paths out — principle above, layout rules + units below */}
                <div style={{ display: 'flex', gap: 7, flexWrap: 'wrap', alignItems: 'center',
                  borderTop: '1px dashed var(--slate-700)', paddingTop: 9 }}>
                  <Chip color="var(--blue-400)">implements → {p.implements}</Chip>
                  {p.rules.map(r => (
                    <button key={r} onClick={() => onOpenRule(r)} title="open in the layout model" style={{
                      cursor: 'pointer', background: 'transparent', fontFamily: 'var(--font-mono)', fontSize: 9.5,
                      color: HOW, border: `1px solid ${HOW}`, borderRadius: 3, padding: '1px 7px' }}>
                      checked by → {r}</button>
                  ))}
                  <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)' }}>
                    applied by {p.units.length} unit{p.units.length > 1 ? 's' : ''}</span>
                </div>
                {prin.text && (
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', lineHeight: 1.5 }}>
                    protects — “{prin.text}”</div>
                )}
              </div>
            );
          })}
        </div>
        <Foot>a pattern is teachable because its shape is declared: the files it lays down are exactly what the layout
          model demands, so scaffolding and verification read the same declaration (§4.3, dual-read). click a
          “checked by” rule to see it against the tree.</Foot>
      </Page>
    );
  }

  Object.assign(window, { DecisionsView, LayoutView, ReificationView, CompositionView, PatternsView });
})();
