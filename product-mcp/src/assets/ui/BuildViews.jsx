/* global React, PF */
/* Build (§5–6) — two views in one file:
   WorkUnitsView     — the Build seam: frozen SPMC bundle out, verdict event back (§5.1)
   VerificationsView — the required verification kinds + the anatomy of a check (§6) */
(function () {
  const BUILD = 'var(--em-view)';
  const V = {
    accepted: { c: 'var(--conf-verified)', t: 'accepted' },
    pass: { c: 'var(--conf-verified)', t: 'pass' },
    escalate: { c: 'var(--em-event)', t: 'escalate' },
    partial: { c: 'var(--em-event)', t: 'partial' },
    fail: { c: 'var(--error, #dc2626)', t: 'fail' },
    rejected: { c: 'var(--error, #dc2626)', t: 'rejected' },
    'n/a': { c: 'var(--slate-500)', t: 'n/a' },
  };

  function Mono({ children, color, size }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: size || 10.5, color: color || 'var(--slate-300)' }}>{children}</span>;
  }
  function SecLabel({ children, color }) {
    return <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
      textTransform: 'uppercase', color: color || 'var(--slate-500)' }}>{children}</div>;
  }
  function Verdict({ v }) {
    const s = V[v] || V['n/a'];
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: s.c }}>● {s.t}</span>;
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

  /* ================= Work units (§5.1) ================= */
  function AxisRow({ axis, children }) {
    return (
      <div style={{ display: 'grid', gridTemplateColumns: '76px 1fr', gap: 10, alignItems: 'baseline',
        borderTop: '1px dashed var(--slate-700)', padding: '7px 0' }}>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, fontWeight: 600, letterSpacing: '.1em',
          textTransform: 'uppercase', color: 'var(--blue-400)' }}>{axis}</span>
        <div style={{ minWidth: 0 }}>{children}</div>
      </div>
    );
  }

  function WorkUnitCard({ wu }) {
    return (
      <div style={{ background: 'var(--slate-800)', border: '1.5px solid var(--slate-600)', borderRadius: 9,
        padding: '13px 16px', boxShadow: 'var(--shadow-graph)' }}>
        {/* envelope header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap' }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.12em',
            textTransform: 'uppercase', color: BUILD }}>work unit</span>
          <Mono size={11.5} color="var(--slate-100)">{wu.id}</Mono>
          <span style={{ marginLeft: 'auto' }}><Verdict v={wu.status} /></span>
        </div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)', marginTop: 4 }}>
          lineage {wu.lineage} · bundle_hash <span style={{ color: 'var(--blue-400)' }}>{wu.hash}</span> — the hash is the identity
        </div>

        {/* the SPMC bundle */}
        <div style={{ marginTop: 11 }}>
          <SecLabel>the frozen bundle — an SPMC bundle, by value</SecLabel>
          <div style={{ marginTop: 6 }}>
            <AxisRow axis="schema">
              <Mono size={10}>{wu.bundle.schema.artifact}</Mono>
              <div style={{ display: 'grid', gap: 3, marginTop: 4 }}>
                {wu.bundle.schema.criteria.map((c, i) => (
                  <Mono key={i} size={9.5} color="var(--slate-400)">· {c}</Mono>
                ))}
              </div>
            </AxisRow>
            <AxisRow axis="prompt"><Mono size={10} color="var(--slate-200)">{wu.bundle.prompt}</Mono></AxisRow>
            <AxisRow axis="model"><Mono size={10} color="var(--slate-400)">{wu.bundle.model}</Mono></AxisRow>
            <AxisRow axis="context">
              <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap' }}>
                {wu.bundle.context.map((c, i) => (
                  <span key={i} style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-300)',
                    background: 'var(--slate-900)', border: '1px solid var(--slate-700)', borderRadius: 3, padding: '1px 6px' }}>{c}</span>
                ))}
              </div>
            </AxisRow>
          </div>
        </div>

        {/* the returned verdict event */}
        <div style={{ marginTop: 11, border: `1.5px dashed ${V[wu.verdict.verdict].c}`, borderRadius: 7,
          padding: '9px 12px', background: `color-mix(in srgb, ${V[wu.verdict.verdict].c} 6%, transparent)` }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 9, flexWrap: 'wrap' }}>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.12em',
              textTransform: 'uppercase', color: V[wu.verdict.verdict].c }}>verdict event</span>
            <Mono size={9} color="var(--slate-500)">{wu.verdict.event} · {wu.verdict.at}</Mono>
            <span style={{ marginLeft: 'auto' }}>
              <Verdict v={wu.verdict.verdict} />
              <Mono size={9} color="var(--slate-500)"> → {wu.verdict.consequence}</Mono>
            </span>
          </div>
          <div style={{ display: 'grid', gap: 3, marginTop: 7 }}>
            {wu.verdict.findings.map((f, i) => (
              <Mono key={i} size={9.5} color={f.includes('fail') ? 'var(--error, #dc2626)' : 'var(--slate-400)'}>· {f}</Mono>
            ))}
          </div>
        </div>
      </div>
    );
  }

  function WorkUnitsView() {
    return (
      <Page max={980}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
          <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>The Build seam</h2>
          <Mono color="var(--slate-500)">a data contract, not a call — two shapes, two emit-points</Mono>
        </div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 6, lineHeight: 1.55, maxWidth: 780 }}>
          the specification side emits a frozen unit and never invokes the executor; the executor emits a verdict
          to a stream and holds no knowledge of any consumer. the freeze is the decoupling (§5.1).
        </div>
        <div style={{ display: 'grid', gap: 14, marginTop: 16 }}>
          {PF.workUnits.map(wu => <WorkUnitCard key={wu.id} wu={wu} />)}
        </div>
        <Foot>every verdict echoes the bundle hash it ran against — provenance is reproducible, and a verdict emitted
          while no one listens is reconciled whenever a consumer next reads the stream. a unit that would need a
          callback mid-execution is not frozen and must not be emitted.</Foot>
      </Page>
    );
  }

  /* ================= Verifications (§6) ================= */
  function VerificationsView() {
    const kinds = PF.verificationKinds;
    const pass = kinds.filter(k => k.verdict === 'pass').length;
    return (
      <Page max={1040}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
          <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>Verification — the conformance bar</h2>
          <Mono color="var(--slate-500)">the framework ships no verifications; it fixes what they must do</Mono>
          <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-400)' }}>
            {pass}/{kinds.length} kinds green</span>
        </div>

        {/* the anatomy */}
        <pre style={{ margin: '14px 0 0', background: 'var(--slate-950)', border: '1px solid var(--slate-700)',
          borderRadius: 7, padding: '12px 15px', fontFamily: 'var(--font-mono)', fontSize: 11.5, lineHeight: 1.65,
          color: 'var(--slate-300)', overflowX: 'auto', whiteSpace: 'pre' }}>{
`verify(artifact, oracle, criteria) -> Verdict { pass | fail, findings[] }`}</pre>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 7, lineHeight: 1.55, maxWidth: 820 }}>
          inputs frozen and declared · the oracle is <span style={{ color: 'var(--slate-300)' }}>derived from the spec, never
          authored in the check</span> · a finding per criterion, never a bare boolean · the verdict is the conjunction, and it gates.
        </div>

        {/* the kinds ledger */}
        <div style={{ marginTop: 16, border: '1px solid var(--slate-700)', borderRadius: 8, overflow: 'hidden' }}>
          <div style={{ display: 'grid', gridTemplateColumns: '190px 1fr 130px 90px', background: 'var(--slate-800)',
            borderBottom: '1px solid var(--slate-700)' }}>
            {['kind', 'oracle — derived from', 'runs', 'verdict'].map(h => (
              <div key={h} style={{ padding: '8px 12px', fontFamily: 'var(--font-mono)', fontSize: 9, fontWeight: 600,
                letterSpacing: '.12em', textTransform: 'uppercase', color: 'var(--slate-500)' }}>{h}</div>
            ))}
          </div>
          {kinds.map((k, i) => (
            <div key={k.kind} style={{ borderTop: i ? '1px dashed var(--slate-800)' : 'none' }}>
              <div style={{ display: 'grid', gridTemplateColumns: '190px 1fr 130px 90px', alignItems: 'baseline' }}>
                <div style={{ padding: '9px 12px' }}><Mono size={10.5} color="var(--slate-100)">{k.kind}</Mono></div>
                <div style={{ padding: '9px 12px' }}><Mono size={9.5} color="var(--slate-400)">{k.oracle}</Mono></div>
                <div style={{ padding: '9px 12px' }}><Mono size={9.5} color={k.when.startsWith('continuous') ? 'var(--em-event)' : 'var(--slate-500)'}>{k.when}</Mono></div>
                <div style={{ padding: '9px 12px' }}><Verdict v={k.verdict} /></div>
              </div>
              {k.finding && (
                <div style={{ padding: '0 12px 9px', fontFamily: 'var(--font-mono)', fontSize: 9.5,
                  color: k.verdict === 'pass' || k.verdict === 'n/a' ? 'var(--slate-500)' : V[k.verdict].c }}>
                  ↳ {k.finding}</div>
              )}
            </div>
          ))}
        </div>

        <Foot>the coherence bar: when realisation is split across work units, the parts must agree at least as well
          as a single unsplit author would achieve from shared context — otherwise the split is not worth it.
          every check cites what it protects; a green verdict is what lets a rationale-trace claim survive (§5).</Foot>
      </Page>
    );
  }

  Object.assign(window, { WorkUnitsView, VerificationsView });
})();
