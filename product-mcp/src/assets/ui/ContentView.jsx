/* global React, PF */
/* Content (§4.6 / §12) — words live in a content store. The keyed catalog as a
   coverage matrix: every (content key, locale) pair the What references must
   resolve. Roles make copy checkable, not just present. One What, many words. */
(function () {
  const CONTENT = 'var(--blue-400)';
  const ROLE_COLOR = { heading: 'var(--blue-400)', 'empty-message': 'var(--em-event)', 'error-message': 'var(--error, #dc2626)' };
  const LOCALES = ['en', 'es'];

  function Mono({ children, color, size }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: size || 10.5, color: color || 'var(--slate-300)' }}>{children}</span>;
  }
  function SecLabel({ children }) {
    return <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
      textTransform: 'uppercase', color: 'var(--slate-500)' }}>{children}</div>;
  }

  function ContentView({ selected, onSelect }) {
    const store = PF.contract.content_store;
    const keys = Object.keys(store);
    const pairs = keys.length * LOCALES.length;
    const resolved = keys.reduce((n, k) => n + LOCALES.filter(l => store[k][l] != null).length, 0);

    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto', background: 'var(--slate-900)' }}>
        <div style={{ maxWidth: 1150, margin: '0 auto', padding: '20px 26px 40px' }}>

          {/* header + coverage verdict */}
          <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
            <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>Content store</h2>
            <Mono color="var(--slate-500)">wire-content · v0.1.0 · resolve(key, locale) {'→'} string</Mono>
            <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 11,
              color: resolved === pairs ? 'var(--conf-verified)' : 'var(--error, #dc2626)' }}>
              coverage {resolved}/{pairs} pairs resolve {resolved === pairs ? '✓' : ''}
            </span>
          </div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', marginTop: 6, lineHeight: 1.55, maxWidth: 760 }}>
            the What references content by key with a declared role — never a literal. locale is the store's
            context dimension, as context-of-use is reification's (§4.6).
          </div>

          {/* the coverage matrix */}
          <div style={{ marginTop: 16, border: '1px solid var(--slate-700)', borderRadius: 8, overflow: 'hidden' }}>
            <div style={{ display: 'grid', gridTemplateColumns: 'minmax(240px, 1.1fr) repeat(2, 1fr)',
              background: 'var(--slate-800)', borderBottom: '1px solid var(--slate-700)' }}>
              <HeadCell>content key · role</HeadCell>
              {LOCALES.map(l => <HeadCell key={l} center>{l}{l === PF.contract.locale ? ' · default' : ''}</HeadCell>)}
            </div>
            {keys.map((k, i) => {
              const e = store[k];
              const on = selected === k;
              return (
                <div key={k} onClick={() => onSelect(k)} style={{
                  display: 'grid', gridTemplateColumns: 'minmax(240px, 1.1fr) repeat(2, 1fr)', cursor: 'pointer',
                  borderTop: i ? '1px dashed var(--slate-800)' : 'none',
                  background: on ? 'var(--slate-800)' : 'transparent',
                  boxShadow: on ? 'inset 2px 0 0 var(--blue-400)' : 'none',
                }}>
                  <div style={{ padding: '10px 13px', minWidth: 0 }}>
                    <Mono size={11} color="var(--slate-100)">{k}</Mono>
                    <div style={{ marginTop: 4 }}>
                      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, letterSpacing: '.08em',
                        color: ROLE_COLOR[e.role] || 'var(--slate-400)',
                        border: `1px solid ${ROLE_COLOR[e.role] || 'var(--slate-600)'}`,
                        borderRadius: 3, padding: '1px 6px' }}>{e.role}</span>
                    </div>
                  </div>
                  {LOCALES.map(l => (
                    <div key={l} style={{ padding: '10px 13px', borderLeft: '1px dashed var(--slate-800)', minWidth: 0 }}>
                      {e[l] != null
                        ? <Mono size={10.5} color="var(--slate-300)">“{e[l]}”</Mono>
                        : <Mono size={10.5} color="var(--error, #dc2626)">⟨missing⟩</Mono>}
                    </div>
                  ))}
                </div>
              );
            })}
          </div>

          {/* role conformance */}
          <div style={{ marginTop: 20 }}>
            <SecLabel>roles make copy checkable — not just present (§12.1)</SecLabel>
            <div style={{ display: 'grid', gap: 6, marginTop: 9 }}>
              {[
                ['empty-message', 'must be non-empty and actionable — conveys the way forward', 'pass'],
                ['error-message', 'must say what went wrong and how to recover', 'pass'],
                ['heading', 'must state the page\u2019s purpose; length bounded by the reified slot', 'pass'],
              ].map(([role, rule, verdict]) => (
                <div key={role} style={{ display: 'flex', gap: 10, alignItems: 'baseline' }}>
                  <span style={{ color: 'var(--conf-verified)', fontSize: 11, width: 12, textAlign: 'center', flex: 'none' }}>●</span>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: ROLE_COLOR[role] || 'var(--slate-300)', width: 110, flex: 'none' }}>{role}</span>
                  <Mono size={10} color="var(--slate-400)">{rule}</Mono>
                  <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--conf-verified)' }}>{verdict}</span>
                </div>
              ))}
            </div>
          </div>

          <div style={{ marginTop: 20, fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
            borderTop: '1px dashed var(--slate-700)', paddingTop: 12, lineHeight: 1.55, maxWidth: 820 }}>
            the content store is to words what the design system is to components: a swappable provider behind a
            conformance profile. couple this store with locale=es and the Screens preview renders the same What in
            Spanish — switch it under Tweaks {'→'} UI.
          </div>
        </div>
      </div>
    );
  }

  function HeadCell({ children, center }) {
    return <div style={{ padding: '8px 13px', fontFamily: 'var(--font-mono)', fontSize: 9, fontWeight: 600,
      letterSpacing: '.12em', textTransform: 'uppercase', color: 'var(--slate-500)', textAlign: center ? 'center' : 'left' }}>{children}</div>;
  }

  Object.assign(window, { ContentView });
})();
