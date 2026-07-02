/* global React, PF */
/* THE GRAPH (§2 / §9) — everything the other views show, as the one connected
   graph it actually is. Radial layout: the product at the centre, the What
   around it, UI / How / Build+Delivery in their sectors. Click any node and
   its full derivation cone lights up — upstream (what it derives from) and
   downstream (its blast radius). "Describe this system" is a query. */
(function () {
  const { useRef, useEffect, useState, useMemo } = React;

  const cssVar = (n, fb) => {
    const v = getComputedStyle(document.documentElement).getPropertyValue(n).trim();
    return v || fb;
  };

  // sectors: angular wedges (degrees), radial rings per abstraction depth
  const SECTORS = {
    what:  { a0: 100, a1: 240, label: 'THE WHAT §3' },
    ui:    { a0: 240, a1: 310, label: 'UI §3.2' },
    how:   { a0: 310, a1: 400, label: 'THE HOW §4' },
    build: { a0: 40,  a1: 100, label: 'BUILD · DELIVERY §5–7' },
  };
  const RING_R = [0, 150, 262, 375, 480];
  const W = 1500, H = 1160, CX = 750, CY = 585;

  function layout(nodes) {
    const groups = {};
    nodes.forEach(n => {
      const k = n.sector + '|' + n.ring;
      (groups[k] = groups[k] || []).push(n);
    });
    const pos = {};
    Object.entries(groups).forEach(([k, arr]) => {
      const [sector, ring] = k.split('|');
      const s = SECTORS[sector];
      const r = RING_R[+ring];
      if (+ring === 0) { arr.forEach(n => { pos[n.id] = { x: CX, y: CY }; }); return; }
      const pad = 6, span = (s.a1 - s.a0) - pad * 2;
      arr.forEach((n, i) => {
        const frac = arr.length === 1 ? 0.5 : i / (arr.length - 1);
        const a = (s.a0 + pad + frac * span) * Math.PI / 180;
        // deterministic jitter so rings don't read as perfect circles
        let h = 0; for (let c = 0; c < n.id.length; c++) h = (h * 31 + n.id.charCodeAt(c)) | 0;
        const jr = ((h % 100) / 100 - 0.5) * 26;
        pos[n.id] = { x: CX + Math.cos(a) * (r + jr), y: CY + Math.sin(a) * (r + jr) };
      });
    });
    return pos;
  }

  function GraphView({ onOpen }) {
    const canvasRef = useRef(null);
    const wrapRef = useRef(null);
    const [selected, setSelected] = useState(null);
    const stateRef = useRef({ hover: null, tf: { x: 0, y: 0, k: 1 }, drag: null });

    const graph = useMemo(() => {
      const g = PF.buildGraph();
      const pos = layout(g.nodes);
      const byId = {}; g.nodes.forEach(n => { byId[n.id] = n; });
      const deg = {}; g.edges.forEach(e => { deg[e.from] = (deg[e.from] || 0) + 1; deg[e.to] = (deg[e.to] || 0) + 1; });
      const out = {}, inn = {};
      g.edges.forEach((e, i) => { (out[e.from] = out[e.from] || []).push(i); (inn[e.to] = inn[e.to] || []).push(i); });
      return { ...g, pos, byId, deg, out, inn };
    }, []);

    // colors resolved once
    const C = useMemo(() => ({
      bg: cssVar('--slate-900', '#0f172a'), panel: cssVar('--slate-800', '#1e293b'),
      faint: cssVar('--slate-700', '#334155'), dim: cssVar('--slate-600', '#475569'),
      text: cssVar('--slate-300', '#cbd5e1'), mute: cssVar('--slate-500', '#64748b'),
      white: cssVar('--slate-100', '#f1f5f9'),
      blue: cssVar('--blue-500', '#3b82f6'), blue4: cssVar('--blue-400', '#60a5fa'),
      cmd: cssVar('--em-command', '#2563eb'), view: cssVar('--em-view', '#16a34a'),
      ev: cssVar('--em-event', '#f59e0b'), trg: cssVar('--em-trigger', '#7c3aed'),
      bridge: cssVar('--em-bridge', '#db2777'),
      entity: cssVar('--kind-entity', '#ea7317'), inv: cssVar('--em-trigger-soft', '#a78bfa'),
      ok: cssVar('--conf-verified', '#22c55e'),
    }), []);

    const nodeColor = (n) => ({
      product: C.white, domain: C.entity, system: C.trg,
      'concept-aggregate': C.entity, 'concept-entity': C.blue, 'concept-value-object': C.blue4,
      'concept-invariant': C.inv, 'concept-external': C.mute, 'concept-reference': C.ev,
      flow: C.cmd, decider: C.cmd, projector: C.view,
      aio: C.trg, screen: C.text, content: C.blue4,
      decision: C.ev, blueprint: C.ev, principle: C.blue4, pattern: C.mute, deployable: C.view, cio: C.view,
      workunit: C.view, feature: C.view, release: C.ok, 'what-version': C.blue, 'how-version': C.ev, target: C.mute,
    }[n.kind] || C.mute);

    const edgeColor = (e) => {
      const a = graph.byId[e.from], b = graph.byId[e.to];
      if (['reifies', 'realised-by', 'frozen-into'].includes(e.rel) || (a.sector !== b.sector && (a.sector === 'what' || b.sector === 'how')))
        return C.bridge;
      if (e.rel === 'realises' && a.kind === 'what-version') return C.bridge;
      if (b.sector === 'build' || a.sector === 'build') return C.view;
      if (a.sector === 'how') return C.ev;
      if (a.sector === 'ui' || b.sector === 'ui') return C.trg;
      return C.blue;
    };

    // derivation cone: upstream + downstream BFS over directed edges
    const cone = useMemo(() => {
      if (!selected) return null;
      const up = new Set([selected]), down = new Set([selected]), edgeSet = new Set();
      const walk = (set, adj, endKey) => {
        const q = [selected];
        while (q.length) {
          const id = q.shift();
          (adj[id] || []).forEach(ei => {
            const e = graph.edges[ei];
            const nxt = e[endKey];
            edgeSet.add(ei);
            if (!set.has(nxt)) { set.add(nxt); q.push(nxt); }
          });
        }
      };
      walk(down, graph.out, 'to');
      walk(up, graph.inn, 'from');
      const all = new Set([...up, ...down]);
      return { up, down, all, edgeSet, upN: up.size - 1, downN: down.size - 1 };
    }, [selected, graph]);
    const coneRef = useRef(null); coneRef.current = cone;
    const selRef = useRef(null); selRef.current = selected;

    // particles drifting along edges — the graph is alive
    const particlesRef = useRef(null);
    if (!particlesRef.current) {
      particlesRef.current = Array.from({ length: 70 }, () => ({
        e: Math.floor(Math.random() * 999), t: Math.random(), v: 0.0016 + Math.random() * 0.0035,
      }));
    }

    useEffect(() => {
      const canvas = canvasRef.current, wrap = wrapRef.current;
      const ctx = canvas.getContext('2d');
      const st = stateRef.current;
      let raf, dpr = window.devicePixelRatio || 1;

      const fit = () => {
        const r = wrap.getBoundingClientRect();
        canvas.width = r.width * dpr; canvas.height = r.height * dpr;
        canvas.style.width = r.width + 'px'; canvas.style.height = r.height + 'px';
        const k = Math.min(r.width / W, r.height / H) * 1.02;
        if (!st.userMoved) st.tf = { k, x: (r.width - W * k) / 2, y: (r.height - H * k) / 2 };
      };
      fit();
      const ro = new ResizeObserver(fit); ro.observe(wrap);

      const world = (mx, my) => ({ x: (mx - st.tf.x) / st.tf.k, y: (my - st.tf.y) / st.tf.k });
      const qpt = (p1, p2, t) => {
        // control point pulled toward centre for radial curvature
        const cx = (p1.x + p2.x) / 2 + (CX - (p1.x + p2.x) / 2) * 0.22;
        const cy = (p1.y + p2.y) / 2 + (CY - (p1.y + p2.y) / 2) * 0.22;
        const u = 1 - t;
        return { x: u * u * p1.x + 2 * u * t * cx + t * t * p2.x, y: u * u * p1.y + 2 * u * t * cy + t * t * p2.y, cx, cy };
      };

      const MAJOR = new Set(['product', 'domain', 'system', 'flow', 'blueprint', 'feature', 'release', 'target']);

      const draw = (time) => {
        try {
        const cone2 = coneRef.current, sel = selRef.current;
        const r = wrap.getBoundingClientRect();
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        ctx.fillStyle = C.bg; ctx.fillRect(0, 0, r.width, r.height);
        ctx.translate(st.tf.x * dpr / dpr, 0); // no-op guard
        ctx.setTransform(dpr * st.tf.k, 0, 0, dpr * st.tf.k, dpr * st.tf.x, dpr * st.tf.y);

        // sector wedges + ring guides
        Object.entries(SECTORS).forEach(([key, s]) => {
          const col = { what: C.blue, ui: C.trg, how: C.ev, build: C.view }[key];
          ctx.beginPath();
          ctx.moveTo(CX, CY);
          ctx.arc(CX, CY, RING_R[4] + 46, s.a0 * Math.PI / 180, s.a1 * Math.PI / 180);
          ctx.closePath();
          ctx.fillStyle = col + '0a'; ctx.fill();
          // sector caption on the outer arc
          const mid = (s.a0 + s.a1) / 2 * Math.PI / 180;
          const lx = CX + Math.cos(mid) * (RING_R[4] + 64), ly = CY + Math.sin(mid) * (RING_R[4] + 64);
          ctx.save();
          ctx.translate(lx, ly);
          ctx.fillStyle = col; ctx.font = '600 15px "IBM Plex Mono", monospace';
          ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
          ctx.fillText(s.label, 0, 0);
          ctx.restore();
        });
        for (let i = 1; i < 5; i++) {
          ctx.beginPath(); ctx.arc(CX, CY, RING_R[i], 0, Math.PI * 2);
          ctx.strokeStyle = C.faint + '55'; ctx.lineWidth = 1; ctx.setLineDash([2, 7]); ctx.stroke(); ctx.setLineDash([]);
        }

        // edges
        graph.edges.forEach((e, i) => {
          const p1 = graph.pos[e.from], p2 = graph.pos[e.to];
          const inCone = cone2 && cone2.edgeSet.has(i);
          const col = edgeColor(e);
          ctx.beginPath();
          const { cx, cy } = qpt(p1, p2, 0.5);
          ctx.moveTo(p1.x, p1.y); ctx.quadraticCurveTo(cx, cy, p2.x, p2.y);
          if (cone2) { ctx.strokeStyle = inCone ? col : C.faint + '30'; ctx.lineWidth = inCone ? 1.7 : 0.7; }
          else { ctx.strokeStyle = col + '73'; ctx.lineWidth = 1; }
          ctx.stroke();
        });

        // particles
        const parts = particlesRef.current;
        parts.forEach(p => {
          if (cone2) { if (!cone2.edgeSet.has(p.e % graph.edges.length)) { p.e = [...cone2.edgeSet][Math.floor(Math.random() * cone2.edgeSet.size)] || 0; } }
          const e = graph.edges[p.e % graph.edges.length];
          p.t += p.v; if (p.t > 1) { p.t = 0; if (!cone2) p.e = Math.floor(Math.random() * graph.edges.length); }
          const pt = qpt(graph.pos[e.from], graph.pos[e.to], p.t);
          ctx.beginPath(); ctx.arc(pt.x, pt.y, 1.7, 0, Math.PI * 2);
          ctx.fillStyle = edgeColor(e); ctx.globalAlpha = 0.85; ctx.fill(); ctx.globalAlpha = 1;
        });

        // nodes
        graph.nodes.forEach(n => {
          const p = graph.pos[n.id];
          const col = nodeColor(n);
          const dimmed = cone2 && !cone2.all.has(n.id);
          const deg = graph.deg[n.id] || 1;
          const rad = n.kind === 'product' ? 15 : Math.min(4 + deg * 0.7, 12);
          ctx.globalAlpha = dimmed ? 0.15 : 1;
          ctx.beginPath(); ctx.arc(p.x, p.y, rad, 0, Math.PI * 2);
          ctx.fillStyle = C.panel; ctx.fill();
          ctx.strokeStyle = col; ctx.lineWidth = n.kind === 'product' ? 2.5 : 1.6;
          if (n.dashed) ctx.setLineDash([3, 3]);
          ctx.stroke(); ctx.setLineDash([]);
          ctx.beginPath(); ctx.arc(p.x, p.y, Math.max(rad - 3.5, 1.6), 0, Math.PI * 2);
          ctx.fillStyle = col + (dimmed ? '44' : '2e'); ctx.fill();
          if (n.id === sel) {
            const pulse = 3 + Math.sin(time / 300) * 1.5;
            ctx.beginPath(); ctx.arc(p.x, p.y, rad + pulse + 2, 0, Math.PI * 2);
            ctx.strokeStyle = C.white; ctx.lineWidth = 1.4; ctx.stroke();
          }
          if (n.id === st.hover && !dimmed) {
            ctx.beginPath(); ctx.arc(p.x, p.y, rad + 3, 0, Math.PI * 2);
            ctx.strokeStyle = col; ctx.lineWidth = 1; ctx.stroke();
          }
          // labels: majors always; everything when zoomed or in cone/hover
          const show = MAJOR.has(n.kind) || st.tf.k > 0.92 || (cone2 && cone2.all.has(n.id)) || n.id === st.hover;
          if (show) {
            ctx.globalAlpha = dimmed ? 0.12 : (MAJOR.has(n.kind) ? 0.95 : 0.7);
            ctx.font = (MAJOR.has(n.kind) ? '600 11.5px' : '10px') + ' "IBM Plex Mono", monospace';
            ctx.fillStyle = n.id === sel ? C.white : C.text;
            ctx.textAlign = 'center';
            ctx.fillText(n.label, p.x, p.y + (n.kind === 'product' ? 30 : 20 + Math.min(4 + deg * 0.7, 12) * 0.4));
          }
          ctx.globalAlpha = 1;
        });

        } catch (err) { console.error('GRAPH DRAW FAIL', err); window.__graphErr = String(err && err.stack || err); stopped = true; }
      };
      let stopped = false, rafSeen = false, ticker = null;
      const loop = (t) => { if (stopped) return; rafSeen = true; draw(t || performance.now()); raf = requestAnimationFrame(loop); };
      raf = requestAnimationFrame(loop);
      draw(performance.now()); // immediate first paint — rAF may be throttled or absent
      const fallback = setTimeout(() => {
        if (!rafSeen && !stopped) ticker = setInterval(() => draw(performance.now()), 50);
      }, 400);

      const hit = (mx, my) => {
        const w = world(mx, my);
        let best = null, bd = 15 / st.tf.k;
        graph.nodes.forEach(n => {
          const p = graph.pos[n.id];
          const d = Math.hypot(p.x - w.x, p.y - w.y);
          if (d < bd) { bd = d; best = n.id; }
        });
        return best;
      };
      const onMove = (ev) => {
        const r = canvas.getBoundingClientRect();
        const mx = ev.clientX - r.left, my = ev.clientY - r.top;
        if (st.drag) {
          st.tf.x += mx - st.drag.x; st.tf.y += my - st.drag.y;
          st.drag = { x: mx, y: my }; st.userMoved = true;
          return;
        }
        st.hover = hit(mx, my);
        canvas.style.cursor = st.hover ? 'pointer' : 'grab';
      };
      const onDown = (ev) => {
        const r = canvas.getBoundingClientRect();
        st.drag = { x: ev.clientX - r.left, y: ev.clientY - r.top, moved: false, t: Date.now() };
      };
      const onUp = (ev) => {
        const wasQuick = st.drag && Date.now() - st.drag.t < 250;
        st.drag = null;
        if (wasQuick) {
          const r = canvas.getBoundingClientRect();
          const id = hit(ev.clientX - r.left, ev.clientY - r.top);
          setSelected(cur => (id === cur ? null : id));
        }
      };
      const onWheel = (ev) => {
        ev.preventDefault();
        const r = canvas.getBoundingClientRect();
        const mx = ev.clientX - r.left, my = ev.clientY - r.top;
        const f = Math.exp(-ev.deltaY * 0.0012);
        const k2 = Math.min(Math.max(st.tf.k * f, 0.3), 3.2);
        st.tf.x = mx - (mx - st.tf.x) * (k2 / st.tf.k);
        st.tf.y = my - (my - st.tf.y) * (k2 / st.tf.k);
        st.tf.k = k2; st.userMoved = true;
      };
      canvas.addEventListener('mousemove', onMove);
      canvas.addEventListener('mousedown', onDown);
      window.addEventListener('mouseup', onUp);
      canvas.addEventListener('wheel', onWheel, { passive: false });
      return () => {
        stopped = true; cancelAnimationFrame(raf); clearTimeout(fallback); if (ticker) clearInterval(ticker);
        ro.disconnect();
        canvas.removeEventListener('mousemove', onMove);
        canvas.removeEventListener('mousedown', onDown);
        window.removeEventListener('mouseup', onUp);
        canvas.removeEventListener('wheel', onWheel);
      };
    }, [graph, C]);

    const selNode = selected ? graph.byId[selected] : null;
    const QUERIES = [
      ['footprint(checkout)', 'feat:checkout'],
      ['blast-radius(Money)', 'c:money'],
      ['rationale-trace(refund decider)', 'wu:wu-checkout-refund-decider-0007'],
      ['distance(What 2.0)', 'tgt:what-2'],
      ['reify(single-select)', 'aio:single-select'],
    ];

    return (
      <div ref={wrapRef} style={{ position: 'absolute', inset: 0, overflow: 'hidden' }}>
        <canvas ref={canvasRef} style={{ position: 'absolute', inset: 0 }} />

        {/* header strip */}
        <div style={{ position: 'absolute', top: 12, left: 16, right: 16, display: 'flex', gap: 10,
          alignItems: 'flex-start', pointerEvents: 'none', flexWrap: 'wrap' }}>
          <div style={{ pointerEvents: 'auto' }}>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, letterSpacing: '.16em',
              textTransform: 'uppercase', color: 'var(--slate-300)' }}>one graph · §2 / §9</div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--slate-500)', marginTop: 3 }}>
              {graph.nodes.length} nodes · {graph.edges.length} typed edges — all derived, none authored
            </div>
            <div style={{ display: 'flex', gap: 6, marginTop: 8, flexWrap: 'wrap' }}>
              {QUERIES.map(([q, id]) => (
                <button key={id} onClick={() => setSelected(s => s === id ? null : id)} style={{
                  cursor: 'pointer', fontFamily: 'var(--font-mono)', fontSize: 9.5, padding: '3px 9px',
                  background: selected === id ? 'var(--slate-700)' : 'var(--slate-800)',
                  color: selected === id ? 'var(--slate-100)' : 'var(--slate-400)',
                  border: `1px solid ${selected === id ? 'var(--blue-400)' : 'var(--slate-600)'}`, borderRadius: 4 }}>
                  {q}</button>
              ))}
              {selected && (
                <button onClick={() => setSelected(null)} style={{ cursor: 'pointer', fontFamily: 'var(--font-mono)',
                  fontSize: 9.5, padding: '3px 9px', background: 'transparent', color: 'var(--slate-500)',
                  border: '1px dashed var(--slate-600)', borderRadius: 4 }}>× clear</button>
              )}
            </div>
          </div>
        </div>

        {/* selection card */}
        {selNode && cone && (
          <div style={{ position: 'absolute', right: 14, top: 12, width: 252, background: 'var(--slate-800)',
            border: '1px solid var(--slate-600)', borderRadius: 8, padding: '12px 14px', boxShadow: 'var(--shadow-graph)' }}>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 8.5, fontWeight: 600, letterSpacing: '.13em',
              textTransform: 'uppercase', color: 'var(--slate-500)' }}>{selNode.kind.replace('concept-', '')}</div>
            <div style={{ fontFamily: 'var(--font-sans)', fontWeight: 700, fontSize: 15, color: 'var(--slate-100)', marginTop: 2 }}>{selNode.label}</div>
            <div style={{ display: 'grid', gap: 4, marginTop: 9, fontFamily: 'var(--font-mono)', fontSize: 10 }}>
              <div><span style={{ color: 'var(--blue-400)' }}>▲ derives from</span>
                <span style={{ color: 'var(--slate-300)' }}> {cone.upN} node{cone.upN === 1 ? '' : 's'} upstream</span></div>
              <div><span style={{ color: 'var(--em-view)' }}>▼ blast radius</span>
                <span style={{ color: 'var(--slate-300)' }}> {cone.downN} node{cone.downN === 1 ? '' : 's'} downstream</span></div>
            </div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, color: 'var(--slate-500)', marginTop: 8, lineHeight: 1.5 }}>
              the cone is a pure traversal — change this node and everything lit is in scope.
            </div>
            {selNode.view && (
              <button onClick={() => onOpen(selNode.view)} style={{ marginTop: 9, cursor: 'pointer', width: '100%',
                fontFamily: 'var(--font-mono)', fontSize: 10, padding: '6px 10px', background: 'var(--blue-600)',
                border: '1px solid var(--blue-500)', color: '#fff', borderRadius: 5 }}>
                open in {selNode.view} {'\u2192'}</button>
            )}
          </div>
        )}

        {/* footer note */}
        <div style={{ position: 'absolute', left: 16, bottom: 12, fontFamily: 'var(--font-mono)', fontSize: 9.5,
          color: 'var(--slate-500)', maxWidth: 520, lineHeight: 1.5, pointerEvents: 'none' }}>
          “describe this system” is a query, not a stale document — impact analysis is a graph traversal.
          scroll to zoom · drag to pan · click a node to trace its cone.
        </div>
      </div>
    );
  }

  Object.assign(window, { GraphView });
})();
