//! Design-token detection source — custom properties as configured rules.
//!
//! A registered token stylesheet (`detect.tokens`, format 4) is a manifest
//! of design decisions: each `--name: value;` declaration becomes one
//! configured rule in the `tokens` namespace, so `ddd why -- --color-ok`
//! resolves a token to its decision the same way an analyzer id resolves,
//! `ddd diff` flags an UNGOVERNED token, plus a manifest entry whose token
//! vanished goes STALE. The "severity" carried is the value's coarse type
//! (color | dimension | keyword | reference | other), which is also what
//! the pair adapter treats as the token's signature.

use crate::configured::ConfiguredRule;

/// Parse the custom-property declarations of one stylesheet.
pub fn parse(text: &str, source: &str) -> Result<Vec<ConfiguredRule>, String> {
    let mut out: Vec<ConfiguredRule> = Vec::new();
    for line in without_comments(text).lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("--") {
            continue;
        }
        let Some((name, value)) = trimmed.split_once(':') else { continue };
        let name = name.trim();
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            continue;
        }
        let value = value.trim().trim_end_matches(';').trim_end_matches('}').trim();
        if out.iter().any(|r| r.rule_id == name) {
            continue;
        }
        out.push(ConfiguredRule {
            rule_id: name.to_string(),
            severity: value_type(value).to_string(),
            disabled: false,
            source: source.to_string(),
            scope: None,
        });
    }
    Ok(out)
}

fn without_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start..].find("*/") {
            Some(end) => rest = &rest[start + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// The coarse value type — kept in lockstep with the pair adapter's
/// signature typing so `diff` and the interceptor describe one vocabulary.
pub fn value_type(value: &str) -> &'static str {
    let v = value.trim();
    if v.is_empty() {
        return "other";
    }
    let lower = v.to_ascii_lowercase();
    if v.starts_with('#')
        || ["rgb(", "rgba(", "hsl(", "hsla(", "oklch(", "oklab(", "color("]
            .iter()
            .any(|f| lower.starts_with(f))
    {
        return "color";
    }
    if lower.starts_with("var(") {
        return "reference";
    }
    let num: String =
        v.chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
    if !num.is_empty() && num.parse::<f64>().is_ok() {
        let unit = &v[num.len()..];
        if unit.is_empty() || unit.chars().all(|c| c.is_ascii_alphabetic() || c == '%') {
            return "dimension";
        }
    }
    if v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return "keyword";
    }
    "other"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_parse_with_their_value_types() {
        let rules = parse(
            ":root {\n  --color-ok: #1a7f37;\n  --gap: 1.5rem;\n  --weight: bold;\n  /* --ghost: 1; */\n}\n.x { color: var(--color-ok); }\n",
            "assets/tokens.css",
        )
        .expect("parse");
        let ids: Vec<(&str, &str)> =
            rules.iter().map(|r| (r.rule_id.as_str(), r.severity.as_str())).collect();
        assert_eq!(
            ids,
            vec![("--color-ok", "color"), ("--gap", "dimension"), ("--weight", "keyword")]
        );
    }
}
