//! Kotlin oracle-only backend — the verification shell for a JVM realiser.
//!
//! The second language backend, deliberately oracle-only (the portable
//! tier): `Oracle.g.kt` carries the wire seam (`WireEvent`/`WireCommand`,
//! the three adapter interfaces, the `PfWire` JSON codec over
//! kotlinx-serialization), `Main.g.kt` speaks the same runner protocol
//! `product {decider,projector} conform` drive, the facts land in
//! `kotlin.test`, and the realiser owns everything behind three
//! scaffolded-once adapters plus the Gradle build. Same graph hash, same
//! oracle, different ecosystem.

use super::decider::Decider;
use super::model::DomainGraph;
use super::projector::Projector;
use super::codegen::{GenFile, ReifyOptions};
use super::codegen_ident::cs_escape;

const ORACLE_KT: &str = r##"@file:Suppress("unused")

package {PKG}

import kotlinx.serialization.json.*

/** An event in wire form (§6.3 protocol): its What-graph id plus the payload fields actually set. */
data class WireEvent(val id: String, val with: Map<String, Any?> = emptyMap())

/** A command in wire form (§6.3 protocol). */
data class WireCommand(val id: String, val with: Map<String, Any?> = emptyMap())

/** The wire-level outcome of one decision: emitted events, or a rejection naming the violated invariant. */
class ConformanceOutcome private constructor(val emit: List<WireEvent>?, val reject: String?) {
    companion object {
        fun emitted(vararg events: WireEvent) = ConformanceOutcome(events.toList(), null)
        fun rejected(invariantId: String) = ConformanceOutcome(null, invariantId)
    }
}

/** The §6.3 seam: fold `given` into fresh aggregate state, decide `command`, answer in wire form. */
interface IConformanceAdapter {
    fun run(deciderId: String, given: List<WireEvent>, command: WireCommand): ConformanceOutcome
}

/** The read-side seam (§3.4/§6.3): fold `given` into a view, answered in wire form (full-state equality). */
interface IProjectionAdapter {
    fun run(projectorId: String, given: List<WireEvent>): Map<String, Any?>
}

/** What a screen actually put in front of the user, in graph vocabulary (§3.2.1). */
data class RenderedScreen(val projections: List<String>, val offeredCommands: List<String>)

/** The UI seam (§4.5/§6.3): render one UI step in one projection state with the given view data. */
interface IScreenAdapter {
    fun render(stepId: String, projectionState: String, view: Map<String, Any?>): RenderedScreen
}

/** JSON codec for the §6.3 conformance wire protocol (scalars: long · boolean · string). */
object PfWire {
    fun parseEvent(el: JsonElement): WireEvent {
        val (id, with) = split(el, "event")
        return WireEvent(id, with)
    }

    fun parseCommand(el: JsonElement): WireCommand {
        val (id, with) = split(el, "command")
        return WireCommand(id, with)
    }

    private fun split(el: JsonElement, idKey: String): Pair<String, Map<String, Any?>> {
        if (el is JsonPrimitive) return Pair(el.content, emptyMap())
        val obj = el.jsonObject
        val id = obj[idKey]?.jsonPrimitive?.content ?: ""
        val with = obj["with"]?.jsonObject?.mapValues { (_, v) -> scalar(v) } ?: emptyMap()
        return Pair(id, with)
    }

    private fun scalar(v: JsonElement): Any? {
        val p = v.jsonPrimitive
        return when {
            p.isString -> p.content
            p.booleanOrNull != null -> p.boolean
            else -> p.longOrNull
        }
    }

    fun toJson(with: Map<String, Any?>): JsonObject = buildJsonObject {
        with.forEach { (k, v) ->
            when (v) {
                is Long -> put(k, v)
                is Int -> put(k, v.toLong())
                is Boolean -> put(k, v)
                is String -> put(k, v)
                else -> {}
            }
        }
    }

