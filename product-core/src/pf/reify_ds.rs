//! Design-system reification — the reify(AIO, context) → CIO evaluation, baked by value.
//!
//! Resolves the How-bound design system against the What graph once at plan
//! time: validity + coupling gaps fail the plan (the §11.2 gate moves *into*
//! reify), and every UI step gets its component map — for each surfaced/offered
//! AIO and each declared context of use, the resolved CIO, the tokens it
//! consumes, and the WCAG criteria it discharges by construction. The result
//! feeds the enriched `ScreenManifest`, the emitted `design-system.g.json` +
//! `tokens.g.css`, and the design-system provider seam.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::error::{ProductError, Result};

use super::manifest::{applies, DsManifest};
use super::model::{ContextOfUse, DomainGraph};
use super::provenance::content_hash;

/// A design system as a reify input: the parsed manifest plus the hash of its
/// stored YAML (the identity `reify check` pins alongside the graph hash).
#[derive(Clone)]
pub struct DsSpec {
    pub manifest: DsManifest,
    pub hash: String,
}

impl DsSpec {
    /// Build a spec from the raw manifest YAML (as stored under
    /// `.product/design-systems/<id>/`).
    pub fn from_source(manifest: DsManifest, source: &str) -> Self {
        Self { manifest, hash: content_hash(source) }
    }
}

/// One AIO reified for one context: the on-system component, its tokens, and
/// the WCAG guarantees the binding inherits (§11.4).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReifiedComponent {
    /// The AIO the step references.
    pub aio: String,
    /// What it binds: the surfaced projection or the offered command.
    pub binds: String,
    /// `surface` | `offer`.
    pub role: String,
    /// The context-of-use id this resolution holds for (`any` when the What
    /// declares none).
    pub context: String,
    /// The resolved on-system component (closed vocabulary).
    pub cio: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<String>,
    /// WCAG 2.2 criteria the component satisfies by construction.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub wcag: Vec<String>,
}

/// The whole resolution: per-step component maps plus the pinned identity.
#[derive(Debug)]
pub struct ResolvedDs {
    pub id: String,
    pub version: String,
    pub hash: String,
    /// UI step id → its reified components (every surface/offer × context).
    pub screens: BTreeMap<String, Vec<ReifiedComponent>>,
}

/// Resolve the design system against the graph — the reify gate. Wholeness
/// findings (§11.3) or coupling gaps (§11.2) fail the plan; nothing is emitted
/// from a design system that cannot realise the What.
pub fn resolve(spec: &DsSpec, graph: &DomainGraph) -> Result<ResolvedDs> {
    let mut findings = super::manifest::validate_ds(&spec.manifest);
    findings.extend(super::manifest::couple_ds(&spec.manifest, graph));
    findings.extend(wildcard_gaps(spec, graph));
    if !findings.is_empty() {
        return Err(ProductError::ConfigError(format!(
            "design system '{}' cannot realise this What — fix before reifying:\n  - {}",
            spec.manifest.design_system.id,
            findings.join("\n  - ")
        )));
    }
    let contexts = context_ids(graph);
    let mut screens = BTreeMap::new();
    for step in super::reify_screen::testable_steps(graph) {
        let mut comps = Vec::new();
        for s in &step.surfaces {
            comps.extend(reify_one(spec, graph, &s.aio, &s.projection, "surface", &contexts));
        }
        for o in &step.offers {
            comps.extend(reify_one(spec, graph, &o.aio, &o.command, "offer", &contexts));
        }
        screens.insert(step.id.clone(), comps);
    }
    let ds = &spec.manifest.design_system;
    Ok(ResolvedDs {
        id: ds.id.clone(),
        version: ds.version.clone(),
        hash: spec.hash.clone(),
        screens,
    })
}

/// The declared contexts of use, or the single wildcard `any`.
fn context_ids(graph: &DomainGraph) -> Vec<String> {
    if graph.contexts_of_use.is_empty() {
        return vec!["any".to_string()];
    }
    graph.contexts_of_use.iter().map(|c| c.id.clone()).collect()
}

/// Coverage for a What that declares no contexts of use: every referenced AIO
/// still needs a rule applying to the `any` wildcard (§11.2 gates coverage
/// per declared context; with none declared, the whole space is one context).
fn wildcard_gaps(spec: &DsSpec, graph: &DomainGraph) -> Vec<String> {
    if !graph.contexts_of_use.is_empty() {
        return Vec::new();
    }
    let any = ContextOfUse { id: "any".to_string(), ..Default::default() };
    let aios: std::collections::BTreeSet<&str> = graph
        .wireframe_steps
        .iter()
        .flat_map(|s| {
            s.surfaces.iter().map(|x| x.aio.as_str()).chain(s.offers.iter().map(|o| o.aio.as_str()))
        })
        .collect();
    aios.iter()
        .filter(|aio| {
            !spec.manifest.design_system.reification.iter().any(|r| &r.aio == *aio && applies(&r.when, &any))
        })
        .map(|aio| format!("no reify({aio}) rule applies (no contexts of use declared — a wildcard rule is required)"))
        .collect()
}

