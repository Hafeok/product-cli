//! Per-artifact validation rules.

use super::super::types::*;
use super::helpers::*;
use super::ValidationContext;
use serde_yaml::Value;
use std::collections::HashMap;

pub fn validate_artifact(
    a: &ArtifactSpec,
    refs: &HashMap<String, (ArtifactType, usize)>,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
    let has_title = matches!(a.fields.get(Value::String("title".into())), Some(Value::String(s)) if !s.trim().is_empty());
    if !has_title {
        findings.push(Finding::error(
            "E006",
            format!("{} artifact requires a non-empty 'title'", a.artifact_type),
            format!("$.artifacts[{}].title", a.index),
        ));
    }

    match a.artifact_type {
        ArtifactType::Feature => validate_feature(a, refs, ctx, findings),
        ArtifactType::Adr => validate_adr(a, refs, ctx, findings),
        ArtifactType::Tc => validate_tc(a, refs, ctx, findings),
        ArtifactType::Dep => validate_dep(a, refs, ctx, findings),
    }
}

fn validate_feature(
    a: &ArtifactSpec,
    refs: &HashMap<String, (ArtifactType, usize)>,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
    if a.fields.get(Value::String("phase".into())).is_none() {
        findings.push(Finding::error(
            "E006",
            "feature artifact requires 'phase'",
            format!("$.artifacts[{}].phase", a.index),
        ));
    }

    check_domains_vocab(
        a.fields.get(Value::String("domains".into())),
        &ctx.config.domains,
        &format!("$.artifacts[{}].domains", a.index),
        findings,
    );

    for (key, kind) in [
        ("adrs", ArtifactType::Adr),
        ("tests", ArtifactType::Tc),
        ("uses", ArtifactType::Dep),
        ("depends-on", ArtifactType::Feature),
    ] {
        check_id_list(
            a.fields.get(Value::String(key.into())),
            kind,
            refs,
            ctx.graph,
            &format!("$.artifacts[{}].{}", a.index, key),
            findings,
        );
    }

    if let Some(Value::Mapping(m)) = a.fields.get(Value::String("domains-acknowledged".into())) {
        for (k, v) in m.iter() {
            let key = k.as_str().unwrap_or("?").to_string();
            let empty = match v {
                Value::String(s) => s.trim().is_empty(),
                _ => true,
            };
            if empty {
                findings.push(Finding::error(
                    "E011",
                    format!("domain acknowledgement for '{}' must include non-empty reasoning", key),
                    format!("$.artifacts[{}].domains-acknowledged.{}", a.index, key),
                ));
            }
        }
    }
}

fn validate_adr(
    a: &ArtifactSpec,
    refs: &HashMap<String, (ArtifactType, usize)>,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
    if let Some(Value::String(s)) = a.fields.get(Value::String("scope".into())) {
        if !matches!(s.as_str(), "cross-cutting" | "domain" | "feature-specific") {
            findings.push(Finding::error(
                "E006",
                format!("invalid scope '{}' — expected cross-cutting, domain, or feature-specific", s),
                format!("$.artifacts[{}].scope", a.index),
            ));
        }
    }

    check_domains_vocab(
        a.fields.get(Value::String("domains".into())),
        &ctx.config.domains,
        &format!("$.artifacts[{}].domains", a.index),
        findings,
    );
    check_id_list(
        a.fields.get(Value::String("features".into())),
        ArtifactType::Feature,
        refs,
        ctx.graph,
        &format!("$.artifacts[{}].features", a.index),
        findings,
    );
    check_id_list(
        a.fields.get(Value::String("governs".into())),
        ArtifactType::Dep,
        refs,
        ctx.graph,
        &format!("$.artifacts[{}].governs", a.index),
        findings,
    );

    if let Some(v) = a.fields.get(Value::String("supersedes".into())) {
        match v {
            Value::String(s) => check_single_id(s, ArtifactType::Adr, refs, ctx.graph,
                &format!("$.artifacts[{}].supersedes", a.index), findings),
            Value::Sequence(seq) => {
                for (i, item) in seq.iter().enumerate() {
                    if let Value::String(s) = item {
                        check_single_id(s, ArtifactType::Adr, refs, ctx.graph,
                            &format!("$.artifacts[{}].supersedes[{}]", a.index, i), findings);
                    }
                }
            }
            _ => {}
        }
    }
}