    fun toResponse(outcome: ConformanceOutcome): JsonObject = buildJsonObject {
        val reject = outcome.reject
        if (reject != null) {
            put("reject", reject)
        } else {
            putJsonArray("emit") {
                (outcome.emit ?: emptyList()).forEach { e ->
                    addJsonObject {
                        put("event", e.id)
                        if (e.with.isNotEmpty()) put("with", PfWire.toJson(e.with))
                    }
                }
            }
        }
    }
}
"##;

const BUILD_GRADLE: &str = r##"// Scaffolded once by `product codegen kotlin` (never overwritten):
// add your app/domain modules and dependencies as your realisation grows.
plugins {
    kotlin("jvm") version "2.0.21"
    kotlin("plugin.serialization") version "2.0.21"
    application
}

repositories { mavenCentral() }

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
    testImplementation(kotlin("test"))
}

application { mainClass.set("{PKG}.MainKt") }

tasks.test { useJUnitPlatform() }
"##;

const CONFORMANCE_STUB_KT: &str = r##"// Scaffolded once by `product codegen kotlin` — never overwritten.
// Implement the §6.3 oracle seam by delegating to your realised domain model.
package {PKG}

class ConformanceAdapter : IConformanceAdapter {
    override fun run(deciderId: String, given: List<WireEvent>, command: WireCommand): ConformanceOutcome {
        // TODO: fold `given` into your aggregate state, decide `command`,
        // return ConformanceOutcome.emitted(...) / .rejected(invariantId).
        TODO("realise the conformance adapter for '$deciderId'")
    }
}
"##;

const PROJECTION_STUB_KT: &str = r##"// Scaffolded once by `product codegen kotlin` — never overwritten.
// Implement the §3.4 read-side seam: fold `given` into your view, answer in wire form.
package {PKG}

class ProjectionAdapter : IProjectionAdapter {
    override fun run(projectorId: String, given: List<WireEvent>): Map<String, Any?> {
        TODO("realise the projection adapter for '$projectorId'")
    }
}
"##;

const SCREEN_STUB_KT: &str = r##"// Scaffolded once by `product codegen kotlin` — never overwritten.
// Implement the §4.5 screen seam over your real UI (Compose test harness,
// Robolectric, whatever your stack renders with) and report what the screen
// surfaced and offered.
package {PKG}

class ScreenAdapter : IScreenAdapter {
    override fun render(stepId: String, projectionState: String, view: Map<String, Any?>): RenderedScreen {
        TODO("realise the screen adapter for '$stepId'")
    }
}
"##;

const README_KT: &str = r##"# {PKG} — What-graph oracle (Kotlin, adapter seam)

Generated by `product codegen kotlin` (oracle-only): no domain types are
generated — you (or the realising agent) own the whole domain design.
The generated artifacts are the incorruptible definition of done:

- `Oracle.g.kt` — the wire seam: `WireEvent`, `WireCommand`,
  `ConformanceOutcome`, and the three adapter interfaces.
- `ConformanceAdapter.kt` / `ProjectionAdapter.kt` / `ScreenAdapter.kt` —
  scaffolded once, never overwritten: implement them over your realisation.
- `src/test/kotlin/**` — every Decider/Projector scenario, flow chain, and
  screen fact as a `kotlin.test` test.
- `Main.g.kt` — the `product decider conform` / `product projector conform`
  runner protocol.
- `openapi.g.json` — the §4.4 interface contract this client speaks.

## Verify

1. `gradle test`
2. `gradle installDist` then, per decider/projector:
   `product decider conform <id> --runner "build/install/{PKG}/bin/{PKG} <id>"`
3. `product codegen check --out <this directory>` — drift gate on the graph hash.
"##;

/// The Kotlin package for a namespace: lowercase alphanumerics.
pub fn package_of(namespace: &str) -> String {
    let pkg: String = namespace
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    if pkg.is_empty() { "product".to_string() } else { pkg }
}

