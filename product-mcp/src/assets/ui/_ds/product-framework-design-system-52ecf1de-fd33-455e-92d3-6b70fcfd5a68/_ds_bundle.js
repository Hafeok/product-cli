/* @ds-bundle: {"format":3,"namespace":"ProductFrameworkDesignSystem_52ecf1","components":[{"name":"Button","sourcePath":"components/core/Button.jsx"},{"name":"Card","sourcePath":"components/core/Card.jsx"},{"name":"ConformanceBadge","sourcePath":"components/core/ConformanceBadge.jsx"},{"name":"StatePill","sourcePath":"components/core/StatePill.jsx"},{"name":"Tag","sourcePath":"components/core/Tag.jsx"},{"name":"EMNode","sourcePath":"components/graph/EMNode.jsx"},{"name":"PhaseStepper","sourcePath":"components/graph/PhaseStepper.jsx"}],"sourceHashes":{"components/core/Button.jsx":"465bdcda8e97","components/core/Card.jsx":"6ac3d835455b","components/core/ConformanceBadge.jsx":"94c282621344","components/core/StatePill.jsx":"e1d7719d529b","components/core/Tag.jsx":"d749e0b61049","components/graph/EMNode.jsx":"de0357088e7d","components/graph/PhaseStepper.jsx":"1faa4b765bf5","ui_kits/aio-renderer/RendererApp.jsx":"f312db627035","ui_kits/aio-renderer/contract.js":"bed9367cab5d","ui_kits/what-graph/WhatGraphApp.jsx":"b8fc6d09bf18","ui_kits/what-graph/data.js":"c899dc1f8060"},"inlinedExternals":[],"unexposedExports":[]} */

(() => {

const __ds_ns = (window.ProductFrameworkDesignSystem_52ecf1 = window.ProductFrameworkDesignSystem_52ecf1 || {});

const __ds_scope = {};

(__ds_ns.__errors = __ds_ns.__errors || []);

// components/core/Button.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/**
 * Button — the primary action control.
 *
 * Visual language: drawn rule (1.5px border), tight 5px radius, IBM Plex Sans
 * 600. Mechanical press (translateY(1px)) — never a scale-bounce. The optional
 * `kicker` renders a tiny uppercase mono eyebrow inside the button, echoing the
 * generic renderer's `trigger-action · cmd-…` labelling.
 */
function Button({
  children,
  variant = 'secondary',
  // 'primary' | 'secondary' | 'ghost' | 'danger'
  size = 'md',
  // 'sm' | 'md' | 'lg'
  icon = null,
  iconRight = null,
  kicker = null,
  fullWidth = false,
  disabled = false,
  as = 'button',
  style: styleProp,
  ...rest
}) {
  const [pressed, setPressed] = React.useState(false);
  const Tag = as;
  const sizes = {
    sm: {
      fontSize: '12px',
      padding: '6px 12px',
      gap: '6px'
    },
    md: {
      fontSize: '14px',
      padding: '9px 16px',
      gap: '8px'
    },
    lg: {
      fontSize: '15px',
      padding: '12px 20px',
      gap: '9px'
    }
  };
  const variants = {
    primary: {
      background: 'var(--accent)',
      color: 'var(--accent-text)',
      border: 'var(--bw) solid var(--accent)'
    },
    secondary: {
      background: 'var(--surface)',
      color: 'var(--accent)',
      border: 'var(--bw) solid var(--accent)'
    },
    ghost: {
      background: 'transparent',
      color: 'var(--text-muted)',
      border: 'var(--bw) solid var(--border-soft)'
    },
    danger: {
      background: 'var(--surface)',
      color: 'var(--error)',
      border: 'var(--bw) solid var(--error)'
    }
  };
  const style = {
    display: 'inline-flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    boxSizing: 'border-box',
    fontFamily: 'var(--font-sans)',
    fontWeight: 600,
    lineHeight: 1.1,
    letterSpacing: '0.01em',
    borderRadius: 'var(--radius)',
    cursor: disabled ? 'not-allowed' : 'pointer',
    width: fullWidth ? '100%' : 'auto',
    opacity: disabled ? 0.45 : 1,
    transition: 'transform var(--dur-fast) var(--ease), filter var(--dur-fast) var(--ease)',
    transform: pressed && !disabled ? 'var(--press-translate)' : 'none',
    userSelect: 'none',
    ...sizes[size],
    ...variants[variant],
    ...styleProp
  };
  const rowStyle = {
    display: 'inline-flex',
    alignItems: 'center',
    gap: sizes[size].gap
  };
  return /*#__PURE__*/React.createElement(Tag, _extends({
    style: style,
    disabled: as === 'button' ? disabled : undefined,
    onMouseDown: () => !disabled && setPressed(true),
    onMouseUp: () => setPressed(false),
    onMouseLeave: () => setPressed(false),
    onMouseEnter: e => {
      if (!disabled) e.currentTarget.style.filter = 'brightness(0.96)';
    }
  }, rest), kicker && /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: '8.5px',
      fontWeight: 500,
      letterSpacing: 'var(--tracking-label)',
      textTransform: 'uppercase',
      opacity: 0.7,
      marginBottom: '2px'
    }
  }, kicker), /*#__PURE__*/React.createElement("span", {
    style: rowStyle
  }, icon && /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-flex'
    }
  }, icon), children, iconRight && /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-flex'
    }
  }, iconRight)));
}
Object.assign(__ds_scope, { Button });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Button.jsx", error: String((e && e.message) || e) }); }

// components/core/Card.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/**
 * Card — a surface container.
 *
 * Two languages, matching the system's two surfaces:
 *  • `elevation` (default) — thin hairline border + soft shadow.
 *  • `draft` — the blueprint signature: a 1.5px ink border with a hard,
 *    un-blurred offset shadow in accent-wash. Reads like ink on drafting paper.
 * An optional `eyebrow` renders the mono uppercase label; `accent` paints a
 * left rule in any Event-Modeling colour.
 */
function Card({
  children,
  variant = 'elevation',
  // 'elevation' | 'draft' | 'flat'
  eyebrow = null,
  title = null,
  accent = null,
  // a CSS colour for the left rule
  padding = '18px 20px',
  style: styleProp,
  ...rest
}) {
  const variants = {
    elevation: {
      background: 'var(--surface)',
      border: 'var(--bw-hair) solid var(--border-soft)',
      boxShadow: 'var(--shadow-2)',
      borderRadius: 'var(--radius-md)'
    },
    draft: {
      background: 'var(--surface)',
      border: 'var(--bw) solid var(--ink)',
      boxShadow: 'var(--shadow-draft)',
      borderRadius: 'var(--radius-md)'
    },
    flat: {
      background: 'var(--surface-2)',
      border: 'var(--bw-hair) solid var(--border-soft)',
      boxShadow: 'none',
      borderRadius: 'var(--radius-md)'
    }
  };
  const style = {
    position: 'relative',
    boxSizing: 'border-box',
    overflow: 'hidden',
    ...variants[variant],
    ...(accent ? {
      borderLeft: `3px solid ${accent}`
    } : null),
    ...styleProp
  };
  return /*#__PURE__*/React.createElement("div", _extends({
    style: style
  }, rest), /*#__PURE__*/React.createElement("div", {
    style: {
      padding
    }
  }, eyebrow && /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: '10px',
      fontWeight: 600,
      letterSpacing: 'var(--tracking-label)',
      textTransform: 'uppercase',
      color: 'var(--text-muted)',
      marginBottom: title ? '4px' : '10px'
    }
  }, eyebrow), title && /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-sans)',
      fontWeight: 600,
      fontSize: '16px',
      color: 'var(--text)',
      marginBottom: '10px'
    }
  }, title), children));
}
Object.assign(__ds_scope, { Card });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Card.jsx", error: String((e && e.message) || e) }); }

// components/core/ConformanceBadge.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/**
 * ConformanceBadge — the cumulative conformance level.
 *
 * The framework claims the highest level it satisfies: Described → Realised →
 * Verified → Delivered. Renders the named level with its colour and an optional
 * ladder of filled/empty rungs showing progress.
 */
