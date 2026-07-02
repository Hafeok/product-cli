/* global React, PF */
/* Scenarios (§3.3–3.4) — behavioural simulation. A flow gives a GIVEN of prior
   events, a WHEN command, a THEN of expected events (or read-model state).
   The oracle is authored once, in the What, and consumed twice: simulated
   before any code exists, then re-run against the realisation (§6.3). */
(function () {
  const CMD = 'var(--em-command)', EV = 'var(--em-event)', VIEW = 'var(--em-view)';
  const V = {
    pass: { c: 'var(--conf-verified)', g: '●', t: 'pass' },
    pending: { c: 'var(--slate-500)', g: '○', t: 'pending' },
    fail: { c: 'var(--error, #dc2626)', g: '●', t: 'fail' },
  };

  function Mono({ children, color, size }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: size || 10.5, color: color || 'var(--slate-300)' }}>{children}</span>;
  }
  function Chip({ children, color }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: color || 'var(--slate-300)',
      border: `1px solid ${color || 'var(--slate-600)'}`, borderRadius: 3, padding: '1px 7px' }}>{children}</span>;
  }
  function Gate({ state }) {
    const s = V[state] || V.pending;
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: s.c }}>{s.g} {s.t}</span>;
  }

  function ScenarioRow({ sc }) {
    const rejected = sc.then.verdict === 'Rejected';
    return (
      <div style={{ background: 'var(--slate-800)', border: '1.5px solid var(--slate-700)', borderRadius: 8,
        padding: '12px 15px', boxShadow: 'var(--shadow-graph)' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 9, flexWrap: 'wrap' }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.12em',
            textTransform: 'uppercase', color: sc.kind === 'decide' ? CMD : VIEW }}>{sc.kind}</span>
          <Mono color="var(--slate-500)" size={9.5}>{sc.decider || sc.projector} · {sc.flow}</Mono>
          <span style={{ marginLeft: 'auto', display: 'flex', gap: 14 }}>
            <span><Mono color="var(--slate-500)" size={9}>simulated </Mono><Gate state={sc.simulated} /></span>
            <span><Mono color="var(--slate-500)" size={9}>realised </Mono><Gate state={sc.realised} /></span>
          </span>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '6px 12px', marginTop: 10, alignItems: 'baseline' }}>
          <Mono color="var(--slate-500)" size={9.5}>given</Mono>
          <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
            {sc.given.length ? sc.given.map(g => <Chip key={g} color={EV}>{g}</Chip>)
              : <Mono color="var(--slate-500)" size={10}>∅ — no prior events</Mono>}
          </div>
          {sc.when && <>
            <Mono color="var(--slate-500)" size={9.5}>when</Mono>
            <div><Chip color={CMD}>{sc.when}</Chip></div>
          </>}
          <Mono color="var(--slate-500)" size={9.5}>then</Mono>
          <div style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
            {sc.then.verdict && (
              <Mono size={11} color={rejected ? 'var(--em-trigger-soft, var(--em-trigger))' : 'var(--conf-verified)'}>
                {sc.then.verdict}{rejected ? `(${sc.then.reason})` : `[${sc.then.events.join(', ')}]`}
              </Mono>
            )}
            {sc.then.state && <Mono size={11} color={VIEW}>{sc.then.state}</Mono>}
          </div>
        </div>

        {sc.note && <div style={{ marginTop: 8, fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--em-event)' }}>{sc.note}</div>}
      </div>
    );
  }

  function ScenariosView() {
    const decides = PF.scenarios.filter(s => s.kind === 'decide');
    const projects = PF.scenarios.filter(s => s.kind === 'project');
    const simPass = PF.scenarios.filter(s => s.simulated === 'pass').length;
    const realPass = PF.scenarios.filter(s => s.realised === 'pass').length;

    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto', background: 'var(--slate-900)' }}>
        <div style={{ maxWidth: 980, margin: '0 auto', padding: '20px 26px 40px' }}>
          <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
            <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>Behavioural simulation</h2>
            <Mono color="var(--slate-500)">flow-derived scenarios — the first gate, and the cheapest</Mono>
            <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-400)' }}>
              simulated {simPass}/{PF.scenarios.length} · realised {realPass}/{PF.scenarios.length}
            </span>
          </div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 6, lineHeight: 1.55, maxWidth: 780 }}>
            the oracle is authored once and consumed twice: <span style={{ color: 'var(--slate-300)' }}>simulated</span> runs
            before any code exists (pure function calls, no infrastructure); <span style={{ color: 'var(--slate-300)' }}>realised</span> re-runs
            the same scenarios against the built behaviour, which must produce identical outputs (§6.3).
          </div>

          <div style={{ marginTop: 16, display: 'grid', gap: 10 }}>
            <SecLabel color={CMD}>decide — commands against Deciders</SecLabel>
            {decides.map(sc => <ScenarioRow key={sc.id} sc={sc} />)}
            <div style={{ marginTop: 8 }}><SecLabel color={VIEW}>project — folds against Projectors</SecLabel></div>
            {projects.map(sc => <ScenarioRow key={sc.id} sc={sc} />)}
          </div>

          <div style={{ marginTop: 18, fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
            borderTop: '1px dashed var(--slate-700)', paddingTop: 11, lineHeight: 1.55, maxWidth: 820 }}>
            a behaviour defect caught here costs a sentence. the Screens preview consumes sc-proj-cart's projected
            output — the same simulation feeds the wireframe's data.
          </div>
        </div>
      </div>
    );
  }
  function SecLabel({ children, color }) {
    return <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
      textTransform: 'uppercase', color: color || 'var(--slate-500)' }}>{children}</div>;
  }

  Object.assign(window, { ScenariosView });
})();
