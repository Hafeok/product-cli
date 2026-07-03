/* global React, PF, PFUI */
/* Domain graph — the Ordering bounded context as an ER-style graph.
   Entities / aggregates / value-objects / invariants are colour-coded; edges
   are relations (has-many, created-from, identity) and invariant governance. */
(function () {
  const { useMemo } = React;
  const { EdgeLayer, ConfDot, FitCanvas } = window.PFUI;

  const COLOR = {
    aggregate: 'var(--kind-entity)', entity: 'var(--blue-500)',
    'value-object': 'var(--blue-400)', invariant: 'var(--kind-invariant)',
    external: 'var(--slate-400)', reference: 'var(--em-event)',
  };
  const KIND_LABEL = {
    aggregate: 'aggregate', entity: 'entity', 'value-object': 'value object',
    invariant: 'invariant', external: 'external · Catalog', reference: 'reference data',
  };

  // Data-driven layout: one column per kind (aggregates → entities → value
  // objects → invariants → …), nodes stacked within, sized to their fields. Lays
  // out any bounded context, not curated coordinates.
  const KIND_ORDER = ['aggregate', 'entity', 'value-object', 'invariant', 'reference', 'external'];

  function nodeH(n) {
    const fields = (n.fields || []).length;
    return (n.kind === 'invariant' ? 40 : 46) + fields * 15;
  }

  function placeByKind(nodes) {
    const cols = KIND_ORDER.filter(k => nodes.some(n => n.kind === k));
    const W = 200, GAP = 44, X0 = 40, Y0 = 46, VGAP = 20;
    const pos = {}; let maxY = Y0;
    cols.forEach((k, ci) => {
      const x = X0 + ci * (W + GAP) + W / 2;
      let y = Y0;
      nodes.filter(n => n.kind === k).forEach(n => {
        const h = nodeH(n);
        pos[n.id] = { x, y: y + h / 2, w: W, h };
        y += h + VGAP;
      });
      maxY = Math.max(maxY, y);
    });
    const width = Math.max(X0 * 2 + cols.length * (W + GAP) - GAP, 640);
    return { pos, width, height: Math.max(maxY + 40, 320) };
  }

  function DomainNode({ n, selected, onClick, showConf }) {
    const c = COLOR[n.kind] || 'var(--slate-400)';
    const isInv = n.kind === 'invariant';
    const isVO = n.kind === 'value-object';
    return (
      <div onClick={onClick} style={{
        cursor: 'pointer', boxSizing: 'border-box', width: '100%', height: '100%',
        background: isInv ? 'var(--slate-900)' : `color-mix(in srgb, ${c} 13%, var(--slate-900))`,
        border: `${isInv || isVO ? '1.5px dashed' : '1.5px solid'} ${selected ? 'var(--blue-400)' : c}`,
        borderRadius: isVO ? 12 : 6,
        boxShadow: selected ? '0 0 0 3px color-mix(in srgb, var(--blue-400) 30%, transparent)' : 'none',
        overflow: 'hidden',
      }}>
        <div style={{ padding: '7px 11px 6px', borderBottom: isInv ? 'none' : '1px solid var(--slate-800)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, fontWeight: 600, letterSpacing: '.12em',
              textTransform: 'uppercase', color: c }}>{KIND_LABEL[n.kind]}</span>
            {showConf && n.kind === 'invariant' && (
              <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 8, color: 'var(--conf-verified)' }}>executable ✓</span>
            )}
          </div>
          <div style={{ fontFamily: isInv ? 'var(--font-mono)' : 'var(--font-sans)', fontWeight: isInv ? 600 : 700,
            fontSize: isInv ? 12 : 14, color: 'var(--slate-100)', marginTop: 1 }}>{n.label}</div>
        </div>
        <div style={{ padding: '6px 11px 8px' }}>
          {n.fields.map((f, i) => (
            <div key={i} style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, color: isInv ? c : 'var(--slate-400)', lineHeight: 1.55 }}>{f}</div>
          ))}
        </div>
      </div>
    );
  }

  function Legend() {
    const items = [['aggregate', COLOR.aggregate], ['entity', COLOR.entity], ['value object', COLOR['value-object']],
      ['invariant', COLOR.invariant], ['reference data', COLOR.reference], ['external context', COLOR.external]];
    return (
      <div style={{ position: 'absolute', left: 16, bottom: 14, display: 'flex', gap: 13, flexWrap: 'wrap',
        background: 'color-mix(in srgb, var(--slate-900) 80%, transparent)', padding: '7px 12px', borderRadius: 6,
        border: '1px solid var(--slate-800)', zIndex: 5 }}>
        {items.map(([label, c]) => (
          <span key={label} style={{ display: 'flex', alignItems: 'center', gap: 6, fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-400)' }}>
            <span style={{ width: 11, height: 11, borderRadius: 3, background: `color-mix(in srgb, ${c} 22%, transparent)`, border: `1.5px solid ${c}` }} />{label}
          </span>
        ))}
      </div>
    );
  }

  function DomainGraph({ selected, onSelect, showConf, showLabels }) {
    const D = PF.domain || { nodes: [], edges: [], contextId: '' };
    const { pos, width: CANVAS_W, height: CANVAS_H } = useMemo(() => placeByKind(D.nodes), [D.contextId, D.nodes.length]);

    const edges = D.edges.map(e => {
      const base = { from: e.from, to: e.to, label: showLabels ? (e.label + (e.card ? '  ' + e.card : '')) : null };
      if (e.kind === 'inv') return { ...base, stroke: 'var(--kind-invariant)', dash: '4 4', width: 1.4, labelColor: 'var(--em-trigger-soft)' };
      if (e.kind === 'cross') return { ...base, stroke: 'var(--em-bridge)', dash: '5 4', width: 1.5, marker: 'mag', labelColor: 'var(--em-bridge)' };
      if (e.kind === 'ref') return { ...base, stroke: 'var(--em-event)', dash: '3 4', width: 1.3, labelColor: 'var(--em-event)' };
      return { ...base, stroke: 'var(--slate-500)', width: 1.6 };
    });

    const ctx = (PF.domains || []).find(d => d.id === D.contextId);

    return (
      <FitCanvas width={CANVAS_W} height={CANVAS_H}>
          {/* context frame label */}
          <div style={{ position: 'absolute', left: 16, top: 12, display: 'flex', alignItems: 'center', gap: 9, zIndex: 5 }}>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, fontWeight: 700, letterSpacing: '.1em',
              textTransform: 'uppercase', color: 'var(--kind-entity)' }}>{(ctx && ctx.name) || D.contextId || 'domain'}</span>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--slate-500)' }}>bounded context · §3.1 structure &amp; data</span>
            {showConf && ctx && <ConfDot level={ctx.conformance} size={8} />}
          </div>

          <EdgeLayer edges={edges} pos={pos} width={CANVAS_W} height={CANVAS_H} showLabels={showLabels} />

          {D.nodes.map(n => pos[n.id] && (
            <div key={n.id} style={{ position: 'absolute', left: pos[n.id].x - pos[n.id].w / 2, top: pos[n.id].y - pos[n.id].h / 2,
              width: pos[n.id].w, height: pos[n.id].h, zIndex: 2 }}>
              <DomainNode n={n} selected={selected === n.id} onClick={() => onSelect(n.id)} showConf={showConf} />
            </div>
          ))}

          <Legend />
      </FitCanvas>
    );
  }

  Object.assign(window, { DomainGraph });
})();
