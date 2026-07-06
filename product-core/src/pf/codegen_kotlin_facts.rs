//! Kotlin fact rendering — the oracle's scenarios in `kotlin.test` form.
//!
//! Renders the same language-neutral fact data the C# emitters consume —
//! Decider/Projector scenarios, oracle-baked flow chains, screen seam
//! checks — as Kotlin test classes driving the scaffolded adapters.
//! Assertion style: `assertEquals` over the wire scalar alphabet (boxed
//! Long/Boolean/String make type mismatches inequality, so no IsType
//! counterpart is needed).

use std::collections::BTreeSet;

use super::decider::Decider;
use super::decider_logic::{EventRef, Expectation, Payload, Scalar};
use super::model_ui::WireframeStep;
use super::projector::Projector;
use super::codegen_flow::FlowFact;
use super::codegen_ident::{cs_escape, method_name, pascal};

/// `<Agg>ScenarioTests.g.kt` — decider scenarios through `ConformanceAdapter`.
pub fn decider_tests(hdr: &str, pkg: &str, agg: &str, decider: &Decider) -> String {
    let mut s = test_header(hdr, pkg);
    s.push_str(&format!("class {agg}ScenarioTests {{\n"));
    let mut seen = BTreeSet::new();
    for sc in &decider.scenarios {
        let name = unique(&mut seen, &sc.name);
        s.push_str(&format!("    @Test\n    fun {name}() {{\n"));
        s.push_str(&format!(
            "        val outcome = ConformanceAdapter().run(\"{}\", {}, {})\n",
            cs_escape(&decider.id),
            given_expr(&sc.given),
            wire_new("WireCommand", sc.when.id(), &sc.when.payload())
        ));
        s.push_str(&expectation(&sc.then));
        s.push_str("    }\n\n");
    }
    s.push_str("}\n");
    s
}

/// `<View>ProjectionTests.g.kt` — projector scenarios through `ProjectionAdapter`.
pub fn projector_tests(hdr: &str, pkg: &str, projector: &Projector) -> String {
    let base = pascal(&projector.projects_for);
    let mut s = test_header(hdr, pkg);
    s.push_str(&format!("class {base}ProjectionTests {{\n"));
    let mut seen = BTreeSet::new();
    for sc in &projector.scenarios {
        let name = unique(&mut seen, &sc.name);
        s.push_str(&format!("    @Test\n    fun {name}() {{\n"));
        s.push_str(&format!(
            "        val wire = ProjectionAdapter().run(\"{}\", {})\n",
            cs_escape(&projector.id),
            given_expr(&sc.given)
        ));
        s.push_str(&format!("        assertEquals({}, wire.size)\n", sc.then.len()));
        for (k, v) in &sc.then {
            s.push_str(&format!("        assertEquals({}, wire[\"{}\"])\n", kt_scalar(v), cs_escape(k)));
        }
        s.push_str("    }\n\n");
    }
    s.push_str("}\n");
    s
}

/// `FlowTests.g.kt` — the oracle-baked chains through both adapters.
pub fn flow_tests(hdr: &str, pkg: &str, facts: &[FlowFact]) -> String {
    use super::decider_sim::Outcome;
    let mut s = test_header(hdr, pkg);
    s.push_str("class FlowTests {\n");
    let mut seen = BTreeSet::new();
    for f in facts {
        let name = unique(&mut seen, &f.name);
        s.push_str(&format!("    @Test\n    fun {name}() {{\n"));
        s.push_str("        val stream = mutableListOf<WireEvent>()\n");
        for (i, c) in f.cmds.iter().enumerate() {
            s.push_str(&format!(
                "        val o{i} = ConformanceAdapter().run(\"{}\", stream, {})\n",
                cs_escape(&c.decider_id),
                wire_new("WireCommand", c.when.id(), &c.when.payload())
            ));
            match &c.outcome {
                Outcome::Rejected(inv) => s.push_str(&format!(
                    "        assertEquals(\"{}\", o{i}.reject)\n",
                    cs_escape(inv)
                )),
                Outcome::Accepted(events) => {
                    s.push_str(&format!("        assertNull(o{i}.reject)\n"));
                    s.push_str(&format!("        assertEquals({}, o{i}.emit!!.size)\n", events.len()));
                    for (j, e) in events.iter().enumerate() {
                        s.push_str(&format!(
                            "        assertEquals(\"{}\", o{i}.emit!![{j}].id)\n",
                            cs_escape(&e.event)
                        ));
                    }
                    s.push_str(&format!("        stream.addAll(o{i}.emit!!)\n"));
                }
            }
        }
        for (i, v) in f.views.iter().enumerate() {
            s.push_str(&format!(
                "        val view{i} = ProjectionAdapter().run(\"{}\", stream)\n",
                cs_escape(&v.projector_id)
            ));
            s.push_str(&format!("        assertEquals({}, view{i}.size)\n", v.view.len()));
            for (k, val) in &v.view {
                s.push_str(&format!(
                    "        assertEquals({}, view{i}[\"{}\"])\n",
                    kt_scalar(val),
                    cs_escape(k)
                ));
            }
        }
        s.push_str("    }\n\n");
    }
    s.push_str("}\n");
    s
}

