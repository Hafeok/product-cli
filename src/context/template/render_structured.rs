//! YAML / JSON structured renderers — top-level mapping with one key per
//! recognised section.

use super::collect::Collected;
use super::loader::Template;
use super::render::ProductInfo;
use super::sections::{build_section, ordered_sections};
use serde_json::{json, Value};

fn yaml_str(s: &str) -> serde_yaml::Value { serde_yaml::Value::String(s.to_string()) }

pub fn render_yaml(c: &Collected, tpl: &Template, product_info: Option<&ProductInfo<'_>>) -> String {
    let mut map = serde_yaml::Mapping::new();
    map.insert(yaml_str("target"), yaml_str(&tpl.template.name));
    map.insert(yaml_str("feature_id"), yaml_str(&c.feature.front.id));
    if let Some(pi) = product_info {
        let mut p = serde_yaml::Mapping::new();
        p.insert(yaml_str("name"), yaml_str(pi.name));
        p.insert(yaml_str("responsibility"), yaml_str(pi.responsibility));
        map.insert(yaml_str("product"), serde_yaml::Value::Mapping(p));
    }
    for name in ordered_sections(tpl) {
        if let Some(body) = build_section(&name, c) {
            map.insert(yaml_str(&name), yaml_str(&body));
        }
    }
    serde_yaml::to_string(&serde_yaml::Value::Mapping(map))
        .unwrap_or_else(|_| "{}\n".to_string())
}

pub fn render_json(c: &Collected, tpl: &Template, product_info: Option<&ProductInfo<'_>>) -> String {
    let mut map = serde_json::Map::new();
    map.insert("target".to_string(), Value::String(tpl.template.name.clone()));
    map.insert("feature_id".to_string(), Value::String(c.feature.front.id.clone()));
    if let Some(pi) = product_info {
        map.insert(
            "product".to_string(),
            json!({"name": pi.name, "responsibility": pi.responsibility}),
        );
    }
    for name in ordered_sections(tpl) {
        if let Some(body) = build_section(&name, c) {
            map.insert(name.clone(), Value::String(body));
        }
    }
    serde_json::to_string_pretty(&Value::Object(map))
        .unwrap_or_else(|_| "{}".to_string())
}
