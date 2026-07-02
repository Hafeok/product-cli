/* global React, PF, PFUI */
/* Features & releases (§7) — Delivery views.
   A feature is a reference to a slice of the event model; its concepts are the
   derived footprint; 'done' is the computed feature_done(f) predicate. Three
   layouts (a tweak): a release board, a footprint explorer, a done ledger. */
(function () {
  const { useState } = React;
  const { EdgeLayer, ConfDot, FitCanvas } = window.PFUI;

  const CONCEPT_COLOR = {
    aggregate: 'var(--kind-entity)', entity: 'var(--blue-500)', 'value-object': 'var(--blue-400)',
    invariant: 'var(--kind-invariant)', external: 'var(--slate-400)', reference: 'var(--em-event)',
  };
  // which derivation link pulls a concept into the footprint (§7.1/§9)
  const DERIVE_LABEL = {
    aggregate: 'targets', entity: 'changes', 'value-object': 'projects',
    invariant: 'enforces', reference: 'reads', external: 'references',
  };
  const BUILD = 'var(--em-view)'; // Delivery / Build phase = green

  // ---- a done-clause status glyph -----------------------------------------
  const CLAUSE = {
    pass: { c: 'var(--conf-verified)', g: '\u25CF', t: 'pass' },
    partial: { c: 'var(--em-event)', g: '\u25D0', t: 'partial' },
    pending: { c: 'var(--slate-500)', g: '\u25CB', t: 'pending' },
    fail: { c: 'var(--error)', g: '\u25CF', t: 'fail' },
  };
  function Clause({ k, state, dense }) {
    const s = CLAUSE[state] || CLAUSE.pending;
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 7 }}>
        <span style={{ color: s.c, fontSize: 11, width: 12, textAlign: 'center' }}>{s.g}</span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: dense ? 9.5 : 10.5, color: 'var(--slate-300)', flex: 1 }}>{k}</span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: s.c, letterSpacing: '.06em' }}>{s.t}</span>
      </div>
    );
  }

  function ProgressBar({ frac, color = BUILD, h = 5 }) {
    return (
      <div style={{ height: h, borderRadius: h, background: 'var(--slate-700)', overflow: 'hidden' }}>
        <div style={{ height: '100%', width: (frac * 100) + '%', background: color,
          transition: 'width .3s var(--ease)' }} />
      </div>
    );
  }

  const doneFrac = (f) => {
    const v = Object.values(f.done);
    return v.reduce((a, s) => a + (s === 'pass' ? 1 : s === 'partial' ? 0.5 : 0), 0) / v.length;
  };

  // ---- a feature card (board) ---------------------------------------------
  function FeatureCard({ f, selected, onClick, showConf, dense }) {
    const planned = f.conformance === 'described';
    return (
      <div onClick={onClick} style={{
        cursor: 'pointer', boxSizing: 'border-box',
        background: planned ? 'var(--slate-900)' : 'var(--slate-800)',
        border: `1.5px ${planned ? 'dashed' : 'solid'} ${selected ? 'var(--blue-400)' : (planned ? 'var(--slate-600)' : 'var(--slate-700)')}`,
        borderRadius: 7, padding: dense ? '9px 11px' : '11px 13px',
        boxShadow: selected ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)' : (planned ? 'none' : 'var(--shadow-graph)'),
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 7 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.12em',
            textTransform: 'uppercase', color: BUILD }}>feature</span>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)' }}>{f.sub.split(' ')[0]}</span>
          {showConf && <span style={{ marginLeft: 'auto' }}><ConfDot level={f.conformance} size={8} /></span>}
        </div>
        <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: dense ? 13 : 14.5,
          color: 'var(--slate-100)', marginTop: 3 }}>{f.name}</div>
        <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap', marginTop: 6 }}>
          {f.flows.map(fl => (
            <span key={fl} style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-300)',
              background: 'var(--slate-900)', border: '1px solid var(--slate-700)', borderRadius: 3, padding: '1px 6px' }}>{fl}</span>
          ))}
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)',
            border: '1px solid var(--slate-800)', borderRadius: 3, padding: '1px 6px' }}>{f.footprint.length} concepts</span>
        </div>
        <div style={{ marginTop: 9, display: 'grid', gap: 4,
          borderTop: '1px dashed var(--slate-700)', paddingTop: 8 }}>
          <Clause k="flows realised & conformant" state={f.done.flows} dense={dense} />
          <Clause k="footprint conformant" state={f.done.footprint} dense={dense} />
          <Clause k="verifications green" state={f.done.verifications} dense={dense} />
          <Clause k="acceptance criteria" state={f.done.acceptance} dense={dense} />
        </div>
        <div style={{ marginTop: 9 }}>
          <ProgressBar frac={doneFrac(f)} color={PF.featureDone(f) ? BUILD : 'var(--em-event)'} />
        </div>
      </div>
    );
  }

  // ---- release column (board) ---------------------------------------------
  const STATUS = {
    delivered: { c: 'var(--conf-delivered)', t: 'delivered' },
    'in-progress': { c: 'var(--em-event)', t: 'in progress' },
    planned: { c: 'var(--slate-500)', t: 'planned' },
  };
  function ReleaseColumn({ rel, selected, onSelect, showConf, dense }) {
    const feats = rel.features.map(id => PF.feature(id));
    const doneN = feats.filter(PF.featureDone).length;
    const st = STATUS[rel.status] || STATUS.planned;
    const planned = rel.status === 'planned';
    return (
      <div style={{ flex: 'none', width: dense ? 268 : 296, display: 'flex', flexDirection: 'column', gap: 10 }}>
        <div style={{ background: 'var(--slate-800)', border: `1.5px ${planned ? 'dashed' : 'solid'} var(--slate-600)`,
          borderRadius: 8, padding: '11px 13px', position: 'relative' }}>
          <span style={{ position: 'absolute', top: 12, right: 13, display: 'flex', alignItems: 'center', gap: 6 }}>
            <span style={{ width: 7, height: 7, borderRadius: '50%', background: st.c }} />
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: st.c, letterSpacing: '.04em' }}>{st.t}</span>
          </span>
          <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 15, color: 'var(--slate-100)',
            paddingRight: 84 }}>{rel.name}</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 6 }}>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--blue-400)',
              border: '1px solid var(--slate-700)', borderRadius: 3, padding: '1px 6px' }}>What {rel.whatVersion}</span>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-400)' }}>{doneN}/{feats.length} done</span>
            <span title="release_done: all members done AND the cut is closed"
              style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9.5,
                color: rel.closed ? 'var(--conf-verified)' : 'var(--slate-500)' }}>
              {rel.closed ? 'cut closed \u2713' : 'cut open'}
            </span>
          </div>
          <div style={{ marginTop: 8 }}><ProgressBar frac={feats.length ? doneN / feats.length : 0}
            color={doneN === feats.length && !planned ? BUILD : 'var(--em-event)'} h={6} /></div>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 9 }}>
          {feats.map(f => (
            <FeatureCard key={f.id} f={f} selected={selected === f.id} onClick={() => onSelect(f.id)}
              showConf={showConf} dense={dense} />
          ))}
        </div>
      </div>
    );
  }

  function BoardLayout({ selected, onSelect, showConf, dense }) {
    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto' }}>
        <div style={{ display: 'flex', gap: 18, padding: '18px 22px 26px', minWidth: 'min-content', alignItems: 'flex-start' }}>
          {PF.delivery.releases.map(rel => (
            <ReleaseColumn key={rel.id} rel={rel} selected={selected} onSelect={onSelect} showConf={showConf} dense={dense} />
          ))}
        </div>
      </div>
    );
  }

  // ---- footprint explorer -------------------------------------------------
  const FP_W = 1140, FP_H = 620;
  function FootprintLayout({ feature, showConf }) {
    const f = feature;
    const flowH = 58, gapF = 22;
    const flowsTop = FP_H / 2 - ((f.flows.length * flowH + (f.flows.length - 1) * gapF) / 2) + flowH / 2;
    const flowPos = {}; const featurePos = { x: 190, y: FP_H / 2, w: 210, h: 72 };
    f.flows.forEach((fl, i) => { flowPos['flow:' + fl] = { x: 540, y: flowsTop + i * (flowH + gapF), w: 200, h: flowH }; });

    const concepts = f.footprint.map(id => PF.concept(id));
    const cH = 62, gapC = 16;
    const cTop = FP_H / 2 - ((concepts.length * cH + (concepts.length - 1) * gapC) / 2) + cH / 2;
    const cPos = {};
    concepts.forEach((c, i) => { cPos['c:' + c.id] = { x: 930, y: cTop + i * (cH + gapC), w: 200, h: cH }; });

    const pos = { feat: featurePos, ...flowPos, ...cPos };
    const edges = [];
    f.flows.forEach(fl => edges.push({ from: 'feat', to: 'flow:' + fl, stroke: BUILD, width: 1.6, dash: '5 4',
      label: 'references', labelColor: BUILD }));
    // flow -> concept, one link per concept, labelled by its derivation kind
    concepts.forEach(c => {
      const fromFlow = 'flow:' + f.flows[0];
      edges.push({ from: fromFlow, to: 'c:' + c.id, stroke: 'var(--slate-500)', width: 1.3, dash: '3 4',
        label: DERIVE_LABEL[c.kind] || 'derives', labelColor: 'var(--slate-400)' });
    });

    return (
      <FitCanvas width={FP_W} height={FP_H}>
        {/* column captions */}
        <Caption x={190} label="feature" color={BUILD} />
        <Caption x={540} label="flow slice · the What" color="var(--slate-400)" />
        <Caption x={930} label="derived footprint · concepts" color="var(--kind-entity)" />

        <EdgeLayer edges={edges} pos={pos} width={FP_W} height={FP_H} showLabels={true} />

        {/* feature node */}
        <Box box={featurePos}>
          <div style={{ width: '100%', height: '100%', boxSizing: 'border-box', background: 'var(--slate-800)',
            border: `1.5px solid ${BUILD}`, borderRadius: 8, padding: '10px 13px', boxShadow: 'var(--shadow-graph)' }}>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.12em',
              textTransform: 'uppercase', color: BUILD }}>feature</div>
            <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 15, color: 'var(--slate-100)', marginTop: 2 }}>{f.name}</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 7, marginTop: 5 }}>
              {showConf && <ConfDot level={f.conformance} size={7} />}
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-400)' }}>{f.sub.split(' ')[0]}</span>
            </div>
          </div>
        </Box>

        {/* flow nodes */}
        {f.flows.map(fl => {
          const flow = PF.flows[fl];
          const b = flowPos['flow:' + fl];
          return (
            <Box key={fl} box={b}>
              <div style={{ width: '100%', height: '100%', boxSizing: 'border-box', background: 'var(--slate-800)',
                border: '1.5px solid var(--slate-600)', borderRadius: 7, padding: '8px 12px', display: 'flex',
                flexDirection: 'column', justifyContent: 'center' }}>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.1em',
                  textTransform: 'uppercase', color: 'var(--em-command)' }}>flow{flow ? ' \u00b7 ' + flow.pattern : ''}</div>
                <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 600, fontSize: 13, color: 'var(--slate-100)', marginTop: 1 }}>
                  {flow ? flow.name : fl}</div>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)', marginTop: 1 }}>{fl}</div>
              </div>
            </Box>
          );
        })}

        {/* concept chips */}
        {concepts.map(c => {
          const b = cPos['c:' + c.id];
          const col = CONCEPT_COLOR[c.kind] || 'var(--slate-400)';
          const dashed = c.kind === 'invariant' || c.kind === 'value-object' || c.sub.includes('not built');
          return (
            <Box key={c.id} box={b}>
              <div style={{ width: '100%', height: '100%', boxSizing: 'border-box',
                background: `color-mix(in srgb, ${col} 13%, var(--slate-900))`,
                border: `1.5px ${dashed ? 'dashed' : 'solid'} ${col}`, borderRadius: c.kind === 'value-object' ? 12 : 6,
                padding: '8px 12px', display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.1em',
                  textTransform: 'uppercase', color: col }}>{c.kind}</div>
                <div style={{ fontFamily: c.kind === 'invariant' ? 'var(--font-mono)' : 'var(--font-sans)',
                  fontWeight: 700, fontSize: c.kind === 'invariant' ? 12 : 13.5, color: 'var(--slate-100)', marginTop: 1 }}>{c.label}</div>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: 'var(--slate-500)', marginTop: 1 }}>{c.sub}</div>
              </div>
            </Box>
          );
        })}

        {/* footer note */}
        <div style={{ position: 'absolute', left: 22, bottom: 14, maxWidth: 560, fontFamily: 'var(--font-mono)',
          fontSize: 10.5, color: 'var(--slate-500)', lineHeight: 1.5 }}>
          footprint({f.id}) is <span style={{ color: 'var(--slate-300)' }}>derived, not declared</span> — a graph traversal of the flow
          slice. You never maintain a feature’s concept list by hand.
        </div>
      </FitCanvas>
    );
  }
  function Caption({ x, label, color }) {
    return (
      <div style={{ position: 'absolute', left: x - 100, top: 16, width: 200, textAlign: 'center',
        fontFamily: 'var(--font-mono)', fontSize: 9.5, fontWeight: 600, letterSpacing: '.14em',
        textTransform: 'uppercase', color }}>{label}</div>
    );
  }
  function Box({ box, children }) {
    return (
      <div style={{ position: 'absolute', left: box.x - box.w / 2, top: box.y - box.h / 2, width: box.w, height: box.h, zIndex: 2 }}>
        {children}
      </div>
    );
  }

  // ---- done ledger --------------------------------------------------------
  function LedgerRow({ state, label, detail }) {
    const s = CLAUSE[state] || CLAUSE.pending;
    return (
      <div style={{ display: 'flex', gap: 11, alignItems: 'flex-start', padding: '8px 0', borderTop: '1px solid var(--slate-800)' }}>
        <span style={{ color: s.c, fontSize: 12, width: 14, textAlign: 'center', flex: 'none', marginTop: 1 }}>{s.g}</span>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 11.5, color: 'var(--slate-200)' }}>{label}</div>
          {detail && <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 2, lineHeight: 1.5 }}>{detail}</div>}
        </div>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: s.c, letterSpacing: '.06em', flex: 'none' }}>{s.t}</span>
      </div>
    );
  }
  function LedgerLayout({ feature, showConf }) {
    const f = feature;
    const done = PF.featureDone(f);
    const concepts = f.footprint.map(id => PF.concept(id));
    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto' }}>
        <div style={{ maxWidth: 900, margin: '0 auto', padding: '22px 26px 40px' }}>
          {/* header */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.14em',
              textTransform: 'uppercase', color: BUILD }}>feature_done</span>
            <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 20, color: 'var(--slate-100)' }}>{f.name}</h2>
            {showConf && <ConfDot level={f.conformance} />}
            <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 12, fontWeight: 600,
              color: done ? 'var(--conf-verified)' : 'var(--em-event)' }}>{done ? 'DONE \u2713' : 'NOT DONE'}</span>
          </div>

          {/* predicate */}
          <pre style={{ margin: '16px 0 0', background: 'var(--slate-950)', border: '1px solid var(--slate-700)',
            borderRadius: 7, padding: '14px 16px', fontFamily: 'var(--font-mono)', fontSize: 11.5, lineHeight: 1.7,
            color: 'var(--slate-300)', overflowX: 'auto', whiteSpace: 'pre' }}>{
`feature_done(${f.id}) := every flow in ${f.id} is realised & passes behavioural conformance
              and every concept in footprint(${f.id}) is realised & passes domain conformance
              and every verification citing a ${f.id}-element is green
              and ${f.id}'s agreed acceptance criteria pass`}</pre>

          {/* the four clauses */}
          <div style={{ marginTop: 20 }}>
            <SectionLabel>the predicate, clause by clause</SectionLabel>
            <LedgerRow state={f.done.flows} label={`flows realised & behaviourally conformant`}
              detail={f.flows.join(', ')} />
            <LedgerRow state={f.done.footprint} label={`footprint concepts realised & domain-conformant`}
              detail={concepts.map(c => c.label).join(', ')} />
            <LedgerRow state={f.done.verifications} label="every verification citing a feature-element is green"
              detail={f.done.verifications === 'pass' ? 'all green' : f.done.verifications === 'partial' ? 'behavioural conformance still amber' : 'not yet run — slice not built'} />
            <LedgerRow state={f.done.acceptance} label="agreed acceptance criteria pass"
              detail={f.acceptance.join('  ·  ')} />
          </div>

          {/* footprint traversal */}
          <div style={{ marginTop: 22 }}>
            <SectionLabel>footprint({f.id}) — derived by traversal (§7.1)</SectionLabel>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginTop: 10 }}>
              {concepts.map(c => {
                const col = CONCEPT_COLOR[c.kind] || 'var(--slate-400)';
                return (
                  <span key={c.id} style={{ display: 'inline-flex', alignItems: 'center', gap: 7, fontFamily: 'var(--font-mono)',
                    fontSize: 10.5, color: 'var(--slate-200)', background: `color-mix(in srgb, ${col} 12%, var(--slate-900))`,
                    border: `1px solid ${col}`, borderRadius: 4, padding: '4px 9px' }}>
                    <span style={{ color: col, fontSize: 8.5, letterSpacing: '.1em', textTransform: 'uppercase' }}>{DERIVE_LABEL[c.kind]}</span>
                    {c.label}
                  </span>
                );
              })}
            </div>
          </div>

          <div style={{ marginTop: 22, fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--slate-400)',
            borderTop: '1px dashed var(--slate-700)', paddingTop: 12 }}>
            value action &nbsp;<span style={{ color: BUILD }}>→ {f.valueAction}</span>
            <span style={{ marginLeft: 14, color: 'var(--slate-500)' }}>progress is computed, not estimated — {Math.round(doneFrac(f) * 100)}% of clauses pass.</span>
          </div>
        </div>
      </div>
    );
  }
  function SectionLabel({ children }) {
    return (
      <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, fontWeight: 600, letterSpacing: '.14em',
        textTransform: 'uppercase', color: 'var(--slate-500)' }}>{children}</div>
    );
  }

  // ---- feature picker (footprint / ledger) --------------------------------
  function FeaturePicker({ active, onPick, showConf }) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '10px 18px', flexWrap: 'wrap',
        borderBottom: '1px solid var(--slate-800)', background: 'var(--slate-900)' }}>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, letterSpacing: '.12em', textTransform: 'uppercase', color: 'var(--slate-500)' }}>feature</span>
        <div style={{ display: 'flex', gap: 7, flexWrap: 'wrap' }}>
          {PF.delivery.features.map(f => {
            const on = f.id === active;
            return (
              <button key={f.id} onClick={() => onPick(f.id)} style={{
                display: 'inline-flex', alignItems: 'center', gap: 7, cursor: 'pointer',
                background: on ? 'var(--slate-700)' : 'transparent', color: on ? 'var(--slate-100)' : 'var(--slate-400)',
                border: `1px solid ${on ? 'var(--slate-500)' : 'var(--slate-700)'}`, borderRadius: 6,
                fontFamily: 'var(--font-sans)', fontSize: 12.5, fontWeight: 500, padding: '5px 11px',
              }}>
                {f.name}
                {showConf && <ConfDot level={f.conformance} size={7} />}
              </button>
            );
          })}
        </div>
      </div>
    );
  }

  // ---- the view -----------------------------------------------------------
  function FeaturesView({ layout, selected, onSelect, showConf, dense }) {
    const [pick, setPick] = useState('checkout');
    const active = (layout === 'board') ? selected : (selected || pick);
    const feature = PF.feature(active) || PF.feature('checkout');
    const choose = (id) => { setPick(id); onSelect(id); };

    if (layout === 'board') {
      return <BoardLayout selected={selected} onSelect={onSelect} showConf={showConf} dense={dense} />;
    }
    return (
      <div style={{ position: 'relative', width: '100%', height: '100%', display: 'flex', flexDirection: 'column' }}>
        <FeaturePicker active={feature.id} onPick={choose} showConf={showConf} />
        <div style={{ position: 'relative', flex: 1, minHeight: 0, background: 'var(--slate-900)' }}>
          {layout === 'footprint'
            ? <FootprintLayout feature={feature} showConf={showConf} />
            : <LedgerLayout feature={feature} showConf={showConf} />}
        </div>
      </div>
    );
  }

  Object.assign(window, { FeaturesView });
})();