const LEVELS = [{
  key: 'described',
  n: 1,
  label: 'Described',
  c: 'var(--conf-described)'
}, {
  key: 'realised',
  n: 2,
  label: 'Realised',
  c: 'var(--conf-realised)'
}, {
  key: 'verified',
  n: 3,
  label: 'Verified',
  c: 'var(--conf-verified)'
}, {
  key: 'delivered',
  n: 4,
  label: 'Delivered',
  c: 'var(--conf-delivered)'
}];
function ConformanceBadge({
  level = 'described',
  showLadder = true,
  style: styleProp,
  ...rest
}) {
  const idx = LEVELS.findIndex(l => l.key === level);
  const cur = LEVELS[Math.max(0, idx)];
  const style = {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '9px',
    padding: '5px 11px 5px 9px',
    borderRadius: 'var(--radius)',
    border: `var(--bw) solid ${cur.c}`,
    background: 'var(--surface)',
    boxSizing: 'border-box',
    ...styleProp
  };
  return /*#__PURE__*/React.createElement("span", _extends({
    style: style
  }, rest), showLadder && /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-flex',
      gap: '3px',
      alignItems: 'center'
    }
  }, LEVELS.map((l, i) => /*#__PURE__*/React.createElement("span", {
    key: l.key,
    style: {
      width: '6px',
      height: '14px',
      borderRadius: '1px',
      background: i <= idx ? l.c : 'var(--border-soft)'
    }
  }))), /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-flex',
      flexDirection: 'column',
      lineHeight: 1.1
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: '8.5px',
      fontWeight: 500,
      letterSpacing: 'var(--tracking-label)',
      textTransform: 'uppercase',
      color: 'var(--text-muted)'
    }
  }, "level ", cur.n, " / 4"), /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-sans)',
      fontSize: '13px',
      fontWeight: 600,
      color: 'var(--text)',
      marginTop: '1px'
    }
  }, cur.label)));
}
Object.assign(__ds_scope, { ConformanceBadge });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/ConformanceBadge.jsx", error: String((e && e.message) || e) }); }

// components/core/StatePill.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/**
 * StatePill — a projection / session state indicator.
 *
 * Mirrors the framework's vocabulary of states: projection states
 * (present / empty / loading / failed), session states (draft / finalized),
 * and verification verdicts (verified / divergent). A leading dot carries the
 * colour; `loading` spins.
 */
function StatePill({
  state = 'present',
  label,
  style: styleProp,
  ...rest
}) {
  const map = {
    present: {
      c: 'var(--ok)',
      text: 'present'
    },
    empty: {
      c: 'var(--text-muted)',
      text: 'empty'
    },
    loading: {
      c: 'var(--warn-bg)',
      text: 'loading'
    },
    failed: {
      c: 'var(--error)',
      text: 'failed'
    },
    draft: {
      c: 'var(--warn-bg)',
      text: 'draft'
    },
    finalized: {
      c: 'var(--ok)',
      text: 'finalized'
    },
    verified: {
      c: 'var(--ok)',
      text: 'verified'
    },
    divergent: {
      c: 'var(--error)',
      text: 'divergent'
    }
  };
  const s = map[state] || map.present;
  const spinning = state === 'loading';
  const style = {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '6px',
    fontFamily: 'var(--font-mono)',
    fontWeight: 600,
    fontSize: '10px',
    letterSpacing: 'var(--tracking-wide)',
    textTransform: 'uppercase',
    color: 'var(--text)',
    padding: '3px 9px 3px 8px',
    border: 'var(--bw-hair) solid var(--border-soft)',
    borderRadius: 'var(--radius-pill)',
    background: 'var(--surface)',
    whiteSpace: 'nowrap',
    ...styleProp
  };
  const dot = {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    background: s.c,
    flex: 'none',
    boxShadow: spinning ? 'none' : `0 0 0 3px color-mix(in srgb, ${s.c} 20%, transparent)`,
    border: spinning ? `1.5px solid color-mix(in srgb, ${s.c} 35%, transparent)` : 'none',
    borderTopColor: spinning ? s.c : undefined,
    animation: spinning ? 'pf-spin 0.8s linear infinite' : 'none',
    boxSizing: 'border-box'
  };
  return /*#__PURE__*/React.createElement("span", _extends({
    style: style
  }, rest), /*#__PURE__*/React.createElement("span", {
    style: dot
  }), label || s.text, /*#__PURE__*/React.createElement("style", null, '@keyframes pf-spin{to{transform:rotate(360deg)}}'));
}
Object.assign(__ds_scope, { StatePill });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/StatePill.jsx", error: String((e && e.message) || e) }); }

// components/core/Tag.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/**
 * Tag — a small uppercase mono label/chip.
 *
 * The system's most-used micro-component: AIO type labels, WCAG codes, section
 * eyebrows, identifier pills. Defaults to the schematic outline look from the
 * generic renderer (`<span class="typ">`). Use `tone` for a coloured wash.
 */
function Tag({
  children,
  tone = 'neutral',
  // 'neutral' | 'accent' | 'command' | 'view' | 'event' | 'trigger' | 'bridge'
  solid = false,
  size = 'md',
  // 'sm' | 'md'
  style: styleProp,
  ...rest
}) {
  const tones = {
    neutral: 'var(--text-muted)',
    accent: 'var(--accent)',
    command: 'var(--em-command)',
    view: 'var(--em-view)',
    event: 'var(--em-event-deep)',
    trigger: 'var(--em-trigger)',
    bridge: 'var(--em-bridge)'
  };
  const c = tones[tone] || tones.neutral;
  const base = {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '5px',
    fontFamily: 'var(--font-mono)',
    fontWeight: 600,
    fontSize: size === 'sm' ? '9px' : '10px',
    letterSpacing: 'var(--tracking-label)',
    textTransform: 'uppercase',
    padding: size === 'sm' ? '1px 5px' : '2px 7px',
    borderRadius: 'var(--radius-xs)',
    lineHeight: 1.4,
    whiteSpace: 'nowrap'
  };
  const style = solid ? {
    ...base,
    background: c,
    color: 'var(--white)',
    border: `var(--bw-hair) solid ${c}`,
    ...styleProp
  } : {
    ...base,
    color: c,
    background: 'transparent',
    border: `var(--bw-hair) solid var(--border-soft)`,
    ...styleProp
  };
  return /*#__PURE__*/React.createElement("span", _extends({
    style: style
  }, rest), children);
}
Object.assign(__ds_scope, { Tag });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Tag.jsx", error: String((e && e.message) || e) }); }

// components/graph/EMNode.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/**
 * EMNode — an Event-Modeling node chip. The signature component.
 *
 * Every construct in a What-graph has a fixed colour (the load-bearing
 * semantic palette). Commands, views, events and triggers are solid fills with
 * dark ink text; a UI step is a dashed outline (it is authored meaning, not a
 * derived fact). Hovering brightens; clicking is supported via `onClick`.
 */
const KINDS = {
  command: {
    c: 'var(--em-command)',
    fill: true,
    abbr: 'CMD'
  },
  view: {
    c: 'var(--em-view)',
    fill: true,
    abbr: 'VIEW'
  },
  'read-model': {
    c: 'var(--em-view)',
    fill: true,
    abbr: 'VIEW'
  },
  event: {
    c: 'var(--em-event)',
    fill: true,
    abbr: 'EVT'
  },
  trigger: {
    c: 'var(--em-trigger)',
    fill: true,
    abbr: 'TRIG'
  },
  'ui-step': {
    c: 'var(--slate-400)',
    fill: false,
    abbr: 'UI'
  }
};
function EMNode({
  kind = 'command',
  label,
  note = null,
  // a secondary id/caption line
  selected = false,
  showKind = true,
  onClick,
  style: styleProp,
  ...rest
}) {
  const [hover, setHover] = React.useState(false);
  const k = KINDS[kind] || KINDS.command;
  const base = {
    display: 'inline-flex',
    flexDirection: 'column',
    gap: '1px',
    minWidth: '150px',
    maxWidth: '210px',
    boxSizing: 'border-box',
    padding: '7px 11px',
    borderRadius: 'var(--radius)',
    fontFamily: 'var(--font-sans)',
    cursor: onClick ? 'pointer' : 'default',
    transition: 'filter var(--dur-fast) var(--ease), box-shadow var(--dur-fast) var(--ease)',
    filter: hover ? 'brightness(1.08)' : 'none',
    boxShadow: selected ? '0 0 0 2px var(--bg), 0 0 0 4px var(--focus-ring)' : 'none'
  };
  const fillStyle = k.fill ? {
    background: k.c,
    border: `var(--bw) solid ${k.c}`,
    color: '#0b1120'
  } : {
    background: 'var(--surface-sunken)',
    border: `var(--bw) dashed ${k.c}`,
    color: 'var(--text)'
  };
  const kindColor = k.fill ? 'rgba(11,17,32,0.62)' : k.c;
  return /*#__PURE__*/React.createElement("div", _extends({
    style: {
      ...base,
      ...fillStyle,
      ...styleProp
    },
    onMouseEnter: () => setHover(true),
    onMouseLeave: () => setHover(false),
    onClick: onClick
  }, rest), showKind && /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: '8px',
      fontWeight: 600,
      letterSpacing: 'var(--tracking-label)',
      textTransform: 'uppercase',
      color: kindColor
    }
  }, k.abbr), /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: '12px',
      fontWeight: 600,
      lineHeight: 1.2
    }
  }, label), note && /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: '9.5px',
      color: k.fill ? 'rgba(11,17,32,0.55)' : 'var(--text-muted)'
    }
  }, note));
}
Object.assign(__ds_scope, { EMNode });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/graph/EMNode.jsx", error: String((e && e.message) || e) }); }

