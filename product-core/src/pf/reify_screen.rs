//! Screen-harness emission — the §4.5 seam checks aimed at a realised UI.
//!
//! The UI oracle is deliberately partial: the graph pins *structure and
//! behaviour* — every surfaced projection rendered in every non-waived
//! state, every offer wired to the command it issues — and says nothing
//! about pixels. `ScreenSeam.g.cs` carries the headless protocol
//! (`IScreenAdapter`: render a step in a state with view data, answer
//! what was surfaced/offered); the realiser implements a scaffolded
//! `ScreenAdapter` over their actual UI (component render, DOM harness,
//! whatever their framework is), and one generated fact per (step, state)
//! holds it to the graph. Present-state fixtures come from the projector
//! oracle — a projector scenario's folded view *is* the screen's data.

use std::collections::BTreeSet;

use super::model::DomainGraph;
use super::model_ui::WireframeStep;
use super::projector::Projector;
use super::projector_sim::project;
use super::reify_ident::{cs_escape, method_name, pascal};
use super::reify_scenarios::count_assert;

const SCREEN_SEAM_CS: &str = r##"#nullable enable

using System.Collections.Generic;

namespace {NS};

/// <summary>What a screen actually put in front of the user, in graph vocabulary:
/// which projections it surfaced and which commands it offered (§3.2.1).</summary>
public sealed record RenderedScreen(IReadOnlyList<string> Projections, IReadOnlyList<string> OfferedCommands);

/// <summary>The UI seam (§4.5/§6.3): render one UI step in one projection state
/// with the given view data, and report the rendered structure. The oracle pins
/// structure and behaviour — pixels, layout, and framework are the realiser's.</summary>
public interface IScreenAdapter
{
    RenderedScreen Render(string stepId, string projectionState, IReadOnlyDictionary<string, object?> view);
}
"##;

const SCREEN_STUB_CS: &str = r##"// Scaffolded once by `product reify csharp` — never overwritten.
// Implement the §4.5 screen seam over your real UI: render the step headlessly
// (component harness, test renderer, DOM driver — your framework's choice) and
// answer which projections were surfaced and which commands were offered.
#nullable enable

using System;
using System.Collections.Generic;

namespace {NS};

public sealed class ScreenAdapter : IScreenAdapter
{
    public RenderedScreen Render(string stepId, string projectionState, IReadOnlyDictionary<string, object?> view)
    {
        // TODO: mount the screen for `stepId` with `view` in `projectionState`,
        // then report what it surfaced/offered, e.g.
        //   return new RenderedScreen(harness.VisibleProjections(), harness.CommandsWired());
        throw new NotImplementedException($"realise the screen adapter for '{stepId}'");
    }
}
"##;

/// `ScreenSeam.g.cs` for the given namespace.
pub fn seam_file(header: &str, ns: &str) -> String {
    format!("{header}{}", SCREEN_SEAM_CS.replace("{NS}", ns))
}

/// The scaffolded-once `ScreenAdapter.cs` stub.
pub fn adapter_stub(ns: &str) -> String {
    SCREEN_STUB_CS.replace("{NS}", ns)
}

/// The UI steps that have anything to hold to the graph.
pub fn testable_steps(graph: &DomainGraph) -> Vec<&WireframeStep> {
    graph
        .wireframe_steps
        .iter()
        .filter(|s| !s.surfaces.is_empty() || !s.offers.is_empty())
        .collect()
}

/// Render `<Step>ScreenTests.g.cs`: a present-state fact asserting every
/// surface + offer, plus one fact per non-waived degraded state meaning.
pub fn tests_file(header: &str, ns: &str, step: &WireframeStep, projectors: &[&Projector]) -> String {
    let base = pascal(&step.id);
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System.Collections.Generic;\nusing Xunit;\n");
    s.push_str(&format!("using {ns};\n\nnamespace {ns}.Tests;\n\n"));
    s.push_str(&format!(
        "/// <summary>§4.5 seam facts for UI step '{}' — structure and state coverage against the realised screen.</summary>\npublic class {base}ScreenTests\n{{\n",
        cs_escape(&step.id)
    ));
    s.push_str(&present_fact(step, projectors));
    let mut seen = BTreeSet::new();
    for m in &step.state_meanings {
        if m.waiver.is_none() && m.state != "present" && seen.insert((m.projection.clone(), m.state.clone())) {
            s.push_str(&state_fact(step, &m.projection, &m.state));
        }
    }
    s.push_str("}\n");
    s
}

fn present_fact(step: &WireframeStep, projectors: &[&Projector]) -> String {
    let mut s = String::from(
        "    [Fact]\n    public void Present_state_surfaces_every_projection_and_offer()\n    {\n",
    );
    s.push_str(&format!(
        "        var screen = new ScreenAdapter().Render(\"{}\", \"present\", {});\n",
        cs_escape(&step.id),
        present_fixture(step, projectors)
    ));
    for surface in &step.surfaces {
        s.push_str(&format!(
            "        Assert.Contains(\"{}\", screen.Projections);\n",
            cs_escape(&surface.projection)
        ));
    }
    s.push_str(&count_assert("screen.OfferedCommands", step.offers.len()));
    for offer in &step.offers {
        s.push_str(&format!(
            "        Assert.Contains(\"{}\", screen.OfferedCommands);\n",
            cs_escape(&offer.command)
        ));
    }
    s.push_str("    }\n\n");
    s
}

/// The present-state view data: the first surfaced projection's projector
/// oracle view (its first scenario), else an empty dictionary.
fn present_fixture(step: &WireframeStep, projectors: &[&Projector]) -> String {
    let view = step.surfaces.iter().find_map(|surface| {
        let p = projectors.iter().find(|p| p.projects_for == surface.projection)?;
        let scenario = p.scenarios.first()?;
        project(p.logic.as_ref()?, &scenario.given).ok()
    });
    let Some(view) = view else {
        return "new Dictionary<string, object?>()".to_string();
    };
    let fields: Vec<String> = view
        .iter()
        .map(|(k, v)| {
            let lit = match v {
                super::decider_logic::Scalar::Bool(b) => b.to_string(),
                super::decider_logic::Scalar::Int(i) => format!("{i}L"),
                super::decider_logic::Scalar::Str(t) => format!("\"{}\"", cs_escape(t)),
            };
            format!("[\"{}\"] = {lit}", cs_escape(k))
        })
        .collect();
    format!("new Dictionary<string, object?> {{ {} }}", fields.join(", "))
}

fn state_fact(step: &WireframeStep, projection: &str, state: &str) -> String {
    let name = method_name(&format!("{projection} {state} state is handled"));
    format!(
        "    [Fact]\n    public void {name}()\n    {{\n        var screen = new ScreenAdapter().Render(\"{}\", \"{}\", new Dictionary<string, object?>());\n        Assert.Contains(\"{}\", screen.Projections);\n    }}\n\n",
        cs_escape(&step.id),
        cs_escape(state),
        cs_escape(projection)
    )
}
