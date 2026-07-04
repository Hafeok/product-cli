//! Flow-fact emission — the §3.2 chain replayed across both seams.
//!
//! A flow is a connected chain of the atomic pattern (command → events →
//! view). At emit time the *Rust* oracle walks the chain: each command
//! step is decided against the state folded from the accumulated stream
//! (payload borrowed from the Decider's first accepting scenario), the
//! emitted events extend the stream, and each read-model step projects
//! the stream into its oracle view. The generated fact replays exactly
//! that chain through the C# adapters and asserts the baked outcomes —
//! cross-slice integration with no new authored artifact.

use std::collections::BTreeSet;

use super::decider::Decider;
use super::decider_logic::{CommandRef, EventRef, Scenario};
use super::decider_sim::{decide, replay, Outcome};
use super::model::DomainGraph;
use super::projector::Projector;
use super::projector_sim::project;
use super::reify_ident::{cs_escape, method_name, pascal};
use super::reify_oracle::wire_new;
use super::reify_projector::view_base;
use super::reify_scenarios::count_assert;

/// One command step of a computed chain: who decides, with what payload,
/// and the oracle's outcome.
struct CmdStep {
    decider_id: String,
    adapter: String,
    when: CommandRef,
    outcome: Outcome,
}

/// One read-model step: the projector and its oracle view over the stream.
struct ViewCheck {
    projector_id: String,
    adapter: String,
    view: super::decider_logic::State,
}

/// A fully computed flow chain, ready to render as a fact.
pub struct FlowFact {
    name: String,
    cmds: Vec<CmdStep>,
    views: Vec<ViewCheck>,
}

/// Compute a chain per flow. A flow is skipped (not a fact) when a command
/// step has no Decider with logic + an accepting scenario, or a read-model
/// step has no Projector with logic — the oracle cannot bake it.
pub fn plan_flows(
    graph: &DomainGraph,
    deciders: &[&Decider],
    projectors: &[&Projector],
    oracle_only: bool,
) -> Vec<FlowFact> {
    graph
        .flows
        .iter()
        .filter_map(|f| plan_one(f, deciders, projectors, oracle_only))
        .collect()
}

fn plan_one(
    flow: &super::model_ui::Flow,
    deciders: &[&Decider],
    projectors: &[&Projector],
    oracle_only: bool,
) -> Option<FlowFact> {
    let mut stream: Vec<EventRef> = Vec::new();
    let mut cmds = Vec::new();
    let mut views = Vec::new();
    for step in &flow.steps {
        if let Some(d) = deciders.iter().find(|d| d.handles.iter().any(|h| h == step)) {
            cmds.push(command_step(d, step, &mut stream, oracle_only)?);
        } else if let Some(p) = projectors.iter().find(|p| &p.projects_for == step) {
            let logic = p.logic.as_ref()?;
            views.push(ViewCheck {
                projector_id: p.id.clone(),
                adapter: projection_adapter(p, oracle_only),
                view: project(logic, &stream).ok()?,
            });
        }
        // Event / trigger / ui-step ids are chain documentation, not drives.
    }
    if cmds.is_empty() {
        return None;
    }
    Some(FlowFact { name: method_name(&flow.label), cmds, views })
}

fn command_step(
    d: &Decider,
    command: &str,
    stream: &mut Vec<EventRef>,
    oracle_only: bool,
) -> Option<CmdStep> {
    let logic = d.logic.as_ref()?;
    let when = accepting_payload(&d.scenarios, command)?;
    let state = replay(logic, stream).ok()?;
    let outcome = decide(logic, &state, &when).ok()?;
    if let Outcome::Accepted(events) = &outcome {
        for e in events {
            stream.push(EventRef::Data { event: e.event.clone(), with: e.payload.clone() });
        }
    }
    Some(CmdStep {
        decider_id: d.id.clone(),
        adapter: if oracle_only {
            "ConformanceAdapter".to_string()
        } else {
            format!("{}Adapter", pascal(&d.decides_for))
        },
        when,
        outcome,
    })
}

