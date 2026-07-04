//! Shared `--field` flags for `product domain new`/`edit`.

use clap::Args;
use serde_json::{json, Map, Value};

/// The full set of node fields, as optional flags shared by `new` and `edit`.
#[derive(Args, Default)]
pub struct NodeFields {
    #[arg(long)]
    label: Option<String>,
    #[arg(long)]
    context: Option<String>,
    #[arg(long)]
    definition: Option<String>,
    #[arg(long)]
    identity: Option<String>,
    #[arg(long = "aggregate-root")]
    aggregate_root: Option<bool>,
    #[arg(long)]
    purpose: Option<String>,
    #[arg(long, value_delimiter = ',')]
    glossary: Option<Vec<String>>,
    #[arg(long)]
    from: Option<String>,
    #[arg(long)]
    to: Option<String>,
    #[arg(long)]
    cardinality: Option<String>,
    #[arg(long)]
    rationale: Option<String>,
    #[arg(long)]
    statement: Option<String>,
    #[arg(long = "applies-to")]
    applies_to: Option<String>,
    #[arg(long = "concept-a")]
    concept_a: Option<String>,
    #[arg(long = "concept-b")]
    concept_b: Option<String>,
    #[arg(long = "mapping-kind")]
    mapping_kind: Option<String>,
    #[arg(long)]
    targets: Option<String>,
    #[arg(long, value_delimiter = ',')]
    emits: Option<Vec<String>>,
    /// §3.2 payload schema: `name:type` pairs (type from the §3.1 datatype
    /// vocabulary — string · integer · number · boolean · date; omit for untyped)
    #[arg(long = "fields", value_delimiter = ',')]
    payload_fields: Option<Vec<String>>,
    #[arg(long)]
    changes: Option<String>,
    #[arg(long, value_delimiter = ',')]
    projects: Option<Vec<String>>,
    #[arg(long)]
    triggers: Option<String>,
    #[arg(long)]
    displays: Option<String>,
    #[arg(long, value_delimiter = ',')]
    steps: Option<Vec<String>>,
    #[arg(long)]
    means: Option<String>,
    #[arg(long)]
    dimension: Option<String>,
    #[arg(long)]
    value: Option<String>,
    #[arg(long)]
    intent: Option<String>,
    /// `projection:aio` pairs (repeatable, comma-separated)
    #[arg(long, value_delimiter = ',')]
    surfaces: Option<Vec<String>>,
    /// `command:aio` pairs (repeatable, comma-separated)
    #[arg(long, value_delimiter = ',')]
    offers: Option<Vec<String>>,
    #[arg(long = "transitions-to", value_delimiter = ',')]
    transitions_to: Option<Vec<String>>,
    #[arg(long = "entry-page")]
    entry_page: Option<String>,
    #[arg(long = "navigates-from-root", value_delimiter = ',')]
    navigates_from_root: Option<Vec<String>>,
    /// Read-model states beyond `present` (e.g. `loading,empty,failed`)
    #[arg(long, value_delimiter = ',')]
    states: Option<Vec<String>>,
    /// WCAG criteria a step or AIO must satisfy (comma-separated)
    #[arg(long = "must-satisfy", value_delimiter = ',')]
    must_satisfy: Option<Vec<String>>,
    /// WCAG level (A/AA/AAA)
    #[arg(long)]
    level: Option<String>,
    /// WCAG verification type (machine/assisted/manual)
    #[arg(long)]
    verification: Option<String>,
    /// Machine-gate result for a WcagCriterion
    #[arg(long)]
    satisfied: Option<bool>,
    /// Attestation: the step / criterion it discharges, who attests
    #[arg(long)]
    step: Option<String>,
    #[arg(long)]
    criterion: Option<String>,
    #[arg(long)]
    date: Option<String>,
    #[arg(long)]
    by: Option<String>,
    /// UI-step content references: `key:role` (repeatable)
    #[arg(long = "content")]
    content: Option<Vec<String>>,
    /// Content-store locales (e.g. `en,es`)
    #[arg(long, value_delimiter = ',')]
    locales: Option<Vec<String>>,
    /// Content-store resolutions: `key:locale:value` (repeatable)
    #[arg(long = "resolves")]
    resolves: Option<Vec<String>>,
    /// Reification: the AIO a rule reifies
    #[arg(long)]
    aio: Option<String>,
    /// Reification: the CIO a rule targets
    #[arg(long)]
    cio: Option<String>,
    /// Design-system CIO catalog (comma-separated)
    #[arg(long, value_delimiter = ',')]
    cios: Option<Vec<String>>,
    /// Design-system token surface (comma-separated)
    #[arg(long, value_delimiter = ',')]
    tokens: Option<Vec<String>>,
    /// UI-step style values (must be design-system tokens, not literals)
    #[arg(long, value_delimiter = ',')]
    styles: Option<Vec<String>>,
    /// `projection:state:meaning` (repeatable)
    #[arg(long = "state-meaning")]
    state_meaning: Option<Vec<String>>,
    /// `projection:state:reason` — waive an ignorable state (repeatable)
    #[arg(long = "waive-state")]
    waive_state: Option<Vec<String>>,
    /// §3.1 reference-set concept (the entity/value-object it is data for)
    #[arg(long)]
    concept: Option<String>,
    /// §3.1 reference-set members (comma-separated)
    #[arg(long, value_delimiter = ',')]
    values: Option<Vec<String>>,
    /// §3.1 data-shape target entity
    #[arg(long)]
    target: Option<String>,
    /// §3.1 data-shape required fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    required: Option<Vec<String>>,
    /// §3.1 data-shape enum constraints: `field=ReferenceSetId` (repeatable)
    #[arg(long = "enum")]
    enums: Option<Vec<String>>,
    /// §3.1 data-shape type constraints: `field=datatype` (repeatable)
    #[arg(long = "type")]
    types: Option<Vec<String>>,
    /// §3.1 production-dataset shape (the shape it conforms to)
    #[arg(long)]
    shape: Option<String>,
    /// §3.1 production-dataset source (JSON file of records)
    #[arg(long)]
    source: Option<String>,
    /// §3.2.5 system kind (application/website/service/cli/…)
    #[arg(long = "system-kind")]
    system_kind: Option<String>,
    /// §3.2.5 system target platforms (e.g. `ios,android,web`)
    #[arg(long = "target-platforms", value_delimiter = ',')]
    target_platforms: Option<Vec<String>>,
    /// §3.2.2 system target interaction classes (e.g. `gui,tui`)
    #[arg(long = "target-classes", value_delimiter = ',')]
    target_classes: Option<Vec<String>>,
    /// §3.2.4 the ApplicationRoot a system roots at
    #[arg(long = "roots-at")]
    roots_at: Option<String>,
    /// §3.2.5 the system a flow belongs to
    #[arg(long)]
    system: Option<String>,
    /// §3.2.0 trigger source (user/external/automated)
    #[arg(long = "trigger-source")]
    trigger_source: Option<String>,
    /// §3.2.0 the command a trigger issues
    #[arg(long)]
    issues: Option<String>,
    /// §3.2.0 the View an automated trigger watches
    #[arg(long)]
    watches: Option<String>,
    /// §3.2.0 the source system a Translation trigger reads from
    #[arg(long = "translates-from")]
    translates_from: Option<String>,
    /// §4.5 the interaction class an AIO is unreifiable in (gui/tui)
    #[arg(long)]
    class: Option<String>,
}