// components/graph/PhaseStepper.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/**
 * PhaseStepper — the What › How › Build progress banner.
 *
 * Straight from the live view's session phase bar. Past phases read "done"
 * (green), the current phase is filled in its phase colour, and phases beyond
 * the session's cap are dimmed. Pills are chevron-separated.
 */
const PHASES = [{
  key: 'what',
  label: 'What',
  c: 'var(--em-command)'
}, {
  key: 'how',
  label: 'How',
  c: 'var(--em-event-deep)'
}, {
  key: 'build',
  label: 'Build',
  c: 'var(--em-view)'
}];
function PhaseStepper({
  current = 'what',
  until = 'build',
  ...rest
}) {
  const ci = PHASES.findIndex(p => p.key === current);
  const ui = PHASES.findIndex(p => p.key === until);
  return /*#__PURE__*/React.createElement("div", _extends({
    style: {
      display: 'inline-flex',
      alignItems: 'center',
      gap: '5px'
    }
  }, rest), PHASES.map((p, i) => {
    const done = i < ci;
    const cur = i === ci;
    const dim = i > ui;
    const style = {
      fontFamily: 'var(--font-mono)',
      fontSize: '11px',
      fontWeight: 600,
      letterSpacing: 'var(--tracking-wide)',
      padding: '3px 12px',
      borderRadius: 'var(--radius-pill)',
      transition: 'all var(--dur) var(--ease)',
      opacity: dim ? 0.35 : 1,
      background: cur ? p.c : done ? 'var(--ok-bg)' : 'var(--surface-2)',
      color: cur ? '#fff' : done ? '#86efac' : 'var(--text-muted)',
      boxShadow: cur ? '0 0 0 2px color-mix(in srgb, var(--text) 16%, transparent)' : 'none'
    };
    return /*#__PURE__*/React.createElement(React.Fragment, {
      key: p.key
    }, i > 0 && /*#__PURE__*/React.createElement("span", {
      style: {
        color: 'var(--text-faint)',
        fontFamily: 'var(--font-mono)',
        fontSize: '11px'
      }
    }, "\u203A"), /*#__PURE__*/React.createElement("span", {
      style: style
    }, p.label));
  }));
}
Object.assign(__ds_scope, { PhaseStepper });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/graph/PhaseStepper.jsx", error: String((e && e.message) || e) }); }

