//! Host routing — one lazily started language server per adapter.
//!
//! Resolves each adapter's host command plus behavior flags from
//! `.ddd/config.yaml` (`adapter.<language>.*`, format 3), spawns hosts on
//! first use, and hands out the [`Host`] for a language or file. This is
//! the entire language-awareness of the layers above: they route here by
//! name or extension, nothing more.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ddd_core::config::DddConfig;

use crate::adapter::{self, Adapter, AdapterFlags};
use crate::error::{LspError, Result};
use crate::host::{Host, Readiness};

pub struct HostManager {
    root: PathBuf,
    config: Option<DddConfig>,
    hosts: BTreeMap<&'static str, Host>,
}

impl HostManager {
    /// A manager for `root`, reading adapter overrides from its config.
    pub fn new(root: PathBuf) -> Self {
        let config = ddd_core::config::load(&root);
        Self { root, config, hosts: BTreeMap::new() }
    }

    /// Override the config (tests; callers that already loaded it).
    pub fn with_config(root: PathBuf, config: Option<DddConfig>) -> Self {
        Self { root, config, hosts: BTreeMap::new() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> Option<&DddConfig> {
        self.config.as_ref()
    }

    /// The adapter for an explicit language override or a file path.
    pub fn resolve_adapter(
        &self,
        language: Option<&str>,
        file: Option<&Path>,
    ) -> Result<&'static Adapter> {
        if let Some(lang) = language {
            return adapter::for_language(lang)
                .ok_or_else(|| LspError::NoAdapter(format!("unknown language `{lang}`")));
        }
        let file = file.ok_or_else(|| LspError::NoAdapter("no file, no language".into()))?;
        adapter::for_path(file)
            .ok_or_else(|| LspError::NoAdapter(format!("no adapter claims {}", file.display())))
    }

    /// The behavior flags for a language (config over adapter defaults).
    pub fn flags(&self, language: &str) -> AdapterFlags {
        let mut flags = AdapterFlags::default();
        if let Some(entry) = self.config.as_ref().and_then(|c| c.adapter_entry(language)) {
            if let Some(v) = entry.internal_is_surface {
                flags.internal_is_surface = v;
            }
            if let Some(attrs) = &entry.exported_attributes {
                flags.exported_attributes = attrs.clone();
            }
        }
        flags
    }

    /// The (possibly newly created) host for a language.
    pub fn host(&mut self, language: &str) -> Result<&mut Host> {
        let adapter = adapter::for_language(language)
            .ok_or_else(|| LspError::NoAdapter(format!("unknown language `{language}`")))?;
        if !self.hosts.contains_key(adapter.language) {
            let command = self.command_for(adapter);
            let host = Host::new(adapter, command, self.root.clone());
            self.hosts.insert(adapter.language, host);
        }
        self.hosts
            .get_mut(adapter.language)
            .ok_or_else(|| LspError::NoAdapter("host vanished".into()))
    }

    /// The host for a language only when one is already running — never
    /// starts one (the `off`-mode disk/overlay sync path).
    pub fn existing_host(&mut self, language: &str) -> Option<&mut Host> {
        self.hosts.get_mut(language)
    }

    fn command_for(&self, adapter: &'static Adapter) -> Vec<String> {
        self.config
            .as_ref()
            .and_then(|c| c.adapter_entry(adapter.language))
            .and_then(|e| e.command.clone())
            .unwrap_or_else(|| adapter.default_command.iter().map(|s| s.to_string()).collect())
    }

    /// Readiness of every registered adapter's host, starting each lazily
    /// (the warmup path).
    pub fn warm_all(&mut self, languages: Option<&[String]>) -> Vec<(String, Result<Readiness>)> {
        let selected: Vec<&'static str> = adapter::all()
            .iter()
            .filter(|a| a.hosted)
            .map(|a| a.language)
            .filter(|l| languages.map(|ls| ls.iter().any(|x| x == l)).unwrap_or(true))
            .collect();
        let mut out = Vec::new();
        for language in selected {
            let r = self.host(language).and_then(Host::readiness);
            out.push((language.to_string(), r));
        }
        out
    }
}
