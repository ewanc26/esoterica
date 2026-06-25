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

// ── FormalRule (Parsed Sound Change) ─────────────────────────────────────

/// A sound change parsed from formal notation, either unconditional or contextual.
#[derive(Debug, Clone, PartialEq)]
pub enum FormalRule {
    /// Applies everywhere the `from` segment appears
    Unconditional { from: String, to: String },
    /// Applies only when the environment matches left/right context
    Contextual { from: String, to: String, left_context: Option<String>, right_context: Option<String> },
}

impl FormalRule {
    /// Parse a rule string into a FormalRule.
    /// Format: `from > to` or `from > to / left_right`
    pub fn parse(input: &str) -> Result<Self, String> {
        parse_formal_rule(input).map(|(_, rule)| rule).map_err(|e| format!("Failed to parse rule '{}': {:?}", input, e))
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

/// Combined engine that applies legacy TOML rules and parsed formal rules in sequence.
pub struct SoundChangeEngine { rules: Vec<SoundChange>, formal_rules: Vec<FormalRule> }

impl SoundChangeEngine {
    pub fn new(rules: Vec<SoundChange>) -> Self { Self { rules, formal_rules: Vec::new() } }
    pub fn from_formal_rules(formal_rule_strings: &[String]) -> Self {
        let formal_rules: Vec<FormalRule> = formal_rule_strings.iter().filter_map(|s| FormalRule::parse(s).ok()).collect();
        Self { rules: Vec::new(), formal_rules }
    }
    pub fn add_formal_rule(&mut self, rule: &str) -> Result<(), String> { let r = FormalRule::parse(rule)?; self.formal_rules.push(r); Ok(()) }
    pub fn apply(&self, word: &str) -> String {
        let mut result = word.to_string();
        for rule in &self.rules { result = self.apply_legacy_rule(&result, rule); }
        for rule in &self.formal_rules { result = rule.apply(&result); }
        result
    }
    fn apply_legacy_rule(&self, word: &str, rule: &SoundChange) -> String {
        let pattern = &rule.pattern; let replacement = &rule.replacement;
        match &rule.context {
            Some(ctx) if ctx == "word_final" => if word.ends_with(pattern) { let e = word.len() - pattern.len(); format!("{}{}", &word[..e], replacement) } else { word.to_string() }
            Some(ctx) if ctx == "word_initial" => if word.starts_with(pattern) { format!("{}{}", replacement, &word[pattern.len()..]) } else { word.to_string() }
            _ => word.replace(pattern, replacement),
        }
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
}