// ui_kits/aio-renderer/RendererApp.jsx
try { (() => {
/* global React, ReactDOM, AIO_CONTRACT */
const {
  useState
} = React;
const NS = window.ProductFrameworkDesignSystem_52ecf1;
const {
  Button,
  Tag
} = NS;
const C = AIO_CONTRACT;
const fmt = (v, t) => t === 'money' ? '€' + (v / 100).toFixed(2) : t === 'integer' ? '×' + v : String(v);
const screenById = id => C.screens.find(s => s.id === id);
const flowOf = pid => (C.flows.find(f => f.pages.includes(pid)) || {}).id || null;
const resolveContent = key => {
  const e = C.content_store[key];
  return e ? e[C.locale] || `⟨missing ${key}⟩` : `⟨missing: ${key}⟩`;
};
const L = ({
  children,
  color
}) => /*#__PURE__*/React.createElement("span", {
  style: {
    fontFamily: 'var(--font-mono)',
    fontSize: 9.5,
    letterSpacing: '.12em',
    textTransform: 'uppercase',
    color: color || 'var(--accent)'
  }
}, children);
function AioLabel({
  el
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 6,
      alignItems: 'center',
      marginBottom: 5
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      border: '1px solid var(--blueprint-line)',
      borderRadius: 2,
      padding: '0 5px',
      background: 'var(--paper)',
      color: 'var(--text-muted)',
      fontFamily: 'var(--font-mono)',
      fontSize: 9.5,
      letterSpacing: '.1em',
      textTransform: 'uppercase'
    }
  }, el.aio), /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--text-muted)',
      fontSize: 11,
      fontStyle: 'italic'
    }
  }, el.role));
}
const Wcag = ({
  el
}) => el.wcag ? /*#__PURE__*/React.createElement("div", {
  style: {
    marginTop: 5,
    display: 'flex',
    gap: 4,
    flexWrap: 'wrap'
  }
}, el.wcag.map(c => /*#__PURE__*/React.createElement("span", {
  key: c,
  style: {
    fontSize: 9,
    color: 'var(--text-muted)',
    border: '1px solid var(--border-soft)',
    borderRadius: 2,
    padding: '0 4px'
  }
}, /*#__PURE__*/React.createElement("span", {
  style: {
    color: 'var(--accent)'
  }
}, "a11y "), c))) : null;
function Element({
  el,
  data,
  current,
  selected,
  onSelect,
  onGo
}) {
  if (el.aio === 'display-collection') {
    const rows = data[el.binds] || [];
    return /*#__PURE__*/React.createElement("div", {
      style: {
        margin: '0 0 16px'
      }
    }, /*#__PURE__*/React.createElement(AioLabel, {
      el: el
    }), /*#__PURE__*/React.createElement("div", {
      style: {
        border: '1.5px solid var(--blueprint-line)',
        borderRadius: 4,
        overflow: 'hidden'
      }
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        display: 'grid',
        gridTemplateColumns: '1fr auto auto',
        gap: 10,
        padding: '7px 11px',
        background: 'var(--paper-2)',
        fontFamily: 'var(--font-mono)',
        fontSize: 9.5,
        letterSpacing: '.1em',
        textTransform: 'uppercase',
        color: 'var(--text-muted)'
      }
    }, el.item_shape.map(c => /*#__PURE__*/React.createElement("span", {
      key: c.field
    }, c.field))), rows.map((r, i) => /*#__PURE__*/React.createElement("div", {
      key: i,
      style: {
        display: 'grid',
        gridTemplateColumns: '1fr auto auto',
        gap: 10,
        padding: '9px 11px',
        borderTop: '1px dashed var(--border-soft)',
        alignItems: 'baseline'
      }
    }, el.item_shape.map((c, j) => /*#__PURE__*/React.createElement("span", {
      key: c.field,
      style: {
        fontWeight: j === 0 ? 600 : 400,
        color: j === 0 ? 'var(--text)' : 'var(--text-muted)',
        fontSize: j === 0 ? 14 : 12.5,
        fontFamily: j === 0 ? 'var(--font-sans)' : 'var(--font-mono)'
      }
    }, fmt(r[c.field], c.type)))))), /*#__PURE__*/React.createElement(Wcag, {
      el: el
    }));
  }
  if (el.aio === 'display-value') {
    const primary = el.emphasis === 'primary';
    return /*#__PURE__*/React.createElement("div", {
      style: {
        margin: '0 0 16px'
      }
    }, /*#__PURE__*/React.createElement(AioLabel, {
      el: el
    }), /*#__PURE__*/React.createElement("div", {
      style: {
        border: `${primary ? '2px' : '1.5px'} solid ${primary ? 'var(--accent)' : 'var(--blueprint-line)'}`,
        borderRadius: 4,
        padding: '11px 13px',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'baseline',
        background: primary ? 'var(--accent-wash)' : 'var(--paper)'
      }
    }, /*#__PURE__*/React.createElement("span", {
      style: {
        fontSize: 13,
        color: 'var(--text-muted)'
      }
    }, el.role), /*#__PURE__*/React.createElement("span", {
      style: {
        fontSize: 22,
        fontWeight: 700,
        fontFamily: 'var(--font-sans)',
        color: 'var(--text)'
      }
    }, fmt(data[el.binds], el.value_type))), /*#__PURE__*/React.createElement(Wcag, {
      el: el
    }));
  }
  if (el.aio === 'single-select') {
    const opts = data[el.binds] || [];
    return /*#__PURE__*/React.createElement("div", {
      style: {
        margin: '0 0 16px'
      }
    }, /*#__PURE__*/React.createElement(AioLabel, {
      el: el
    }), /*#__PURE__*/React.createElement("div", {
      style: {
        border: '1.5px solid var(--blueprint-line)',
        borderRadius: 4,
        overflow: 'hidden'
      }
    }, opts.map((o, i) => {
      const on = selected[current] === i;
      return /*#__PURE__*/React.createElement("div", {
        key: i,
        onClick: () => onSelect(i),
        style: {
          padding: '10px 12px',
          borderTop: i ? '1px dashed var(--border-soft)' : 'none',
          cursor: 'pointer',
          display: 'flex',
          gap: 9,
          alignItems: 'center',
          background: on ? 'var(--accent-wash)' : 'transparent'
        }
      }, /*#__PURE__*/React.createElement("span", {
        style: {
          color: on ? 'var(--accent)' : 'var(--blueprint-line)'
        }
      }, on ? '●' : '○'), o[el.option_field]);
    })), /*#__PURE__*/React.createElement(Wcag, {
      el: el
    }));
  }
  if (el.aio === 'trigger-action') {
    return /*#__PURE__*/React.createElement("div", {
      style: {
        margin: '0 0 16px'
      }
    }, /*#__PURE__*/React.createElement(Button, {
      variant: el.emphasis === 'primary' ? 'primary' : 'secondary',
      fullWidth: true,
      kicker: `trigger-action · ${el.issues}`,
      onClick: () => el.transitions_to && onGo(el.transitions_to)
    }, el.role), /*#__PURE__*/React.createElement(Wcag, {
      el: el
    }));
  }
  return null;
}
function StateBox({
  state,
  msg,
  ckey
}) {
  const failed = state === 'failed';
  return /*#__PURE__*/React.createElement("div", {
    style: {
      border: `1.5px dashed ${failed ? 'var(--warn)' : 'var(--blueprint-line)'}`,
      borderRadius: 4,
      padding: '26px 16px',
      textAlign: 'center',
      color: failed ? 'var(--warn)' : 'var(--text-muted)'
    }
  }, /*#__PURE__*/React.createElement(L, {
    color: failed ? 'var(--warn)' : 'var(--text-muted)'
  }, state), state === 'loading' && /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'inline-block',
      width: 16,
      height: 16,
      border: '2px solid var(--border-soft)',
      borderTopColor: 'var(--accent)',
      borderRadius: '50%',
      animation: 'aio-spin .8s linear infinite',
      margin: '8px auto 0'
    }
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: 13.5,
      marginTop: 7
    }
  }, msg), ckey && /*#__PURE__*/React.createElement("div", {
    style: {
      marginTop: 6
    }
  }, /*#__PURE__*/React.createElement(L, null, "content: ", ckey)));
}
function Device({
  current,
  setCurrent,
  forced,
  setForced,
  selected,
  setSelected
}) {
  const screen = screenById(current);
  const data = C.scenario.projected[screen.projection] || {};
  const state = forced && screen.state_space.includes(forced) ? forced : data.state || 'present';
  const curFlow = flowOf(current);
  const go = to => {
    setCurrent(to);
    setForced(null);
  };
  return /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: 430,
      margin: '0 auto'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 6,
      alignItems: 'center',
      fontSize: 11,
      color: 'var(--text-muted)',
      margin: '0 auto 14px',
      flexWrap: 'wrap',
      fontFamily: 'var(--font-mono)'
    }
  }, /*#__PURE__*/React.createElement(L, null, curFlow, ":"), (C.flows.find(f => f.id === curFlow)?.pages || [current]).map((pid, i) => /*#__PURE__*/React.createElement(React.Fragment, {
    key: pid
  }, i > 0 && /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--blueprint-line)'
    }
  }, "\u2192"), /*#__PURE__*/React.createElement("span", {
    onClick: () => go(pid),
    style: {
      padding: '2px 7px',
      border: '1px solid var(--border-soft)',
      borderRadius: 2,
      cursor: 'pointer',
      background: pid === current ? 'var(--ink)' : 'transparent',
      color: pid === current ? '#fff' : 'inherit',
      borderColor: pid === current ? 'var(--ink)' : 'var(--border-soft)'
    }
  }, screenById(pid).name)))), /*#__PURE__*/React.createElement("div", {
    style: {
      background: '#fff',
      border: '1.5px solid var(--ink)',
      borderRadius: 10,
      overflow: 'hidden',
      boxShadow: 'var(--shadow-draft)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      background: 'var(--ink)',
      color: '#fff',
      fontSize: 10.5,
      letterSpacing: '.1em',
      padding: '6px 12px',
      display: 'flex',
      justifyContent: 'space-between',
      fontFamily: 'var(--font-mono)'
    }
  }, /*#__PURE__*/React.createElement("span", null, "generic \xB7 no design system"), /*#__PURE__*/React.createElement("span", {
    style: {
      opacity: .7
    }
  }, C.context.form_factor, " \xB7 ", C.context.modality)), /*#__PURE__*/React.createElement("nav", {
    style: {
      display: 'flex',
      background: 'var(--paper-2)',
      borderBottom: '1.5px solid var(--ink)'
    }
  }, C.root.destinations.map(d => {
    const cur = flowOf(d.to) === curFlow;
    return /*#__PURE__*/React.createElement("button", {
      key: d.to,
      onClick: () => go(d.to),
      style: {
        flex: 1,
        border: 0,
        borderRight: '1px solid var(--border-soft)',
        background: cur ? '#fff' : 'transparent',
        fontFamily: 'var(--font-mono)',
        fontSize: 11,
        padding: '9px 4px',
        cursor: 'pointer',
        color: cur ? 'var(--accent)' : 'var(--text-muted)',
        fontWeight: cur ? 700 : 400,
        boxShadow: cur ? 'inset 0 -2px 0 var(--accent)' : 'none'
      }
    }, /*#__PURE__*/React.createElement("span", {
      style: {
        display: 'block',
        fontSize: 8,
        letterSpacing: '.12em',
        color: 'var(--blueprint-line)',
        marginBottom: 2
      }
    }, "NAVIGATE"), d.label);
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '18px 16px 22px',
      minHeight: 380
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      justifyContent: 'space-between',
      fontFamily: 'var(--font-mono)',
      fontSize: 10,
      letterSpacing: '.14em',
      textTransform: 'uppercase',
      color: 'var(--text-muted)',
      borderBottom: '1px dashed var(--border-soft)',
      paddingBottom: 8,
      marginBottom: 14
    }
  }, /*#__PURE__*/React.createElement("span", null, screen.name), /*#__PURE__*/React.createElement("span", null, screen.projection)), screen.content?.heading && /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: 18,
      fontWeight: 700,
      fontFamily: 'var(--font-sans)',
      margin: '0 0 14px',
      color: 'var(--text)'
    }
  }, /*#__PURE__*/React.createElement(L, null, "content: ", screen.content.heading), /*#__PURE__*/React.createElement("div", null, resolveContent(screen.content.heading))), state !== 'present' ? /*#__PURE__*/React.createElement(StateBox, {
    state: state,
    msg: screen.state_content?.[state] && resolveContent(screen.state_content[state]) || screen.state_meanings?.[state] || state,
    ckey: screen.state_content?.[state]
  }) : screen.elements.map((el, i) => /*#__PURE__*/React.createElement(Element, {
    key: i,
    el: el,
    data: data,
    current: current,
    selected: selected,
    onSelect: idx => setSelected({
      ...selected,
      [current]: idx
    }),
    onGo: go
  }))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      justifyContent: 'flex-end',
      gap: 8,
      padding: '8px 12px',
      borderTop: '1px dashed var(--border-soft)',
      background: 'var(--paper-2)'
    }
  }, C.root.global_actions.map(a => /*#__PURE__*/React.createElement("button", {
    key: a.issues,
    style: {
      border: '1px solid var(--blueprint-line)',
      background: '#fff',
      fontFamily: 'var(--font-mono)',
      fontSize: 10.5,
      padding: '4px 10px',
      borderRadius: 3,
      cursor: 'pointer',
      color: 'var(--text-muted)'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: 8,
      letterSpacing: '.1em',
      color: 'var(--blueprint-line)',
      marginRight: 5
    }
  }, "TRIGGER-ACTION"), a.label)))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexWrap: 'wrap',
      gap: 14,
      margin: '18px auto 0',
      paddingTop: 14,
      borderTop: '1px dashed var(--border-soft)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 5
    }
  }, /*#__PURE__*/React.createElement(L, {
    color: "var(--text-muted)"
  }, "Projection state"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      border: '1px solid var(--blueprint-line)',
      borderRadius: 3,
      overflow: 'hidden'
    }
  }, screen.state_space.map(st => /*#__PURE__*/React.createElement("button", {
    key: st,
    onClick: () => setForced(st),
    style: {
      border: 0,
      background: st === state ? 'var(--accent)' : '#fff',
      color: st === state ? '#fff' : 'var(--text-muted)',
      fontFamily: 'var(--font-mono)',
      fontSize: 11,
      padding: '5px 9px',
      cursor: 'pointer',
      borderRight: '1px solid var(--border-soft)'
    }
  }, st)))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 5
    }
  }, /*#__PURE__*/React.createElement(L, {
    color: "var(--text-muted)"
  }, "Reset"), /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    variant: "ghost",
    onClick: () => {
      setCurrent(C.start);
      setForced(null);
      setSelected({});
    }
  }, "\u21BA start over"))));
}
function Inspector({
  current
}) {
  const [tab, setTab] = useState('screen');
  const obj = tab === 'screen' ? screenById(current) : tab === 'scenario' ? C.scenario : tab === 'content' ? {
    locale: C.locale,
    content_store: C.content_store
  } : C;
  const tabs = [['screen', 'This screen'], ['scenario', 'Scenario data'], ['content', 'Content store'], ['full', 'Full contract']];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      background: 'var(--paper-2)',
      border: '1px solid var(--border-soft)',
      borderRadius: 4,
      overflow: 'hidden'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      borderBottom: '1px solid var(--border-soft)'
    }
  }, tabs.map(([k, lbl]) => /*#__PURE__*/React.createElement("button", {
    key: k,
    onClick: () => setTab(k),
    style: {
      flex: 1,
      border: 0,
      borderRight: '1px solid var(--border-soft)',
      background: tab === k ? '#fff' : 'transparent',
      fontFamily: 'var(--font-mono)',
      fontSize: 11,
      letterSpacing: '.08em',
      textTransform: 'uppercase',
      color: tab === k ? 'var(--accent)' : 'var(--text-muted)',
      padding: '9px 6px',
      cursor: 'pointer',
      fontWeight: tab === k ? 700 : 400,
      boxShadow: tab === k ? 'inset 0 -2px 0 var(--accent)' : 'none'
    }
  }, lbl))), /*#__PURE__*/React.createElement("pre", {
    style: {
      margin: 0,
      padding: 16,
      fontSize: 12,
      lineHeight: 1.55,
      overflow: 'auto',
      maxHeight: 560,
      fontFamily: 'var(--font-mono)',
      color: 'var(--ink)'
    }
  }, JSON.stringify(obj, null, 2)));
}
function App() {
  const [current, setCurrent] = useState(C.start);
  const [forced, setForced] = useState(null);
  const [selected, setSelected] = useState({});
  return /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: 1180,
      margin: '0 auto',
      padding: '28px 22px 60px'
    }
  }, /*#__PURE__*/React.createElement("header", {
    style: {
      borderBottom: '1.5px solid var(--ink)',
      paddingBottom: 14,
      marginBottom: 22
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 12,
      marginBottom: 8
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/logo-mark.svg",
    height: "22",
    alt: ""
  }), /*#__PURE__*/React.createElement("h1", {
    style: {
      fontSize: 15,
      letterSpacing: '.14em',
      textTransform: 'uppercase',
      margin: 0,
      fontWeight: 700,
      fontFamily: 'var(--font-sans)',
      color: 'var(--ink)'
    }
  }, "What \u2192 Preview"), /*#__PURE__*/React.createElement(Tag, {
    tone: "accent"
  }, "Preview"), /*#__PURE__*/React.createElement(Tag, null, "Generic renderer")), /*#__PURE__*/React.createElement("p", {
    style: {
      margin: 0,
      color: 'var(--text-muted)',
      fontSize: 12.5,
      maxWidth: '70ch',
      fontFamily: 'var(--font-mono)',
      lineHeight: 1.5
    }
  }, "A generic Abstract-Interaction-Object renderer consuming a ", /*#__PURE__*/React.createElement("b", null, "render contract"), " \u2014 the derived projection of a What. No design system is coupled: every control is its schematic default.")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: '1fr 1.15fr',
      gap: 26,
      alignItems: 'start'
    }
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("h2", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: 11,
      letterSpacing: '.16em',
      textTransform: 'uppercase',
      color: 'var(--text-muted)',
      margin: '0 0 12px',
      fontWeight: 700
    }
  }, "Render contract (consumed)"), /*#__PURE__*/React.createElement(Inspector, {
    current: current
  })), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("h2", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: 11,
      letterSpacing: '.16em',
      textTransform: 'uppercase',
      color: 'var(--text-muted)',
      margin: '0 0 12px',
      fontWeight: 700
    }
  }, "Rendered surface"), /*#__PURE__*/React.createElement(Device, {
    current: current,
    setCurrent: setCurrent,
    forced: forced,
    setForced: setForced,
    selected: selected,
    setSelected: setSelected
  }))));
}
ReactDOM.createRoot(document.getElementById('root')).render(/*#__PURE__*/React.createElement(App, null));
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/aio-renderer/RendererApp.jsx", error: String((e && e.message) || e) }); }

