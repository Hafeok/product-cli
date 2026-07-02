/* global React, PF */
/* Screens (preview) — the generic AIO renderer, recreated from the repo's
   preview/renderer.html. Consumes the render contract: render(contract) → surface.
   Deliberately schematic — a blueprint, not a product. Walkable via transitions;
   the same contract reifies differently per context of use (phone / desktop). */
(function () {
  const { useState } = React;

  const CSS = `
  .bp{position:absolute;inset:0;overflow:auto;background:var(--paper);color:var(--ink);
    font-family:var(--font-mono);font-size:13.5px;line-height:1.5}
  .bp *{box-sizing:border-box}
  .bp .wrap{max-width:1280px;margin:0 auto;padding:22px 22px 46px}
  .bp .cols{display:grid;grid-template-columns:1fr 1.2fr;gap:24px;align-items:start}
  .bp .cols.desktop{grid-template-columns:1fr 1.75fr}
  .bp h2.sec{font-size:11px;letter-spacing:.16em;text-transform:uppercase;color:var(--text-faint,#5b7790);
    margin:0 0 11px;font-weight:700;display:flex;align-items:center;gap:8px;font-family:var(--font-mono)}
  .bp h2.sec::after{content:"";flex:1;height:1px;background:var(--blueprint-line-soft)}

  .bp .contract{background:var(--paper-2);border:1px solid var(--blueprint-line-soft);border-radius:4px;overflow:hidden}
  .bp .bar{display:flex;border-bottom:1px solid var(--blueprint-line-soft)}
  .bp .bar button{flex:1;border:0;background:transparent;font-family:inherit;font-size:10.5px;letter-spacing:.08em;
    text-transform:uppercase;color:var(--text-faint,#5b7790);padding:9px 5px;cursor:pointer;border-right:1px solid var(--blueprint-line-soft)}
  .bp .bar button:last-child{border-right:0}
  .bp .bar button[aria-selected="true"]{background:#fff;color:var(--blue-700);font-weight:700;box-shadow:inset 0 -2px 0 var(--blue-700)}
  .bp pre{margin:0;padding:15px;font-size:11.5px;line-height:1.55;overflow:auto;max-height:600px;white-space:pre;tab-size:2}
  .bp pre .k{color:var(--blue-700)} .bp pre .s{color:#117a4d} .bp pre .n{color:#9a3b00}

  .bp .device{background:#fff;border:1.5px solid var(--ink);border-radius:10px;overflow:hidden;margin:0 auto;
    box-shadow:var(--shadow-draft)}
  .bp .device.phone{max-width:410px}
  .bp .device.desktop{max-width:none}
  .bp .chrome{background:var(--ink);color:#fff;font-size:10.5px;letter-spacing:.1em;padding:6px 12px;
    display:flex;justify-content:space-between}
  .bp .chrome .ctx{opacity:.7}

  .bp .appnav{display:flex;background:var(--paper-2);border-bottom:1.5px solid var(--ink)}
  .bp .appnav button{flex:1;border:0;border-right:1px solid var(--blueprint-line-soft);background:transparent;
    font-family:inherit;font-size:11px;letter-spacing:.06em;padding:9px 4px;cursor:pointer;color:var(--text-faint,#5b7790)}
  .bp .appnav button:last-child{border-right:0}
  .bp .appnav button .typ{display:block;font-size:8px;letter-spacing:.12em;color:var(--blueprint-line);margin-bottom:2px}
  .bp .appnav button[aria-current="true"]{background:#fff;color:var(--blue-700);font-weight:700;box-shadow:inset 0 -2px 0 var(--blue-700)}

  .bp .deskgrid{display:grid;grid-template-columns:176px 1fr}
  .bp .sidenav{border-right:1.5px solid var(--ink);background:var(--paper-2);padding:10px 0;display:flex;flex-direction:column}
  .bp .sidenav button{border:0;background:transparent;font-family:inherit;font-size:11.5px;text-align:left;
    padding:9px 14px;cursor:pointer;color:var(--text-faint,#5b7790);border-left:2px solid transparent}
  .bp .sidenav button .typ{display:block;font-size:8px;letter-spacing:.12em;color:var(--blueprint-line)}
  .bp .sidenav button[aria-current="true"]{background:#fff;color:var(--blue-700);font-weight:700;border-left-color:var(--blue-700)}
  .bp .sidenav .navlab{font-size:8.5px;letter-spacing:.14em;text-transform:uppercase;color:var(--blueprint-line);padding:0 14px 7px}

  .bp .globalbar{display:flex;justify-content:flex-end;gap:8px;padding:8px 12px;border-top:1px dashed var(--blueprint-line-soft);background:var(--paper-2)}
  .bp .globalbar button{border:1px solid var(--blueprint-line);background:#fff;font-family:inherit;font-size:10.5px;
    padding:4px 10px;border-radius:3px;cursor:pointer;color:var(--text-faint,#5b7790)}
  .bp .globalbar .typ{font-size:8px;letter-spacing:.1em;color:var(--blueprint-line);margin-right:5px}

  .bp .screen{padding:18px 16px 22px;min-height:400px}
  .bp .stepname{font-size:10px;letter-spacing:.14em;text-transform:uppercase;color:var(--text-faint,#5b7790);
    border-bottom:1px dashed var(--blueprint-line-soft);padding-bottom:8px;margin-bottom:14px;display:flex;justify-content:space-between}
  .bp .pageheading{font-size:18px;font-weight:700;margin:0 0 14px}
  .bp .ck{display:block;font-size:8.5px;letter-spacing:.1em;text-transform:uppercase;color:var(--blue-700);
    font-weight:400;margin-top:3px;opacity:.7}

  .bp .aio{margin:0 0 16px}
  .bp .lab{font-size:9.5px;letter-spacing:.12em;text-transform:uppercase;color:var(--blue-700);margin-bottom:5px;
    display:flex;gap:6px;align-items:center}
  .bp .lab .typ{border:1px solid var(--blueprint-line);border-radius:2px;padding:0 5px;background:var(--paper);color:var(--text-faint,#5b7790)}
  .bp .lab .role{color:var(--text-faint,#5b7790);font-size:11px;text-transform:none;letter-spacing:0;font-style:italic}

  .bp .gc-list{border:1.5px solid var(--blueprint-line);border-radius:4px;overflow:hidden}
  .bp .gc-list .row{display:grid;gap:10px;padding:9px 11px;border-bottom:1px dashed var(--blueprint-line-soft);align-items:baseline}
  .bp .gc-list .row:last-child{border-bottom:0}
  .bp .gc-list .row .nm{font-weight:600}
  .bp .gc-list .row .dim{color:var(--text-faint,#5b7790);font-size:12.5px}
  .bp .gc-list .head{background:var(--paper-2);font-size:9.5px;letter-spacing:.1em;text-transform:uppercase;color:var(--text-faint,#5b7790)}
  .bp .gc-stack{border:1.5px solid var(--blueprint-line);border-radius:4px;overflow:hidden}
  .bp .gc-stack .item{padding:10px 12px;border-bottom:1px dashed var(--blueprint-line-soft)}
  .bp .gc-stack .item:last-child{border-bottom:0}
  .bp .gc-stack .item .nm{font-weight:600}
  .bp .gc-stack .item .dim{color:var(--text-faint,#5b7790);font-size:12px;display:flex;gap:12px}

  .bp .gc-value{border:1.5px solid var(--blueprint-line);border-radius:4px;padding:11px 13px;display:flex;
    justify-content:space-between;align-items:baseline;background:var(--paper)}
  .bp .gc-value.primary{border-width:2px;border-color:var(--blue-700);background:var(--accent-wash,#dbe8fb)}
  .bp .gc-value .big{font-size:22px;font-weight:700}

  .bp .gc-button{border:1.5px solid var(--blue-700);background:#fff;color:var(--blue-700);font-family:inherit;
    font-size:14px;letter-spacing:.04em;padding:12px;border-radius:4px;cursor:pointer;font-weight:700;width:100%}
  .bp .gc-button.inline{width:auto;padding:10px 22px}
  .bp .gc-button.primary{background:var(--blue-700);color:#fff}
  .bp .gc-button:active{transform:translateY(1px)}
  .bp .gc-button .typ{display:block;font-size:9px;letter-spacing:.12em;opacity:.65;font-weight:400;margin-bottom:2px}

  .bp .gc-select{border:1.5px solid var(--blueprint-line);border-radius:4px;overflow:hidden}
  .bp .gc-select .opt{padding:10px 12px;border-bottom:1px dashed var(--blueprint-line-soft);cursor:pointer;display:flex;gap:9px}
  .bp .gc-select .opt:last-child{border-bottom:0}
  .bp .gc-select .opt::before{content:"○";color:var(--blueprint-line)}
  .bp .gc-select .opt[aria-checked="true"]{background:var(--accent-wash,#dbe8fb)}
  .bp .gc-select .opt[aria-checked="true"]::before{content:"●";color:var(--blue-700)}
  .bp .gc-seg{display:flex;border:1.5px solid var(--blueprint-line);border-radius:4px;overflow:hidden}
  .bp .gc-seg .opt{flex:1;padding:10px 8px;text-align:center;cursor:pointer;border-right:1px solid var(--blueprint-line-soft);font-size:12.5px}
  .bp .gc-seg .opt:last-child{border-right:0}
  .bp .gc-seg .opt[aria-checked="true"]{background:var(--blue-700);color:#fff;font-weight:700}

  .bp .wcag{margin-top:5px;display:flex;gap:4px;flex-wrap:wrap}
  .bp .wcag span{font-size:9px;color:var(--text-faint,#5b7790);border:1px solid var(--blueprint-line-soft);border-radius:2px;padding:0 4px}
  .bp .wcag span::before{content:"a11y ";color:var(--blue-700)}

  .bp .state-box{border:1.5px dashed var(--blueprint-line);border-radius:4px;padding:26px 16px;text-align:center;color:var(--text-faint,#5b7790)}
  .bp .state-box.failed{border-color:#9a5b00;color:#9a5b00}
  .bp .state-box .st{font-size:10px;letter-spacing:.14em;text-transform:uppercase;display:block;margin-bottom:7px;opacity:.8}
  .bp .spinner{display:inline-block;width:16px;height:16px;border:2px solid var(--blueprint-line-soft);
    border-top-color:var(--blue-700);border-radius:50%;animation:bpspin .8s linear infinite;margin-bottom:8px}
  @keyframes bpspin{to{transform:rotate(360deg)}}

  .bp .crumb{display:flex;gap:6px;align-items:center;font-size:11px;color:var(--text-faint,#5b7790);margin:0 0 13px;flex-wrap:wrap}
  .bp .crumb .flowlab{font-size:10px;letter-spacing:.1em;text-transform:uppercase;color:var(--blue-700)}
  .bp .crumb .c{padding:2px 7px;border:1px solid var(--blueprint-line-soft);border-radius:2px;cursor:pointer}
  .bp .crumb .c.here{background:var(--ink);color:#fff;border-color:var(--ink)}
  .bp .crumb .arr{color:var(--blueprint-line)}

  .bp .controls{display:flex;flex-wrap:wrap;gap:14px;margin-top:14px;padding-top:13px;border-top:1px dashed var(--blueprint-line-soft)}
  .bp .grp .glab{display:block;font-size:9.5px;letter-spacing:.12em;text-transform:uppercase;color:var(--text-faint,#5b7790);margin-bottom:5px}
  .bp .seg{display:flex;border:1px solid var(--blueprint-line);border-radius:3px;overflow:hidden}
  .bp .seg button{border:0;background:#fff;font-family:inherit;font-size:11px;padding:5px 9px;cursor:pointer;
    border-right:1px solid var(--blueprint-line-soft);color:var(--text-faint,#5b7790)}
  .bp .seg button:last-child{border-right:0}
  .bp .seg button[aria-pressed="true"]{background:var(--blue-700);color:#fff}
  .bp .note{margin-top:22px;padding-top:12px;border-top:1px solid var(--blueprint-line-soft);color:var(--text-faint,#5b7790);
    font-size:11px;max-width:82ch;line-height:1.55}
  .bp .note code{background:var(--paper-2);padding:1px 4px;border-radius:2px}

  /* ---- web · full-viewport mode ---- */
  .bp.web{overflow:hidden;display:flex;flex-direction:column}
  .bp.web .chrome{flex:none}
  .bp .chrome .url{background:rgba(255,255,255,.14);border-radius:3px;padding:1px 12px;letter-spacing:.04em}
  .bp .webbody{flex:1;display:flex;min-height:0;background:#fff}
  .bp .webbody .sidenav{width:188px;flex:none;display:flex;flex-direction:column;border-right:1.5px solid var(--ink);background:var(--paper-2);padding:10px 0}
  .bp .webbody .sidenav .globalslot{margin-top:auto;border-top:1px dashed var(--blueprint-line-soft);padding:10px 14px 4px}
  .bp .webbody .sidenav .globalslot button{width:100%;border:1px solid var(--blueprint-line);background:#fff;font-family:inherit;
    font-size:10.5px;padding:5px 10px;border-radius:3px;cursor:pointer;color:var(--text-faint,#5b7790);text-align:left}
  .bp .webmain{flex:1;min-width:0;display:flex;flex-direction:column}
  .bp .webtoolbar{flex:none;display:flex;align-items:center;gap:14px;flex-wrap:wrap;padding:8px 20px;
    border-bottom:1px dashed var(--blueprint-line-soft);background:var(--paper-2)}
  .bp .webtoolbar .crumb{margin:0}
  .bp .webtoolbar .seg{background:#fff}
  .bp .contractbtn{border:1px solid var(--blueprint-line);background:#fff;font-family:inherit;font-size:10.5px;
    padding:4px 12px;border-radius:3px;cursor:pointer;color:var(--blue-700)}
  .bp .contractbtn[aria-pressed="true"]{background:var(--blue-700);color:#fff}
  .bp .webscreen{flex:1;overflow:auto}
  .bp .webinner{max-width:880px;margin:0 auto;width:100%;padding:10px 24px 40px}
  .bp .webinner .screen{min-height:0;padding:14px 0 0}
  .bp .contractpanel{position:absolute;top:29px;bottom:0;right:0;width:480px;max-width:62%;z-index:5;
    background:var(--paper-2);border-left:1.5px solid var(--ink);box-shadow:-8px 0 24px rgba(11,31,51,.18);
    display:flex;flex-direction:column}
  .bp .contractpanel .bar{flex:none}
  .bp .contractpanel pre{flex:1;max-height:none}
  .bp .contractpanel .closex{position:absolute;top:6px;right:10px;border:0;background:transparent;cursor:pointer;
    font-family:inherit;font-size:13px;color:var(--text-faint,#5b7790)}
  `;

  const C = () => PF.contract;
  const fmt = (v, t) => t === 'money' ? '€' + (v / 100).toFixed(2) : t === 'integer' ? '×' + v : String(v);
  const flowOf = (id) => { const f = C().flows.find(f => f.pages.includes(id)); return f ? f.id : null; };

  function syntax(obj) {
    let s = JSON.stringify(obj, null, 2).replace(/&/g, '&amp;').replace(/</g, '&lt;');
    s = s.replace(/"([^"]+)":/g, '"<span class="k">$1</span>":');
    s = s.replace(/: "([^"]*)"/g, ': "<span class="s">$1</span>"');
    s = s.replace(/: (\d+)/g, ': <span class="n">$1</span>');
    return s;
  }

  function Wcag({ el }) {
    if (!el.wcag || !el.wcag.length) return null;
    return <div className="wcag">{el.wcag.map(c => <span key={c}>{c}</span>)}</div>;
  }
  function Lab({ el }) {
    return <div className="lab"><span className="typ">{el.aio}</span><span className="role">{el.role}</span></div>;
  }

  function Element({ el, data, ctx, chosen, onChoose, onGo }) {
    if (el.aio === 'display-collection') {
      const rows = data[el.binds] || [];
      if (ctx === 'desktop') {
        const gtc = `repeat(${el.item_shape.length}, 1fr)`;
        return (
          <div className="aio"><Lab el={el} />
            <div className="gc-list">
              <div className="row head" style={{ gridTemplateColumns: gtc }}>
                {el.item_shape.map(c => <span key={c.field}>{c.field}</span>)}
              </div>
              {rows.map((r, i) => (
                <div key={i} className="row" style={{ gridTemplateColumns: gtc }}>
                  {el.item_shape.map((c, j) => <span key={c.field} className={j === 0 ? 'nm' : 'dim'}>{fmt(r[c.field], c.type)}</span>)}
                </div>
              ))}
            </div><Wcag el={el} />
          </div>
        );
      }
      return (
        <div className="aio"><Lab el={el} />
          <div className="gc-stack">
            {rows.map((r, i) => (
              <div key={i} className="item">
                <div className="nm">{fmt(r[el.item_shape[0].field], el.item_shape[0].type)}</div>
                <div className="dim">{el.item_shape.slice(1).map(c => <span key={c.field}>{fmt(r[c.field], c.type)}</span>)}</div>
              </div>
            ))}
          </div><Wcag el={el} />
        </div>
      );
    }
    if (el.aio === 'display-value') {
      return (
        <div className="aio"><Lab el={el} />
          <div className={'gc-value' + (el.emphasis === 'primary' ? ' primary' : '')}>
            <span>{el.role}</span><span className="big">{fmt(data[el.binds], el.value_type)}</span>
          </div><Wcag el={el} />
        </div>
      );
    }
    if (el.aio === 'single-select') {
      const opts = data[el.binds] || [];
      // context of use decides the reification: phone → option list, desktop → segmented control (§4.5)
      const cls = ctx === 'desktop' && opts.length <= 5 ? 'gc-seg' : 'gc-select';
      return (
        <div className="aio"><Lab el={el} />
          <div className={cls} role="radiogroup">
            {opts.map((o, i) => (
              <div key={i} className="opt" role="radio" aria-checked={chosen === i} onClick={() => onChoose(i)}>{o[el.option_field]}</div>
            ))}
          </div><Wcag el={el} />
        </div>
      );
    }
    if (el.aio === 'trigger-action') {
      const cls = 'gc-button' + (el.emphasis === 'primary' ? ' primary' : '') + (ctx === 'desktop' ? ' inline' : '');
      return (
        <div className="aio" style={ctx === 'desktop' ? { display: 'flex', justifyContent: 'flex-end' } : null}>
          <button className={cls} onClick={() => el.transitions_to && onGo(el.transitions_to)}>
            <span className="typ">trigger-action · {el.issues}</span>{el.role}
          </button><Wcag el={el} />
        </div>
      );
    }
    return <div className="aio"><Lab el={el} /><div className="gc-value">unrendered AIO: {el.aio}</div></div>;
  }

  function ScreenPreview({ context, locale, screenId, setScreenId }) {
    const loc = locale || C().locale;
    const isWeb = context === 'web' || context === 'desktop';
    const elCtx = isWeb ? 'desktop' : 'phone';
    const [contractOpen, setContractOpen] = useState(false);
    const [forced, setForced] = useState(null);
    const [chosen, setChosen] = useState({});
    const [tab, setTab] = useState('screen');
    const current = screenId || C().start;
    const screen = PF.screen(current);
    const data = C().scenario.projected[screen.projection] || {};
    const state = (forced && screen.state_space.includes(forced)) ? forced : (data.state || 'present');
    const curFlow = flowOf(current);
    const flow = C().flows.find(f => f.id === curFlow);
    const go = (id) => { setScreenId(id); setForced(null); };

    const jsonObj = tab === 'screen' ? screen
      : tab === 'scenario' ? C().scenario
      : tab === 'content' ? { locale: loc, content_store: C().content_store }
      : C();

    const nav = C().root.destinations.map(d => {
      const cur = flowOf(d.to) === curFlow;
      return (
        <button key={d.to} aria-current={cur ? 'true' : 'false'} onClick={() => go(d.to)}>
          <span className="typ">navigate</span>{d.label}
        </button>
      );
    });

    const screenBody = (
      <div className="screen">
        <div className="stepname"><span>{screen.name}</span><span>{screen.projection}</span></div>
        {screen.content && screen.content.heading && (
          <div className="pageheading">
            <span className="ck">content: {screen.content.heading}</span>
            {PF.resolveContent(screen.content.heading, loc)}
          </div>
        )}
        {state !== 'present' ? (
          <div className={'state-box' + (state === 'failed' ? ' failed' : '')}>
            <span className="st">{state}</span>
            {state === 'loading' && <span className="spinner"></span>}
            <div>{(screen.state_content && screen.state_content[state])
              ? PF.resolveContent(screen.state_content[state], loc)
              : (screen.state_meanings[state] || state)}</div>
            {screen.state_content && screen.state_content[state] &&
              <span className="ck">content: {screen.state_content[state]}</span>}
          </div>
        ) : (
          screen.elements.map((el, i) => (
            <Element key={i} el={el} data={data} ctx={elCtx} chosen={chosen[current]}
              onChoose={(idx) => setChosen(c => ({ ...c, [current]: idx }))} onGo={go} />
          ))
        )}
      </div>
    );

    const crumbEl = (
      <div className="crumb">
        <span className="flowlab">{curFlow || '—'}:</span>
        {(flow ? flow.pages : [current]).map((pid, i) => (
          <React.Fragment key={pid}>
            {i > 0 && <span className="arr">→</span>}
            <span className={'c' + (pid === current ? ' here' : '')} onClick={() => go(pid)}>{PF.screen(pid).name}</span>
          </React.Fragment>
        ))}
      </div>
    );
    const stateSeg = (
      <div className="seg">
        {screen.state_space.map(st => (
          <button key={st} aria-pressed={st === state} onClick={() => setForced(st)}>{st}</button>
        ))}
      </div>
    );
    const inspector = (
      <React.Fragment>
        <div className="bar" role="tablist">
          {[['screen', 'This screen'], ['scenario', 'Scenario data'], ['content', 'Content store'], ['full', 'Full contract']].map(([k, l]) => (
            <button key={k} role="tab" aria-selected={tab === k} onClick={() => setTab(k)}>{l}</button>
          ))}
        </div>
        <pre dangerouslySetInnerHTML={{ __html: syntax(jsonObj) }} />
      </React.Fragment>
    );

    /* ---- web · full viewport: the surface IS the view ---- */
    if (isWeb) {
      return (
        <div className="bp web" style={{ position: 'absolute', inset: 0 }}>
          <style>{CSS}</style>
          <div className="chrome">
            <span>generic · no design system</span>
            <span className="url">shop.acme.com/{screen.id.replace('ui-', '')}</span>
            <span className="ctx">web · pointer · {loc}</span>
          </div>
          <div className="webbody">
            <nav className="sidenav" aria-label="Primary">
              <span className="navlab">navigate · root</span>
              {nav}
              <div className="globalslot">
                {C().root.global_actions.map(a => (
                  <button key={a.issues}><span className="typ">trigger-action · </span>{a.label}</button>
                ))}
              </div>
            </nav>
            <div className="webmain">
              <div className="webtoolbar">
                {crumbEl}
                <span style={{ marginLeft: 'auto', display: 'flex', gap: 12, alignItems: 'center' }}>
                  <span className="grp" style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                    <span className="glab" style={{ margin: 0 }}>projection state</span>
                    {stateSeg}
                  </span>
                  <div className="seg"><button onClick={() => { go(C().start); setChosen({}); }}>↺ start over</button></div>
                  <button className="contractbtn" aria-pressed={contractOpen}
                    onClick={() => setContractOpen(o => !o)}>{'{ }'} contract</button>
                </span>
              </div>
              <div className="webscreen">
                <div className="webinner">{screenBody}</div>
              </div>
            </div>
          </div>
          {contractOpen && (
            <div className="contractpanel">
              <button className="closex" onClick={() => setContractOpen(false)} aria-label="Close contract">×</button>
              {inspector}
            </div>
          )}
        </div>
      );
    }

    return (
      <div className="bp">
        <style>{CSS}</style>
        <div className="wrap">
          <div className="cols">
            <div>
              <h2 className="sec">render contract (consumed)</h2>
              <div className="contract">{inspector}</div>
            </div>

            <div>
              <h2 className="sec">rendered surface — phone · touch</h2>
              {crumbEl}

              <div className="device phone">
                <div className="chrome"><span>generic · no design system</span><span className="ctx">phone · touch · {loc}</span></div>
                <nav className="appnav" aria-label="Primary">{nav}</nav>
                {screenBody}
                <div className="globalbar">
                  {C().root.global_actions.map(a => (
                    <button key={a.issues}><span className="typ">trigger-action</span>{a.label}</button>
                  ))}
                </div>
              </div>

              <div className="controls">
                <div className="grp">
                  <span className="glab">projection state</span>
                  {stateSeg}
                </div>
                <div className="grp">
                  <span className="glab">reset</span>
                  <div className="seg"><button onClick={() => { go(C().start); setChosen({}); }}>↺ start over</button></div>
                </div>
              </div>
            </div>
          </div>

          <div className="note">
            The renderer is a pure function of the contract: <code>render(contract) → surface</code>. Every control is the
            schematic default for its AIO — no design system is coupled. The same contract reifies differently per context
            of use: the root's navigate edges become a tab bar on the phone and a full-viewport sidebar on the web; the same
            single-select becomes an option list or a segmented control (§4.5). If this is legible and walkable, the
            contract carries enough to build against.
          </div>
        </div>
      </div>
    );
  }

  Object.assign(window, { ScreenPreview });
})();