fn validate_tc(
    a: &ArtifactSpec,
    refs: &HashMap<String, (ArtifactType, usize)>,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
    let tc_type = a
        .fields
        .get(Value::String("tc-type".into()))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    if let Some(ref s) = tc_type {
        if !ctx.config.is_known_tc_type(s.as_str()) {
            findings.push(
                Finding::error(
                    "E006",
                    format!("unknown tc-type '{}'", s),
                    format!("$.artifacts[{}].tc-type", a.index),
                )
                .with_hint(ctx.config.tc_type_hint()),
            );
        }
    }

    if let Some(Value::Mapping(m)) = a.fields.get(Value::String("validates".into())) {
        if let Some(v) = m.get(Value::String("features".into())) {
            check_id_list_value(v, ArtifactType::Feature, refs, ctx.graph,
                &format!("$.artifacts[{}].validates.features", a.index), findings);
        }
        if let Some(v) = m.get(Value::String("adrs".into())) {
            check_id_list_value(v, ArtifactType::Adr, refs, ctx.graph,
                &format!("$.artifacts[{}].validates.adrs", a.index), findings);
        }
    }

    // FT-047 / ADR-041: absence TCs must have empty validates.features and
    // non-empty validates.adrs.
    if tc_type.as_deref() == Some("absence") {
        let validates = a.fields.get(Value::String("validates".into()));
        let (features_nonempty, adrs_nonempty) = match validates {
            Some(Value::Mapping(m)) => {
                let f = matches!(
                    m.get(Value::String("features".into())),
                    Some(Value::Sequence(s)) if !s.is_empty()
                );
                let aa = matches!(
                    m.get(Value::String("adrs".into())),
                    Some(Value::Sequence(s)) if !s.is_empty()
                );
                (f, aa)
            }
            _ => (false, false),
        };
        if features_nonempty {
            findings.push(Finding::error(
                "E006",
                "absence TC requires empty `validates.features` (cross-cutting only)",
                format!("$.artifacts[{}].validates.features", a.index),
            ));
        }
        if !adrs_nonempty {
            findings.push(Finding::error(
                "E006",
                "absence TC requires non-empty `validates.adrs` linking the governing ADR(s)",
                format!("$.artifacts[{}].validates.adrs", a.index),
            ));
        }
    }

    if let Some(Value::String(r)) = a.fields.get(Value::String("runner".into())) {
        if !matches!(r.as_str(), "cargo-test" | "bash" | "pytest" | "custom") {
            findings.push(Finding::error(
                "E006",
                format!("invalid runner '{}'", r),
                format!("$.artifacts[{}].runner", a.index),
            ));
        }
    }
}

fn validate_dep(
    a: &ArtifactSpec,
    refs: &HashMap<String, (ArtifactType, usize)>,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
    if let Some(Value::String(s)) = a.fields.get(Value::String("dep-type".into())) {
        if !matches!(
            s.as_str(),
            "library" | "service" | "api" | "tool" | "hardware" | "runtime"
        ) {
            findings.push(Finding::error(
                "E006",
                format!("invalid dep-type '{}'", s),
                format!("$.artifacts[{}].dep-type", a.index),
            ));
        }
    }

    if let Some(v) = a.fields.get(Value::String("adrs".into())) {
        check_id_list_value(v, ArtifactType::Adr, refs, ctx.graph,
            &format!("$.artifacts[{}].adrs", a.index), findings);
    }

    if let Some(Value::String(r)) = a.fields.get(Value::String("breaking-change-risk".into())) {
        if r == "high" {
            findings.push(Finding::warning(
                "W013",
                format!(
                    "new dependency '{}' has breaking-change-risk: high",
                    a.fields.get(Value::String("title".into()))
                        .and_then(|v| v.as_str()).unwrap_or("")
                ),
                format!("$.artifacts[{}].breaking-change-risk", a.index),
            ));
        }
    }
}
