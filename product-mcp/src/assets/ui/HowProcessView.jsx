/* global React, PF */
/* The How process — How is binding resolution. The What declares placeholders;
   the How resolves them in dependency order, each binding gated. The single
   visual: a worklist draining to zero. DONE = worklist empty ∧ all gates green. */
(function () {
  const HOW = 'var(--em-event)';
  const G = { green: 'var(--conf-verified)', amber: 'var(--em-event)', red: 'var(--error, #dc2626)', waived: 'var(--em-event)' };

  function Mono({ children, color, size }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: size || 10.5, color: color || 'var(--slate-300)' }}>{children}</span>;
  }
  function Dot({ state }) {
    return <span style={{ width: 8, height: 8, borderRadius: '50%', background: G[state] || 'var(--slate-600)', flex: 'none', display: 'inline-block' }} />;
  }

  /* ---- derive the worklist from the model, exactly as the tool would ---- */
  function worklist() {
    const h1 = PF.how.principles.map(p => {
      const loc = PF.principleLocated(p.id);
      return { label: p.id, note: loc ? 'located \u00b7 ' + loc.via : 'no rule locates it \u2014 unenforceable prose', state: loc ? 'green' : 'red' };
    });
    const h2 = PF.how.layout.map(r => ({
      label: r.id, note: r.verdict === 'pass' ? 'rule holds against the tree' : r.finding,
      state: r.verdict === 'pass' ? 'green' : 'amber',
    }));
    const h3 = [
      ...PF.manifest.reification.map(r => ({ label: `${r.aio} \u00d7 ${r.when}`, note: '\u2192 ' + r.cio, state: 'green' })),
      ...PF.manifest.unreifiable.map(u => ({ label: `${u.aio} \u00d7 ${u.cls}`, note: 'waived \u2014 ' + u.rationale, state: 'waived' })),
    ];
    const h4 = Object.keys(PF.contract.content_store).map(k => ({
      label: k, note: 'en \u2713 \u00b7 es \u2713 \u00b7 role: ' + PF.contract.content_store[k].role, state: 'green',
    }));
    const h5 = PF.howProcess.surfaces.map(s => ({
      label: s.id, note: s.drift ? 'DRIFT \u2014 hand-edited since generation' : 'generated ' + s.generated + ' \u00b7 no drift', state: s.drift ? 'red' : 'green',
    }));
    const h6 = PF.how.deployableUnits.map(du => ({
      label: du.id + ' · ' + du.env, note: 'realises ' + du.system + ' · ' + du.identity + (du.frozen ? ' · frozen ✓' : ''), state: 'green',
    }));
    return { H1: h1, H2: h2, H3: h3, H4: h4, H5: h5, H6: h6 };
  }
  const gateState = (rows) => rows.some(r => r.state === 'red') ? 'red' : rows.some(r => r.state === 'amber') ? 'amber' : 'green';

  function HowProcessView({ onOpen }) {
    const wl = React.useMemo(worklist, []);
    const all = Object.values(wl).flat();
    const resolved = all.filter(r => r.state !== 'red').length;
    const gates = PF.howProcess.steps.map(s => ({ ...s, state: gateState(wl[s.id] || []) }));
    const done = resolved === all.length && gates.every(g => g.state === 'green');
    const [open, setOpen] = React.useState('H2');

    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto', background: 'var(--slate-900)' }}>
        <div style={{ maxWidth: 1080, margin: '0 auto', padding: '20px 26px 40px' }}>

          {/* header: the one idea + the computed progress */}
          <div>
            <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>The How is binding resolution</h2>
            <div style={{ marginTop: 4 }}>
              <Mono color="var(--slate-500)">the What declares placeholders; the How resolves them, in dependency order, each binding gated</Mono>
            </div>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginTop: 12 }}>
            <div style={{ flex: 1, height: 7, borderRadius: 7, background: 'var(--slate-700)', overflow: 'hidden' }}>
              <div style={{ height: '100%', width: (resolved / all.length * 100) + '%', background: HOW }} />
            </div>
            <Mono size={12} color="var(--slate-100)">{resolved} / {all.length} bindings resolved</Mono>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10.5, fontWeight: 600,
              color: done ? G.green : HOW }}>{done ? 'DONE \u2713' : 'NOT DONE \u2014 gates open'}</span>
          </div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 6, lineHeight: 1.55 }}>
            the worklist is extracted from the settled What — (AIO × context) pairs, (key × locale) pairs, systems,
            principles, surfaces — never authored. done was never a judgment call.
          </div>

          {/* the pipeline */}
          <div style={{ display: 'flex', alignItems: 'stretch', gap: 8, marginTop: 18, flexWrap: 'wrap' }}>
            <PipeChip label="enumerate" sub={all.length + ' bindings'} state={null} />
            <Arrow />
            {gates.filter(s => !s.parallel).slice(0, 2).map((s, i) => (
              <React.Fragment key={s.id}>
                {i > 0 && <Arrow />}
                <PipeChip label={s.id} sub={s.name} state={s.state} onClick={() => setOpen(s.id)} />
              </React.Fragment>
            ))}
            <Arrow />
            <div style={{ display: 'flex', gap: 6, border: '1.5px dashed var(--slate-600)', borderRadius: 8, padding: 6 }}>
              {gates.filter(s => s.parallel).map(s => (
                <PipeChip key={s.id} label={s.id} sub={s.name} state={s.state} onClick={() => setOpen(s.id)} />
              ))}
            </div>
            <Arrow />
            <PipeChip label="H6" sub="Deployment identity" state={gates.find(s => s.id === 'H6').state} onClick={() => setOpen('H6')} />
            <Arrow />
            <PipeChip label="done" sub={done ? 'all gates green' : 'gates open'} state={done ? 'green' : 'amber'} />
          </div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 7 }}>
            enforced dependencies: H1 → H2 (locate what you declared) · H2 → H3, H4 (the tables need a home) ·
            H3–H5 run parallel · H5 depends only on the settled domain model
          </div>

          {/* the draining worklist, per step */}
          <div style={{ marginTop: 18, display: 'grid', gap: 10 }}>
            {gates.map(s => {
              const rows = wl[s.id] || [];
              const isOpen = open === s.id;
              const ok = rows.filter(r => r.state !== 'red').length;
              return (
                <div key={s.id} style={{ background: 'var(--slate-800)', borderRadius: 9, overflow: 'hidden',
                  border: `1.5px solid ${isOpen ? 'var(--slate-500)' : 'var(--slate-700)'}` }}>
                  <div onClick={() => setOpen(isOpen ? null : s.id)} style={{ display: 'flex', alignItems: 'flex-start', gap: 11,
                    padding: '11px 15px', cursor: 'pointer', flexWrap: 'wrap' }}>
                    <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 700, fontSize: 13, color: HOW, width: 28, flex: 'none' }}>{s.id}</span>
                    <span style={{ flex: '1 1 auto', minWidth: 0, display: 'flex', alignItems: 'baseline', gap: 9, flexWrap: 'wrap' }}>
                      <span style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 14, color: 'var(--slate-100)' }}>{s.name}</span>
                      <Mono size={9} color="var(--slate-500)">{s.spec}{s.dep ? ' \u00b7 needs ' + s.dep : ''}</Mono>
                    </span>
                    <span style={{ display: 'flex', alignItems: 'center', gap: 9, flex: 'none' }}>
                      <Mono size={9.5} color="var(--slate-400)">{ok}/{rows.length}</Mono>
                      <Dot state={s.state} />
                      <Mono size={9.5} color={G[s.state]}>{s.state === 'green' ? 'gate green' : s.state === 'amber' ? 'gate open' : 'gate red'}</Mono>
                      {s.view && (
                        <button onClick={(e) => { e.stopPropagation(); onOpen(s.view); }} style={{
                          cursor: 'pointer', background: 'transparent', border: '1px solid var(--slate-600)',
                          color: 'var(--blue-400)', borderRadius: 4, padding: '2px 9px',
                          fontFamily: 'var(--font-mono)', fontSize: 9.5 }}>open {'\u2192'}</button>
                      )}
                    </span>
                  </div>
                  {isOpen && (
                    <div style={{ borderTop: '1px dashed var(--slate-700)', padding: '9px 15px 12px' }}>
                      <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-400)', fontStyle: 'italic', marginBottom: 4 }}>
                        {s.question}</div>
                      <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: G[s.state], marginBottom: 9 }}>
                        gate: {s.gate}</div>
                      <div style={{ display: 'grid', gap: 4 }}>
                        {rows.map((r, i) => (
                          <div key={i} style={{ display: 'flex', gap: 10, alignItems: 'baseline' }}>
                            <Dot state={r.state} />
                            <Mono size={10.5} color="var(--slate-100)">{r.label}</Mono>
                            <Mono size={9.5} color={r.state === 'red' ? G.red : 'var(--slate-500)'}>{r.note}</Mono>
                          </div>
                        ))}
                      </div>
                      {s.id === 'H6' && (
                        <div style={{ marginTop: 10, fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)' }}>
                          a DeployableUnit is the DORA unit — what deployment frequency and lead time count per. staging and
                          production are two units of the same system on the same blueprint; only the How column changes.</div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          <div style={{ marginTop: 18, fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
            borderTop: '1px dashed var(--slate-700)', paddingTop: 11, lineHeight: 1.55, maxWidth: 840 }}>
            a worklist, not a wizard: the What tells you the work, the order tells you the sequence, the gates tell
            you when each piece is truly done. the tool refuses to mark a later step done while an earlier gate is red.
          </div>
        </div>
      </div>
    );
  }

  function PipeChip({ label, sub, state, onClick }) {
    return (
      <div onClick={onClick} style={{ display: 'flex', flexDirection: 'column', justifyContent: 'center', gap: 2,
        background: 'var(--slate-800)', border: '1.5px solid var(--slate-600)', borderRadius: 7,
        padding: '7px 12px', cursor: onClick ? 'pointer' : 'default', minWidth: 74 }}>
        <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.1em',
            textTransform: 'uppercase', color: 'var(--slate-100)' }}>{label}</span>
          {state && <Dot state={state} />}
        </span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: 'var(--slate-500)' }}>{sub}</span>
      </div>
    );
  }
  function Arrow() {
    return <span style={{ alignSelf: 'center', fontFamily: 'var(--font-mono)', fontSize: 13, color: 'var(--slate-500)' }}>{'\u2192'}</span>;
  }

  Object.assign(window, { HowProcessView });
})();
