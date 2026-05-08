//! Per-model context bundle templates (FT-063, ADR-049).
//!
//! Templates are TOML data files declaring how to render a context bundle for
//! a target model. Resolution order is repo → user → built-in; first match
//! wins. Built-in templates are embedded via `include_str!` and are
//! read-only.

pub mod collect;
pub mod loader;
pub mod render;
pub mod render_structured;
pub mod render_text;
pub mod render_xml;
pub mod resolve;
pub mod sections;
pub mod validate;

pub use loader::{Template, TemplateError};
pub use render::{render_feature, ProductInfo, RenderedBundle};
pub use resolve::{
    builtin_names, builtin_toml, resolve_all, resolve_one, ResolveOutcome, ResolvedTemplate,
    TemplateSource,
};
