/* global React, PF, PFUI */
/* Systems map — product → systems & journeys. A graph: product owns systems &
   domains; systems reference domains; a journey crosses two systems via a
   Translation. */
(function () {
  const { useMemo } = React;
  const { EdgeLayer, ConfDot, FitCanvas } = window.PFUI;

  const CANVAS_W = 1120, CANVAS_H = 680;

  function layout() {
    // curated coordinates (x,y = node centers); arranged for orthogonal routing
    const pos = {
      acme: { x: 560, y: 92, w: 250, h: 92 },
      ordering: { x: 470, y: 300, w: 156, h: 72 },
      catalog: { x: 650, y: 300, w: 156, h: 72 },
      'acme-shop': { x: 230, y: 503, w: 300, h: 172 },
      'acme-admin': { x: 890, y: 503, w: 300, h: 172 },
    };
    return pos;
  }

  // ---- orthogonal (Manhattan) edge renderer ----
  function roundedPath(pts, r) {
    if (pts.length < 2) return '';
    let d = `M ${pts[0].x} ${pts[0].y}`;
    for (let i = 1; i < pts.length - 1; i++) {
      const p0 = pts[i - 1], p1 = pts[i], p2 = pts[i + 1];
      const v1 = { x: p1.x - p0.x, y: p1.y - p0.y }, v2 = { x: p2.x - p1.x, y: p2.y - p1.y };
      const l1 = Math.hypot(v1.x, v1.y) || 1, l2 = Math.hypot(v2.x, v2.y) || 1;
      const rr = Math.min(r, l1 / 2, l2 / 2);
      const a = { x: p1.x - v1.x / l1 * rr, y: p1.y - v1.y / l1 * rr };
      const b = { x: p1.x + v2.x / l2 * rr, y: p1.y + v2.y / l2 * rr };
      d += ` L ${a.x} ${a.y} Q ${p1.x} ${p1.y} ${b.x} ${b.y}`;
    }
    const last = pts[pts.length - 1];
    d += ` L ${last.x} ${last.y}`;
    return d;
  }

  function OrthoEdges({ edges, showLabels }) {
    return (
      <svg width={CANVAS_W} height={CANVAS_H} style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'visible' }}>
        <defs>
          <marker id="sm-arr" viewBox="0 0 10 10" refX="8.5" refY="5" markerWidth="6.5" markerHeight="6.5" orient="auto-start-reverse">
            <path d="M0,0 L10,5 L0,10 z" fill="var(--slate-500)" />
          </marker>
        </defs>
        {edges.map((e, i) => (
          <g key={i}>
            <path d={roundedPath(e.pts, 9)} fill="none" stroke={e.stroke} strokeWidth={e.width || 1.5}
              strokeDasharray={e.dash || 'none'} opacity={e.opacity == null ? 1 : e.opacity}
              markerEnd={e.arrow === false ? undefined : 'url(#sm-arr)'} />
            {showLabels && e.label && (
              <g>
                <rect x={e.lx - e.label.length * 3.1 - 5} y={e.ly - 8} rx="3"
                  width={e.label.length * 6.2 + 10} height={16} fill="var(--slate-900)" opacity="0.9" />
                <text x={e.lx} y={e.ly + 3.5} textAnchor="middle"
                  style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, letterSpacing: '.04em', fill: 'var(--slate-400)' }}>{e.label}</text>
              </g>
            )}
          </g>
        ))}
      </svg>
    );
  }

  function ProductNode({ p, selected, onClick, showConf }) {
    return (
      <div onClick={onClick} style={{
        cursor: 'pointer', boxSizing: 'border-box', width: '100%', height: '100%',
        background: 'var(--slate-800)', border: `1.5px solid ${selected ? 'var(--blue-400)' : 'var(--slate-600)'}`,
        borderRadius: 8, padding: '12px 14px', display: 'flex', gap: 12, alignItems: 'center',
        boxShadow: selected ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 30%, transparent)' : 'var(--shadow-graph)',
      }}>
        <img src="assets/logo-mark.svg" height="40" alt="" style={{ flex: 'none' }} />
        <div style={{ minWidth: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <span style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 17, color: 'var(--slate-100)' }}>{p.name}</span>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)' }}>product · {p.id}</span>
            {showConf && <ConfDot level={p.conformance} />}
          </div>
          <div style={{ fontFamily: 'var(--font-sans)', fontSize: 12, color: 'var(--slate-400)', marginTop: 3, lineHeight: 1.35 }}>{p.purpose}</div>
          <div style={{ display: 'flex', gap: 6, marginTop: 7, flexWrap: 'wrap' }}>
            {['direction ' + p.direction, p.quality].map(t => (
              <span key={t} style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--blue-400)',
                border: '1px solid var(--slate-700)', borderRadius: 3, padding: '1px 6px', whiteSpace: 'nowrap' }}>{t}</span>
            ))}
          </div>
        </div>
      </div>
    );
  }

  function DomainNode({ d, selected, onClick, showConf }) {
    return (
      <div onClick={onClick} style={{
        cursor: 'pointer', boxSizing: 'border-box', width: '100%', height: '100%',
        background: 'color-mix(in srgb, var(--kind-entity) 14%, var(--slate-900))',
        border: `1.5px solid ${selected ? 'var(--blue-400)' : 'var(--kind-entity)'}`,
        borderRadius: 7, padding: '9px 12px',
        boxShadow: selected ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 30%, transparent)' : 'none',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 7 }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.12em',
            textTransform: 'uppercase', color: 'var(--blue-400)' }}>domain</span>
          {showConf && <ConfDot level={d.conformance} size={7} />}
        </div>
        <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 600, fontSize: 14, color: 'var(--slate-100)', marginTop: 1 }}>{d.name}</div>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-400)', marginTop: 2 }}>{d.language.join(' · ')}</div>
      </div>
    );
  }

  function SystemNode({ s, selected, onSelect, onOpen, showConf, dense }) {
    const flowCount = s.flows.length;
    return (
      <div onClick={onSelect} style={{
        cursor: 'pointer', boxSizing: 'border-box', width: '100%', height: '100%',
        background: 'var(--slate-800)', border: `1.5px solid ${selected ? 'var(--blue-400)' : 'var(--slate-600)'}`,
        borderRadius: 8, overflow: 'hidden', display: 'flex', flexDirection: 'column',
        boxShadow: selected ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 30%, transparent)' : 'var(--shadow-graph)',
      }}>
        <div style={{ padding: dense ? '9px 13px' : '11px 14px', borderBottom: '1px solid var(--slate-700)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <span style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 15, color: 'var(--slate-100)', whiteSpace: 'nowrap', flexShrink: 0 }}>{s.name}</span>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', whiteSpace: 'nowrap', flexShrink: 0 }}>{s.kind}</span>
            <span style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 6, flexShrink: 0 }}>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--em-trigger-soft)',
                border: '1px solid var(--slate-700)', borderRadius: 3, padding: '0 5px' }}>{s.cls}</span>
              {showConf && <ConfDot level={s.conformance} size={8} />}
            </span>
          </div>
          <div style={{ fontFamily: 'var(--font-sans)', fontSize: 11.5, color: 'var(--slate-400)', marginTop: 3 }}>{s.purpose}</div>
        </div>
        <div style={{ padding: dense ? '8px 13px' : '10px 14px', display: 'flex', flexDirection: 'column', gap: 7, flex: 1 }}>
          <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap' }}>
            {s.platforms.map(pl => (
              <span key={pl} style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-300)',
                background: 'var(--slate-900)', border: '1px solid var(--slate-700)', borderRadius: 3, padding: '1px 6px' }}>{pl}</span>
            ))}
          </div>
          {s.demands.length > 0 && !dense && (
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', lineHeight: 1.5 }}>
              {s.demands.map(d => <div key={d}>· {d}</div>)}
            </div>
          )}
        </div>
        <button onClick={(e) => { e.stopPropagation(); onOpen(); }} style={{
          margin: 0, border: 0, borderTop: '1px solid var(--slate-700)', background: 'var(--slate-900)',
          color: 'var(--blue-400)', fontFamily: 'var(--font-mono)', fontSize: 11, padding: '8px 14px',
          cursor: 'pointer', textAlign: 'left', display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        }}>
          <span>{flowCount} event-model flow{flowCount > 1 ? 's' : ''}</span>
          <span style={{ color: 'var(--slate-300)' }}>open →</span>
        </button>
      </div>
    );
  }

  function SystemsMap({ onOpenSystem, selected, onSelect, showConf, showLabels, dense }) {
    const pos = useMemo(layout, []);
    const P = PF.product;
    const slate5 = 'var(--slate-500)', slate6 = 'var(--slate-600)';

    // orthogonal edges — explicit waypoints routed through clear channels
    const edges = [
      // product owns domains (fork down the centre)
      { pts: [{ x: 560, y: 138 }, { x: 560, y: 202 }, { x: 470, y: 202 }, { x: 470, y: 264 }], stroke: slate6, width: 1.4 },
      { pts: [{ x: 560, y: 138 }, { x: 560, y: 202 }, { x: 650, y: 202 }, { x: 650, y: 264 }], stroke: slate6, width: 1.4 },
      // product owns systems (brackets down the outside)
      { pts: [{ x: 435, y: 92 }, { x: 230, y: 92 }, { x: 230, y: 417 }], stroke: slate5, width: 1.6, label: 'owns', lx: 230, ly: 250 },
      { pts: [{ x: 685, y: 92 }, { x: 890, y: 92 }, { x: 890, y: 417 }], stroke: slate5, width: 1.6, label: 'owns', lx: 890, ly: 250 },
      // shop references domains (dashed)
      { pts: [{ x: 300, y: 417 }, { x: 300, y: 300 }, { x: 392, y: 300 }], stroke: slate5, width: 1.3, dash: '5 4', opacity: 0.6, label: 'references', lx: 300, ly: 350 },
      { pts: [{ x: 345, y: 417 }, { x: 345, y: 362 }, { x: 650, y: 362 }, { x: 650, y: 336 }], stroke: slate5, width: 1.3, dash: '5 4', opacity: 0.6 },
      // admin references domains (dashed)
      { pts: [{ x: 835, y: 417 }, { x: 835, y: 374 }, { x: 470, y: 374 }, { x: 470, y: 336 }], stroke: slate5, width: 1.3, dash: '5 4', opacity: 0.6 },
      { pts: [{ x: 880, y: 417 }, { x: 880, y: 300 }, { x: 728, y: 300 }], stroke: slate5, width: 1.3, dash: '5 4', opacity: 0.6, label: 'references', lx: 880, ly: 350 },
    ];

    // the journey — a Translation crossing the two systems (magenta, orthogonal bracket below)
    const j = PF.journeys[0];
    const jy = 591, jbus = 648;
    const jPath = roundedPath([{ x: 230, y: jy }, { x: 230, y: jbus }, { x: 890, y: jbus }, { x: 890, y: jy }], 10);

    return (
      <FitCanvas width={CANVAS_W} height={CANVAS_H}>
          <OrthoEdges edges={edges} showLabels={showLabels} />

          {/* journey — drawn separately, bracketed below the systems */}
          <svg width={CANVAS_W} height={CANVAS_H} style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'visible' }}>
            <path d={jPath} fill="none" stroke="var(--em-bridge)" strokeWidth="1.8" strokeDasharray="2 5" strokeLinecap="round" />
            <g transform={`translate(560, ${jbus})`}>
              <rect x={-150} y={-13} rx="4" width={300} height={26} fill="var(--slate-900)" stroke="var(--em-bridge)" strokeOpacity="0.5" />
              <text x={0} y={-1} textAnchor="middle" style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 600, fill: 'var(--em-bridge)' }}>journey · {j.name}</text>
              <text x={0} y={9} textAnchor="middle" style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fill: 'var(--slate-400)' }}>Translation: ev-order-placed → cmd-accept-fulfilment</text>
            </g>
          </svg>

          {/* nodes */}
          <Positioned box={pos.acme}><ProductNode p={P} selected={selected === 'acme'} onClick={() => onSelect('acme')} showConf={showConf} /></Positioned>
          {PF.domains.map(d => (
            <Positioned key={d.id} box={pos[d.id]}>
              <DomainNode d={d} selected={selected === d.id} onClick={() => onSelect(d.id)} showConf={showConf} />
            </Positioned>
          ))}
          {PF.systems.map(s => (
            <Positioned key={s.id} box={pos[s.id]}>
              <SystemNode s={s} selected={selected === s.id} onSelect={() => onSelect(s.id)} onOpen={() => onOpenSystem(s.id)} showConf={showConf} dense={dense} />
            </Positioned>
          ))}
      </FitCanvas>
    );
  }

  function Positioned({ box, children }) {
    return (
      <div style={{ position: 'absolute', left: box.x - box.w / 2, top: box.y - box.h / 2, width: box.w, height: box.h, zIndex: 2 }}>
        {children}
      </div>
    );
  }

  Object.assign(window, { SystemsMap });
})();
