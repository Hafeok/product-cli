//! Typed generation parameters for a registry instance.
//!
//! Validation here is about *meaning* — a date that is a date, an IRI the RDF
//! parser accepts — never about which characters a value may contain. A value
//! carrying `&`, `$`, a backslash or a brace is data the renderer copies
//! verbatim; rejecting sigils would be an escaping rule in disguise, and an
//! escaping rule is what corrupted the first generated instance.

use crate::error::{ProductError, Result};

/// The parameters a registry instance is minted from.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RegistryParams {
    /// The owning organisation (`--owner`).
    pub owner: String,
    /// The repository name (`--repo`).
    pub repo: String,
    /// The named person whose merge is ratification (`--ratifier`).
    pub ratifier: String,
    /// The registry's display name (`--display-name`).
    pub display_name: String,
    /// The base IRI every identifier is minted under (`--base-iri`).
    pub base_iri: String,
    /// The mint date, `YYYY-MM-DD` (`--date`) — when the authority was
    /// demonstrably controlled, not when the tree was written.
    pub mint_date: String,
    /// Who ran the generation (`--generated-by`), recorded in the provenance.
    pub generated_by: String,
}

impl RegistryParams {
    /// Check every parameter carries a usable meaning. Returns the first fault.
    pub fn validate(&self) -> Result<()> {
        self.check_present()?;
        self.check_name("--owner", &self.owner)?;
        self.check_name("--repo", &self.repo)?;
        check_date(&self.mint_date)?;
        check_base_iri(&self.base_iri)
    }

    fn check_present(&self) -> Result<()> {
        let fields = [
            ("--owner", &self.owner),
            ("--repo", &self.repo),
            ("--ratifier", &self.ratifier),
            ("--display-name", &self.display_name),
            ("--base-iri", &self.base_iri),
            ("--date", &self.mint_date),
            ("--generated-by", &self.generated_by),
        ];
        for (flag, value) in fields {
            if value.trim().is_empty() {
                return Err(fault(&format!("{flag} is empty — every parameter reaches the birth provenance, so none may be blank")));
            }
        }
        Ok(())
    }

    fn check_name(&self, flag: &str, value: &str) -> Result<()> {
        if value.chars().any(|c| c.is_whitespace() || c == '/') {
            return Err(fault(&format!(
                "{flag} '{value}' carries whitespace or a path separator — it names one segment of a repository path"
            )));
        }
        Ok(())
    }
}

fn check_date(value: &str) -> Result<()> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| fault(&format!("--date '{value}' is not a YYYY-MM-DD calendar date")))
}

/// Ask the real RDF parser whether the base IRI can carry identifiers — no
/// regex, no character allow-list.
fn check_base_iri(value: &str) -> Result<()> {
    if !(value.ends_with('#') || value.ends_with('/')) {
        return Err(fault(&format!(
            "--base-iri '{value}' must end in '#' or '/' — identifiers are minted directly beneath it"
        )));
    }
    let probe = format!("@prefix probe: <{value}> .\nprobe:subject probe:predicate probe:object .\n");
    crate::pf::sparql_rules::select(&probe, "SELECT * WHERE { ?s ?p ?o }").map_err(|e| {
        fault(&format!("--base-iri '{value}' is not an IRI the RDF parser accepts: {e}"))
    })?;
    Ok(())
}

fn fault(message: &str) -> ProductError {
    ProductError::ConfigError(format!("registry parameter: {message}"))
}

#[cfg(test)]
#[path = "params_tests.rs"]
mod tests;
