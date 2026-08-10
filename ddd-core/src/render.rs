//! `ddd render` — the graph projected to one self-contained HTML page.
//!
//! A projection, never a second source of truth (PRD §3 non-goals): no
//! server, no scripts, no external assets, no editable state, nothing
//! time-stamped — regenerating from an unchanged graph is byte-identical.
//! Four sections: status-coloured claims, decision chains with pin state,
//! manifest coverage, the escapes dashboard; plus the M4 seam map.

use std::fmt::Write as _;

use crate::report::EscapesReport;
use crate::store::DddStore;

/// The whole page as one string; the caller owns writing it to disk.
pub fn render_html(store: &DddStore, escapes: &EscapesReport, today: &str) -> String {
    let mut b = String::with_capacity(32 * 1024);
    let _ = writeln!(b,
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>DDD governance graph</title>\n<style>{CSS}</style>\n</head>\n<body>\n<h1>DDD governance graph</h1>\n<p class=\"muted\">A static projection of <code>.ddd/</code> — regenerated, never edited. Cadence judged against {}.</p>",
        esc(today)
    );
    claims_section(&mut b, store);
    decisions_section(&mut b, store);
    manifest_section(&mut b, store);
    escapes_section(&mut b, escapes);
    seams_section(&mut b, store);
    b.push_str("</body>\n</html>\n");
    b
}

/// The page's stylesheet — a real `.css` asset so the M7 pair governance
/// (interception, the class contract check, the token discipline) applies
/// to it; inlined into `<style>` at render time so the output stays one
/// self-contained file.
const CSS: &str = include_str!("../assets/render.css");

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn claims_section(b: &mut String, store: &DddStore) {
    let _ = writeln!(b, "<h2>Claims ({})</h2>", store.claims.len());
    b.push_str("<table><tr><th>id</th><th>status</th><th>statement</th><th>falsifier</th><th>changed</th><th>revalidate by</th></tr>\n");
    for c in &store.claims {
        let status = c.status.as_str();
        let _ = writeln!(b,
            "<tr><td><code>{}</code></td><td><span class=\"chip {status}\">{status}</span></td><td>{}</td><td class=\"muted\">{}</td><td>{}</td><td>{}</td></tr>",
            esc(&c.id),
            esc(c.statement.trim()),
            esc(c.falsifier.as_deref().unwrap_or("—").trim()),
            esc(&c.changed),
            esc(c.revalidate_by.as_deref().unwrap_or("—")),
        );
    }
    b.push_str("</table>\n");
}

fn decisions_section(b: &mut String, store: &DddStore) {
    let _ = writeln!(b, "<h2>Decisions ({})</h2>", store.decisions.len());
    for d in &store.decisions {
        let kind = match d.kind {
            crate::decision::DecisionKind::Decision => "decision",
            crate::decision::DecisionKind::RiskAcceptance => "risk acceptance",
        };
        let _ = writeln!(b,
            "<div class=\"card\"><h3><code>{}</code> — {}</h3>\n<p class=\"muted\">{kind} · principal {}{}</p>\n<p>{}</p>",
            esc(&d.id),
            esc(&d.title),
            esc(&d.principal),
            d.date.as_deref().map(|x| format!(" · {}", esc(x))).unwrap_or_default(),
            esc(d.rationale.trim()),
        );
        if !d.based_on.is_empty() {
            b.push_str("<ul>\n");
            for basis in &d.based_on {
                basis_line(b, store, basis);
            }
            b.push_str("</ul>\n");
        }
        b.push_str("</div>\n");
    }
}

/// One basedOn edge with its pin state against the current claim.
fn basis_line(b: &mut String, store: &DddStore, basis: &crate::decision::BasedOn) {
    let id = basis.claim_id();
    let current = store.claims.iter().find(|c| c.id == id);
    let state = match (basis.pin(), current) {
        (_, None) => "<span class=\"bad\">claim not found</span>".to_string(),
        (None, Some(c)) => {
            format!("<span class=\"muted\">unpinned (format 1) — currently {}</span>", c.status.as_str())
        }
        (Some(pin), Some(c)) => {
            if pin.status == c.status && pin.changed == c.changed {
                format!("<span class=\"ok\">pin holds ({}@{})</span>", pin.status.as_str(), esc(&pin.changed))
            } else {
                format!(
                    "<span class=\"bad\">basis moved: pinned {}@{}, now {}@{}</span>",
                    pin.status.as_str(),
                    esc(&pin.changed),
                    c.status.as_str(),
                    esc(&c.changed)
                )
            }
        }
    };
    let _ = writeln!(b, "<li>basedOn <code>{}</code> — {state}</li>", esc(id));
}

