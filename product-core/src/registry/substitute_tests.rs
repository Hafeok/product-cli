//! Unit tests for the single-pass renderer.

use super::*;

fn tok<'a>(label: &'a str, token: &'a str, value: &'a str) -> Token<'a> {
    Token { label, token, value }
}

#[test]
fn substitutes_every_occurrence_once() {
    let out = render("a {{X}} b {{X}}", &[tok("--x", "{{X}}", "V")]);
    assert_eq!(out.text, "a V b V");
    assert_eq!(out.sites.len(), 2);
}

#[test]
fn recorded_sites_address_the_written_value() {
    let out = render("owner: {{X}}!", &[tok("--x", "{{X}}", "Ø-value")]);
    let site = &out.sites[0];
    assert_eq!(&out.text[site.offset..site.offset + site.len], "Ø-value");
}

#[test]
fn a_value_that_looks_like_a_token_is_not_re_read() {
    let tokens = [tok("--x", "{{X}}", "{{Y}}"), tok("--y", "{{Y}}", "SECOND")];
    let out = render("{{X}}", &tokens);
    assert_eq!(out.text, "{{Y}}", "an emitted value must never be scanned again");
}

#[test]
fn sequential_replace_would_have_re_read_it() {
    // The mechanism this renderer replaces, shown failing on the same input.
    let naive = "{{X}}".replace("{{X}}", "{{Y}}").replace("{{Y}}", "SECOND");
    assert_eq!(naive, "SECOND");
    assert_eq!(render("{{X}}", &[tok("--x", "{{X}}", "{{Y}}"), tok("--y", "{{Y}}", "SECOND")]).text, "{{Y}}");
}

#[test]
fn regex_and_shell_sigils_are_data() {
    for value in ["&", "\\1", "$0", "a|b", "a/b", "back\\slash", "$(whoami)", "`id`"] {
        let out = render("[{{X}}]", &[tok("--x", "{{X}}", value)]);
        assert_eq!(out.text, format!("[{value}]"), "sigil {value:?} was not carried verbatim");
    }
}

#[test]
fn multibyte_text_around_a_token_survives() {
    let out = render("— {{X}} — æøå", &[tok("--x", "{{X}}", "værdi")]);
    assert_eq!(out.text, "— værdi — æøå");
}

#[test]
fn longest_token_wins() {
    let tokens = [tok("short", "{{X}}", "S"), tok("long", "{{X}}Y", "L")];
    assert_eq!(render("{{X}}Y", &tokens).text, "L");
}

#[test]
fn disjointness_is_enforced() {
    let tokens = [tok("short", "{{X}}", "S"), tok("long", "{{X}}Y", "L")];
    assert!(tokens_are_disjoint(&tokens).is_err());
    assert!(tokens_are_disjoint(&[tok("a", "{{A}}", "1"), tok("b", "{{B}}", "2")]).is_ok());
}

#[test]
fn text_without_tokens_is_unchanged() {
    let input = "no placeholders here {{lowercase}} $ & \\";
    assert_eq!(render(input, &[tok("--x", "{{X}}", "V")]).text, input);
}
