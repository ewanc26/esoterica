//! Sound change engine with two backends:
//! 1. Legacy TOML rules (pattern/replacement/context) from data files.
//! 2. Formal parser using nom that supports notation like `p > b / V_V`.
//!
//! Spec notation: FROM > TO / LEFT_RIGHT
//!   # = word boundary, V = any vowel, C = any consonant
//!   ∅ (U+2205) = deletion

use crate::archetypes::SoundChange;
use nom::{
    IResult,
    bytes::complete::{tag, take_while1},
    character::complete::multispace0,
    sequence::{delimited, preceded, tuple},
    branch::alt,
    combinator::{opt, recognize},
    Parser,
};
use serde::{Deserialize, Serialize};

// ── FormalRule (Parsed Sound Change) ─────────────────────────────────────

/// A sound change parsed from formal notation, either unconditional or contextual.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FormalRule {
    /// Applies everywhere the `from` segment appears
    Unconditional { from: String, to: String },
    /// Applies only when the environment matches left/right context
    Contextual { from: String, to: String, left_context: Option<String>, right_context: Option<String> },
}

impl FormalRule {
    /// Parse a rule string into a FormalRule.
    /// Format: `from > to` or `from > to / left_right`
    ///
    /// Trailing input is rejected, so a malformed environment such as
    /// `p > b / V_V_V` is an error rather than a silently truncated rule.
    pub fn parse(input: &str) -> Result<Self, String> {
        let (rest, rule) = parse_formal_rule(input.trim())
            .map_err(|e| format!("Failed to parse rule '{}': {:?}", input, e))?;
        if !rest.trim().is_empty() {
            return Err(format!(
                "Failed to parse rule '{}': unexpected trailing input '{}'",
                input,
                rest.trim()
            ));
        }
        Ok(rule)
    }

    /// The segment this rule rewrites.
    pub fn from(&self) -> &str {
        match self {
            FormalRule::Unconditional { from, .. } | FormalRule::Contextual { from, .. } => from,
        }
    }

    /// The segment this rule rewrites to; `∅` marks deletion.
    pub fn to(&self) -> &str {
        match self {
            FormalRule::Unconditional { to, .. } | FormalRule::Contextual { to, .. } => to,
        }
    }

    /// Render the rule back into formal notation.
    pub fn to_notation(&self) -> String {
        match self {
            FormalRule::Unconditional { from, to } => format!("{} > {}", from, to),
            FormalRule::Contextual { from, to, left_context, right_context } => format!(
                "{} > {} / {}_{}",
                from,
                to,
                left_context.as_deref().unwrap_or(""),
                right_context.as_deref().unwrap_or(""),
            ),
        }
    }

    /// Apply this rule to a word, returning the transformed string.
    pub fn apply(&self, word: &str) -> String {
        match self {
            FormalRule::Unconditional { from, to } => {
                let repl = if to == "\u{2205}" { "" } else { to.as_str() };
                word.replace(from.as_str(), repl)
            }
            FormalRule::Contextual { from, to, left_context, right_context } => {
                let repl = if to == "\u{2205}" { "" } else { to.as_str() };
                let chars: Vec<char> = word.chars().collect();
                let from_chars: Vec<char> = from.chars().collect();
                let from_len = from_chars.len();
                if from_len == 0 || chars.len() < from_len { return word.to_string(); }
                let mut result = String::new();
                let mut i = 0;
                while i < chars.len() {
                    if i + from_len <= chars.len() && chars[i..i + from_len] == from_chars[..] {
                        let left_ok = match left_context {
                            None => true,
                            Some(ref ctx) => match ctx.as_str() {
                                "#" => i == 0,
                                "V" => i > 0 && is_vowel(chars[i - 1]),
                                "C" => i > 0 && is_consonant(chars[i - 1]),
                                _ => { let cl = ctx.chars().count(); i >= cl && chars[i - cl..i].iter().collect::<String>() == *ctx }
                            }
                        };
                        let right_ok = match right_context {
                            None => true,
                            Some(ref ctx) => match ctx.as_str() {
                                "#" => i + from_len == chars.len(),
                                "V" => i + from_len < chars.len() && is_vowel(chars[i + from_len]),
                                "C" => i + from_len < chars.len() && is_consonant(chars[i + from_len]),
                                _ => { let cl = ctx.chars().count(); i + from_len + cl <= chars.len() && chars[i + from_len..i + from_len + cl].iter().collect::<String>() == *ctx }
                            }
                        };
                        if left_ok && right_ok { result.push_str(repl); i += from_len; continue; }
                    }
                    result.push(chars[i]); i += 1;
                }
                result
            }
        }
    }
}