fn manifest_section(b: &mut String, store: &DddStore) {
    let total: usize = store.manifests.iter().map(|m| m.file.rules.len()).sum();
    let _ = writeln!(b, "<h2>Manifest coverage ({total} rules)</h2>");
    for set in &store.manifests {
        if set.file.rules.is_empty() {
            continue;
        }
        let _ = writeln!(b, "<h3><code>{}</code></h3>", esc(&set.name));
        b.push_str("<table><tr><th>rule</th><th>severity</th><th>governance</th><th>suppression</th></tr>\n");
        for rule in &set.file.rules {
            let governance = match (&rule.decision, &rule.governance) {
                (Some(d), _) => format!("<code>{}</code>", esc(d)),
                (None, Some(g)) => format!("<span class=\"warn\">{}</span>", esc(g)),
                (None, None) => "<span class=\"bad\">no decision (rule 4)</span>".to_string(),
            };
            let suppression = match &rule.suppression {
                None => "—".to_string(),
                Some(s) => match s.risk_acceptance.as_deref() {
                    Some(id) if store.decisions.iter().any(|d| d.id == id) => {
                        format!("cites <code>{}</code>", esc(id))
                    }
                    Some(id) => format!("<span class=\"bad\">cites missing <code>{}</code></span>", esc(id)),
                    None => "<span class=\"bad\">uncited</span>".to_string(),
                },
            };
            let _ = writeln!(b,
                "<tr><td><code>{}</code></td><td>{}</td><td>{governance}</td><td>{suppression}</td></tr>",
                esc(&rule.rule_id),
                esc(&rule.severity),
            );
        }
        b.push_str("</table>\n");
    }
}

fn escapes_section(b: &mut String, escapes: &EscapesReport) {
    b.push_str("<h2>Escapes dashboard</h2>\n");
    if escapes.is_clean() {
        b.push_str("<p class=\"ok\">No escapes: no diff findings, no cadence violations, no basis loss.</p>\n");
    }
    diff_findings_table(b, escapes);
    cadence_table(b, escapes);
    basis_loss_table(b, escapes);
    for note in &escapes.diff.notes {
        let _ = writeln!(b, "<p class=\"muted\">note: {}</p>", esc(note));
    }
}

fn diff_findings_table(b: &mut String, escapes: &EscapesReport) {
    if escapes.diff.findings.is_empty() {
        return;
    }
    let _ = writeln!(b, "<h3>Governance diff ({})</h3>", escapes.diff.findings.len());
    b.push_str("<table><tr><th>kind</th><th>namespace</th><th>rule</th><th>detail</th></tr>\n");
    for f in &escapes.diff.findings {
        let _ = writeln!(b,
            "<tr><td class=\"bad\">{}</td><td><code>{}</code></td><td><code>{}</code></td><td>{}</td></tr>",
            f.kind.label(),
            esc(&f.namespace),
            esc(&f.rule_id),
            esc(&f.detail),
        );
    }
    b.push_str("</table>\n");
}

fn cadence_table(b: &mut String, escapes: &EscapesReport) {
    if escapes.cadence.is_empty() {
        return;
    }
    let _ = writeln!(b, "<h3>Cadence violations ({})</h3>", escapes.cadence.len());
    b.push_str("<table><tr><th>claim</th><th>status</th><th>revalidate by</th></tr>\n");
    for c in &escapes.cadence {
        let _ = writeln!(b,
            "<tr><td><code>{}</code></td><td>{}</td><td class=\"bad\">{}</td></tr>",
            esc(&c.claim),
            c.status.as_str(),
            esc(&c.revalidate_by),
        );
    }
    b.push_str("</table>\n");
}

fn basis_loss_table(b: &mut String, escapes: &EscapesReport) {
    if escapes.basis_loss.is_empty() {
        return;
    }
    let _ = writeln!(b, "<h3>Basis loss ({})</h3>", escapes.basis_loss.len());
    b.push_str("<table><tr><th>decision</th><th>claim</th><th>pinned</th><th>current</th></tr>\n");
    for l in &escapes.basis_loss {
        let current = match (&l.current_status, &l.current_changed) {
            (Some(s), Some(c)) => format!("{}@{}", s.as_str(), esc(c)),
            _ => "gone".to_string(),
        };
        let _ = writeln!(b,
            "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}@{}</td><td class=\"bad\">{current}</td></tr>",
            esc(&l.decision),
            esc(&l.claim),
            l.pinned_status.as_str(),
            esc(&l.pinned_changed),
        );
    }
    b.push_str("</table>\n");
}

/// The M4 seam map: declarations plus interception rows.
fn seams_section(b: &mut String, store: &DddStore) {
    if store.seams.is_empty() && store.seam_events.is_empty() {
        return;
    }
    let _ = writeln!(b,
        "<h2>Seam map ({} declarations, {} interception rows)</h2>",
        store.seams.len(),
        store.seam_events.len()
    );
    for s in &store.seams {
        let vk = if s.verdict_knowledge.trim().is_empty() {
            "<span class=\"bad\">no verdict knowledge — seam cost with no demand absorbed</span>".to_string()
        } else {
            esc(&s.verdict_knowledge)
        };
        let _ = writeln!(b,
            "<div class=\"card\"><h3><code>{}</code> — {}</h3><p>{vk}</p><p class=\"muted\">contract: {}</p></div>",
            esc(&s.id),
            esc(&s.boundary),
            esc(&s.contract_location),
        );
    }
    if !store.seam_events.is_empty() {
        b.push_str("<table><tr><th>row</th><th>file</th><th>symbol</th><th>change</th><th>outcome</th><th>refs</th><th>linked</th></tr>\n");
        for e in &store.seam_events {
            let _ = writeln!(b,
                "<tr><td><code>{}</code></td><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                esc(&e.id),
                esc(&e.file),
                esc(&e.symbol),
                esc(&e.change),
                esc(&e.outcome),
                e.reference_count.map(|n| n.to_string()).unwrap_or_else(|| "—".into()),
                e.linked_declaration.as_deref().map(esc).unwrap_or_else(|| "—".into()),
            );
        }
        b.push_str("</table>\n");
    }
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