/// The first scenario that accepts this command supplies the payload.
fn accepting_payload(scenarios: &[Scenario], command: &str) -> Option<CommandRef> {
    scenarios
        .iter()
        .find(|s| s.when.id() == command && s.then.emit.is_some())
        .map(|s| s.when.clone())
}

fn projection_adapter(p: &Projector, oracle_only: bool) -> String {
    if oracle_only {
        "ProjectionAdapter".to_string()
    } else {
        format!("{}ProjectionAdapter", view_base(p))
    }
}

/// Render `FlowTests.g.cs` — one fact per computed flow chain.
pub fn tests_file(header: &str, ns: &str, facts: &[FlowFact]) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System.Collections.Generic;\nusing Xunit;\n");
    s.push_str(&format!("using {ns};\n\nnamespace {ns}.Tests;\n\n"));
    s.push_str("/// <summary>§3.2 flow chains replayed across both seams — the oracle baked each step's outcome at generation time.</summary>\npublic class FlowTests\n{\n");
    let mut seen = BTreeSet::new();
    for f in facts {
        s.push_str(&fact(f, &mut seen));
    }
    s.push_str("}\n");
    s
}

fn fact(f: &FlowFact, seen: &mut BTreeSet<String>) -> String {
    let mut name = f.name.clone();
    while !seen.insert(name.clone()) {
        name.push('_');
    }
    let mut s = format!("    [Fact]\n    public void {name}()\n    {{\n");
    s.push_str("        var stream = new List<WireEvent>();\n");
    for (i, c) in f.cmds.iter().enumerate() {
        s.push_str(&cmd_block(i, c));
    }
    for (i, v) in f.views.iter().enumerate() {
        s.push_str(&view_block(i, v));
    }
    s.push_str("    }\n\n");
    s
}

fn cmd_block(i: usize, c: &CmdStep) -> String {
    let mut s = format!(
        "        var o{i} = new {}().Run(\"{}\", stream, {});\n",
        c.adapter,
        cs_escape(&c.decider_id),
        wire_new("WireCommand", c.when.id(), &c.when.payload())
    );
    match &c.outcome {
        Outcome::Rejected(inv) => {
            s.push_str(&format!("        Assert.Equal(\"{}\", o{i}.Reject);\n", cs_escape(inv)));
        }
        Outcome::Accepted(events) => {
            s.push_str(&format!("        Assert.Null(o{i}.Reject);\n"));
            s.push_str(&count_assert(&format!("o{i}.Emit!"), events.len()));
            for (j, e) in events.iter().enumerate() {
                s.push_str(&format!(
                    "        Assert.Equal(\"{}\", o{i}.Emit![{j}].Id);\n",
                    cs_escape(&e.event)
                ));
            }
            s.push_str(&format!("        stream.AddRange(o{i}.Emit!);\n"));
        }
    }
    s
}

fn view_block(i: usize, v: &ViewCheck) -> String {
    let mut s = format!(
        "        var view{i} = new {}().Run(\"{}\", stream);\n",
        v.adapter,
        cs_escape(&v.projector_id)
    );
    s.push_str(&count_assert(&format!("view{i}"), v.view.len()));
    for (k, val) in &v.view {
        let (ty, lit) = match val {
            super::decider_logic::Scalar::Bool(b) => ("bool", b.to_string()),
            super::decider_logic::Scalar::Int(n) => ("long", format!("{n}L")),
            super::decider_logic::Scalar::Str(t) => ("string", format!("\"{}\"", cs_escape(t))),
        };
        s.push_str(&format!(
            "        Assert.Equal({lit}, Assert.IsType<{ty}>(view{i}[\"{}\"]));\n",
            cs_escape(k)
        ));
    }
    s
}