impl NodeFields {
    /// Project the provided flags into a JSON field map keyed by model field
    /// names (the merge target in `pf::edit`).
    pub(crate) fn to_map(&self) -> Map<String, Value> {
        let mut m = Map::new();
        let mut put = |k: &str, v: Value| { m.insert(k.to_string(), v); };
        if let Some(v) = &self.label { put("label", json!(v)); }
        if let Some(v) = &self.context { put("context", json!(v)); }
        if let Some(v) = &self.definition { put("definition", json!(v)); }
        if let Some(v) = &self.identity { put("identity", json!(v)); }
        if let Some(v) = self.aggregate_root { put("is_aggregate_root", json!(v)); }
        if let Some(v) = &self.purpose { put("purpose", json!(v)); }
        if let Some(v) = &self.glossary { put("glossary", json!(v)); }
        if let Some(v) = &self.from { put("from", json!(v)); }
        if let Some(v) = &self.to { put("to", json!(v)); }
        if let Some(v) = &self.cardinality { put("cardinality", json!(v)); }
        if let Some(v) = &self.rationale { put("rationale", json!(v)); }
        if let Some(v) = &self.statement { put("statement", json!(v)); }
        if let Some(v) = &self.applies_to { put("applies_to", json!(v)); }
        if let Some(v) = &self.concept_a { put("concept_a", json!(v)); }
        if let Some(v) = &self.concept_b { put("concept_b", json!(v)); }
        if let Some(v) = &self.mapping_kind { put("kind", json!(v)); }
        if let Some(v) = &self.targets { put("targets", json!(v)); }
        if let Some(v) = &self.emits { put("emits", json!(v)); }
        if let Some(v) = &self.payload_fields { put("fields", typed_fields(v)); }
        if let Some(v) = &self.changes { put("changes", json!(v)); }
        if let Some(v) = &self.projects { put("projects", json!(v)); }
        if let Some(v) = &self.triggers { put("triggers", json!(v)); }
        if let Some(v) = &self.displays { put("displays", json!(v)); }
        if let Some(v) = &self.steps { put("steps", json!(v)); }
        if let Some(v) = &self.means { put("means", json!(v)); }
        if let Some(v) = &self.dimension { put("dimension", json!(v)); }
        if let Some(v) = &self.value { put("value", json!(v)); }
        self.put_ui_fields(&mut m);
        self.put_data_fields(&mut m);
        self.put_system_fields(&mut m);
        m
    }

