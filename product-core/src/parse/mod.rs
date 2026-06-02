//! Parsing — front-matter extraction, formal block parsing, TOML processing.

pub use crate::formal::*;
pub use crate::parser::*;

/// Deserialize `due-date` as an ISO 8601 date (YYYY-MM-DD). On failure the
/// error carries the marker substring `due-date: expected YYYY-MM-DD` which
/// the graph-validation layer detects to emit E006 with a field-specific
/// hint (FT-053, ADR-045).
pub fn deserialize_due_date<'de, D>(de: D) -> std::result::Result<Option<chrono::NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde::Deserialize;
    let opt: Option<serde_yaml::Value> = Option::deserialize(de)?;
    let value = match opt {
        None | Some(serde_yaml::Value::Null) => return Ok(None),
        Some(v) => v,
    };
    let s = match &value {
        serde_yaml::Value::String(s) => s.clone(),
        other => {
            return Err(D::Error::custom(format!(
                "due-date: expected YYYY-MM-DD, got {:?}",
                other
            )))
        }
    };
    chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| D::Error::custom(format!("due-date: expected YYYY-MM-DD, got {:?}", s)))
}
