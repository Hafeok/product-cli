/* global React */
/* Shared graph primitives for the product-cli "What" views.
   Everything here is exported to window.PFUI so the per-view Babel
   scripts (each its own scope) can use it. */
(function () {
  const { useState } = React;

  // ---- geometry: where an edge should touch a box -------------------------
  // Given a box {x,y,w,h} (x,y = center) and a target point, return the point on
  // the box perimeter along the line to the target.
  function anchor(box, tx, ty) {
    const dx = tx - box.x, dy = ty - box.y;
    if (dx === 0 && dy === 0) return { x: box.x, y: box.y };
    const hw = box.w / 2, hh = box.h / 2;
    const sx = dx === 0 ? Infinity : hw / Math.abs(dx);
    const sy = dy === 0 ? Infinity : hh / Math.abs(dy);
    const s = Math.min(sx, sy);
    return { x: box.x + dx * s, y: box.y + dy * s };
  }

  // cubic path between two anchors; tangent follows the dominant axis
  function curve(a, b) {
    const dx = b.x - a.x, dy = b.y - a.y;
    if (Math.abs(dy) >= Math.abs(dx)) {
      const my = (a.y + b.y) / 2;
      return `M${a.x},${a.y} C${a.x},${my} ${b.x},${my} ${b.x},${b.y}`;
    }
    const mx = (a.x + b.x) / 2;
    return `M${a.x},${a.y} C${mx},${a.y} ${mx},${b.y} ${b.x},${b.y}`;
  }

  // orthogonal elbow between two anchors — straight segments, small rounded knees
  function elbow(a, b, r = 10) {
    const dx = b.x - a.x, dy = b.y - a.y;
    if (Math.abs(dx) < 1 || Math.abs(dy) < 1) return `M${a.x},${a.y} L${b.x},${b.y}`;
    if (Math.abs(dy) >= Math.abs(dx)) {
      const my = (a.y + b.y) / 2;
      const rr = Math.min(r, Math.abs(dx) / 2, Math.abs(my - a.y), Math.abs(b.y - my));
      const sx = Math.sign(b.x - a.x), sy1 = Math.sign(my - a.y), sy2 = Math.sign(b.y - my);
      return `M${a.x},${a.y} L${a.x},${my - sy1 * rr}` +
        ` Q${a.x},${my} ${a.x + sx * rr},${my} L${b.x - sx * rr},${my}` +
        ` Q${b.x},${my} ${b.x},${my + sy2 * rr} L${b.x},${b.y}`;
    }
    const mx = (a.x + b.x) / 2;
    const rr = Math.min(r, Math.abs(dy) / 2, Math.abs(mx - a.x), Math.abs(b.x - mx));
    const sy = Math.sign(b.y - a.y), sx1 = Math.sign(mx - a.x), sx2 = Math.sign(b.x - mx);
    return `M${a.x},${a.y} L${mx - sx1 * rr},${a.y}` +
      ` Q${mx},${a.y} ${mx},${a.y + sy * rr} L${mx},${b.y - sy * rr}` +
      ` Q${mx},${b.y} ${mx + sx2 * rr},${b.y} L${b.x},${b.y}`;
  }

  // ---- the SVG edge layer -------------------------------------------------
  // edges: [{ from, to, label, stroke, dash, width, opacity, arrow, marker, dim }]
  // pos:   { id -> {x,y,w,h} }  (x,y = node center)
  function EdgeLayer({ edges, pos, width, height, showLabels = true, ortho = false }) {
    return (
      <svg width={width} height={height}
        style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'visible' }}>
        <defs>
          <marker id="pf-arr" viewBox="0 0 10 10" refX="8.5" refY="5" markerWidth="6.5" markerHeight="6.5" orient="auto-start-reverse">
            <path d="M0,0 L10,5 L0,10 z" fill="var(--slate-500)" />
          </marker>
          <marker id="pf-arr-mag" viewBox="0 0 10 10" refX="8.5" refY="5" markerWidth="6.5" markerHeight="6.5" orient="auto-start-reverse">
            <path d="M0,0 L10,5 L0,10 z" fill="var(--em-bridge)" />
          </marker>
        </defs>
        {edges.map((e, i) => {
          const A = pos[e.from], B = pos[e.to];
          if (!A || !B) return null;
          const a = anchor(A, B.x, B.y);
          const b = anchor(B, A.x, A.y);
          const useOrtho = e.ortho != null ? e.ortho : ortho;
          const d = useOrtho ? elbow(a, b) : curve(a, b);
          const mx = (a.x + b.x) / 2, my = (a.y + b.y) / 2;
          const marker = e.marker === 'mag' ? 'url(#pf-arr-mag)' : 'url(#pf-arr)';
          return (
            <g key={i} style={{ transition: 'opacity 140ms var(--ease)', opacity: e.opacity == null ? 1 : e.opacity }}>
              <path d={d} fill="none"
                stroke={e.stroke || 'var(--slate-600)'} strokeWidth={e.width || 1.5}
                strokeDasharray={e.dash || 'none'}
                markerEnd={e.arrow === false ? undefined : marker} />
              {showLabels && e.label && (
                <g>
                  <rect x={mx - e.label.length * 3.1 - 5} y={my - 8} rx="3"
                    width={e.label.length * 6.2 + 10} height={16}
                    fill="var(--slate-900)" opacity="0.86" />
                  <text x={mx} y={my + 3.5} textAnchor="middle"
                    style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, letterSpacing: '.04em',
                      fill: e.labelColor || 'var(--slate-400)' }}>{e.label}</text>
                </g>
              )}
            </g>
          );
        })}
      </svg>
    );
  }

  // ---- a conformance dot --------------------------------------------------
  const CONF = {
    described: 'var(--conf-described)', realised: 'var(--conf-realised)',
    verified: 'var(--conf-verified)', delivered: 'var(--conf-delivered)',
  };
  function ConfDot({ level, size = 9 }) {
    const c = CONF[level] || CONF.described;
    return <span title={'conformance: ' + level} style={{
      width: size, height: size, borderRadius: '50%', background: c, flex: 'none',
      display: 'inline-block', boxShadow: `0 0 0 3px color-mix(in srgb, ${c} 22%, transparent)`,
    }} />;
  }

  // ---- the shared right-hand detail panel --------------------------------
  function DetailPanel({ title, kindTag, rows, onClose, children }) {
    return (
      <aside style={{
        position: 'absolute', top: 12, right: 12, bottom: 12, width: 300,
        background: 'var(--slate-800)', border: '1px solid var(--slate-600)',
        borderRadius: 8, padding: 16, boxShadow: 'var(--shadow-graph)', zIndex: 8,
        overflow: 'auto',
      }}>
        {kindTag}
        <h2 style={{ margin: '8px 0 2px', fontSize: 16, fontFamily: 'var(--font-sans)', color: 'var(--slate-100)' }}>{title}</h2>
        {rows && (
          <dl style={{ margin: '10px 0 0' }}>
            {rows.map((r, i) => r && (
              <div key={i} style={{ marginTop: 10 }}>
                <dt style={{ color: 'var(--slate-400)', fontSize: 10, fontFamily: 'var(--font-mono)',
                  letterSpacing: '.12em', textTransform: 'uppercase' }}>{r.k}</dt>
                {(Array.isArray(r.v) ? r.v : [r.v]).map((t, j) => (
                  <dd key={j} style={{ margin: '3px 0 0', fontSize: 12, fontFamily: 'var(--font-mono)',
                    color: r.color || 'var(--slate-200)', lineHeight: 1.45 }}>{t}</dd>
                ))}
              </div>
            ))}
          </dl>
        )}
        {children}
        <button onClick={onClose} style={{
          marginTop: 16, background: 'none', border: '1px solid var(--slate-600)',
          color: 'var(--slate-300)', borderRadius: 5, padding: '5px 12px', cursor: 'pointer',
          fontFamily: 'var(--font-mono)', fontSize: 11,
        }}>× close</button>
      </aside>
    );
  }

  // ---- hover bookkeeping hook --------------------------------------------
  function useHover() {
    const [h, setH] = useState(null);
    return [h, setH];
  }

  // ---- fit-to-view: scale a fixed canvas to fit its container (contain) ----
  function FitCanvas({ width, height, children }) {
    const ref = React.useRef(null);
    const [scale, setScale] = useState(1);
    React.useEffect(() => {
      const el = ref.current; if (!el) return;
      const measure = () => {
        const r = el.getBoundingClientRect();
        const pad = 28;
        const s = Math.min((r.width - pad) / width, (r.height - pad) / height, 1.15);
        setScale(s > 0 ? s : 1);
      };
      const ro = new ResizeObserver(measure);
      ro.observe(el); measure();
      return () => ro.disconnect();
    }, [width, height]);
    return (
      <div ref={ref} style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'center',
        justifyContent: 'center', overflow: 'hidden' }}>
        <div style={{ width, height, position: 'relative', flex: 'none',
          transform: `scale(${scale})`, transformOrigin: 'center center' }}>
          {children}
        </div>
      </div>
    );
  }

  Object.assign(window, { PFUI: { EdgeLayer, anchor, curve, ConfDot, DetailPanel, useHover, CONF, FitCanvas } });
})();