// ── Helper Predicates ────────────────────────────────────────────────────

fn is_vowel(c: char) -> bool { matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | '\u{00e4}' | '\u{00f6}' | 'y' | '\u{00e6}' | '\u{00f8}') }
fn is_consonant(c: char) -> bool { c.is_alphabetic() && !is_vowel(c) }

// ── Nom Parsers ──────────────────────────────────────────────────────────

/// Parse one alphabetic/IPA segment (e.g. "p", "sh", "ŋ").
fn parse_segment(input: &str) -> IResult<&str, String> {
    let (input, seg) = recognize(take_while1(|c: char| c.is_alphabetic() || "\u{0283}\u{0292}\u{03b8}\u{00f0}\u{014b}\u{0294}\u{0295}\u{027e}".contains(c)))(input)?;
    Ok((input, seg.to_string()))
}

/// Parse the empty-set symbol ∅ for deletion rules.
fn parse_empty(input: &str) -> IResult<&str, String> { let (input, _) = tag("\u{2205}")(input)?; Ok((input, "\u{2205}".to_string())) }
fn parse_target(input: &str) -> IResult<&str, String> { alt((parse_empty, parse_segment))(input) }

/// Parse a context element: word boundary (#), vowel class (V), consonant class (C), or literal segment.
fn parse_context_element(input: &str) -> IResult<&str, String> {
    alt((tag("#").map(|s: &str| s.to_string()), tag("V").map(|s: &str| s.to_string()), tag("C").map(|s: &str| s.to_string()), parse_segment))(input)
}

/// Parse the environment part of a rule: left_/right context around `_`.
fn parse_environment(input: &str) -> IResult<&str, (Option<String>, Option<String>)> {
    let parse_both = tuple((parse_context_element, delimited(multispace0, tag("_"), multispace0), parse_context_element));
    let parse_right_only = tuple((delimited(multispace0, tag("_"), multispace0), parse_context_element));
    let parse_left_only = tuple((parse_context_element, delimited(multispace0, tag("_"), multispace0)));
    alt((parse_both.map(|(l, _, r)| (Some(l), Some(r))), parse_right_only.map(|(_, r)| (None, Some(r))), parse_left_only.map(|(l, _)| (Some(l), None))))(input)
}

/// Top-level parser: `from > to / left_right` or `from > to`.
fn parse_formal_rule(input: &str) -> IResult<&str, FormalRule> {
    let (input, from) = parse_segment(input)?;
    let (input, _) = delimited(multispace0, tag(">"), multispace0)(input)?;
    let (input, to) = parse_target(input)?;
    let (input, has_context) = opt(preceded(delimited(multispace0, tag("/"), multispace0), parse_environment))(input)?;
    let rule = match has_context {
        Some((l, r)) => FormalRule::Contextual { from, to, left_context: l, right_context: r },
        None => FormalRule::Unconditional { from, to },
    };
    Ok((input, rule))
}

// ── SoundChangeEngine ────────────────────────────────────────────────────

/// One step of a derivation: the rule applied and the form it produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStep {
    /// The rule in its source notation.
    pub rule: String,
    /// The form before this rule ran.
    pub before: String,
    /// The form after this rule ran.
    pub after: String,
    /// Whether this rule actually changed anything.
    pub changed: bool,
}

/// Combined engine that applies legacy TOML rules and parsed formal rules in sequence.
#[derive(Debug, Clone, Default)]
pub struct SoundChangeEngine { rules: Vec<SoundChange>, formal_rules: Vec<FormalRule> }

impl SoundChangeEngine {
    pub fn new(rules: Vec<SoundChange>) -> Self { Self { rules, formal_rules: Vec::new() } }

    /// Build an engine from formal notation, discarding rules that do not parse.
    ///
    /// Prefer [`try_from_formal_rules`](Self::try_from_formal_rules): a typo in
    /// a rule silently disables it here.
    pub fn from_formal_rules(formal_rule_strings: &[String]) -> Self {
        let formal_rules: Vec<FormalRule> = formal_rule_strings.iter().filter_map(|s| FormalRule::parse(s).ok()).collect();
        Self { rules: Vec::new(), formal_rules }
    }

