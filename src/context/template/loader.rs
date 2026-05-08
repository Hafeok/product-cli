//! Template TOML parser — typed deserialisation of the schema (no validation).

use serde::Deserialize;

/// A parsed template document. Fields use serde defaults so tests and
/// hand-written templates can omit optional sections; the `validate` module
/// applies the closed-allowlist checks downstream.
#[derive(Debug, Clone, Deserialize)]
pub struct Template {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub template: TemplateMeta,
    pub format: FormatBlock,
    pub ordering: OrderingBlock,
    #[serde(default)]
    pub token_budget: TokenBudget,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub target_model: String,
    #[serde(default)]
    pub context_window: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormatBlock {
    pub structure: String,
    #[serde(default = "default_content_format")]
    pub content_format: String,
    #[serde(default)]
    pub xml: XmlOptions,
    #[serde(default)]
    pub markdown: MarkdownOptions,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct XmlOptions {
    #[serde(default)]
    pub include_attributes: bool,
    #[serde(default = "default_empty_section_handling")]
    pub empty_section_handling: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MarkdownOptions {
    #[serde(default = "default_heading_levels")]
    pub heading_levels: String,
    #[serde(default = "default_table_format")]
    pub table_format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderingBlock {
    #[serde(default)]
    pub sections: Vec<String>,
    #[serde(default)]
    pub deliverables_at_top: bool,
    #[serde(default)]
    pub critical_first: bool,
    #[serde(default = "default_adrs_ordered_by")]
    pub adrs_ordered_by: String,
    #[serde(default = "default_tcs_ordered_by")]
    pub tcs_ordered_by: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TokenBudget {
    #[serde(default)]
    pub target_max: u64,
    #[serde(default)]
    pub hard_max: u64,
}

fn default_schema_version() -> u32 { 1 }
fn default_content_format() -> String { "markdown".to_string() }
fn default_empty_section_handling() -> String { "omit".to_string() }
fn default_heading_levels() -> String { "h2-h3".to_string() }
fn default_table_format() -> String { "github".to_string() }
fn default_adrs_ordered_by() -> String { "centrality".to_string() }
fn default_tcs_ordered_by() -> String { "type".to_string() }

/// Errors that arise from parsing a template TOML string. Validation errors
/// (E030 surface) live in `validate.rs`.
#[derive(Debug)]
pub enum TemplateError {
    /// TOML parse failure — wraps the underlying `toml::de::Error`.
    Parse(String),
    /// `schema_version` is newer than the binary supports.
    SchemaVersion { declared: u32, supported: u32 },
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(s) => write!(f, "TOML parse error: {}", s),
            Self::SchemaVersion { declared, supported } => write!(
                f,
                "template schema_version {} not supported (binary supports {}); upgrade Product",
                declared, supported,
            ),
        }
    }
}

impl std::error::Error for TemplateError {}

/// Maximum supported template schema version. Bumped when a new field
/// changes parse semantics.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// Parse a template from a TOML string. Returns the parsed document or a
/// `TemplateError`. Validation (E030) is applied separately in
/// `validate::validate_template`.
pub fn parse_template(toml_text: &str) -> Result<Template, TemplateError> {
    let parsed: Template = toml::from_str(toml_text)
        .map_err(|e| TemplateError::Parse(e.to_string()))?;
    if parsed.schema_version > SUPPORTED_SCHEMA_VERSION {
        return Err(TemplateError::SchemaVersion {
            declared: parsed.schema_version,
            supported: SUPPORTED_SCHEMA_VERSION,
        });
    }
    Ok(parsed)
}