/// The static (template) half of the Kotlin plan.
pub fn static_files(
    opts: &ReifyOptions,
    pkg: &str,
    hdr: &str,
    hash: &str,
    has_projectors: bool,
    has_screens: bool,
) -> Vec<GenFile> {
    let main_dir = format!("src/main/kotlin/{pkg}");
    let mut out = vec![
        gen(&format!("{main_dir}/Oracle.g.kt"), format!("{hdr}{}", ORACLE_KT.replace("{PKG}", pkg))),
        gen(&format!("{main_dir}/Provenance.g.kt"), provenance_kt(hdr, opts, pkg, hash)),
        gen("README.g.md", README_KT.replace("{PKG}", pkg)),
        once("settings.gradle.kts", format!("rootProject.name = \"{pkg}\"\n")),
        once("build.gradle.kts", BUILD_GRADLE.replace("{PKG}", pkg)),
        once(&format!("{main_dir}/ConformanceAdapter.kt"), CONFORMANCE_STUB_KT.replace("{PKG}", pkg)),
    ];
    if has_projectors {
        out.push(once(&format!("{main_dir}/ProjectionAdapter.kt"), PROJECTION_STUB_KT.replace("{PKG}", pkg)));
    }
    if has_screens {
        out.push(once(&format!("{main_dir}/ScreenAdapter.kt"), SCREEN_STUB_KT.replace("{PKG}", pkg)));
    }
    out
}

fn provenance_kt(hdr: &str, opts: &ReifyOptions, pkg: &str, hash: &str) -> String {
    format!(
        "{hdr}package {pkg}\n\n/** Binds this realisation to the exact What it realises (§7.3). */\nobject PfProvenance {{\n    const val PRODUCT = \"{}\"\n    const val WHAT_VERSION = \"{}\"\n    const val GRAPH_HASH = \"sha256:{hash}\"\n}}\n",
        cs_escape(&opts.product),
        cs_escape(&opts.what_version),
    )
}

/// `Main.g.kt` — the runner protocol, routing ids to the scaffolded adapters.
pub fn main_file(hdr: &str, pkg: &str, deciders: &[&Decider], projectors: &[&Projector], graph: &DomainGraph) -> String {
    let _ = graph;
    let default_id = deciders
        .first()
        .map(|d| d.id.as_str())
        .or_else(|| projectors.first().map(|p| p.id.as_str()))
        .unwrap_or("");
    let mut s = String::new();
    s.push_str(hdr);
    // The `.g.kt` suffix would otherwise derive class `Main_gKt` — pin the
    // JVM name the Gradle `application` block launches.
    s.push_str("@file:JvmName(\"MainKt\")\n\n");
    s.push_str(&format!("package {pkg}\n\nimport kotlinx.serialization.json.*\n\n"));
    s.push_str("fun main(args: Array<String>) {\n");
    s.push_str(&format!("    val id = if (args.isNotEmpty()) args[0] else \"{}\"\n", cs_escape(default_id)));
    s.push_str("    val projection = resolveProjection(id)\n");
    s.push_str("    val input = generateSequence(::readLine).joinToString(\"\\n\")\n");
    s.push_str("    val doc = Json.parseToJsonElement(input).jsonArray\n");
    s.push_str("    val responses = buildJsonArray {\n        doc.forEach { request ->\n");
    s.push_str("            val obj = request.jsonObject\n");
    s.push_str("            val given = obj[\"given\"]!!.jsonArray.map { PfWire.parseEvent(it) }\n");
    s.push_str("            if (projection != null) {\n                add(PfWire.toJson(projection.run(id, given)))\n            } else {\n");
    s.push_str("                val command = PfWire.parseCommand(obj[\"when\"]!!)\n");
    s.push_str("                add(PfWire.toResponse(ConformanceAdapter().run(id, given, command)))\n            }\n");
    s.push_str("        }\n    }\n    print(responses.toString())\n}\n\n");
    s.push_str(&projection_resolver(projectors));
    s
}

fn projection_resolver(projectors: &[&Projector]) -> String {
    if projectors.is_empty() {
        return "private fun resolveProjection(id: String): IProjectionAdapter? = null\n".to_string();
    }
    let mut s = String::from("private fun resolveProjection(id: String): IProjectionAdapter? = when (id) {\n");
    for p in projectors {
        s.push_str(&format!("    \"{}\" -> ProjectionAdapter()\n", cs_escape(&p.id)));
    }
    s.push_str("    else -> null\n}\n");
    s
}