// ui_kits/aio-renderer/contract.js
try { (() => {
/* The render contract — a derived projection of the checkout What.
   Ported from product-framework/preview/renderer.html. In product-cli this is
   generated from the live What-graph, never authored. */
window.AIO_CONTRACT = {
  contract_version: 'preview-0',
  title: 'Checkout',
  context: {
    form_factor: 'phone',
    modality: 'touch'
  },
  locale: 'en',
  content_store: {
    'checkout.review.heading': {
      role: 'heading',
      en: 'Review your order'
    },
    'cart.empty.message': {
      role: 'empty-message',
      en: 'Nothing to check out yet — add something to get started.'
    },
    'cart.failed.message': {
      role: 'error-message',
      en: "Couldn't load your cart. Check your connection and retry."
    },
    'checkout.payment.heading': {
      role: 'heading',
      en: 'How would you like to pay?'
    },
    'browse.heading': {
      role: 'heading',
      en: 'Browse the shop'
    },
    'orders.heading': {
      role: 'heading',
      en: 'Your orders'
    },
    'confirm.heading': {
      role: 'heading',
      en: "You're all set"
    }
  },
  start: 'ui-review-cart',
  root: {
    destinations: [{
      to: 'ui-browse',
      label: 'Browse'
    }, {
      to: 'ui-orders',
      label: 'Orders'
    }, {
      to: 'ui-review-cart',
      label: 'Cart'
    }],
    global_actions: [{
      issues: 'cmd-sign-out',
      label: 'Sign out'
    }]
  },
  flows: [{
    id: 'flow-checkout',
    entry: 'ui-review-cart',
    pages: ['ui-review-cart', 'ui-choose-payment', 'ui-confirmation']
  }, {
    id: 'flow-browse',
    entry: 'ui-browse',
    pages: ['ui-browse']
  }, {
    id: 'flow-orders',
    entry: 'ui-orders',
    pages: ['ui-orders']
  }],
  screens: [{
    id: 'ui-review-cart',
    name: 'Review cart',
    projection: 'rm-cart-summary',
    content: {
      heading: 'checkout.review.heading'
    },
    state_space: ['present', 'empty', 'loading', 'failed'],
    state_meanings: {
      empty: 'Nothing to check out yet — add something to get started.',
      loading: 'Fetching your cart…',
      failed: "Couldn't load your cart. Check your connection and retry."
    },
    state_content: {
      empty: 'cart.empty.message',
      failed: 'cart.failed.message'
    },
    elements: [{
      aio: 'display-collection',
      role: 'the line items',
      binds: 'items',
      item_shape: [{
        field: 'name',
        type: 'string'
      }, {
        field: 'qty',
        type: 'integer'
      }, {
        field: 'price',
        type: 'money'
      }],
      wcag: ['1.3.1']
    }, {
      aio: 'display-value',
      role: 'the total owed',
      binds: 'total',
      value_type: 'money',
      emphasis: 'primary',
      wcag: ['1.3.1']
    }, {
      aio: 'trigger-action',
      role: 'proceed to payment',
      issues: 'cmd-begin-payment',
      emphasis: 'primary',
      transitions_to: 'ui-choose-payment',
      wcag: ['2.5.8', '2.4.7']
    }]
  }, {
    id: 'ui-choose-payment',
    name: 'Choose payment',
    projection: 'rm-payment-options',
    content: {
      heading: 'checkout.payment.heading'
    },
    state_space: ['present', 'loading', 'failed'],
    state_meanings: {
      loading: 'Loading payment methods…',
      failed: "Couldn't load payment methods. Retry."
    },
    elements: [{
      aio: 'single-select',
      role: 'a payment method',
      binds: 'methods',
      option_field: 'label',
      issues: 'cmd-select-method',
      wcag: ['1.3.1', '4.1.2']
    }, {
      aio: 'trigger-action',
      role: 'confirm and pay',
      issues: 'cmd-authorize-payment',
      emphasis: 'primary',
      transitions_to: 'ui-confirmation',
      wcag: ['2.5.8', '2.4.7']
    }]
  }, {
    id: 'ui-confirmation',
    name: 'Order placed',
    projection: 'rm-order-confirmation',
    content: {
      heading: 'confirm.heading'
    },
    state_space: ['present', 'loading'],
    state_meanings: {
      loading: 'Placing your order…'
    },
    elements: [{
      aio: 'display-value',
      role: 'the confirmation message',
      binds: 'message',
      value_type: 'string',
      emphasis: 'primary',
      wcag: ['1.3.1']
    }, {
      aio: 'display-value',
      role: 'the order number',
      binds: 'order_no',
      value_type: 'string',
      wcag: ['1.3.1']
    }, {
      aio: 'trigger-action',
      role: 'keep shopping',
      issues: 'cmd-continue-shopping',
      transitions_to: 'ui-browse',
      wcag: ['2.5.8']
    }]
  }, {
    id: 'ui-browse',
    name: 'Browse',
    projection: 'rm-catalog',
    content: {
      heading: 'browse.heading'
    },
    state_space: ['present', 'loading', 'empty'],
    state_meanings: {
      loading: 'Loading products…',
      empty: 'No products match.'
    },
    elements: [{
      aio: 'display-collection',
      role: 'the catalog',
      binds: 'products',
      item_shape: [{
        field: 'name',
        type: 'string'
      }, {
        field: 'price',
        type: 'money'
      }],
      wcag: ['1.3.1']
    }, {
      aio: 'trigger-action',
      role: 'go to cart',
      issues: 'cmd-open-cart',
      emphasis: 'primary',
      transitions_to: 'ui-review-cart',
      wcag: ['2.5.8']
    }]
  }, {
    id: 'ui-orders',
    name: 'Orders',
    projection: 'rm-order-history',
    content: {
      heading: 'orders.heading'
    },
    state_space: ['present', 'loading', 'empty'],
    state_meanings: {
      loading: 'Loading orders…',
      empty: "You haven't ordered yet."
    },
    elements: [{
      aio: 'display-collection',
      role: 'past orders',
      binds: 'orders',
      item_shape: [{
        field: 'order_no',
        type: 'string'
      }, {
        field: 'total',
        type: 'money'
      }],
      wcag: ['1.3.1']
    }]
  }],
  scenario: {
    projected: {
      'rm-cart-summary': {
        state: 'present',
        items: [{
          name: 'Coffee beans',
          qty: 2,
          price: 1800
        }, {
          name: 'Filter papers',
          qty: 1,
          price: 600
        }],
        total: 4200
      },
      'rm-payment-options': {
        state: 'present',
        methods: [{
          label: 'Card ending 4242'
        }, {
          label: 'Apple Pay'
        }, {
          label: 'Pay on delivery'
        }]
      },
      'rm-order-confirmation': {
        state: 'present',
        message: 'Order placed — thanks!',
        order_no: '#A7F-3192'
      },
      'rm-catalog': {
        state: 'present',
        products: [{
          name: 'Coffee beans',
          price: 1800
        }, {
          name: 'Filter papers',
          price: 600
        }, {
          name: 'Grinder',
          price: 4500
        }]
      },
      'rm-order-history': {
        state: 'present',
        orders: [{
          order_no: '#A7F-3192',
          total: 4200
        }, {
          order_no: '#9C1-0088',
          total: 1800
        }]
      }
    }
  }
};
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/aio-renderer/contract.js", error: String((e && e.message) || e) }); }

// ui_kits/what-graph/WhatGraphApp.jsx
try { (() => {
/* global React, ReactDOM, WG */
const {
  useState,
  useMemo
} = React;
const NS = window.ProductFrameworkDesignSystem_52ecf1;
const {
  EMNode,
  PhaseStepper,
  Tag,
  StatePill
} = NS;
const NODE_W = 170;
const KIND_FILTERS = ['trigger', 'ui-step', 'command', 'view', 'event'];
const FILTER_COLOR = {
  trigger: 'var(--em-trigger)',
  'ui-step': 'var(--slate-400)',
  command: 'var(--em-command)',
  view: 'var(--em-view)',
  event: 'var(--em-event)'
};
function computeLayout() {
  const {
    gutter,
    colW
  } = WG.geometry;
  const padTop = 16;
  let y = padTop;
  const laneBox = {};
  for (const ln of WG.lanes) {
    laneBox[ln.id] = {
      top: y,
      h: ln.h,
      center: y + ln.h / 2
    };
    y += ln.h;
  }
  const totalH = y + 8;
  const width = gutter + WG.cols * colW + 24;
  // group nodes by (lane,col) cell so co-located nodes stack instead of overlap
  const cell = {};
  for (const n of WG.nodes) {
    const k = n.lane + ':' + n.col;
    (cell[k] = cell[k] || []).push(n.id);
  }
  const STACK = 46;
  const pos = {};
  for (const n of WG.nodes) {
    const x = gutter + n.col * colW + colW / 2;
    const box = laneBox[n.lane];
    const group = cell[n.lane + ':' + n.col];
    let yy = box.center;
    if (group.length > 1) {
      const i = group.indexOf(n.id);
      yy = box.center - (group.length - 1) * STACK / 2 + i * STACK;
    }
    pos[n.id] = {
      x,
      y: yy
    };
  }
  return {
    laneBox,
    totalH,
    width,
    pos
  };
}
function Connectors({
  pos,
  hovered,
  hidden
}) {
  const live = id => !hidden.has(WG.nodes.find(n => n.id === id)?.kind);
  return /*#__PURE__*/React.createElement("svg", {
    width: "100%",
    height: "100%",
    style: {
      position: 'absolute',
      inset: 0,
      pointerEvents: 'none',
      overflow: 'visible'
    }
  }, /*#__PURE__*/React.createElement("defs", null, /*#__PURE__*/React.createElement("marker", {
    id: "wg-arr",
    viewBox: "0 0 10 10",
    refX: "9",
    refY: "5",
    markerWidth: "6",
    markerHeight: "6",
    orient: "auto-start-reverse"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M0,0 L10,5 L0,10 z",
    fill: "var(--slate-500)"
  }))), WG.edges.map((e, i) => {
    const a = pos[e.from],
      b = pos[e.to];
    if (!a || !b || !live(e.from) || !live(e.to)) return null;
    const my = (a.y + b.y) / 2;
    const d = `M${a.x},${a.y} C${a.x},${my} ${b.x},${my} ${b.x},${b.y}`;
    if (e.type === 'spine') {
      return /*#__PURE__*/React.createElement("path", {
        key: i,
        d: d,
        fill: "none",
        stroke: "var(--slate-600)",
        strokeWidth: "1.4",
        markerEnd: "url(#wg-arr)",
        opacity: "0.6"
      });
    }
    const show = hovered === e.from || hovered === e.to;
    return /*#__PURE__*/React.createElement("path", {
      key: i,
      d: d,
      fill: "none",
      stroke: "#38bdf8",
      strokeWidth: "1.5",
      strokeDasharray: "5 3",
      opacity: show ? 0.95 : 0,
      style: {
        transition: 'opacity 120ms'
      }
    });
  }));
}
function Timeline({
  hidden,
  onSelect,
  selected
}) {
  const [hovered, setHovered] = useState(null);
  const {
    laneBox,
    totalH,
    width,
    pos
  } = useMemo(computeLayout, []);
  const {
    gutter
  } = WG.geometry;
  return /*#__PURE__*/React.createElement("div", {
    style: {
      position: 'relative',
      width,
      height: totalH,
      minWidth: '100%'
    }
  }, WG.lanes.map(ln => {
    const box = laneBox[ln.id];
    const isStream = ln.kind === 'stream';
    return /*#__PURE__*/React.createElement("div", {
      key: ln.id
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        position: 'absolute',
        left: gutter,
        right: 0,
        top: box.top,
        height: box.h,
        background: isStream ? 'rgba(245,158,11,.07)' : ln.id === 'cmdview' ? 'rgba(37,99,235,.05)' : 'rgba(148,163,184,.05)',
        borderTop: '1px solid var(--slate-800)'
      }
    }), /*#__PURE__*/React.createElement("div", {
      style: {
        position: 'absolute',
        left: 0,
        width: gutter - 14,
        top: box.center - 8,
        textAlign: 'right',
        fontFamily: 'var(--font-mono)',
        fontSize: '11px',
        fontWeight: isStream ? 600 : 700,
        color: isStream ? 'var(--em-event)' : 'var(--slate-300)',
        letterSpacing: '.02em',
        opacity: isStream ? 0.9 : 1
      }
    }, ln.label));
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      position: 'absolute',
      left: gutter,
      top: 0,
      bottom: 0,
      width: 1,
      background: 'var(--slate-700)'
    }
  }), /*#__PURE__*/React.createElement(Connectors, {
    pos: pos,
    hovered: hovered,
    hidden: hidden
  }), WG.nodes.filter(n => !hidden.has(n.kind)).map(n => {
    const p = pos[n.id];
    return /*#__PURE__*/React.createElement("div", {
      key: n.id,
      onMouseEnter: () => setHovered(n.id),
      onMouseLeave: () => setHovered(null),
      style: {
        position: 'absolute',
        left: p.x - NODE_W / 2,
        top: p.y - 23,
        width: NODE_W
      }
    }, /*#__PURE__*/React.createElement(EMNode, {
      kind: n.kind,
      label: n.label,
      note: n.sub,
      showKind: false,
      selected: selected === n.id,
      onClick: () => onSelect(n.id),
      style: {
        minWidth: NODE_W,
        maxWidth: NODE_W
      }
    }));
  }));
}
function StructuralView({
  hidden
}) {
  const domain = [{
    kind: 'entity',
    label: 'Order',
    sub: 'aggregate'
  }, {
    kind: 'entity',
    label: 'Payment',
    sub: 'aggregate'
  }, {
    kind: 'value-object',
    label: 'Money',
    sub: 'amount + currency'
  }, {
    kind: 'invariant',
    label: 'Order total ≥ 0',
    sub: 'read by Begin payment'
  }];
  const colorFor = k => ({
    entity: 'var(--kind-entity)',
    'value-object': 'var(--kind-value-object)',
    invariant: 'var(--kind-invariant)'
  })[k];
  const event = WG.nodes.filter(n => ['command', 'view', 'event'].includes(n.kind) && !hidden.has(n.kind));
  const Box = ({
    c,
    label,
    sub
  }) => /*#__PURE__*/React.createElement("div", {
    style: {
      border: `1.5px solid ${c}`,
      background: `color-mix(in srgb, ${c} 13%, transparent)`,
      borderRadius: 'var(--radius)',
      padding: '8px 11px',
      marginBottom: 8
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-sans)',
      fontWeight: 600,
      fontSize: 12,
      color: 'var(--slate-200)'
    }
  }, label), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: 9.5,
      color: 'var(--slate-400)'
    }
  }, sub));
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: '1fr 1fr',
      minWidth: '100%'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '14px 22px',
      borderRight: '1px dashed var(--slate-700)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: 11,
      fontWeight: 700,
      letterSpacing: '.04em',
      color: 'var(--em-command)',
      marginBottom: 12
    }
  }, "DOMAIN MODEL \xB7 \xA73.1 structure"), domain.map((d, i) => /*#__PURE__*/React.createElement(Box, {
    key: i,
    c: colorFor(d.kind),
    label: d.label,
    sub: d.sub
  }))), /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '14px 22px'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-mono)',
      fontSize: 11,
      fontWeight: 700,
      letterSpacing: '.04em',
      color: 'var(--em-event)',
      marginBottom: 12
    }
  }, "EVENT MODEL \xB7 \xA73.2 behaviour"), event.map(n => /*#__PURE__*/React.createElement(Box, {
    key: n.id,
    c: n.kind === 'command' ? 'var(--em-command)' : n.kind === 'view' ? 'var(--em-view)' : 'var(--em-event)',
    label: n.label,
    sub: n.sub
  }))));
}
function DetailPanel({
  id,
  onClose
}) {
  const n = WG.nodes.find(x => x.id === id);
  if (!n) return null;
  const m = WG.meta[id] || {};
  const Row = ({
    k,
    items
  }) => items && items.length ? /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("dt", {
    style: {
      color: 'var(--slate-400)',
      fontSize: 11,
      fontFamily: 'var(--font-mono)',
      marginTop: 10
    }
  }, k), items.map((t, i) => /*#__PURE__*/React.createElement("dd", {
    key: i,
    style: {
      margin: '2px 0 0',
      fontSize: 12,
      fontFamily: 'var(--font-mono)',
      color: 'var(--slate-200)'
    }
  }, t))) : null;
  return /*#__PURE__*/React.createElement("aside", {
    style: {
      position: 'absolute',
      top: 0,
      right: 0,
      width: 290,
      maxHeight: '100%',
      overflow: 'auto',
      background: 'var(--slate-800)',
      border: '1px solid var(--slate-600)',
      borderRadius: '8px 0 0 8px',
      padding: 16,
      boxShadow: 'var(--shadow-graph)',
      zIndex: 5
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 8,
      marginBottom: 6
    }
  }, /*#__PURE__*/React.createElement(Tag, {
    tone: n.kind === 'command' ? 'command' : n.kind === 'view' ? 'view' : n.kind === 'event' ? 'event' : n.kind === 'trigger' ? 'trigger' : 'neutral',
    solid: true
  }, n.kind)), /*#__PURE__*/React.createElement("h2", {
    style: {
      margin: '4px 0 2px',
      fontSize: 15,
      fontFamily: 'var(--font-sans)',
      color: 'var(--slate-100)'
    }
  }, n.label), /*#__PURE__*/React.createElement("dl", {
    style: {
      margin: 0
    }
  }, /*#__PURE__*/React.createElement(Row, {
    k: "id",
    items: [n.sub || n.id]
  }), /*#__PURE__*/React.createElement(Row, {
    k: "out",
    items: m.out
  }), /*#__PURE__*/React.createElement(Row, {
    k: "in",
    items: m.in
  })), /*#__PURE__*/React.createElement("button", {
    onClick: onClose,
    style: {
      marginTop: 14,
      background: 'none',
      border: '1px solid var(--slate-600)',
      color: 'var(--slate-200)',
      borderRadius: 5,
      padding: '5px 10px',
      cursor: 'pointer',
      fontFamily: 'var(--font-mono)',
      fontSize: 11
    }
  }, "close"));
}
function App() {
  const [mode, setMode] = useState('timeline');
  const [hidden, setHidden] = useState(new Set());
  const [selected, setSelected] = useState(null);
  const toggle = k => setHidden(h => {
    const n = new Set(h);
    n.has(k) ? n.delete(k) : n.add(k);
    return n;
  });
  return /*#__PURE__*/React.createElement("div", {
    "data-surface": "graph",
    style: {
      minHeight: '100vh',
      display: 'flex',
      flexDirection: 'column'
    }
  }, /*#__PURE__*/React.createElement("header", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 16,
      padding: '10px 16px',
      background: 'var(--slate-800)',
      borderBottom: '1px solid var(--slate-600)',
      flexWrap: 'wrap'
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/logo-mark.svg",
    height: "22",
    alt: ""
  }), /*#__PURE__*/React.createElement("h1", {
    style: {
      fontSize: 15,
      margin: 0,
      fontWeight: 600,
      fontFamily: 'var(--font-sans)',
      color: 'var(--slate-100)'
    }
  }, "What Graph ", /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--slate-400)',
      fontWeight: 400
    }
  }, "\u2014 Live View")), /*#__PURE__*/React.createElement(PhaseStepper, {
    current: "how",
    until: "build"
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: 4
    }
  }, /*#__PURE__*/React.createElement(StatePill, {
    state: "draft",
    label: "checkout \xB7 DRAFT"
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 6,
      color: 'var(--slate-400)',
      fontSize: 12,
      fontFamily: 'var(--font-mono)'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      width: 9,
      height: 9,
      borderRadius: '50%',
      background: '#22c55e'
    }
  }), "live"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      border: '1px solid var(--slate-600)',
      borderRadius: 6,
      overflow: 'hidden',
      marginLeft: 'auto'
    }
  }, ['timeline', 'lanes'].map(m => /*#__PURE__*/React.createElement("button", {
    key: m,
    onClick: () => setMode(m),
    style: {
      background: mode === m ? 'var(--slate-600)' : 'none',
      border: 0,
      color: mode === m ? 'var(--slate-100)' : 'var(--slate-400)',
      fontFamily: 'var(--font-mono)',
      fontSize: 12,
      padding: '4px 12px',
      cursor: 'pointer'
    }
  }, m === 'timeline' ? 'Timeline' : 'Two-lane')))), mode === 'timeline' && /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 14,
      padding: '8px 16px',
      background: 'var(--slate-900)',
      borderBottom: '1px solid var(--slate-800)',
      fontFamily: 'var(--font-mono)',
      fontSize: 12,
      color: 'var(--slate-400)',
      flexWrap: 'wrap'
    }
  }, KIND_FILTERS.map(k => /*#__PURE__*/React.createElement("label", {
    key: k,
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 5,
      cursor: 'pointer'
    }
  }, /*#__PURE__*/React.createElement("input", {
    type: "checkbox",
    checked: !hidden.has(k),
    onChange: () => toggle(k)
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      width: 9,
      height: 9,
      borderRadius: 2,
      background: FILTER_COLOR[k]
    }
  }), k))), /*#__PURE__*/React.createElement("div", {
    style: {
      position: 'relative',
      flex: 1,
      overflowX: 'auto',
      overflowY: 'hidden',
      background: 'var(--slate-900)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '8px 0 20px'
    }
  }, mode === 'timeline' ? /*#__PURE__*/React.createElement(Timeline, {
    hidden: hidden,
    onSelect: setSelected,
    selected: selected
  }) : /*#__PURE__*/React.createElement(StructuralView, {
    hidden: hidden
  })), selected && /*#__PURE__*/React.createElement(DetailPanel, {
    id: selected,
    onClose: () => setSelected(null)
  })));
}
ReactDOM.createRoot(document.getElementById('root')).render(/*#__PURE__*/React.createElement(App, null));
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/what-graph/WhatGraphApp.jsx", error: String((e && e.message) || e) }); }