/// Reify one (AIO, binding) across every context. Coupling was already gated,
/// so a missing rule here only happens for the `any` wildcard — skipped.
fn reify_one(
    spec: &DsSpec,
    graph: &DomainGraph,
    aio: &str,
    binds: &str,
    role: &str,
    contexts: &[String],
) -> Vec<ReifiedComponent> {
    let ds = &spec.manifest.design_system;
    let mut out = Vec::new();
    for ctx_id in contexts {
        let ctx = graph.contexts_of_use.iter().find(|c| &c.id == ctx_id).cloned().unwrap_or_else(
            || ContextOfUse { id: ctx_id.clone(), ..Default::default() },
        );
        let Some(rule) = ds.reification.iter().find(|r| r.aio == aio && applies(&r.when, &ctx))
        else {
            continue;
        };
        let component = ds.components.iter().find(|c| c.id == rule.cio);
        out.push(ReifiedComponent {
            aio: aio.to_string(),
            binds: binds.to_string(),
            role: role.to_string(),
            context: ctx_id.clone(),
            cio: rule.cio.clone(),
            tokens: component.map(|c| c.tokens.clone()).unwrap_or_default(),
            wcag: component
                .map(|c| c.satisfies.iter().map(|s| s.criterion.clone()).collect())
                .unwrap_or_default(),
        });
    }
    out
}

/// `design-system.g.json` — the resolved component map + token values, by
/// value, hash-pinned (the same philosophy as the oracle manifest).
pub fn ds_json(resolved: &ResolvedDs, spec: &DsSpec, graph_hash: &str) -> String {
    let ds = &spec.manifest.design_system;
    let tokens: Vec<serde_json::Value> = ds
        .tokens
        .iter()
        .map(|t| {
            serde_json::json!({ "id": t.id, "type": t.kind, "values": t.values })
        })
        .collect();
    let v = serde_json::json!({
        "design_system": { "id": resolved.id, "version": resolved.version, "hash": format!("sha256:{}", resolved.hash) },
        "graph_hash": format!("sha256:{graph_hash}"),
        "themes": ds.themes,
        "targets": ds.targets,
        "tokens": tokens,
        "screens": resolved.screens,
    });
    let mut s = serde_json::to_string_pretty(&v).unwrap_or_default();
    s.push('\n');
    s
}

/// A token id as a CSS custom-property name (`color.on-accent` → `--color-on-accent`).
pub fn css_var(token_id: &str) -> String {
    format!("--{}", token_id.replace('.', "-"))
}

/// `tokens.g.css` — the token surface as CSS custom properties: the first
/// declared theme (or valueless declarations) on `:root`, every other theme
/// under `[data-theme="…"]`. Screens style through these variables only.
pub fn tokens_css(spec: &DsSpec) -> String {
    let ds = &spec.manifest.design_system;
    let themes: Vec<&str> = if ds.themes.is_empty() {
        vec![]
    } else {
        ds.themes.iter().map(String::as_str).collect()
    };
    let mut s = format!(
        "/* generated by `product reify` — design system '{}' v{} — tokens, not literals (§4.5) */\n",
        ds.id, ds.version
    );
    let block = |selector: &str, theme: Option<&str>| -> String {
        let mut b = format!("{selector} {{\n");
        for t in &ds.tokens {
            let value = theme.and_then(|th| t.values.get(th)).cloned();
            match value {
                Some(v) => b.push_str(&format!("  {}: {v};\n", css_var(&t.id))),
                None => b.push_str(&format!("  {}: initial; /* {} — no value declared */\n", css_var(&t.id), t.kind)),
            }
        }
        b.push_str("}\n");
        b
    };
    match themes.split_first() {
        None => s.push_str(&block(":root", None)),
        Some((first, rest)) => {
            s.push_str(&block(":root", Some(first)));
            for th in rest {
                s.push_str(&block(&format!("[data-theme=\"{th}\"]"), Some(th)));
            }
        }
    }
    s
}

/// `DesignSystem.g.cs` — the design-system provider seam, parallel to the
/// conformance adapter: a token/component lookup interface plus the resolved
/// (step, aio, context) → CIO catalog baked as data.
pub fn catalog_cs(header: &str, ns: &str, resolved: &ResolvedDs) -> String {
    let mut rows = String::new();
    for (step, comps) in &resolved.screens {
        for c in comps {
            rows.push_str(&format!(
                "        new(\"{}\", \"{}\", \"{}\", \"{}\", \"{}\"),\n",
                cs(step), cs(&c.aio), cs(&c.binds), cs(&c.context), cs(&c.cio)
            ));
        }
    }
    format!(
        "{header}#nullable enable\n\nusing System.Collections.Generic;\n\nnamespace {ns};\n\n\
/// <summary>One reify(AIO, context) → CIO resolution for a UI step (§4.5/§11.2).</summary>\n\
public sealed record ReifiedComponent(string StepId, string Aio, string Binds, string Context, string Cio);\n\n\
/// <summary>The design-system seam: resolve a component and its token values at render\n\
/// time. The realiser implements this over the bound design system's bundle.</summary>\n\
public interface IDesignSystemProvider\n{{\n    object Component(string cio);\n    string Token(string tokenId);\n}}\n\n\
/// <summary>The resolved catalog — design system '{id}' v{version} (sha256:{hash}).</summary>\n\
public static class DesignSystemCatalog\n{{\n    public static readonly IReadOnlyList<ReifiedComponent> Resolutions = new ReifiedComponent[]\n    {{\n{rows}    }};\n}}\n",
        id = resolved.id, version = resolved.version, hash = resolved.hash,
    )
}

fn cs(s: &str) -> String {
    super::reify_ident::cs_escape(s)
}

#[cfg(test)]
#[path = "reify_ds_tests.rs"]
mod tests;