    /// The §3.2.5 system-side field puts (kind, purpose reuse, platforms, root).
    fn put_system_fields(&self, m: &mut Map<String, Value>) {
        let mut put = |k: &str, v: Value| { m.insert(k.to_string(), v); };
        if let Some(v) = &self.system_kind { put("kind", json!(v)); }
        if let Some(v) = &self.target_platforms { put("target_platforms", json!(v)); }
        if let Some(v) = &self.target_classes { put("target_classes", json!(v)); }
        if let Some(v) = &self.roots_at { put("root", json!(v)); }
        if let Some(v) = &self.system { put("system", json!(v)); }
        if let Some(v) = &self.trigger_source { put("source", json!(v)); }
        if let Some(v) = &self.issues { put("issues", json!(v)); }
        if let Some(v) = &self.watches { put("watches", json!(v)); }
        if let Some(v) = &self.translates_from { put("translates_from", json!(v)); }
        if let Some(v) = &self.class { put("class", json!(v)); }
    }

    /// The §3.1 data-side field puts (reference sets, shapes, datasets).
    fn put_data_fields(&self, m: &mut Map<String, Value>) {
        let mut put = |k: &str, v: Value| { m.insert(k.to_string(), v); };
        if let Some(v) = &self.concept { put("concept", json!(v)); }
        if let Some(v) = &self.values { put("values", json!(v)); }
        if let Some(v) = &self.target { put("target", json!(v)); }
        if let Some(v) = &self.required { put("required", json!(v)); }
        if let Some(v) = &self.enums { put("enums", field_pairs(v, "reference_set")); }
        if let Some(v) = &self.types { put("types", field_pairs(v, "datatype")); }
        if let Some(v) = &self.shape { put("shape", json!(v)); }
        if let Some(v) = &self.source { put("source", json!(v)); }
    }

