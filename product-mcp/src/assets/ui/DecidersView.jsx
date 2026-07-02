/* global React, PF, PFUI */
/* Deciders & Projectors (§3.3–3.4) — the executable form of behaviour.
   Signature DERIVED from the event model; only the logic is authored.
   decide(state, cmd) → Accepted[events] | Rejected[reason]
   project(state, ev) → state.  Plus the justification checks. */
(function () {
  const { ConfDot } = window.PFUI;
  const CMD = 'var(--em-command)', EV = 'var(--em-event)', VIEW = 'var(--em-view)', INV = 'var(--em-trigger-soft, var(--em-trigger))';

  function Label({ children, color }) {
    return <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, fontWeight: 600, letterSpacing: '.14em',
      textTransform: 'uppercase', color: color || 'var(--slate-500)' }}>{children}</div>;
  }
  function Mono({ children, color, size }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: size || 10.5, color: color || 'var(--slate-300)' }}>{children}</span>;
  }
  function Chip({ children, color }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: color || 'var(--slate-300)',
      border: `1px solid ${color || 'var(--slate-600)'}`, borderRadius: 3, padding: '1px 7px' }}>{children}</span>;
  }
  function RuleCheck({ ok, children }) {
    return (
      <div style={{ display: 'flex', gap: 8, alignItems: 'baseline' }}>
        <span style={{ color: ok ? 'var(--conf-verified)' : 'var(--em-event)', fontSize: 11, width: 12, textAlign: 'center', flex: 'none' }}>{ok ? '\u25cf' : '\u25d0'}</span>
        <Mono size={10} color="var(--slate-400)">{children}</Mono>
      </div>
    );
  }

  function DeciderCard({ d, showConf }) {
    return (
      <div style={{ background: 'var(--slate-800)', borderRadius: 9, padding: '14px 16px',
        border: `1.5px ${d.planned ? 'dashed' : 'solid'} var(--slate-600)`, boxShadow: d.planned ? 'none' : 'var(--shadow-graph)' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.12em',
            textTransform: 'uppercase', color: CMD }}>decider</span>
          <span style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 16, color: 'var(--slate-100)' }}>{d.aggregate}</span>
          <Mono color="var(--slate-500)" size={9.5}>decides_for {d.aggregate}</Mono>
          {showConf && <span style={{ marginLeft: 'auto' }}><ConfDot level={d.conformance} size={8} /></span>}
        </div>

        {/* the derived signature */}
        <pre style={{ margin: '11px 0 0', background: 'var(--slate-950)', border: '1px solid var(--slate-700)', borderRadius: 6,
          padding: '10px 13px', fontFamily: 'var(--font-mono)', fontSize: 11, lineHeight: 1.65, color: 'var(--slate-300)',
          overflowX: 'auto', whiteSpace: 'pre' }}>{
`decide(${d.aggregate.toLowerCase()}, command) -> Accepted[events] | Rejected[reason]
evolve(${d.aggregate.toLowerCase()}, event)   -> ${d.aggregate.toLowerCase()}`}</pre>

        {/* handles table — derived */}
        <div style={{ marginTop: 12 }}>
          <Label color={CMD}>handles — exactly the commands targeting {d.aggregate} · derived</Label>
          <div style={{ marginTop: 7, display: 'grid', gap: 6 }}>
            {d.handles.map(h => (
              <div key={h.cmd} style={{ display: 'flex', gap: 9, alignItems: 'baseline', flexWrap: 'wrap' }}>
                <Chip color={CMD}>{h.cmd}</Chip>
                <Mono color="var(--slate-500)" size={9.5}>{'\u2192'}</Mono>
                {h.emits.map(e => <Chip key={e} color={EV}>{e}</Chip>)}
                {h.rejects.length > 0 && <Mono color={INV} size={9.5}>| rejects: {h.rejects.join(', ')}</Mono>}
              </div>
            ))}
          </div>
        </div>

        {/* rejections — the authored substance */}
        <div style={{ marginTop: 12 }}>
          <Label color={INV}>rejections — the authored logic; the substance of a Decider</Label>
          <div style={{ marginTop: 7, display: 'grid', gap: 6 }}>
            {d.rejections.map(r => (
              <div key={r.id} style={{ display: 'flex', gap: 9, alignItems: 'baseline', flexWrap: 'wrap' }}>
                <Chip color={INV}>{r.id}</Chip>
                <Mono size={10.5}>{r.rule}</Mono>
                <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9,
                  color: r.reachable ? 'var(--conf-verified)' : 'var(--em-event)' }}>
                  {r.reachable ? 'reachable \u2713' : 'unreachable \u2014 ' + (r.note || 'finding')}</span>
              </div>
            ))}
          </div>
        </div>

        {/* state justification */}
        <div style={{ marginTop: 12 }}>
          <Label>state justification — every field has a reading decide()</Label>
          <div style={{ marginTop: 7, display: 'grid', gap: 5 }}>
            {d.stateRead.map(s => (
              <div key={s.field} style={{ display: 'flex', gap: 9, alignItems: 'baseline', flexWrap: 'wrap' }}>
                <Mono size={10.5} color="var(--slate-100)">{s.field}</Mono>
                <Mono size={9.5} color="var(--slate-500)">read by {s.readBy}</Mono>
              </div>
            ))}
          </div>
        </div>

        {/* conformance rules */}
        <div style={{ marginTop: 12, borderTop: '1px dashed var(--slate-700)', paddingTop: 10, display: 'grid', gap: 5 }}>
          <RuleCheck ok={d.coverage.foreign === 0}>no foreign commands — handles only commands targeting {d.aggregate}</RuleCheck>
          <RuleCheck ok={true}>command coverage {d.coverage.commands} — no command left unspecified</RuleCheck>
          <RuleCheck ok={d.coverage.outputs === 'contained'}>output-alphabet containment — emits only declared events</RuleCheck>
          <RuleCheck ok={d.rejections.some(r => r.reachable)}>decider justification — at least one reachable rejection</RuleCheck>
        </div>
      </div>
    );
  }

  function ProjectorCard({ p, showConf }) {
    return (
      <div style={{ background: 'var(--slate-800)', borderRadius: 9, padding: '13px 15px',
        border: '1.5px solid var(--slate-700)', boxShadow: 'var(--shadow-graph)' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.12em',
            textTransform: 'uppercase', color: VIEW }}>projector</span>
          <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 700, fontSize: 13, color: 'var(--slate-100)' }}>{p.readModel}</span>
          {showConf && <span style={{ marginLeft: 'auto' }}><ConfDot level={p.conformance} size={7} /></span>}
        </div>
        <div style={{ marginTop: 9, display: 'grid', gap: 5 }}>
          {p.folds.map(f => (
            <div key={f.ev} style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
              <Chip color={EV}>{f.ev}</Chip>
              <Mono size={9.5} color="var(--slate-500)">{'\u2192'} {f.into}</Mono>
            </div>
          ))}
        </div>
        <div style={{ marginTop: 9, fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-400)' }}>
          outputs <span style={{ color: 'var(--slate-200)' }}>{p.outputs.join(' \u00b7 ')}</span>
        </div>
        <div style={{ marginTop: 4, fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-400)' }}>
          consumed by <span style={{ color: VIEW }}>{p.consumers.join(', ')}</span>
          <span style={{ color: 'var(--slate-500)' }}> — every projected field has a reader</span>
        </div>
        <div style={{ marginTop: 9, borderTop: '1px dashed var(--slate-700)', paddingTop: 8, display: 'grid', gap: 4 }}>
          <RuleCheck ok={true}>no foreign events · event coverage {p.coverage.events} · output containment</RuleCheck>
        </div>
      </div>
    );
  }

  function DecidersView({ showConf }) {
    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto', background: 'var(--slate-900)' }}>
        <div style={{ maxWidth: 1180, margin: '0 auto', padding: '20px 26px 40px' }}>
          <div style={{ display: 'grid', gridTemplateColumns: '1.35fr 1fr', gap: 20, alignItems: 'start' }}>
            <div style={{ display: 'grid', gap: 14 }}>
              <Label color={CMD}>deciders — decide, the write half</Label>
              {PF.deciders.map(d => <DeciderCard key={d.id} d={d} showConf={showConf} />)}
            </div>
            <div style={{ display: 'grid', gap: 14 }}>
              <Label color={VIEW}>projectors — project, the read half · symmetric in every respect</Label>
              {PF.projectors.map(p => <ProjectorCard key={p.id} p={p} showConf={showConf} />)}
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
                border: '1.5px dashed var(--slate-700)', borderRadius: 8, padding: '11px 14px', lineHeight: 1.6 }}>
                orphaned state and toothless Deciders fail toward the same diagnosis from opposite
                sides — an invariant that exists in the business but not yet in the model. a pincer
                on unmodelled rules (§3.4).
              </div>
            </div>
          </div>
          <div style={{ marginTop: 18, fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
            borderTop: '1px dashed var(--slate-700)', paddingTop: 11, lineHeight: 1.55, maxWidth: 860 }}>
            the signature is pure What — derived from the graph and validated against it; only the decision logic
            is authored. the Decider becomes the oracle the realised behaviour is checked against (§6.3) — simulate
            it under Scenarios.
          </div>
        </div>
      </div>
    );
  }

  Object.assign(window, { DecidersView });
})();
