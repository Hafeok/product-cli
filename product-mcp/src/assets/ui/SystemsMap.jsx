/* global React, PF, PFUI */
/* Systems map — product → systems & journeys. A graph: product owns systems &
   domains; systems reference domains; a journey crosses two systems via a
   Translation. */
(function () {
  const { useMemo } = React;
  const { EdgeLayer, ConfDot, FitCanvas } = window.PFUI;

  // Card sizes; the canvas is sized to the graph, so any product / domain /
  // system count lays out (this view is data-driven, not curated coordinates).
  const PROD = { w: 280, h: 96 }, DOM = { w: 176, h: 78 }, SYS = { w: 288, h: 176 };
  const GAP = 40, MARGIN = 60;
  const PROD_Y = 74, DOM_Y = 300, SYS_Y = 540;

  // Compute node centres for the live product/domains/systems, plus the canvas
  // size and the y of each row's connecting bus.
  function computeLayout(product, domains, systems) {
    const rowWidth = (n, cw) => n > 0 ? n * cw + (n - 1) * GAP : 0;
    const wDom = rowWidth(domains.length, DOM.w);
    const wSys = rowWidth(systems.length, SYS.w);
    const W = Math.max(PROD.w, wDom, wSys, 640) + 2 * MARGIN;
    const journeyBand = 0; // reserved below the systems row for journey brackets
    const H = SYS_Y + SYS.h / 2 + 60 + journeyBand;
    const pos = {};
    if (product) pos[product.id] = { x: W / 2, y: PROD_Y, w: PROD.w, h: PROD.h };
    const place = (arr, cw, ch, y) => {
      const start = (W - rowWidth(arr.length, cw)) / 2 + cw / 2;
      arr.forEach((n, i) => { pos[n.id] = { x: start + i * (cw + GAP), y, w: cw, h: ch }; });
    };
    place(domains, DOM.w, DOM.h, DOM_Y);
    place(systems, SYS.w, SYS.h, SYS_Y);
    return { pos, W, H };
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

  function OrthoEdges({ edges, showLabels, w, h }) {
    return (
      <svg width={w} height={h} style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'visible' }}>
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
    const P = PF.product || { id: '', name: '', purpose: '', direction: '', quality: '' };
    const domains = PF.domains || [];
    const systems = PF.systems || [];
    const journeys = PF.journeys || [];
    const slate5 = 'var(--slate-500)', slate6 = 'var(--slate-600)';

    const { pos, W, H } = useMemo(
      () => computeLayout(P, domains, systems),
      [P.id, domains.map(d => d.id).join(','), systems.map(s => s.id).join(',')],
    );

    const cx = id => (pos[id] || null);
    const top = b => ({ x: b.x, y: b.y - b.h / 2 });
    const bot = b => ({ x: b.x, y: b.y + b.h / 2 });

    // Edges computed from the graph: product owns each domain + system, and each
    // system references its domains (dashed). Buses sit between the rows.
    const edges = [];
    const ownDomBus = (PROD_Y + PROD.h / 2 + DOM_Y - DOM.h / 2) / 2;
    const ownSysBus = DOM_Y + DOM.h / 2 + 30;
    const refBus = DOM_Y + DOM.h / 2 + 16;
    const pb = pos[P.id] ? bot(pos[P.id]) : null;
    if (pb) {
      const owned = new Set(P.ownsDomains || domains.map(d => d.id));
      domains.filter(d => owned.has(d.id)).forEach((d, i) => {
        const b = cx(d.id); if (!b) return;
        edges.push({ pts: [pb, { x: pb.x, y: ownDomBus }, { x: b.x, y: ownDomBus }, top(b)], stroke: slate6, width: 1.4, label: i === 0 ? 'owns' : null, lx: (pb.x + b.x) / 2, ly: ownDomBus - 8 });
      });
      const ownedSys = new Set(P.ownsSystems || systems.map(s => s.id));
      systems.filter(s => ownedSys.has(s.id)).forEach((s, i) => {
        const b = cx(s.id); if (!b) return;
        edges.push({ pts: [pb, { x: pb.x, y: ownSysBus }, { x: b.x, y: ownSysBus }, top(b)], stroke: slate5, width: 1.6, label: i === 0 ? 'owns' : null, lx: b.x, ly: ownSysBus - 8 });
      });
    }
    systems.forEach(s => {
      const sb = cx(s.id); if (!sb) return;
      (s.references || []).forEach((dref, i) => {
        const b = cx(dref); if (!b) return;
        edges.push({ pts: [top(sb), { x: sb.x, y: refBus }, { x: b.x, y: refBus }, bot(b)], stroke: slate5, width: 1.3, dash: '5 4', opacity: 0.6, label: i === 0 ? 'references' : null, lx: (sb.x + b.x) / 2, ly: refBus + 10, arrow: false });
      });
    });

    // Journeys — a Translation bracket below the systems it crosses.
    const jbus0 = SYS_Y + SYS.h / 2 + 24;
    const journeyEls = journeys.map((j, idx) => {
      const a = cx(j.from && j.from.system), b = cx(j.to && j.to.system);
      if (!a || !b) return null;
      const jbus = jbus0 + idx * 34;
      const jPath = roundedPath([{ x: a.x, y: bot(a).y }, { x: a.x, y: jbus }, { x: b.x, y: jbus }, { x: b.x, y: bot(b).y }], 10);
      return (
        <g key={j.id}>
          <path d={jPath} fill="none" stroke="var(--em-bridge)" strokeWidth="1.8" strokeDasharray="2 5" strokeLinecap="round" />
          <g transform={`translate(${(a.x + b.x) / 2}, ${jbus})`}>
            <rect x={-170} y={-13} rx="4" width={340} height={26} fill="var(--slate-900)" stroke="var(--em-bridge)" strokeOpacity="0.5" />
            <text x={0} y={-1} textAnchor="middle" style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 600, fill: 'var(--em-bridge)' }}>journey · {j.name}</text>
            <text x={0} y={9} textAnchor="middle" style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fill: 'var(--slate-400)' }}>{j.translation ? ('Translation: ' + j.translation) : 'crosses via Translation'}</text>
          </g>
        </g>
      );
    }).filter(Boolean);

    const HH = H + (journeyEls.length ? journeyEls.length * 34 : 0);

    return (
      <FitCanvas width={W} height={HH}>
          <OrthoEdges edges={edges} showLabels={showLabels} w={W} h={HH} />
          {journeyEls.length > 0 && (
            <svg width={W} height={HH} style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'visible' }}>{journeyEls}</svg>
          )}
          {pos[P.id] && (
            <Positioned box={pos[P.id]}><ProductNode p={P} selected={selected === P.id} onClick={() => onSelect(P.id)} showConf={showConf} /></Positioned>
          )}
          {domains.map(d => (
            <Positioned key={d.id} box={pos[d.id]}>
              <DomainNode d={d} selected={selected === d.id} onClick={() => onSelect(d.id)} showConf={showConf} />
            </Positioned>
          ))}
          {systems.map(s => (
            <Positioned key={s.id} box={pos[s.id]}>
              <SystemNode s={s} selected={selected === s.id} onSelect={() => onSelect(s.id)} onOpen={() => onOpenSystem(s.id)} showConf={showConf} dense={dense} />
            </Positioned>
          ))}
      </FitCanvas>
    );
  }

  // Guard against a missing position (e.g. an id present in an edge but not laid
  // out) so a single gap never white-screens the whole app.
  function Positioned({ box, children }) {
    if (!box) return null;
    return (
      <div style={{ position: 'absolute', left: box.x - box.w / 2, top: box.y - box.h / 2, width: box.w, height: box.h, zIndex: 2 }}>
        {children}
      </div>
    );
  }

  Object.assign(window, { SystemsMap });
})();