    /// The §3.2.1–§3.2.4 UI-layer field puts, split out to keep `to_map` small.
    fn put_ui_fields(&self, m: &mut Map<String, Value>) {
        {
            let mut put = |k: &str, v: Value| { m.insert(k.to_string(), v); };
            if let Some(v) = &self.intent { put("intent", json!(v)); }
            if let Some(v) = &self.transitions_to { put("transitions_to", json!(v)); }
            if let Some(v) = &self.surfaces { put("surfaces", pairs(v, "projection", "aio")); }
            if let Some(v) = &self.offers { put("offers", pairs(v, "command", "aio")); }
            if let Some(v) = &self.entry_page { put("entry_page", json!(v)); }
            if let Some(v) = &self.navigates_from_root { put("navigates_from_root", json!(v)); }
            if let Some(v) = &self.states { put("states", json!(v)); }
        if let Some(v) = &self.must_satisfy { put("must_satisfy", json!(v)); }
        if let Some(v) = &self.content { put("content_refs", pairs(v, "key", "role")); }
        if let Some(v) = &self.locales { put("locales", json!(v)); }
        if let Some(v) = &self.resolves { put("resolutions", triples(v)); }
        if let Some(v) = &self.aio { put("aio", json!(v)); }
        if let Some(v) = &self.cio { put("cio", json!(v)); }
        if let Some(v) = &self.cios { put("cios", json!(v)); }
        if let Some(v) = &self.tokens { put("tokens", json!(v)); }
        if let Some(v) = &self.styles { put("styles", json!(v)); }
        if let Some(v) = &self.level { put("level", json!(v)); }
        if let Some(v) = &self.verification { put("verification", json!(v)); }
        if let Some(v) = self.satisfied { put("satisfied", json!(v)); }
        if let Some(v) = &self.step { put("step", json!(v)); }
        if let Some(v) = &self.criterion { put("criterion", json!(v)); }
        if let Some(v) = &self.date { put("date", json!(v)); }
        if let Some(v) = &self.by { put("by", json!(v)); }
        }
        let annotations = state_annotations(self.state_meaning.as_deref(), self.waive_state.as_deref());
        if !annotations.is_empty() {
            m.insert("state_meanings".to_string(), json!(annotations));
        }
    }
}

/// Parse `--state-meaning`/`--waive-state` strings (`projection:state:text`)
/// into `[{projection, state, meaning|waiver}]` for a UI step's `state_meanings`.
fn state_annotations(meanings: Option<&[String]>, waivers: Option<&[String]>) -> Vec<Value> {
    let parse = |items: Option<&[String]>, key: &str| -> Vec<Value> {
        items
            .unwrap_or(&[])
            .iter()
            .filter_map(|s| {
                let mut it = s.splitn(3, ':');
                let projection = it.next()?.to_string();
                let state = it.next()?.to_string();
                let text = it.next().unwrap_or("").to_string();
                Some(json!({ "projection": projection, "state": state, key: text }))
            })
            .collect()
    };
    let mut out = parse(meanings, "meaning");
    out.extend(parse(waivers, "waiver"));
    out
}

/// Parse `a:b` pair strings into `[{<k1>: a, <k2>: b}, …]` (e.g. `proj:aio` for
/// `surfaces`, `key:role` for content refs). A pair missing its `:b` half keeps b empty.
/// Parse `name:type` payload-field pairs; a missing `:type` leaves the field
/// untyped (no `type` key at all, so it round-trips as `None`).
fn typed_fields(items: &[String]) -> Value {
    let arr: Vec<Value> = items
        .iter()
        .map(|s| match s.split_once(':') {
            Some((name, ty)) if !ty.is_empty() => json!({ "name": name, "type": ty }),
            _ => json!({ "name": s.trim_end_matches(':') }),
        })
        .collect();
    json!(arr)
}

fn pairs(items: &[String], k1: &str, k2: &str) -> Value {
    let arr: Vec<Value> = items
        .iter()
        .map(|s| {
            let (a, b) = s.split_once(':').unwrap_or((s.as_str(), ""));
            json!({ k1: a, k2: b })
        })
        .collect();
    json!(arr)
}

/// Parse `field=value` strings into `[{field, <value_key>: value}, …]` for a
/// data shape's enum (`reference_set`) or type (`datatype`) constraints.
fn field_pairs(items: &[String], value_key: &str) -> Value {
    let arr: Vec<Value> = items
        .iter()
        .map(|s| {
            let (field, val) = s.split_once('=').unwrap_or((s.as_str(), ""));
            json!({ "field": field, value_key: val })
        })
        .collect();
    json!(arr)
}

/// Parse `key:locale:value` strings into `[{key, locale, value}, …]` for a
/// content store's resolutions.
fn triples(items: &[String]) -> Value {
    let arr: Vec<Value> = items
        .iter()
        .map(|s| {
            let mut it = s.splitn(3, ':');
            json!({
                "key": it.next().unwrap_or(""),
                "locale": it.next().unwrap_or(""),
                "value": it.next().unwrap_or(""),
            })
        })
        .collect();
    json!(arr)
}

