/* global React, PF, PFUI */
/* Flows timeline — the event-model of one flow in a system, as an Event-Modeling
   swimlane: Triggers/UI · Commands·Views · per-aggregate event streams. Spine
   edges are the always-on flow; cross edges (a view feeding a UI step) stay
   visible and brighten on hover. */
(function () {
  const { useState, useMemo } = React;
  const { EdgeLayer, ConfDot } = window.PFUI;
  const NS = window.ProductFrameworkDesignSystem_52ecf1;
  const EMNode = NS.EMNode;

  const GUTTER = 190, COLW = 224, NODE_W = 190, STACK = 50;

  function computeLayout(flow, dense) {
    const railH = dense ? 96 : 116, streamH = dense ? 74 : 86;
    let y = 20;
    const laneBox = {};
    flow.lanes.forEach(ln => {
      const h = ln.kind === 'stream' ? streamH : railH;
      laneBox[ln.id] = { top: y, h, center: y + h / 2, kind: ln.kind, label: ln.label };
      y += h;
    });
    const totalH = y + 16;
    const width = GUTTER + flow.cols * COLW + 40;
    const cell = {};
    flow.nodes.forEach(n => { const k = n.lane + ':' + n.col; (cell[k] = cell[k] || []).push(n.id); });
    const pos = {};
    flow.nodes.forEach(n => {
      const x = GUTTER + n.col * COLW + COLW / 2;
      const box = laneBox[n.lane];
      const group = cell[n.lane + ':' + n.col];
      let yy = box.center;
      if (group.length > 1) {
        const i = group.indexOf(n.id);
        yy = box.center - ((group.length - 1) * STACK) / 2 + i * STACK;
      }
      pos[n.id] = { x, y: yy, w: NODE_W, h: 44 };
    });
    return { laneBox, totalH, width, pos };
  }

  function FlowsTimeline({ flow, hidden, selected, onSelect, showLabels, dense }) {
    const [hovered, setHovered] = useState(null);
    const { laneBox, totalH, width, pos } = useMemo(() => computeLayout(flow, dense), [flow, dense]);

    const visible = (id) => { const n = flow.nodes.find(x => x.id === id); return n && !hidden.has(n.kind); };

    const edges = flow.edges.filter(e => visible(e.from) && visible(e.to)).map(e => {
      const touch = hovered && (e.from === hovered || e.to === hovered);
      if (e.type === 'spine') {
        return { from: e.from, to: e.to, stroke: 'var(--slate-500)', width: 1.5, opacity: touch ? 1 : 0.62, arrow: true };
      }
      return { from: e.from, to: e.to, stroke: '#38bdf8', width: 1.5, dash: '5 3', opacity: touch ? 1 : 0.4, arrow: true };
    });

    return (
      <div style={{ position: 'relative', width, height: totalH, minWidth: '100%' }}>
        {/* lane backgrounds + gutter labels */}
        {flow.lanes.map(ln => {
          const box = laneBox[ln.id];
          const isStream = ln.kind === 'stream';
          return (
            <div key={ln.id}>
              <div style={{ position: 'absolute', left: GUTTER, right: 0, top: box.top, height: box.h,
                background: isStream ? 'rgba(245,158,11,.06)' : (ln.id === 'cmdview' ? 'rgba(37,99,235,.05)' : 'rgba(148,163,184,.045)'),
                borderTop: '1px solid var(--slate-800)' }} />
              <div style={{ position: 'absolute', left: 0, width: GUTTER - 16, top: box.center - 9, textAlign: 'right',
                fontFamily: 'var(--font-mono)', fontSize: 11, fontWeight: isStream ? 600 : 700,
                color: isStream ? 'var(--em-event)' : 'var(--slate-300)', letterSpacing: '.02em' }}>{ln.label}</div>
            </div>
          );
        })}
        <div style={{ position: 'absolute', left: GUTTER, top: 0, bottom: 0, width: 1, background: 'var(--slate-700)' }} />

        <EdgeLayer edges={edges} pos={pos} width={width} height={totalH} showLabels={false} />

        {flow.nodes.filter(n => !hidden.has(n.kind)).map(n => {
          const p = pos[n.id];
          return (
            <div key={n.id} onMouseEnter={() => setHovered(n.id)} onMouseLeave={() => setHovered(null)}
              style={{ position: 'absolute', left: p.x - NODE_W / 2, top: p.y - 23, width: NODE_W, zIndex: 2 }}>
              <EMNode kind={n.kind} label={n.label} note={n.sub} showKind={false}
                selected={selected === n.id} onClick={() => onSelect(n.id)}
                style={{ minWidth: NODE_W, maxWidth: NODE_W }} />
            </div>
          );
        })}
      </div>
    );
  }

  // wrapper: flow selector chips for the system + the timeline + scroll
  function FlowsView({ systemId, flowId, setFlowId, hidden, selected, onSelect, showConf, showLabels, dense }) {
    const system = PF.systems.find(s => s.id === systemId);
    const flow = PF.flows[flowId];
    if (!system || !flow) {
      return <window.PFUI.EmptyHint what="event-model flows"
        hint="flows appear here once a system and its trigger → command → event chains are captured (§3.2)." />;
    }

    return (
      <div style={{ position: 'relative', width: '100%', height: '100%', display: 'flex', flexDirection: 'column' }}>
        {/* flow selector */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '10px 18px', flexWrap: 'wrap',
          borderBottom: '1px solid var(--slate-800)', background: 'var(--slate-900)' }}>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, letterSpacing: '.12em', textTransform: 'uppercase', color: 'var(--slate-500)' }}>flows in {system.name}</span>
          <div style={{ display: 'flex', gap: 7, flexWrap: 'wrap' }}>
            {system.flows.map(fid => {
              const f = PF.flows[fid];
              const on = fid === flowId;
              return (
                <button key={fid} onClick={() => setFlowId(fid)} style={{
                  display: 'inline-flex', alignItems: 'center', gap: 7, cursor: 'pointer',
                  background: on ? 'var(--slate-700)' : 'transparent', color: on ? 'var(--slate-100)' : 'var(--slate-400)',
                  border: `1px solid ${on ? 'var(--slate-500)' : 'var(--slate-700)'}`, borderRadius: 6,
                  fontFamily: 'var(--font-sans)', fontSize: 12.5, fontWeight: 500, padding: '5px 11px',
                }}>
                  {f.name}
                  {showConf && <ConfDot level={f.conformance} size={7} />}
                </button>
              );
            })}
          </div>
          <span style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8,
            fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--slate-500)' }}>
            <span style={{ color: 'var(--em-command)' }}>pattern</span> {flow.pattern}
          </span>
        </div>

        {/* canvas */}
        <div style={{ flex: 1, overflow: 'auto', background: 'var(--slate-900)' }}>
          <div style={{ padding: '10px 0 24px' }}>
            <FlowsTimeline flow={flow} hidden={hidden} selected={selected} onSelect={onSelect}
              showLabels={showLabels} dense={dense} />
          </div>
        </div>
      </div>
    );
  }

  Object.assign(window, { FlowsView });
})();
