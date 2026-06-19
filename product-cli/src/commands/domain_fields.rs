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
        if let Some(v) = &self.changes { put("changes", json!(v)); }
        if let Some(v) = &self.projects { put("projects", json!(v)); }
        if let Some(v) = &self.triggers { put("triggers", json!(v)); }
        if let Some(v) = &self.displays { put("displays", json!(v)); }
        if let Some(v) = &self.steps { put("steps", json!(v)); }
        if let Some(v) = &self.means { put("means", json!(v)); }
        if let Some(v) = &self.dimension { put("dimension", json!(v)); }
        if let Some(v) = &self.value { put("value", json!(v)); }
        if let Some(v) = &self.intent { put("intent", json!(v)); }
        if let Some(v) = &self.transitions_to { put("transitions_to", json!(v)); }
        if let Some(v) = &self.surfaces {
            put("surfaces", pairs(v, "projection"));
        }
        if let Some(v) = &self.offers {
            put("offers", pairs(v, "command"));
        }
        if let Some(v) = &self.entry_page { put("entry_page", json!(v)); }
        if let Some(v) = &self.navigates_from_root { put("navigates_from_root", json!(v)); }
        m
    }
}

/// Parse `target:aio` pair strings into `[{<target_key>, aio}, …]` for a UI
/// step's `surfaces`/`offers`. A pair missing its `:aio` half keeps an empty aio.
fn pairs(items: &[String], target_key: &str) -> Value {
    let arr: Vec<Value> = items
        .iter()
        .map(|s| {
            let (target, aio) = s.split_once(':').unwrap_or((s.as_str(), ""));
            json!({ target_key: target, "aio": aio })
        })
        .collect();
    json!(arr)
}