    /// Build an engine from formal notation, reporting the first invalid rule.
    pub fn try_from_formal_rules(formal_rule_strings: &[String]) -> Result<Self, String> {
        let formal_rules = formal_rule_strings
            .iter()
            .map(|s| FormalRule::parse(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { rules: Vec::new(), formal_rules })
    }

    pub fn add_formal_rule(&mut self, rule: &str) -> Result<(), String> { let r = FormalRule::parse(rule)?; self.formal_rules.push(r); Ok(()) }

    /// Legacy TOML rules held by this engine.
    pub fn legacy_rules(&self) -> &[SoundChange] { &self.rules }

    /// Parsed formal rules held by this engine.
    pub fn formal_rules(&self) -> &[FormalRule] { &self.formal_rules }

    /// Total number of rules that will be applied.
    pub fn rule_count(&self) -> usize { self.rules.len() + self.formal_rules.len() }

    pub fn apply(&self, word: &str) -> String {
        let mut result = word.to_string();
        for rule in &self.rules { result = Self::apply_legacy_rule(&result, rule); }
        for rule in &self.formal_rules { result = rule.apply(&result); }
        result
    }

    /// Apply every rule in order, recording the form after each one.
    ///
    /// This is the derivation a historical linguist would write out, and it is
    /// what the rule editor needs in order to show why a form came out as it did.
    pub fn trace(&self, word: &str) -> Vec<TraceStep> {
        let mut current = word.to_string();
        let mut steps = Vec::with_capacity(self.rule_count());

        for rule in &self.rules {
            let after = Self::apply_legacy_rule(&current, rule);
            let before = std::mem::replace(&mut current, after.clone());
            steps.push(TraceStep {
                rule: legacy_notation(rule),
                changed: before != after,
                before,
                after,
            });
        }

        for rule in &self.formal_rules {
            let after = rule.apply(&current);
            let before = std::mem::replace(&mut current, after.clone());
            steps.push(TraceStep {
                rule: rule.to_notation(),
                changed: before != after,
                before,
                after,
            });
        }

        steps
    }

    fn apply_legacy_rule(word: &str, rule: &SoundChange) -> String {
        let pattern = &rule.pattern; let replacement = &rule.replacement;
        if pattern.is_empty() { return word.to_string(); }
        match &rule.context {
            Some(ctx) if ctx == "word_final" => if word.ends_with(pattern) { let e = word.len() - pattern.len(); format!("{}{}", &word[..e], replacement) } else { word.to_string() }
            Some(ctx) if ctx == "word_initial" => if word.starts_with(pattern) { format!("{}{}", replacement, &word[pattern.len()..]) } else { word.to_string() }
            _ => word.replace(pattern, replacement),
        }
    }
}

/// Render a legacy TOML rule in a readable one-line form.
fn legacy_notation(rule: &SoundChange) -> String {
    let target = if rule.replacement.is_empty() { "\u{2205}" } else { rule.replacement.as_str() };
    match rule.context.as_deref() {
        Some("word_final") => format!("{} > {} / _#", rule.pattern, target),
        Some("word_initial") => format!("{} > {} / #_", rule.pattern, target),
        Some(other) => format!("{} > {} ({})", rule.pattern, target, other),
        None => format!("{} > {}", rule.pattern, target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_parse_unconditional() { let r = FormalRule::parse("p > b").unwrap(); assert_eq!(r, FormalRule::Unconditional { from: "p".into(), to: "b".into() }); }
    #[test] fn test_parse_with_context() { let r = FormalRule::parse("p > b / V_V").unwrap(); assert_eq!(r, FormalRule::Contextual { from: "p".into(), to: "b".into(), left_context: Some("V".into()), right_context: Some("V".into()) }); }
    #[test] fn test_parse_word_final() { let r = FormalRule::parse("k > h / _#").unwrap(); assert_eq!(r, FormalRule::Contextual { from: "k".into(), to: "h".into(), left_context: None, right_context: Some("#".into()) }); }
    #[test] fn test_parse_word_initial() { let r = FormalRule::parse("s > \u{2205} / #_").unwrap(); assert_eq!(r, FormalRule::Contextual { from: "s".into(), to: "\u{2205}".into(), left_context: Some("#".into()), right_context: None }); }
    #[test] fn test_parse_deletion() { let r = FormalRule::parse("h > \u{2205} / _#").unwrap(); assert_eq!(r, FormalRule::Contextual { from: "h".into(), to: "\u{2205}".into(), left_context: None, right_context: Some("#".into()) }); }
    #[test] fn test_apply_unconditional() { let r = FormalRule::Unconditional { from: "p".into(), to: "b".into() }; assert_eq!(r.apply("pata"), "bata"); }
    #[test] fn test_apply_intervocalic() { let r = FormalRule::Contextual { from: "p".into(), to: "b".into(), left_context: Some("V".into()), right_context: Some("V".into()) }; assert_eq!(r.apply("apa"), "aba"); assert_eq!(r.apply("pa"), "pa"); }
    #[test] fn test_apply_word_final() { let r = FormalRule::Contextual { from: "k".into(), to: "h".into(), left_context: None, right_context: Some("#".into()) }; assert_eq!(r.apply("tak"), "tah"); assert_eq!(r.apply("takka"), "takka"); }
    #[test] fn test_apply_word_initial_deletion() { let r = FormalRule::Contextual { from: "s".into(), to: "\u{2205}".into(), left_context: Some("#".into()), right_context: None }; assert_eq!(r.apply("stop"), "top"); assert_eq!(r.apply("fast"), "fast"); }
    #[test] fn test_engine_formal_rules() { let e = SoundChangeEngine::from_formal_rules(&["p > b / V_V".to_string(), "k > h / _#".to_string()]); assert_eq!(e.apply("paka"), "paka"); assert_eq!(e.apply("apaka"), "abaka"); assert_eq!(e.apply("pak"), "pah"); assert_eq!(e.apply("apak"), "abah"); }
    #[test] fn test_legacy_engine() { let e = SoundChangeEngine::new(vec![SoundChange { pattern: "p".into(), replacement: "b".into(), context: Some("word_final".into()) }]); assert_eq!(e.apply("tap"), "tab"); assert_eq!(e.apply("pat"), "pat"); }
    #[test] fn test_grimms_law_voiceless_stops() { let e = SoundChangeEngine::from_formal_rules(&["p > f".to_string(), "t > \u{03b8}".to_string(), "k > h".to_string()]); assert_eq!(e.apply("pater"), "fa\u{03b8}er"); }
    #[test] fn test_verners_law() { let e = SoundChangeEngine::from_formal_rules(&["f > v / V_V".to_string(), "\u{03b8} > \u{00f0} / V_V".to_string(), "s > z / V_V".to_string()]); assert_eq!(e.apply("afa"), "ava"); assert_eq!(e.apply("a\u{03b8}a"), "a\u{00f0}a"); assert_eq!(e.apply("asa"), "aza"); }
    #[test] fn test_palatalization() { let e = SoundChangeEngine::from_formal_rules(&["k > ch / _i".to_string()]); assert_eq!(e.apply("kina"), "china"); assert_eq!(e.apply("kata"), "kata"); }
    #[test] fn test_final_devoicing() { let e = SoundChangeEngine::from_formal_rules(&["b > p / _#".to_string(), "d > t / _#".to_string(), "g > k / _#".to_string()]); assert_eq!(e.apply("tab"), "tap"); assert_eq!(e.apply("tod"), "tot"); assert_eq!(e.apply("tag"), "tak"); }
    #[test] fn test_nasal_assimilation() { let e = SoundChangeEngine::from_formal_rules(&["n > m / _p".to_string(), "n > ng / _k".to_string()]); assert_eq!(e.apply("tanpa"), "tampa"); assert_eq!(e.apply("tanka"), "tangka"); }
    #[test] fn test_compound_rules() { let e = SoundChangeEngine::from_formal_rules(&["p > f / _#".to_string(), "t > s / _#".to_string(), "a > e / _#".to_string()]); assert_eq!(e.apply("pata"), "pate"); assert_eq!(e.apply("tap"), "taf"); assert_eq!(e.apply("pat"), "pas"); }

    // ── Parse strictness ─────────────────────────────────────────────────

    #[test]
    fn surrounding_whitespace_is_tolerated() {
        assert_eq!(
            FormalRule::parse("  p > b  ").unwrap(),
            FormalRule::Unconditional { from: "p".into(), to: "b".into() }
        );
    }

    #[test]
    fn trailing_junk_is_rejected() {
        // Previously the parser returned the prefix it understood and dropped
        // the rest, silently changing what the rule meant.
        let err = FormalRule::parse("p > b / V_V nonsense").unwrap_err();
        assert!(err.contains("trailing input"), "unhelpful error: {}", err);
    }

    #[test]
    fn a_missing_arrow_is_rejected() {
        assert!(FormalRule::parse("p b").is_err());
        assert!(FormalRule::parse("").is_err());
    }

    #[test]
    fn try_from_formal_rules_reports_the_bad_rule() {
        let err = SoundChangeEngine::try_from_formal_rules(&[
            "p > b".to_string(),
            "%%%".to_string(),
        ])
        .unwrap_err();
        assert!(err.contains("%%%"), "unhelpful error: {}", err);
    }

    #[test]
    fn from_formal_rules_still_skips_bad_rules() {
        let e = SoundChangeEngine::from_formal_rules(&["p > b".to_string(), "%%%".to_string()]);
        assert_eq!(e.rule_count(), 1);
    }

    // ── Notation round-trip ──────────────────────────────────────────────

    #[test]
    fn rules_round_trip_through_their_notation() {
        for source in ["p > b", "k > h / _#", "s > \u{2205} / #_", "p > b / V_V"] {
            let rule = FormalRule::parse(source).unwrap();
            let reparsed = FormalRule::parse(&rule.to_notation()).unwrap();
            assert_eq!(rule, reparsed, "{} did not round-trip", source);
        }
    }

    #[test]
    fn accessors_expose_the_rule_segments() {
        let rule = FormalRule::parse("k > h / _#").unwrap();
        assert_eq!(rule.from(), "k");
        assert_eq!(rule.to(), "h");
    }

    #[test]
    fn rules_serialise_with_a_kind_tag() {
        let rule = FormalRule::parse("p > b / V_V").unwrap();
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("\"kind\":\"contextual\""), "got: {}", json);
        let restored: FormalRule = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, rule);
    }

    // ── Tracing ──────────────────────────────────────────────────────────

    #[test]
    fn trace_records_one_step_per_rule() {
        let e = SoundChangeEngine::try_from_formal_rules(&[
            "p > f".to_string(),
            "k > h".to_string(),
        ])
        .unwrap();
        let trace = e.trace("paka");
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].after, "faka");
        assert_eq!(trace[1].after, "faha");
        assert_eq!(trace[1].before, "faka");
    }

    #[test]
    fn trace_covers_legacy_rules_too() {
        let e = SoundChangeEngine::new(vec![SoundChange {
            pattern: "p".into(),
            replacement: "b".into(),
            context: Some("word_final".into()),
        }]);
        let trace = e.trace("tap");
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].rule, "p > b / _#");
        assert_eq!(trace[0].after, "tab");
        assert!(trace[0].changed);
    }

    #[test]
    fn trace_final_form_matches_apply() {
        let e = SoundChangeEngine::try_from_formal_rules(&[
            "p > f".to_string(),
            "a > e / _#".to_string(),
        ])
        .unwrap();
        assert_eq!(e.trace("papa").last().unwrap().after, e.apply("papa"));
    }

    #[test]
    fn trace_of_an_empty_engine_is_empty() {
        assert!(SoundChangeEngine::default().trace("mana").is_empty());
    }

    // ── Legacy rule edge cases ───────────────────────────────────────────

    #[test]
    fn an_empty_legacy_pattern_is_a_no_op() {
        // `str::replace("", x)` would otherwise splice the replacement between
        // every character.
        let e = SoundChangeEngine::new(vec![SoundChange {
            pattern: String::new(),
            replacement: "z".into(),
            context: None,
        }]);
        assert_eq!(e.apply("mana"), "mana");
    }

    #[test]
    fn legacy_deletion_renders_as_the_empty_set() {
        let e = SoundChangeEngine::new(vec![SoundChange {
            pattern: "a".into(),
            replacement: String::new(),
            context: Some("word_final".into()),
        }]);
        assert_eq!(e.apply("mana"), "man");
        assert_eq!(e.trace("mana")[0].rule, "a > \u{2205} / _#");
    }

    #[test]
    fn every_bundled_sound_change_set_applies_without_panicking() {
        for (name, rules) in crate::archetypes::get_sound_change_registry() {
            let e = SoundChangeEngine::new(rules);
            for word in ["kata", "t\u{00e4}m\u{00f6}", "\u{014b}a\u{0283}u", "", "a"] {
                let out = e.apply(word);
                assert_eq!(out, e.trace(word).last().map(|s| s.after.clone()).unwrap_or(out.clone()), "{} on '{}'", name, word);
            }
        }
    }
}
