//! The registry template, embedded from the repository at compile time.
//!
//! The template's home stays `docs/g-track/registry-template/` — the path the
//! PRD and every generated instance's provenance pin. Nothing is vendored into
//! this crate: a second copy would be a projection presenting as a source. The
//! manifest below is hand-maintained, with a drift test that fails when the
//! tree on disk holds a file the manifest does not.

/// The template's own version, as declared in its `TEMPLATE.md` header. A test
/// fails if the two drift apart.
pub const TEMPLATE_VERSION: &str = "0.2.0";

/// The base-IRI placeholder. G-1 Gate 3 kept it IRI-valid so the template's own
/// Turtle parses; it is a token like any other, not an exception.
pub const BASE_IRI_TOKEN: &str = "https://REGISTRY-HOST.example/ns#";

/// The host substring the gate scans for. Any survivor means a base IRI reached
/// the output unsubstituted.
pub const HOST_SENTINEL: &str = "REGISTRY-HOST.example";

/// One file of the template.
pub struct TemplateFile {
    /// Path relative to the instance root.
    pub path: &'static str,
    /// The file's contents as they stand in the template.
    pub text: &'static str,
    /// Whether the file is written with the executable bit set.
    pub executable: bool,
    /// Whether parameters are substituted into it. `TEMPLATE.md` travels
    /// verbatim: it is the instance's record of what its placeholders were,
    /// which is how it re-pins when the template moves.
    pub substituted: bool,
}

macro_rules! template_file {
    ($path:literal, $exec:literal, $subst:literal) => {
        TemplateFile {
            path: $path,
            text: include_str!(concat!("../../../docs/g-track/registry-template/", $path)),
            executable: $exec,
            substituted: $subst,
        }
    };
}

/// Every file a generated instance carries.
pub const TEMPLATE: &[TemplateFile] = &[
    template_file!("TEMPLATE.md", false, false),
    template_file!("README.md", false, true),
    template_file!("GENERATION.ttl", false, true),
    template_file!(".github/workflows/validate.yml", false, true),
    template_file!("graphs/canonical/_exemplar.ttl", false, true),
    template_file!("graphs/canonical/_exemplar-decision.ttl", false, true),
    template_file!("scripts/build-projection.sh", true, true),
    template_file!("shapes/decision.ttl", false, true),
    template_file!("shapes/reading.ttl", false, true),
    template_file!("shapes/structural.ttl", false, true),
];

/// The template's source directory, relative to the workspace root — where the
/// drift test walks.
pub const TEMPLATE_DIR: &str = "docs/g-track/registry-template";

#[cfg(test)]
#[path = "template_tests.rs"]
mod tests;