/// `<Step>ScreenTests.g.kt` — §4.5 seam facts through `ScreenAdapter`.
pub fn screen_tests(hdr: &str, pkg: &str, step: &WireframeStep, projectors: &[&Projector]) -> String {
    let base = pascal(&step.id);
    let mut s = test_header(hdr, pkg);
    s.push_str(&format!("class {base}ScreenTests {{\n"));
    s.push_str("    @Test\n    fun Present_state_surfaces_every_projection_and_offer() {\n");
    let fixture = super::codegen_screen::present_state(step, projectors)
        .map(|v| kt_map(&v))
        .unwrap_or_else(|| "emptyMap()".to_string());
    s.push_str(&format!(
        "        val screen = ScreenAdapter().render(\"{}\", \"present\", {fixture})\n",
        cs_escape(&step.id)
    ));
    for surface in &step.surfaces {
        s.push_str(&format!(
            "        assertTrue(\"{}\" in screen.projections)\n",
            cs_escape(&surface.projection)
        ));
    }
    s.push_str(&format!("        assertEquals({}, screen.offeredCommands.size)\n", step.offers.len()));
    for offer in &step.offers {
        s.push_str(&format!(
            "        assertTrue(\"{}\" in screen.offeredCommands)\n",
            cs_escape(&offer.command)
        ));
    }
    s.push_str("    }\n\n");
    let mut seen = BTreeSet::new();
    for m in &step.state_meanings {
        if m.waiver.is_none() && m.state != "present" && seen.insert((m.projection.clone(), m.state.clone())) {
            s.push_str(&state_fact(step, &m.projection, &m.state));
        }
    }
    s.push_str("}\n");
    s
}

fn state_fact(step: &WireframeStep, projection: &str, state: &str) -> String {
    let name = method_name(&format!("{projection} {state} state is handled"));
    format!(
        "    @Test\n    fun {name}() {{\n        val screen = ScreenAdapter().render(\"{}\", \"{}\", emptyMap())\n        assertTrue(\"{}\" in screen.projections)\n    }}\n\n",
        cs_escape(&step.id),
        cs_escape(state),
        cs_escape(projection)
    )
}

fn test_header(hdr: &str, pkg: &str) -> String {
    format!("{hdr}package {pkg}\n\nimport kotlin.test.*\n\n")
}

fn unique(seen: &mut BTreeSet<String>, name: &str) -> String {
    let mut n = method_name(name);
    while !seen.insert(n.clone()) {
        n.push('_');
    }
    n
}

fn given_expr(given: &[EventRef]) -> String {
    if given.is_empty() {
        return "emptyList()".to_string();
    }
    let items: Vec<String> = given
        .iter()
        .map(|ev| wire_new("WireEvent", ev.id(), &ev.payload()))
        .collect();
    format!("listOf({})", items.join(", "))
}

fn expectation(then: &Expectation) -> String {
    if let Some(inv) = &then.reject {
        return format!("        assertEquals(\"{}\", outcome.reject)\n", cs_escape(inv));
    }
    let expected = then.emit.clone().unwrap_or_default();
    let mut s = String::from("        assertNull(outcome.reject)\n");
    s.push_str(&format!("        assertEquals({}, outcome.emit!!.size)\n", expected.len()));
    for (i, ev) in expected.iter().enumerate() {
        s.push_str(&format!(
            "        assertEquals(\"{}\", outcome.emit!![{i}].id)\n",
            cs_escape(ev.id())
        ));
        let payload = ev.payload();
        s.push_str(&format!("        assertEquals({}, outcome.emit!![{i}].with.size)\n", payload.len()));
        for (k, v) in &payload {
            s.push_str(&format!(
                "        assertEquals({}, outcome.emit!![{i}].with[\"{}\"])\n",
                kt_scalar(v),
                cs_escape(k)
            ));
        }
    }
    s
}

/// `WireEvent("id", mapOf("f" to v))` construction expression.
fn wire_new(ty: &str, id: &str, payload: &Payload) -> String {
    if payload.is_empty() {
        return format!("{ty}(\"{}\")", cs_escape(id));
    }
    format!("{ty}(\"{}\", {})", cs_escape(id), kt_map(payload))
}

fn kt_map(payload: &Payload) -> String {
    let fields: Vec<String> = payload
        .iter()
        .map(|(k, v)| format!("\"{}\" to {}", cs_escape(k), kt_scalar(v)))
        .collect();
    format!("mapOf({})", fields.join(", "))
}

fn kt_scalar(v: &Scalar) -> String {
    match v {
        Scalar::Bool(b) => b.to_string(),
        Scalar::Int(i) => format!("{i}L"),
        Scalar::Str(s) => format!("\"{}\"", cs_escape(s)),
    }
}
