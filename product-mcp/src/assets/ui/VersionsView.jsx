/* global React, PF, PFUI */
/* Versions & direction (§7.3) — Delivery views.
   The What and the How carry independent semantic versions that reference each
   other; a target version is a declared future partition; direction is the
   computed gap. Two layouts (a tweak): a two-axis ladder, a target partition. */
(function () {
  const { EdgeLayer, ConfDot, FitCanvas } = window.PFUI;

  const WHAT = 'var(--blue-500)';   // What phase = blue
  const HOW = 'var(--em-event)';    // How phase = amber
  const BUILD = 'var(--em-view)';   // Build / done = green

  const BUMP = {
    major: { t: 'major', c: 'var(--slate-100)', b: 'var(--slate-500)' },
    minor: { t: 'minor', c: 'var(--slate-300)', b: 'var(--slate-600)' },
    patch: { t: 'patch', c: 'var(--slate-500)', b: 'var(--slate-700)' },
  };
  function BumpTag({ bump }) {
    const b = BUMP[bump] || BUMP.patch;
    return (
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.1em',
        textTransform: 'uppercase', color: b.c, border: `1px solid ${b.b}`, borderRadius: 3, padding: '1px 6px' }}>{b.t}</span>
    );
  }

  // ================= LADDER =================
  const L_W = 1160, L_H = 690;
  const WHAT_POS = { '2.0': 115, '1.1': 350, '1.0': 565 };
  const HOW_POS = { '1.1.2': 95, '1.1.1': 235, '1.1.0': 375, '1.0.0': 565 };
  const WCARD = { x: 340, w: 306, h: 132 };
  const HCARD = { x: 820, w: 262, h: 96 };

  function LadderLayout({ selected, onSelect }) {
    const V = PF.delivery.versions;
    const pos = {};
    V.what.forEach(w => { pos['w:' + w.v] = { x: WCARD.x, y: WHAT_POS[w.v], w: WCARD.w, h: WCARD.h }; });
    V.how.forEach(h => { pos['h:' + h.v] = { x: HCARD.x, y: HOW_POS[h.v], w: HCARD.w, h: HCARD.h }; });
    const edges = V.how.map(h => ({
      from: 'h:' + h.v, to: 'w:' + h.realises, stroke: 'var(--em-bridge)', width: 1.4, dash: '5 4',
      marker: 'mag', label: 'realises', labelColor: 'var(--em-bridge)',
    }));

    return (
      <FitCanvas width={L_W} height={L_H}>
        <ColCaption x={WCARD.x} label="What-version · the specified meaning" color={WHAT} />
        <ColCaption x={HCARD.x} label="How-version · the realisation" color={HOW} />

        {/* track spines */}
        <div style={{ position: 'absolute', left: WCARD.x - 0.5, top: 60, bottom: 40, width: 1.5,
          background: 'linear-gradient(var(--slate-700), var(--slate-800))', zIndex: 0 }} />
        <div style={{ position: 'absolute', left: HCARD.x - 0.5, top: 60, bottom: 40, width: 1.5,
          background: 'linear-gradient(var(--slate-700), var(--slate-800))', zIndex: 0 }} />

        <EdgeLayer edges={edges} pos={pos} width={L_W} height={L_H} showLabels={true} />

        {V.what.map(w => {
          const p = pos['w:' + w.v];
          return (
            <Node key={w.v} p={p}>
              <VersionCard sel={selected === 'w:' + w.v} onClick={() => onSelect('w:' + w.v)}
                accent={WHAT} axis="What" v={w} adds={w.adds} />
            </Node>
          );
        })}
        {V.how.map(h => {
          const p = pos['h:' + h.v];
          return (
            <Node key={h.v} p={p}>
              <VersionCard sel={selected === 'h:' + h.v} onClick={() => onSelect('h:' + h.v)}
                accent={HOW} axis="How" v={h} realises={h.realises} compact />
            </Node>
          );
        })}

        <div style={{ position: 'absolute', left: 24, bottom: 14, maxWidth: 620, fontFamily: 'var(--font-mono)',
          fontSize: 10.5, color: 'var(--slate-500)', lineHeight: 1.55, zIndex: 3 }}>
          each bump is <span style={{ color: 'var(--slate-300)' }}>derivable from what the diff touched</span> — behaviour (major),
          an added slice (minor), or realisation only (How-side). The How versions beneath a stable What.
        </div>
      </FitCanvas>
    );
  }

  function VersionCard({ v, axis, accent, adds, realises, sel, onClick, compact }) {
    const target = v.target, current = v.current;
    return (
      <div onClick={onClick} style={{
        cursor: 'pointer', boxSizing: 'border-box', width: '100%', height: '100%',
        background: 'var(--slate-800)',
        border: `1.5px ${target ? 'dashed' : 'solid'} ${sel ? 'var(--blue-400)' : (current ? accent : 'var(--slate-600)')}`,
        borderRadius: 8, padding: compact ? '9px 12px' : '11px 13px', overflow: 'hidden',
        boxShadow: sel ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)'
          : (current ? `0 0 0 1px color-mix(in srgb, ${accent} 40%, transparent)` : 'var(--shadow-graph)'),
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 700, fontSize: compact ? 14 : 17, color: accent }}>{axis} {v.v}</span>
          <BumpTag bump={v.bump} />
          {current && <Pill c={accent} t="current" />}
          {target && <Pill c="var(--slate-400)" t="target" />}
          {realises && <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 9,
            color: 'var(--em-bridge)' }}>{'\u2192'} What {realises}</span>}
        </div>
        <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 600, fontSize: compact ? 12.5 : 14,
          color: 'var(--slate-100)', marginTop: compact ? 2 : 4 }}>{v.name}</div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: compact ? 9.5 : 10, color: 'var(--slate-400)',
          marginTop: 3, lineHeight: 1.45 }}>{v.diff}</div>
        {adds && (
          <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap', marginTop: 8 }}>
            {adds.map(id => {
              const f = PF.feature(id);
              return (
                <span key={id} style={{ display: 'inline-flex', alignItems: 'center', gap: 5, fontFamily: 'var(--font-mono)',
                  fontSize: 9, color: 'var(--slate-300)', background: 'var(--slate-900)', border: '1px solid var(--slate-700)',
                  borderRadius: 3, padding: '1px 6px' }}>
                  <span style={{ width: 6, height: 6, borderRadius: '50%', background: PF.featureDone(f) ? BUILD : 'var(--slate-500)' }} />
                  {f ? f.name : id}
                </span>
              );
            })}
          </div>
        )}
      </div>
    );
  }
  function Pill({ c, t }) {
    return (
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.08em',
        color: '#0b1120', background: c, borderRadius: 3, padding: '1px 6px' }}>{t}</span>
    );
  }
  function ColCaption({ x, label, color }) {
    return (
      <div style={{ position: 'absolute', left: x - 220, top: 22, width: 440, textAlign: 'center',
        fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 600, letterSpacing: '.14em',
        textTransform: 'uppercase', color, zIndex: 3 }}>{label}</div>
    );
  }
  function Node({ p, children }) {
    return (
      <div style={{ position: 'absolute', left: p.x - p.w / 2, top: p.y - p.h / 2, width: p.w, height: p.h, zIndex: 2 }}>
        {children}
      </div>
    );
  }

  // ================= TARGET PARTITION =================
  function Gauge({ frac, size = 168 }) {
    const r = size / 2 - 14, c = 2 * Math.PI * r, cx = size / 2;
    return (
      <svg width={size} height={size} style={{ display: 'block' }}>
        <circle cx={cx} cy={cx} r={r} fill="none" stroke="var(--slate-700)" strokeWidth="12" />
        <circle cx={cx} cy={cx} r={r} fill="none" stroke={BUILD} strokeWidth="12" strokeLinecap="round"
          strokeDasharray={c} strokeDashoffset={c * (1 - frac)}
          transform={`rotate(-90 ${cx} ${cx})`} style={{ transition: 'stroke-dashoffset .5s var(--ease)' }} />
        <text x={cx} y={cx - 2} textAnchor="middle" style={{ fontFamily: 'var(--font-sans)', fontWeight: 700,
          fontSize: 34, fill: 'var(--slate-100)' }}>{Math.round(frac * 100)}%</text>
        <text x={cx} y={cx + 20} textAnchor="middle" style={{ fontFamily: 'var(--font-mono)', fontSize: 9,
          letterSpacing: '.1em', fill: 'var(--slate-400)', textTransform: 'uppercase' }}>toward goal</text>
      </svg>
    );
  }

  function PartitionLayout({ selected, onSelect, showConf }) {
    const target = PF.delivery.targets[0];
    const members = target.partition.map(id => PF.feature(id));
    const distance = members.filter(f => !PF.featureDone(f));
    const frac = 1 - distance.length / members.length;

    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto' }}>
        <div style={{ maxWidth: 1060, margin: '0 auto', padding: '22px 26px 40px' }}>
          {/* header */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.14em',
              textTransform: 'uppercase', color: WHAT }}>target version</span>
            <h2 style={{ margin: 0, fontFamily: 'var(--font-sans)', fontSize: 21, color: 'var(--slate-100)' }}>{target.name}</h2>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-400)',
              border: '1px dashed var(--slate-600)', borderRadius: 4, padding: '2px 8px' }}>declared future partition</span>
          </div>
          <p style={{ fontFamily: 'var(--font-mono)', fontSize: 11.5, color: 'var(--slate-400)', maxWidth: 720,
            lineHeight: 1.6, margin: '10px 0 0' }}>{target.note}</p>

          {/* gauge + partition */}
          <div style={{ display: 'flex', gap: 30, marginTop: 22, alignItems: 'flex-start', flexWrap: 'wrap' }}>
            <div style={{ flex: 'none', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12 }}>
              <Gauge frac={frac} />
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', textAlign: 'center' }}>
                {members.length - distance.length} of {members.length} slices done
              </div>
            </div>

            <div style={{ flex: 1, minWidth: 300 }}>
              <SecLabel>partition — the named set of feature-slices</SecLabel>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 9, marginTop: 10 }}>
                {members.map(f => {
                  const done = PF.featureDone(f);
                  return (
                    <div key={f.id} onClick={() => onSelect(f.id)} style={{
                      cursor: 'pointer', display: 'flex', alignItems: 'center', gap: 12, padding: '10px 13px',
                      background: 'var(--slate-800)', borderRadius: 7,
                      border: `1.5px ${done ? 'solid' : 'dashed'} ${selected === f.id ? 'var(--blue-400)' : (done ? BUILD : 'var(--slate-600)')}`,
                    }}>
                      <span style={{ color: done ? BUILD : 'var(--slate-500)', fontSize: 13, width: 14, textAlign: 'center' }}>{done ? '\u25CF' : '\u25CB'}</span>
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 600, fontSize: 14, color: 'var(--slate-100)' }}>{f.name}</div>
                        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 1 }}>{f.sub.split(' ')[0]} · {f.flows.join(', ')}</div>
                      </div>
                      {showConf && <ConfDot level={f.conformance} size={8} />}
                      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: done ? BUILD : 'var(--slate-500)',
                        letterSpacing: '.05em', flex: 'none' }}>{done ? 'done' : 'not done'}</span>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>

          {/* distance query */}
          <div style={{ marginTop: 26 }}>
            <SecLabel>direction is the computed gap — a query, not a status report</SecLabel>
            <pre style={{ margin: '10px 0 0', background: 'var(--slate-950)', border: '1px solid var(--slate-700)',
              borderRadius: 7, padding: '13px 16px', fontFamily: 'var(--font-mono)', fontSize: 11.5, lineHeight: 1.7,
              color: 'var(--slate-300)', overflowX: 'auto', whiteSpace: 'pre' }}>{
`distance(${target.name}) = { slice \u2208 partition : not feature_done(slice) }
             = { ${distance.map(f => f.id).join(', ')} }

progress = 1 \u2212 |distance| / |partition| = 1 \u2212 ${distance.length}/${members.length} = ${Math.round(frac * 100)}%`}</pre>
          </div>

          {/* what remains pulls in */}
          <div style={{ marginTop: 22 }}>
            <SecLabel>what each remaining slice pulls in — via its footprint (§7.1)</SecLabel>
            <div style={{ display: 'flex', gap: 12, marginTop: 10, flexWrap: 'wrap' }}>
              {distance.map(f => (
                <div key={f.id} style={{ flex: '1 1 260px', minWidth: 240, background: 'var(--slate-900)',
                  border: '1.5px dashed var(--slate-600)', borderRadius: 7, padding: '11px 13px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <span style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 13.5, color: 'var(--slate-100)' }}>{f.name}</span>
                    {showConf && <span style={{ marginLeft: 'auto' }}><ConfDot level={f.conformance} size={7} /></span>}
                  </div>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginTop: 8 }}>
                    {f.footprint.map(id => {
                      const c = PF.concept(id);
                      return (
                        <span key={id} style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-300)',
                          border: '1px solid var(--slate-700)', borderRadius: 3, padding: '1px 6px' }}>{c.label}</span>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div style={{ marginTop: 22, fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--slate-500)',
            borderTop: '1px dashed var(--slate-700)', paddingTop: 12, lineHeight: 1.6 }}>
            You cannot have a stale roadmap because there is no roadmap — there is a query over declared targets.
          </div>
        </div>
      </div>
    );
  }
  function SecLabel({ children }) {
    return (
      <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, fontWeight: 600, letterSpacing: '.14em',
        textTransform: 'uppercase', color: 'var(--slate-500)' }}>{children}</div>
    );
  }

  // ================= view =================
  function VersionsView({ layout, selected, onSelect, showConf }) {
    return (
      <div style={{ position: 'relative', width: '100%', height: '100%', background: 'var(--slate-900)' }}>
        {layout === 'partition'
          ? <PartitionLayout selected={selected} onSelect={onSelect} showConf={showConf} />
          : <LadderLayout selected={selected} onSelect={onSelect} />}
      </div>
    );
  }

  Object.assign(window, { VersionsView });
})();
