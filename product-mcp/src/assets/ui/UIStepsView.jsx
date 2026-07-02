/* global React, PF, PFUI */
/* UI step spec sheet (§3.2.1) — one step's two layers: the BUILDABLE CORE
   (intent / information shown / actions / transitions, all references into
   the model) and MODELLED MEANING (emphasis, state meanings, WCAG
   obligations, content keys). Intent is rendered as what it is: debt. */
(function () {
  const { ConfDot } = window.PFUI;

  const AIO_COLOR = { 'display-collection': 'var(--em-view)', 'display-value': 'var(--em-view)',
    'trigger-action': 'var(--em-command)', 'single-select': 'var(--em-command)',
    'text-entry': 'var(--em-command)', navigate: 'var(--em-trigger)' };
  const VTYPE = {
    machine: { c: 'var(--conf-verified)', t: 'machine' },
    assisted: { c: 'var(--em-event)', t: 'assisted' },
    manual: { c: 'var(--em-trigger)', t: 'manual' },
  };

  function Label({ children }) {
    return <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, fontWeight: 600, letterSpacing: '.14em',
      textTransform: 'uppercase', color: 'var(--slate-500)' }}>{children}</div>;
  }
  function Mono({ children, color }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10.5, color: color || 'var(--slate-300)' }}>{children}</span>;
  }
  function Chip({ children, color = 'var(--slate-600)', text = 'var(--slate-300)' }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: text,
      border: `1px solid ${color}`, borderRadius: 3, padding: '1px 7px' }}>{children}</span>;
  }
  function Panel({ title, accent, children }) {
    return (
      <section style={{ background: 'var(--slate-800)', border: '1px solid var(--slate-700)', borderRadius: 8, padding: '13px 15px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
          <span style={{ width: 8, height: 8, borderRadius: 2, background: accent, flex: 'none' }} />
          <Label>{title}</Label>
        </div>
        {children}
      </section>
    );
  }

  function StepPicker({ active, onPick }) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '10px 18px', flexWrap: 'wrap',
        borderBottom: '1px solid var(--slate-800)', background: 'var(--slate-900)' }}>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, letterSpacing: '.12em', textTransform: 'uppercase',
          color: 'var(--slate-500)' }}>ui step</span>
        <div style={{ display: 'flex', gap: 7, flexWrap: 'wrap' }}>
          {PF.contract.screens.map(s => (
            <button key={s.id} onClick={() => onPick(s.id)} style={{
              cursor: 'pointer', background: s.id === active ? 'var(--slate-700)' : 'transparent',
              color: s.id === active ? 'var(--slate-100)' : 'var(--slate-400)',
              border: `1px solid ${s.id === active ? 'var(--slate-500)' : 'var(--slate-700)'}`, borderRadius: 6,
              fontFamily: 'var(--font-sans)', fontSize: 12.5, fontWeight: 500, padding: '5px 11px',
            }}>{s.name}</button>
          ))}
        </div>
      </div>
    );
  }

  function UIStepsView({ stepId, setStepId, onPreview }) {
    const s = PF.screen(stepId) || PF.contract.screens[0];
    const spec = PF.stepSpecs[s.id] || {};
    const displays = s.elements.filter(e => e.aio.startsWith('display'));
    const actions = s.elements.filter(e => !e.aio.startsWith('display'));
    const allWcag = Object.keys(spec.inheritedWcag || {});

    return (
      <div style={{ position: 'relative', width: '100%', height: '100%', display: 'flex', flexDirection: 'column' }}>
        <StepPicker active={s.id} onPick={setStepId} />
        <div style={{ position: 'absolute', inset: 0, top: 53, overflow: 'auto', background: 'var(--slate-900)' }}>
          <div style={{ maxWidth: 1060, margin: '0 auto', padding: '20px 26px 40px' }}>

            {/* header */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.14em',
                textTransform: 'uppercase', color: 'var(--slate-400)' }}>ui step</span>
              <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 20, color: 'var(--slate-100)' }}>{s.name}</h2>
              <Mono color="var(--slate-500)">{s.id}</Mono>
              <button onClick={() => onPreview(s.id)} style={{
                marginLeft: 'auto', cursor: 'pointer', background: 'var(--blue-600)', border: '1px solid var(--blue-500)',
                color: '#fff', borderRadius: 5, padding: '6px 13px', fontFamily: 'var(--font-mono)', fontSize: 11 }}>
                preview screen {'\u2192'}</button>
            </div>

            {/* intent = debt */}
            <div style={{ marginTop: 14, border: '1.5px dashed var(--em-event)', borderRadius: 7,
              background: 'color-mix(in srgb, var(--em-event) 7%, transparent)', padding: '10px 14px',
              display: 'flex', gap: 12, alignItems: 'baseline', flexWrap: 'wrap' }}>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, fontWeight: 600, letterSpacing: '.12em',
                textTransform: 'uppercase', color: 'var(--em-event)', flex: 'none' }}>intent · debt</span>
              <span style={{ fontFamily: 'var(--font-serif, var(--font-sans))', fontStyle: 'italic', fontSize: 14,
                color: 'var(--slate-200)' }}>“{s.intent}”</span>
              <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9.5,
                color: spec.intentReliance ? 'var(--em-event)' : 'var(--conf-verified)' }}>
                intent-reliance: {spec.intentReliance || 0}{spec.intentNote ? ' \u00b7 ' + spec.intentNote : ''}</span>
            </div>

            {/* two layers */}
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 14, marginTop: 16, alignItems: 'start' }}>
              {/* buildable core */}
              <div style={{ display: 'grid', gap: 12 }}>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
                  textTransform: 'uppercase', color: 'var(--blue-400)' }}>the buildable core — references, not artifacts</div>

                <Panel title="information shown · projects" accent="var(--em-view)">
                  <div style={{ display: 'flex', alignItems: 'baseline', gap: 8, marginBottom: 8 }}>
                    <Chip color="var(--em-view)" text="var(--em-view)">{s.projection}</Chip>
                    <Mono color="var(--slate-500)">the read model this step surfaces</Mono>
                  </div>
                  <div style={{ display: 'grid', gap: 7 }}>
                    {displays.map((e, i) => (
                      <div key={i} style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                        <Chip color={AIO_COLOR[e.aio]} text={AIO_COLOR[e.aio]}>{e.aio}</Chip>
                        <Mono>{e.role}</Mono>
                        <Mono color="var(--slate-500)">binds {s.projection}.{e.binds}</Mono>
                      </div>
                    ))}
                  </div>
                </Panel>

                <Panel title="actions available · commands" accent="var(--em-command)">
                  <div style={{ display: 'grid', gap: 7 }}>
                    {actions.map((e, i) => (
                      <div key={i} style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                        <Chip color={AIO_COLOR[e.aio]} text={AIO_COLOR[e.aio]}>{e.aio}</Chip>
                        <Mono>{e.role}</Mono>
                        <Mono color="var(--em-command)">issues {e.issues}</Mono>
                      </div>
                    ))}
                    {!actions.length && <Mono color="var(--slate-500)">none — a read-only step</Mono>}
                  </div>
                </Panel>

                <Panel title="transitions · navigate edges" accent="var(--em-trigger)">
                  <div style={{ display: 'grid', gap: 7 }}>
                    {(spec.transitions || []).map((tr, i) => (
                      <div key={i} style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                        <Mono color="var(--slate-400)">on {tr.on}</Mono>
                        <Mono color="var(--em-trigger)">{'\u2192'} {tr.to}</Mono>
                        {tr.note && <Mono color="var(--slate-500)">({tr.note})</Mono>}
                      </div>
                    ))}
                    {!(spec.transitions || []).length && <Mono color="var(--slate-500)">terminal within its flow</Mono>}
                  </div>
                </Panel>
              </div>

              {/* modelled meaning */}
              <div style={{ display: 'grid', gap: 12 }}>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
                  textTransform: 'uppercase', color: 'var(--em-bridge)' }}>modelled meaning — shapes realisation, checked at the seam</div>

                <Panel title="emphasis" accent="var(--em-bridge)">
                  <Mono>{spec.emphasis || '—'}</Mono>
                </Panel>

                <Panel title="projection states · covering" accent="var(--em-event)">
                  <div style={{ display: 'grid', gap: 7 }}>
                    {s.state_space.map(st => {
                      const meaning = (s.state_meanings || {})[st];
                      const waiver = (s.state_waivers || {})[st];
                      const ckey = (s.state_content || {})[st];
                      return (
                        <div key={st} style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                          <Chip color={st === 'failed' ? 'var(--error, #dc2626)' : 'var(--slate-600)'}
                            text={st === 'failed' ? 'var(--error, #dc2626)' : 'var(--slate-300)'}>{st}</Chip>
                          {st === 'present'
                            ? <Mono color="var(--slate-500)">the projected value — rendered via the elements</Mono>
                            : <Mono>{meaning}</Mono>}
                          {waiver && <Mono color="var(--em-event)">waived: {waiver}</Mono>}
                          {ckey && <Mono color="var(--blue-400)">content: {ckey}</Mono>}
                        </div>
                      );
                    })}
                  </div>
                </Panel>

                <Panel title="accessibility · union, computed" accent="var(--conf-verified)">
                  <div style={{ display: 'grid', gap: 7 }}>
                    {allWcag.map(c => {
                      const w = PF.wcag[c] || {};
                      const vt = VTYPE[w.vtype] || VTYPE.manual;
                      return (
                        <div key={c} style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                          <Chip>{c} {w.name}</Chip>
                          <Mono color="var(--slate-500)">A{w.level === 'AA' ? 'A' : ''} · inherited from {spec.inheritedWcag[c]}</Mono>
                          <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9, color: vt.c }}>{vt.t}</span>
                        </div>
                      );
                    })}
                  </div>
                </Panel>

                <Panel title="content references · keys, not literals" accent="var(--blue-400)">
                  <div style={{ display: 'grid', gap: 7 }}>
                    {s.content && Object.entries(s.content).map(([role, key]) => (
                      <div key={key} style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                        <Chip color="var(--blue-400)" text="var(--blue-400)">{role}</Chip>
                        <Mono>{key}</Mono>
                        <Mono color="var(--slate-500)">{'→'} “{PF.resolveContent(key)}” (en)</Mono>
                      </div>
                    ))}
                    {s.state_content && Object.entries(s.state_content).map(([st, key]) => (
                      <div key={key} style={{ display: 'flex', gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
                        <Chip color="var(--blue-400)" text="var(--blue-400)">{st}-message</Chip>
                        <Mono>{key}</Mono>
                      </div>
                    ))}
                  </div>
                </Panel>
              </div>
            </div>

            <div style={{ marginTop: 18, fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
              borderTop: '1px dashed var(--slate-700)', paddingTop: 11, lineHeight: 1.55 }}>
              the step names the projection and the abstract interaction — never a control. “a ui step naming a
              dropdown” is a structural violation the seam verification rejects (§6.3), not a style lapse.
            </div>
          </div>
        </div>
      </div>
    );
  }

  Object.assign(window, { UIStepsView });
})();
