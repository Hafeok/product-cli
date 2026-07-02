/* global React, PF */
/* AIO catalog (§3.2.2) — the core vocabulary of abstract interaction.
   Each AIO: what it means, what it's typed over, the obligations it carries,
   and how it reifies per context of use. Meaning, not widgets. */
(function () {
  const ACTION = 'var(--em-command)', DISPLAY = 'var(--em-view)', NAVC = 'var(--em-trigger)';
  const groupOf = (id) =>
    id.startsWith('display') ? 'display' : id === 'navigate' ? 'navigate' : 'action / input';
  const colorOf = (id) =>
    id.startsWith('display') ? DISPLAY : id === 'navigate' ? NAVC : ACTION;

  const VT_COLOR = { machine: 'var(--conf-verified)', assisted: 'var(--em-event)', manual: 'var(--em-trigger)' };

  function AIOCard({ a, selected, onClick }) {
    const col = colorOf(a.id);
    const usage = PF.aioUsage(a.id);
    return (
      <div onClick={onClick} style={{
        cursor: 'pointer', boxSizing: 'border-box', background: 'var(--slate-800)',
        border: `1.5px solid ${selected ? 'var(--blue-400)' : 'var(--slate-700)'}`, borderRadius: 8,
        padding: '13px 15px', boxShadow: selected ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)' : 'var(--shadow-graph)',
        display: 'flex', flexDirection: 'column', gap: 9,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ width: 8, height: 8, borderRadius: 2, background: col, flex: 'none' }} />
          <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 700, fontSize: 14, color: 'var(--slate-100)' }}>{a.id}</span>
          <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 8.5, letterSpacing: '.1em',
            textTransform: 'uppercase', color: col }}>{groupOf(a.id)}</span>
        </div>
        <div style={{ fontFamily: 'var(--font-sans)', fontSize: 13, color: 'var(--slate-200)' }}>{a.means}</div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-400)' }}>
          typed over <span style={{ color: 'var(--slate-200)' }}>{a.typedOver}</span>
        </div>

        {/* reification per context — the point of the abstraction */}
        <div style={{ borderTop: '1px dashed var(--slate-700)', paddingTop: 9, display: 'grid', gap: 5 }}>
          {[['phone', a.reify.phone], ['desktop', a.reify.desktop], ['tui', a.reify.tui]].map(([ctx, r]) => (
            <div key={ctx} style={{ display: 'flex', gap: 9, alignItems: 'baseline' }}>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, letterSpacing: '.08em', color: 'var(--slate-500)',
                width: 52, flex: 'none', textTransform: 'uppercase' }}>{ctx}</span>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-300)' }}>{r}</span>
            </div>
          ))}
        </div>

        <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap', alignItems: 'center' }}>
          {a.wcag.map(c => {
            const w = PF.wcag[c] || {};
            return (
              <span key={c} title={w.name} style={{ fontFamily: 'var(--font-mono)', fontSize: 9,
                color: VT_COLOR[w.vtype] || 'var(--slate-400)', border: '1px solid var(--slate-700)',
                borderRadius: 3, padding: '1px 6px' }}>{c}</span>
            );
          })}
          {usage.length > 0 && (
            <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)' }}>
              used by {usage.length} screen{usage.length > 1 ? 's' : ''}</span>
          )}
        </div>
      </div>
    );
  }

  function AIOCatalogView({ selected, onSelect }) {
    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto', background: 'var(--slate-900)' }}>
        <div style={{ maxWidth: 1240, margin: '0 auto', padding: '20px 26px 40px' }}>
          <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, flexWrap: 'wrap' }}>
            <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 19, color: 'var(--slate-100)' }}>
              Abstract Interaction Objects</h2>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)' }}>
              the base set is normative; the set is extensible — an AIO is meaning, not a widget
            </span>
          </div>

          {/* a11y legend */}
          <div style={{ display: 'flex', gap: 16, marginTop: 10, fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)' }}>
            {Object.entries(VT_COLOR).map(([t, c]) => (
              <span key={t} style={{ display: 'inline-flex', alignItems: 'center', gap: 5 }}>
                <span style={{ width: 7, height: 7, borderRadius: '50%', background: c }} />a11y {t}
              </span>
            ))}
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(340px, 1fr))', gap: 13, marginTop: 16 }}>
            {PF.aios.map(a => (
              <AIOCard key={a.id} a={a} selected={selected === a.id} onClick={() => onSelect(a.id)} />
            ))}
          </div>

          <div style={{ marginTop: 20, fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
            borderTop: '1px dashed var(--slate-700)', paddingTop: 12, lineHeight: 1.55, maxWidth: 760 }}>
            a single-select over three options on a tablet is well served by a segmented control; the same
            single-select over forty options on a phone by a searchable list. the choice is a real UX decision —
            about realisation in a context (§4.5), never about what the step means.
          </div>
        </div>
      </div>
    );
  }

  Object.assign(window, { AIOCatalogView });
})();
