//! Formal block types — AISP notation data structures (ADR-011, ADR-016)

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FormalBlock {
    Types(Vec<TypeDef>),
    Invariants(Vec<Invariant>),
    Scenario(ScenarioBlock),
    ExitCriteria(Vec<ExitField>),
    Evidence(EvidenceBlock),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TypeDef {
    pub name: String,
    pub expr: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Invariant {
    pub raw: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScenarioBlock {
    pub given: Option<String>,
    pub when: Option<String>,
    pub then: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExitField {
    pub raw: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EvidenceBlock {
    pub delta: f64,
    pub phi: u8,
    pub tau: Stability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stability {
    Stable,
    Unstable,
    Unknown,
}

impl std::fmt::Display for Stability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "\u{25CA}\u{207A}"),
            Self::Unstable => write!(f, "\u{25CA}\u{207B}"),
            Self::Unknown => write!(f, "\u{25CA}?"),
        }
    }
}

/// Parse type definitions from a Types block
pub(crate) fn parse_type_defs(content: &str) -> Vec<TypeDef> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(';');
            if line.is_empty() {
                return None;
            }
            let parts: Vec<&str> = line.splitn(2, '\u{225C}').collect();
            if parts.len() == 2 {
                Some(TypeDef {
                    name: parts[0].trim().to_string(),
                    expr: parts[1].trim().to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Parse invariants from an Invariants block
pub(crate) fn parse_invariants(content: &str) -> Vec<Invariant> {
    let mut invariants = Vec::new();
    let mut current = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current.is_empty() {
                invariants.push(Invariant { raw: current.clone() });
                current.clear();
            }
            continue;
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }
    if !current.is_empty() {
        invariants.push(Invariant { raw: current });
    }
    invariants
}

/// Parse a scenario block (given/when/then)
pub(crate) fn parse_scenario(content: &str) -> ScenarioBlock {
    let mut given = None;
    let mut when = None;
    let mut then = None;

    let mut current_field: Option<&str> = None;
    let mut current_value = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("given\u{225C}") {
            if let Some(field) = current_field {
                set_field(field, &current_value, &mut given, &mut when, &mut then);
            }
            current_field = Some("given");
            current_value = val.trim().to_string();
        } else if let Some(val) = trimmed.strip_prefix("when\u{225C}") {
            if let Some(field) = current_field {
                set_field(field, &current_value, &mut given, &mut when, &mut then);
            }
            current_field = Some("when");
            current_value = val.trim().to_string();
        } else if let Some(val) = trimmed.strip_prefix("then\u{225C}") {
            if let Some(field) = current_field {
                set_field(field, &current_value, &mut given, &mut when, &mut then);
            }
            current_field = Some("then");
            current_value = val.trim().to_string();
        } else if current_field.is_some() && !trimmed.is_empty() {
            current_value.push('\n');
            current_value.push_str(trimmed);
        }
    }
    if let Some(field) = current_field {
        set_field(field, &current_value, &mut given, &mut when, &mut then);
    }

    ScenarioBlock { given, when, then }
}

fn set_field(
    field: &str,
    value: &str,
    given: &mut Option<String>,
    when: &mut Option<String>,
    then: &mut Option<String>,
) {
    let v = value.trim().to_string();
    if v.is_empty() {
        return;
    }
    match field {
        "given" => *given = Some(v),
        "when" => *when = Some(v),
        "then" => *then = Some(v),
        _ => {}
    }
}

/// Parse exit criteria lines
pub(crate) fn parse_exit_criteria(content: &str) -> Vec<ExitField> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| ExitField {
            raw: l.trim().to_string(),
        })
        .collect()
}
