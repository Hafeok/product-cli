//! Formal block parser for AISP-influenced notation in test criteria (ADR-011, ADR-016)

use regex::Regex;

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
    pub expr: String, // stored as raw expression
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Invariant {
    pub raw: String, // verbatim for context bundle output
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
    pub delta: f64,           // confidence [0.0, 1.0]
    pub phi: u8,              // coverage [0, 100]
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
            Self::Stable => write!(f, "\u{25CA}\u{207A}"),   // ◊⁺
            Self::Unstable => write!(f, "\u{25CA}\u{207B}"), // ◊⁻
            Self::Unknown => write!(f, "\u{25CA}?"),         // ◊?
        }
    }
}

/// Result of parsing formal blocks — blocks + any diagnostics
pub struct FormalParseResult {
    pub blocks: Vec<FormalBlock>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Parse all formal blocks from a test criterion body
pub fn parse_formal_blocks(body: &str) -> Vec<FormalBlock> {
    parse_formal_blocks_with_diagnostics(body).blocks
}

/// Parse formal blocks with full diagnostic reporting (ADR-016)
pub fn parse_formal_blocks_with_diagnostics(body: &str) -> FormalParseResult {
    let mut result = FormalParseResult {
        blocks: Vec::new(),
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Parse evidence blocks: ⟦Ε⟧⟨...⟩
    let evidence_re = Regex::new(r"⟦Ε⟧⟨([^⟩]+)⟩").expect("constant regex");
    for cap in evidence_re.captures_iter(body) {
        match parse_evidence_fields_validated(&cap[1]) {
            Ok(eb) => result.blocks.push(FormalBlock::Evidence(eb)),
            Err(msg) => result.errors.push(msg),
        }
    }

    // Detect unclosed block delimiters: ⟦ without matching ⟧
    let open_count = body.matches('⟦').count();
    let close_count = body.matches('⟧').count();
    if open_count > close_count {
        result.errors.push(format!(
            "E001: unclosed block delimiter — found {} ⟦ but only {} ⟧",
            open_count, close_count
        ));
    }

    // Detect unrecognised block types: ⟦X:Unknown⟧
    let block_type_re = Regex::new(r"⟦([^⟧]+)⟧").expect("constant regex");
    let known = ["Σ:Types", "Γ:Invariants", "Λ:Scenario", "Λ:ExitCriteria", "Λ:ExitCritera", "Ε"];
    for cap in block_type_re.captures_iter(body) {
        let bt = &cap[1];
        if !known.contains(&bt) {
            result.errors.push(format!(
                "E001: unrecognised block type ⟦{}⟧",
                bt
            ));
        }
    }

    // Parse typed blocks: ⟦BlockType⟧{ ... }
    let block_starts: Vec<(usize, &str)> = find_block_starts(body);

    for (start, block_type) in block_starts {
        let rest = &body[start..];
        if let Some(brace_start) = rest.find('{') {
            let content_start = start + brace_start + 1;
            if let Some(brace_end) = find_matching_brace(body, content_start) {
                let content = body[content_start..brace_end].trim();

                // W004: empty block body
                if content.is_empty() {
                    result.warnings.push(format!(
                        "W004: empty block body ⟦{}⟧{{}}",
                        block_type
                    ));
                    continue;
                }

                match block_type {
                    "Σ:Types" => {
                        result.blocks.push(FormalBlock::Types(parse_type_defs(content)));
                    }
                    "Γ:Invariants" => {
                        result.blocks.push(FormalBlock::Invariants(parse_invariants(content)));
                    }
                    "Λ:Scenario" => {
                        result.blocks.push(FormalBlock::Scenario(parse_scenario(content)));
                    }
                    "Λ:ExitCriteria" | "Λ:ExitCritera" => {
                        result.blocks.push(FormalBlock::ExitCriteria(parse_exit_criteria(content)));
                    }
                    _ => {}
                }
            } else {
                // Unclosed brace
                let line = body[..start].lines().count() + 1;
                result.errors.push(format!(
                    "E001: unclosed '{{' for ⟦{}⟧ block at line {}",
                    block_type, line
                ));
            }
        }
    }

    result
}

/// Validate evidence fields and return error on out-of-range values
fn parse_evidence_fields_validated(fields: &str) -> std::result::Result<EvidenceBlock, String> {
    let mut delta = None;
    let mut phi = None;
    let mut tau = Stability::Unknown;

    for field in fields.split(';') {
        let field = field.trim();
        if let Some(val) = field.strip_prefix("δ≜") {
            delta = val.trim().parse::<f64>().ok();
        } else if let Some(val) = field.strip_prefix("φ≜") {
            phi = val.trim().parse::<u8>().ok();
        } else if let Some(val) = field.strip_prefix("τ≜") {
            let val = val.trim();
            tau = if val.contains("◊⁺") || val.contains("\u{25CA}\u{207A}") {
                Stability::Stable
            } else if val.contains("◊⁻") || val.contains("\u{25CA}\u{207B}") {
                Stability::Unstable
            } else {
                Stability::Unknown
            };
        }
    }

    let d = delta.unwrap_or(0.0);
    let p = phi.unwrap_or(0);

    if !(0.0..=1.0).contains(&d) {
        return Err(format!("E001: evidence δ={:.2} is out of range [0.0, 1.0]", d));
    }
    // phi is u8 so max is 255, but we restrict to 100
    if p > 100 {
        return Err(format!("E001: evidence φ={} is out of range [0, 100]", p));
    }

    Ok(EvidenceBlock { delta: d, phi: p, tau })
}

fn find_block_starts(body: &str) -> Vec<(usize, &str)> {
    let mut results = Vec::new();
    let known_types = ["Σ:Types", "Γ:Invariants", "Λ:Scenario", "Λ:ExitCriteria", "Λ:ExitCritera"];
    for bt in &known_types {
        let pattern = format!("⟦{}⟧", bt);
        let mut search_start = 0;
        while let Some(pos) = body[search_start..].find(&pattern) {
            results.push((search_start + pos, *bt));
            search_start += pos + pattern.len();
        }
    }
    results.sort_by_key(|(pos, _)| *pos);
    results
}

fn find_matching_brace(body: &str, start: usize) -> Option<usize> {
    let mut depth = 1;
    for (i, c) in body[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_type_defs(content: &str) -> Vec<TypeDef> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(';');
            if line.is_empty() {
                return None;
            }
            // Split on ≜
            let parts: Vec<&str> = line.splitn(2, '≜').collect();
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

fn parse_invariants(content: &str) -> Vec<Invariant> {
    // Each line (or multi-line expression) is an invariant
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

fn parse_scenario(content: &str) -> ScenarioBlock {
    let mut given = None;
    let mut when = None;
    let mut then = None;

    let mut current_field: Option<&str> = None;
    let mut current_value = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("given≜") {
            if let Some(field) = current_field {
                set_field(field, &current_value, &mut given, &mut when, &mut then);
            }
            current_field = Some("given");
            current_value = val.trim().to_string();
        } else if let Some(val) = trimmed.strip_prefix("when≜") {
            if let Some(field) = current_field {
                set_field(field, &current_value, &mut given, &mut when, &mut then);
            }
            current_field = Some("when");
            current_value = val.trim().to_string();
        } else if let Some(val) = trimmed.strip_prefix("then≜") {
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

fn parse_exit_criteria(content: &str) -> Vec<ExitField> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| ExitField {
            raw: l.trim().to_string(),
        })
        .collect()
}

/// Check if formal blocks have any evidence blocks and return aggregate metrics
#[allow(dead_code)]
pub fn aggregate_evidence(blocks: &[FormalBlock]) -> Option<EvidenceBlock> {
    let evidence: Vec<&EvidenceBlock> = blocks
        .iter()
        .filter_map(|b| match b {
            FormalBlock::Evidence(e) => Some(e),
            _ => None,
        })
        .collect();

    if evidence.is_empty() {
        return None;
    }

    let avg_delta = evidence.iter().map(|e| e.delta).sum::<f64>() / evidence.len() as f64;
    let avg_phi = evidence.iter().map(|e| e.phi as f64).sum::<f64>() / evidence.len() as f64;

    // Worst-case stability
    let tau = if evidence.iter().any(|e| e.tau == Stability::Unstable) {
        Stability::Unstable
    } else if evidence.iter().any(|e| e.tau == Stability::Unknown) {
        Stability::Unknown
    } else {
        Stability::Stable
    };

    Some(EvidenceBlock {
        delta: avg_delta,
        phi: avg_phi as u8,
        tau,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_evidence_block() {
        let body = "Some text\n\n⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩\n";
        let blocks = parse_formal_blocks(body);
        let evidence = blocks.iter().find_map(|b| match b {
            FormalBlock::Evidence(e) => Some(e),
            _ => None,
        });
        assert!(evidence.is_some());
        let e = evidence.unwrap();
        assert!((e.delta - 0.95).abs() < 0.001);
        assert_eq!(e.phi, 100);
        assert_eq!(e.tau, Stability::Stable);
    }

    #[test]
    fn parse_types_block() {
        let body = "⟦Σ:Types⟧{\n  Node≜IRI\n  Role≜Leader|Follower|Learner\n}\n";
        let blocks = parse_formal_blocks(body);
        let types = blocks.iter().find_map(|b| match b {
            FormalBlock::Types(t) => Some(t),
            _ => None,
        });
        assert!(types.is_some());
        let t = types.unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].name, "Node");
        assert_eq!(t[0].expr, "IRI");
        assert_eq!(t[1].name, "Role");
    }

    #[test]
    fn parse_scenario_block() {
        let body = "⟦Λ:Scenario⟧{\n  given≜cluster_init(nodes:2)\n  when≜elapsed(10s)\n  then≜∃n∈nodes: roles(n)=Leader\n}\n";
        let blocks = parse_formal_blocks(body);
        let scenario = blocks.iter().find_map(|b| match b {
            FormalBlock::Scenario(s) => Some(s),
            _ => None,
        });
        assert!(scenario.is_some());
        let s = scenario.unwrap();
        assert!(s.given.is_some());
        assert!(s.when.is_some());
        assert!(s.then.is_some());
        assert!(s.given.as_ref().unwrap().contains("cluster_init"));
    }

    #[test]
    fn parse_invariants_block() {
        let body = "⟦Γ:Invariants⟧{\n  ∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1\n}\n";
        let blocks = parse_formal_blocks(body);
        let invs = blocks.iter().find_map(|b| match b {
            FormalBlock::Invariants(i) => Some(i),
            _ => None,
        });
        assert!(invs.is_some());
        assert_eq!(invs.unwrap().len(), 1);
    }

    #[test]
    fn evidence_delta_out_of_range() {
        let body = "⟦Ε⟧⟨δ≜1.5;φ≜100;τ≜◊⁺⟩\n";
        let result = parse_formal_blocks_with_diagnostics(body);
        assert!(!result.errors.is_empty(), "should report error for delta > 1.0");
        assert!(result.errors[0].contains("E001"));
        assert!(result.errors[0].contains("out of range"));
    }

    #[test]
    fn empty_block_warning() {
        let body = "⟦Γ:Invariants⟧{}\n";
        let result = parse_formal_blocks_with_diagnostics(body);
        assert!(!result.warnings.is_empty(), "should warn on empty block");
        assert!(result.warnings[0].contains("W004"));
    }

    #[test]
    fn unrecognised_block_type_error() {
        let body = "⟦X:Unknown⟧{ stuff }\n";
        let result = parse_formal_blocks_with_diagnostics(body);
        assert!(!result.errors.is_empty(), "should error on unknown block type");
        assert!(result.errors[0].contains("unrecognised"));
    }

    #[test]
    fn unclosed_delimiter_error() {
        let body = "⟦Γ:Invariants⟧ no closing brace\n";
        // No brace at all — the block just won't be found by find_block_starts
        // But unclosed ⟦ without ⟧ is also detected
        let body2 = "⟦Γ:Invariants some text\n";
        let result = parse_formal_blocks_with_diagnostics(body2);
        assert!(!result.errors.is_empty(), "should detect unclosed ⟦");
    }

    #[test]
    fn valid_evidence_passes() {
        let body = "⟦Ε⟧⟨δ≜0.0;φ≜0;τ≜◊?⟩\n";
        let result = parse_formal_blocks_with_diagnostics(body);
        assert!(result.errors.is_empty());
        assert_eq!(result.blocks.len(), 1);
    }
}
