/* global React, PF, PFUI */
/* Data (§3.1 / §13) — reference data is What; production data is an oracle.
   Left: the constitutive sets behaviour depends on. Right: bound datasets with
   shapes asserted continuously, the data-divergence trend, and bidirectional
   triage — a failure reads both ways: the data is wrong, or the spec went stale. */
(function () {
  const { ConfDot } = window.PFUI;
  const REF = 'var(--kind-entity)';      // domain-structure orange
  const ORACLE = 'var(--em-event)';      // amber — a signal, not a spec

  function SecLabel({ children, color }) {
    return <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
      textTransform: 'uppercase', color: color || 'var(--slate-500)' }}>{children}</div>;
  }
  function Mono({ children, color, size }) {
    return <span style={{ fontFamily: 'var(--font-mono)', fontSize: size || 10.5, color: color || 'var(--slate-300)' }}>{children}</span>;
  }

  function Sparkline({ trend, rising, w = 120, h = 34 }) {
    const max = Math.max(...trend) * 1.2;
    const pts = trend.map((v, i) => `${(i / (trend.length - 1)) * (w - 6) + 3},${h - 4 - (v / max) * (h - 10)}`).join(' ');
    const col = rising ? ORACLE : 'var(--conf-verified)';
    return (
      <svg width={w} height={h} style={{ display: 'block' }}>
        <polyline points={pts} fill="none" stroke={col} strokeWidth="1.6" />
        {trend.map((v, i) => {
          const [x, y] = pts.split(' ')[i].split(',');
          return <circle key={i} cx={x} cy={y} r={i === trend.length - 1 ? 2.6 : 1.4} fill={col} />;
        })}
      </svg>
    );
  }

  function RefCard({ r, selected, onClick, showConf }) {
    return (
      <div onClick={onClick} style={{
        cursor: 'pointer', background: 'var(--slate-800)', borderRadius: 8, padding: '12px 14px',
        border: `1.5px solid ${selected ? 'var(--blue-400)' : 'var(--slate-700)'}`,
        boxShadow: selected ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)' : 'var(--shadow-graph)',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.12em',
            textTransform: 'uppercase', color: REF }}>reference data</span>
          <Mono color="var(--slate-500)" size={9}>{r.id}</Mono>
          {showConf && <span style={{ marginLeft: 'auto' }}><ConfDot level={r.conformance} size={8} /></span>}
        </div>
        <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 14.5, color: 'var(--slate-100)', marginTop: 3 }}>{r.name}</div>
        <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap', marginTop: 8 }}>
          {r.values.map(v => (
            <span key={v} style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-200)',
              background: `color-mix(in srgb, ${REF} 12%, var(--slate-900))`, border: `1px solid ${REF}`,
              borderRadius: 4, padding: '2px 8px' }}>{v}</span>
          ))}
        </div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-400)', marginTop: 9 }}>
          reference_data_for {'\u2192'} <span style={{ color: 'var(--slate-200)' }}>{r.referenceFor}</span>
        </div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 4, lineHeight: 1.5 }}>{r.note}</div>
      </div>
    );
  }

  function DatasetCard({ d, selected, onClick }) {
    return (
      <div onClick={onClick} style={{
        cursor: 'pointer', background: 'var(--slate-800)', borderRadius: 8, padding: '12px 14px',
        border: `1.5px dashed ${selected ? 'var(--blue-400)' : 'var(--slate-600)'}`,
        boxShadow: selected ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)' : 'none',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.12em',
            textTransform: 'uppercase', color: ORACLE }}>oracle · populated</span>
          <Mono color="var(--slate-500)" size={9}>{d.id}</Mono>
        </div>
        <div style={{ display: 'flex', alignItems: 'flex-start', gap: 14, marginTop: 3 }}>
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 14.5, color: 'var(--slate-100)' }}>{d.name}</div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-400)', marginTop: 5, lineHeight: 1.6 }}>
              {d.rows} rows · conforms_to_shape {'→'} <span style={{ color: 'var(--slate-200)' }}>{d.shape}</span><br />
              asserted <span style={{ color: 'var(--slate-200)' }}>{d.assertion}</span> — a standing signal, not a one-time check
            </div>
          </div>
          <div style={{ flex: 'none', textAlign: 'right' }}>
            <Sparkline trend={d.trend} rising={d.rising} />
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10.5, marginTop: 3,
              color: d.rising ? ORACLE : 'var(--conf-verified)' }}>
              divergence {d.rate}%{d.rising ? ' \u2197' : ' \u2192'}</div>
          </div>
        </div>
      </div>
    );
  }

  function DataView({ selected, onSelect, showConf }) {
    return (
      <div style={{ position: 'absolute', inset: 0, overflow: 'auto', background: 'var(--slate-900)' }}>
        <div style={{ maxWidth: 1150, margin: '0 auto', padding: '20px 26px 40px' }}>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1.15fr', gap: 22, alignItems: 'start' }}>

            {/* reference data — the What side */}
            <div>
              <SecLabel color={REF}>reference data — constitutive · part of the What</SecLabel>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', margin: '7px 0 12px', lineHeight: 1.55 }}>
                behaviour is undefined without these sets; events and invariants reference them (§3.1)
              </div>
              <div style={{ display: 'grid', gap: 11 }}>
                {PF.refData.map(r => (
                  <RefCard key={r.id} r={r} selected={selected === r.id} onClick={() => onSelect(r.id)} showConf={showConf} />
                ))}
              </div>
            </div>

            {/* production data — the oracle side */}
            <div>
              <SecLabel color={ORACLE}>production data — the oracle · not the What</SecLabel>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)', margin: '7px 0 12px', lineHeight: 1.55 }}>
                populated facts, validated against declared shapes — authored once, asserted continuously (§13)
              </div>
              <div style={{ display: 'grid', gap: 11 }}>
                {PF.oracle.datasets.map(d => (
                  <DatasetCard key={d.id} d={d} selected={selected === d.id} onClick={() => onSelect(d.id)} />
                ))}
              </div>

              {/* triage ledger */}
              <div style={{ marginTop: 18 }}>
                <SecLabel>bidirectional triage — a failure reads both ways</SecLabel>
                <div style={{ marginTop: 9, border: '1px solid var(--slate-700)', borderRadius: 7, overflow: 'hidden' }}>
                  {PF.oracle.violations.map((v, i) => (
                    <div key={v.id} onClick={() => onSelect(v.id)} style={{
                      cursor: 'pointer', padding: '10px 13px', display: 'flex', gap: 12, alignItems: 'baseline',
                      borderTop: i ? '1px dashed var(--slate-700)' : 'none',
                      background: selected === v.id ? 'var(--slate-800)' : 'transparent',
                    }}>
                      <span style={{ flex: 'none', fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600,
                        letterSpacing: '.08em', textTransform: 'uppercase', borderRadius: 3, padding: '2px 7px',
                        color: '#0b1120', background: v.triage === 'spec drift' ? ORACLE : 'var(--blue-400)' }}>{v.triage}</span>
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <Mono size={11}>{v.shape}</Mono>
                        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 2, lineHeight: 1.5 }}>{v.note}</div>
                      </div>
                      <Mono color="var(--slate-400)" size={10}>{v.count} rows</Mono>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          <div style={{ marginTop: 22, fontFamily: 'var(--font-mono)', fontSize: 10.5, color: 'var(--slate-500)',
            borderTop: '1px dashed var(--slate-700)', paddingTop: 12, lineHeight: 1.55, maxWidth: 820 }}>
            the constitutive/populated test: if changing a datum changes what the system means, it is reference data
            and belongs in the What. the 4,200 products do not; the set of valid payment methods does.
          </div>
        </div>
      </div>
    );
  }

  Object.assign(window, { DataView });
})();