// ui_kits/what-graph/data.js
try { (() => {
/* The reference "checkout" What-graph, shaped for the timeline.
   Geometry is fixed so the SVG connector layer and the node chips align. */
window.WG = {
  geometry: {
    gutter: 188,
    colW: 210,
    x0: 0
  },
  lanes: [{
    id: 'ui',
    label: 'Triggers / UI',
    kind: 'rail',
    h: 100
  }, {
    id: 'cmdview',
    label: 'Commands · Views',
    kind: 'rail',
    h: 64
  }, {
    id: 'order',
    label: 'Order',
    kind: 'stream',
    h: 72
  }, {
    id: 'payment',
    label: 'Payment',
    kind: 'stream',
    h: 72
  }],
  cols: 3,
  nodes: [{
    id: 'trg-pay',
    kind: 'trigger',
    label: 'Shopper opens cart',
    col: 0,
    lane: 'ui',
    sub: 'user trigger'
  }, {
    id: 'ui-review',
    kind: 'ui-step',
    label: 'Review cart',
    col: 0,
    lane: 'ui'
  }, {
    id: 'ui-choose',
    kind: 'ui-step',
    label: 'Choose payment',
    col: 1,
    lane: 'ui'
  }, {
    id: 'ui-confirm',
    kind: 'ui-step',
    label: 'Order placed',
    col: 2,
    lane: 'ui'
  }, {
    id: 'cmd-begin',
    kind: 'command',
    label: 'Begin payment',
    col: 0,
    lane: 'cmdview',
    sub: 'cmd-begin-payment'
  }, {
    id: 'cmd-auth',
    kind: 'command',
    label: 'Authorize payment',
    col: 1,
    lane: 'cmdview',
    sub: 'cmd-authorize-payment'
  }, {
    id: 'rm-confirm',
    kind: 'view',
    label: 'Order confirmation',
    col: 2,
    lane: 'cmdview',
    sub: 'rm-order-confirmation'
  }, {
    id: 'ev-begun',
    kind: 'event',
    label: 'Payment begun',
    col: 0,
    lane: 'order',
    sub: 'ev-payment-begun'
  }, {
    id: 'ev-placed',
    kind: 'event',
    label: 'Order placed',
    col: 1,
    lane: 'order',
    sub: 'ev-order-placed'
  }, {
    id: 'ev-auth',
    kind: 'event',
    label: 'Payment authorized',
    col: 1,
    lane: 'payment',
    sub: 'ev-payment-authorized'
  }],
  // spine = always-on vertical flow; cross = shown on hover (feeds a view)
  edges: [{
    from: 'trg-pay',
    to: 'cmd-begin',
    type: 'spine'
  }, {
    from: 'ui-review',
    to: 'cmd-begin',
    type: 'spine'
  }, {
    from: 'cmd-begin',
    to: 'ev-begun',
    type: 'spine'
  }, {
    from: 'ui-choose',
    to: 'cmd-auth',
    type: 'spine'
  }, {
    from: 'cmd-auth',
    to: 'ev-auth',
    type: 'spine'
  }, {
    from: 'cmd-auth',
    to: 'ev-placed',
    type: 'spine'
  }, {
    from: 'ev-placed',
    to: 'rm-confirm',
    type: 'cross'
  }, {
    from: 'ui-confirm',
    to: 'rm-confirm',
    type: 'cross'
  }],
  // detail-panel metadata
  meta: {
    'cmd-begin': {
      model: 'event',
      context: 'Checkout',
      out: ['→ ev-payment-begun (emits)'],
      in: ['← ui-review (triggers)', '← trg-pay (issues)']
    },
    'cmd-auth': {
      model: 'event',
      context: 'Checkout',
      out: ['→ ev-payment-authorized (emits)', '→ ev-order-placed (emits)'],
      in: ['← ui-choose (triggers)']
    },
    'ev-placed': {
      model: 'event',
      context: 'Order',
      out: ['→ rm-order-confirmation (projects)'],
      in: ['← cmd-authorize-payment (emits)']
    },
    'rm-confirm': {
      model: 'event',
      context: 'Checkout',
      out: ['→ ui-confirmation (displays)'],
      in: ['← ev-order-placed (projects, bridge)']
    },
    'ui-review': {
      model: 'event',
      context: 'Checkout',
      out: ['→ cmd-begin-payment (triggers)'],
      in: []
    }
  }
};
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/what-graph/data.js", error: String((e && e.message) || e) }); }

__ds_ns.Button = __ds_scope.Button;

__ds_ns.Card = __ds_scope.Card;

__ds_ns.ConformanceBadge = __ds_scope.ConformanceBadge;

__ds_ns.StatePill = __ds_scope.StatePill;

__ds_ns.Tag = __ds_scope.Tag;

__ds_ns.EMNode = __ds_scope.EMNode;

__ds_ns.PhaseStepper = __ds_scope.PhaseStepper;

})();
