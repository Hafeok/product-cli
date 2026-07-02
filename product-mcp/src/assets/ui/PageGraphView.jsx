/* global React, PF, PFUI */
/* Page graph (§3.2.4) — navigation as ONE graph. Nodes are pages (UI steps),
   edges are navigate transitions. Each system has a declared root whose
   out-edges ARE the primary navigation; "top-level" is derived, not tagged.
   Flows are tinted named regions — subgraphs, not owners. */
(function () {
  const { EdgeLayer, ConfDot, FitCanvas } = window.PFUI;

  const NAV = 'var(--em-trigger)';        // navigate = interaction = violet
  const UI = 'var(--slate-400)';
  const W = 1250, H = 780;

  // fixed layout — shop above, admin below
  const POS = {
    'root-shop':        { x: 130, y: 235, w: 168, h: 76, root: true },
    'ui-browse':        { x: 420, y: 105, w: 172, h: 58 },
    'ui-product':       { x: 700, y: 105, w: 172, h: 58 },
    'ui-review-cart':   { x: 420, y: 360, w: 172, h: 58 },
    'ui-choose-payment':{ x: 700, y: 360, w: 172, h: 58 },
    'ui-confirmation':  { x: 980, y: 360, w: 172, h: 58 },
    'ui-orders':        { x: 420, y: 232, w: 172, h: 58 },
    'ui-order-detail':  { x: 700, y: 232, w: 172, h: 58 },

    'root-admin':       { x: 130, y: 640, w: 168, h: 76, root: true },
    'ui-queue':         { x: 420, y: 578, w: 172, h: 58 },
    'ui-ship':          { x: 700, y: 578, w: 172, h: 58 },
    'ui-order-admin':   { x: 420, y: 702, w: 172, h: 58 },
    'ui-issue-refund':  { x: 700, y: 702, w: 172, h: 58 },
  };

  // flow regions — drawn as tinted dashes behind the pages they partition
  const REGIONS = [
    { flow: 'flow-browse',   label: 'flow-browse',   x: 316, y: 62,  w: 486, h: 88 },
    { flow: 'flow-orders',   label: 'flow-orders',   x: 316, y: 190, w: 486, h: 88 },
    { flow: 'flow-checkout', label: 'flow-checkout', x: 316, y: 318, w: 766, h: 88 },
    { flow: 'flow-fulfil',   label: 'flow-fulfil',   x: 316, y: 536, w: 486, h: 88 },
    { flow: 'flow-refunds',  label: 'flow-refunds',  x: 316, y: 660, w: 486, h: 88 },
  ];

  function allSystems() { return PF.pageGraph.systems; }
  function pageName(id) {
    for (const s of allSystems()) {
      const p = s.pages.find(p => p.id === id);
      if (p) return p.name;
    }
    return id;
  }
  // derived: a page is top-level iff it has an inbound edge from a root
  function isTopLevel(id) {
    return allSystems().some(s => s.edges.some(e => e.from === s.root && e.to === id));
  }

  function PageGraphView({ selected, onSelect, onOpenStep }) {
    const edges = [];
    allSystems().forEach(sys => {
      sys.edges.forEach(e => {
        const fromRoot = e.from === sys.root;
        edges.push({
          from: e.from, to: e.to,
          stroke: fromRoot ? NAV : 'var(--slate-500)',
          width: fromRoot ? 1.7 : 1.3,
          dash: fromRoot ? undefined : '4 4',
          label: e.label, labelColor: fromRoot ? NAV : 'var(--slate-400)',
        });
      });
    });

    return (
      <FitCanvas width={W} height={H}>
        {/* system band labels */}
        <BandLabel y={20} name="acme-shop" note="one page graph · root out-edges = primary nav (derived)" />
        <BandLabel y={498} name="acme-admin" note="its own root, its own subgraph \u2014 shared domain, separate journey" />
        <div style={{ position: 'absolute', left: 30, right: 30, top: 468, height: 1,
          background: 'var(--slate-700)', zIndex: 0 }} />

        {/* flow regions */}
        {REGIONS.map(r => (
          <div key={r.flow} style={{ position: 'absolute', left: r.x, top: r.y, width: r.w, height: r.h,
            border: '1.5px dashed var(--slate-700)', borderRadius: 10,
            background: 'color-mix(in srgb, var(--em-command) 4%, transparent)', zIndex: 0 }}>
            <span style={{ position: 'absolute', top: -8, left: 14, fontFamily: 'var(--font-mono)', fontSize: 9,
              letterSpacing: '.1em', color: 'var(--slate-500)', background: 'var(--slate-900)', padding: '0 6px' }}>{r.label}</span>
          </div>
        ))}

        <EdgeLayer edges={edges} pos={POS} width={W} height={H} showLabels={true} />

        {/* roots */}
        {allSystems().map(sys => {
          const p = POS[sys.root];
          const on = selected === sys.root;
          return (
            <div key={sys.root} onClick={() => onSelect(sys.root)} style={{
              position: 'absolute', left: p.x - p.w / 2, top: p.y - p.h / 2, width: p.w, height: p.h, zIndex: 2,
              cursor: 'pointer', boxSizing: 'border-box', background: 'var(--slate-800)',
              border: `2px solid ${on ? 'var(--blue-400)' : NAV}`, borderRadius: 9, padding: '9px 13px',
              boxShadow: on ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)' : 'var(--shadow-graph)',
            }}>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.12em',
                textTransform: 'uppercase', color: NAV }}>system root</div>
              <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 14.5, color: 'var(--slate-100)', marginTop: 2 }}>{sys.name}</div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, color: 'var(--slate-500)', marginTop: 2 }}>
                global: {sys.globalActions.join(', ')}</div>
            </div>
          );
        })}

        {/* pages */}
        {allSystems().map(sys => sys.pages.map(pg => {
          const p = POS[pg.id];
          if (!p) return null;
          const on = selected === pg.id;
          const top = isTopLevel(pg.id);
          const entry = sys.flows.some(f => f.entry === pg.id);
          return (
            <div key={pg.id} onClick={() => onSelect(pg.id)} onDoubleClick={() => pg.specced && onOpenStep(pg.id)} style={{
              position: 'absolute', left: p.x - p.w / 2, top: p.y - p.h / 2, width: p.w, height: p.h, zIndex: 2,
              cursor: 'pointer', boxSizing: 'border-box', background: 'var(--slate-800)',
              border: `1.5px ${pg.specced ? 'solid' : 'dashed'} ${on ? 'var(--blue-400)' : 'var(--slate-600)'}`,
              borderRadius: 6, padding: '7px 11px',
              boxShadow: on ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 28%, transparent)' : 'var(--shadow-graph)',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                <span style={{ fontFamily: 'var(--font-mono)', fontSize: 7.5, fontWeight: 600, letterSpacing: '.11em',
                  textTransform: 'uppercase', color: UI }}>page</span>
                {top && <span style={{ fontFamily: 'var(--font-mono)', fontSize: 7.5, letterSpacing: '.06em',
                  color: NAV, border: `1px solid ${NAV}`, borderRadius: 2, padding: '0 4px' }}>top-level · derived</span>}
                {entry && !top && <span style={{ fontFamily: 'var(--font-mono)', fontSize: 7.5, letterSpacing: '.06em',
                  color: 'var(--slate-400)', border: '1px solid var(--slate-600)', borderRadius: 2, padding: '0 4px' }}>entry</span>}
              </div>
              <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 600, fontSize: 12.5, color: 'var(--slate-100)', marginTop: 2 }}>{pg.name}</div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8, color: 'var(--slate-500)' }}>{pg.id}</div>
            </div>
          );
        }))}

        <div style={{ position: 'absolute', left: 30, bottom: 12, maxWidth: 700, fontFamily: 'var(--font-mono)',
          fontSize: 10.5, color: 'var(--slate-500)', lineHeight: 1.5, zIndex: 3 }}>
          nothing is tagged “top-level” by hand — it falls out of the root’s out-edges, the way a feature’s
          footprint falls out of its flow slice (§7). solid border = fully specced in the render contract.
        </div>
      </FitCanvas>
    );
  }

  function BandLabel({ y, name, note }) {
    return (
      <div style={{ position: 'absolute', left: 30, top: y, display: 'flex', alignItems: 'baseline', gap: 12, zIndex: 3 }}>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, fontWeight: 700, letterSpacing: '.14em',
          textTransform: 'uppercase', color: 'var(--slate-300)' }}>{name}</span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)' }}>{note}</span>
      </div>
    );
  }

  // detail for app.jsx
  window.pageGraphDetail = function (sel) {
    for (const sys of allSystems()) {
      if (sel === sys.root) {
        const dests = sys.edges.filter(e => e.from === sys.root).map(e => `\u2192 ${pageName(e.to)} (${e.label})`);
        return {
          kind: 'root', title: sys.name + ' \u2014 root', rows: [
            { k: 'out-edges = primary nav (derived)', v: dests },
            { k: 'global actions', v: sys.globalActions.join(', ') },
            { k: 'reifies per context', v: 'phone \u2192 tab bar \u00b7 desktop \u2192 sidebar \u00b7 tui \u2192 menu bar' },
          ],
        };
      }
      const pg = sys.pages.find(p => p.id === sel);
      if (pg) {
        const inbound = sys.edges.filter(e => e.to === pg.id).map(e => '\u2190 ' + (e.from === sys.root ? sys.name + ' root' : pageName(e.from)));
        const outbound = sys.edges.filter(e => e.from === pg.id).map(e => '\u2192 ' + pageName(e.to));
        return {
          kind: 'page', title: pg.name, specced: pg.specced, id: pg.id, rows: [
            { k: 'id', v: pg.id },
            { k: 'flow (named subgraph)', v: pg.flow },
            { k: 'top-level', v: isTopLevel(pg.id) ? 'yes \u2014 derived from a root edge' : 'no \u2014 nested, reachable only via pages' },
            inbound.length ? { k: 'inbound', v: inbound } : null,
            outbound.length ? { k: 'outbound (navigate)', v: outbound } : null,
          ],
        };
      }
    }
    return null;
  };

  Object.assign(window, { PageGraphView });
})();