pub(crate) fn gen(path: &str, content: String) -> GenFile {
    GenFile { path: path.to_string(), content, overwrite: true }
}

pub(crate) fn once(path: &str, content: String) -> GenFile {
    GenFile { path: path.to_string(), content, overwrite: false }
}

/// Plan the Kotlin projection — the oracle-only tier for a JVM realiser:
/// wire seam, runner, `kotlin.test` facts, scaffolded adapters + Gradle
/// build, provenance. No domain types; the realiser owns the design.
pub fn plan_kotlin(
    graph: &DomainGraph,
    deciders: &[Decider],
    projectors: &[super::projector::Projector],
    opts: &ReifyOptions,
) -> crate::error::Result<super::codegen::ReifyPlan> {
    use super::codegen::{aggregate_names, gen as rgen, input_hash, header, provenance_json, GenFile as GF, ReifyPlan};
    let graph_hash = input_hash(graph, &opts.product, deciders, projectors)?;
    let hdr = header(&graph_hash, &opts.what_version);
    let mut sorted: Vec<&Decider> = deciders.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    let mut sorted_p: Vec<&super::projector::Projector> = projectors.iter().collect();
    sorted_p.sort_by(|a, b| a.id.cmp(&b.id));
    let aggs = aggregate_names(&sorted)?;

    let pkg = package_of(&opts.namespace);
    let steps = super::codegen_screen::testable_steps(graph);
    let mut files = static_files(opts, &pkg, &hdr, &graph_hash, !sorted_p.is_empty(), !steps.is_empty());
    files.push(rgen(
        &format!("src/main/kotlin/{pkg}/Main.g.kt"),
        main_file(&hdr, &pkg, &sorted, &sorted_p, graph),
    ));
    files.extend(fact_files(graph, &hdr, &pkg, &sorted, &aggs, &sorted_p, &steps));
    files.push(rgen(
        "openapi.g.json",
        super::codegen_openapi::openapi_file(graph, &sorted, &sorted_p, &opts.product, &opts.what_version, &graph_hash),
    ));
    files.push(rgen(
        "realise-kotlin.cell.g.yaml",
        super::codegen_cell::cell_file_kotlin(&graph_hash, &pkg)?,
    ));
    files.push(GF {
        path: "provenance.g.json".to_string(),
        content: provenance_json(opts, &graph_hash, &files),
        overwrite: true,
    });
    Ok(ReifyPlan { files, graph_hash, aggregates: aggs })
}

fn fact_files(
    graph: &DomainGraph,
    hdr: &str,
    pkg: &str,
    sorted: &[&Decider],
    aggs: &[String],
    sorted_p: &[&super::projector::Projector],
    steps: &[&super::model_ui::WireframeStep],
) -> Vec<GenFile> {
    use super::codegen_kotlin_facts as kf;
    let test_dir = format!("src/test/kotlin/{pkg}");
    let mut out = Vec::new();
    for (d, agg) in sorted.iter().zip(aggs) {
        if !d.scenarios.is_empty() {
            out.push(gen(&format!("{test_dir}/{agg}ScenarioTests.g.kt"), kf::decider_tests(hdr, pkg, agg, d)));
        }
    }
    for p in sorted_p {
        if !p.scenarios.is_empty() {
            let base = super::codegen_projector::view_base(p);
            out.push(gen(&format!("{test_dir}/{base}ProjectionTests.g.kt"), kf::projector_tests(hdr, pkg, p)));
        }
    }
    let flows = super::codegen_flow::plan_flows(graph, sorted, sorted_p, true);
    if !flows.is_empty() {
        out.push(gen(&format!("{test_dir}/FlowTests.g.kt"), kf::flow_tests(hdr, pkg, &flows)));
    }
    for step in steps {
        out.push(gen(
            &format!("{test_dir}/{}ScreenTests.g.kt", super::codegen_ident::pascal(&step.id)),
            kf::screen_tests(hdr, pkg, step, sorted_p),
        ));
    }
    out
}
