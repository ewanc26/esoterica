//! Morphological transformation engine.
//! Applies affixation and reduplication rules to root words produced by the
//! phonology engine.
//!
//! Rules run in the order they appear in the profile, each operating on the
//! output of the last, so a prefix declared after a reduplication attaches to
//! the reduplicated stem rather than to the bare root.

use crate::archetypes::{MorphRule, Morphology};
use crate::rng::SharedRng;
use rand::seq::SliceRandom;

pub struct MorphologyEngine {
    morphology: Morphology,
    rng: SharedRng,
}

impl MorphologyEngine {
    /// Build an engine seeded from system entropy.
    pub fn new(morphology: Morphology) -> Self {
        Self { morphology, rng: SharedRng::from_entropy() }
    }

    /// Build a reproducible engine; noun-class assignment becomes deterministic.
    pub fn seeded(morphology: Morphology, seed: u64) -> Self {
        Self { morphology, rng: SharedRng::from_seed(seed) }
    }

    /// The morphological profile this engine applies.
    pub fn morphology(&self) -> &Morphology {
        &self.morphology
    }

    /// Apply every morphology rule in order to a root word.
    /// Returns the transformed word and an optional randomly-assigned noun class.
    pub fn apply_rules(&self, root: &str) -> (String, Option<String>) {
        let word = self.apply_affixes(root);
        let noun_class = self
            .morphology
            .noun_classes
            .as_ref()
            .and_then(|classes| self.rng.with(|rng| classes.choose(rng).cloned()));
        (word, noun_class)
    }

    /// Apply the affix rules alone, without assigning a noun class.
    pub fn apply_affixes(&self, root: &str) -> String {
        let mut word = root.to_string();
        for rule in &self.morphology.rules {
            word = apply_rule(&word, rule);
        }
        word
    }
}

/// Apply a single morphological rule to a stem.
fn apply_rule(word: &str, rule: &MorphRule) -> String {
    match rule {
        MorphRule::Suffix(suffix) => format!("{}{}", word, suffix),
        MorphRule::Prefix(prefix) => format!("{}{}", prefix, word),
        MorphRule::Infix(infix) => insert_infix(word, infix),
        MorphRule::Circumfix(prefix, suffix) => format!("{}{}{}", prefix, word, suffix),
        MorphRule::Reduplication => format!("{}-{}", word, word),
        MorphRule::PartialReduplication(n) => {
            let copied: String = word.chars().take(*n).collect();
            if copied.is_empty() {
                word.to_string()
            } else {
                format!("{}-{}", copied, word)
            }
        }
    }
}

/// Insert an infix at the stem's midpoint, counted in characters.
///
/// Splitting on the byte midpoint would panic on any non-ASCII root, which
/// every phonology with `ä`, `ŋ`, or an IPA inventory produces.
fn insert_infix(word: &str, infix: &str) -> String {
    let char_count = word.chars().count();
    if char_count == 0 {
        return infix.to_string();
    }
    let split = word
        .char_indices()
        .nth(char_count / 2)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(word.len());
    format!("{}{}{}", &word[..split], infix, &word[split..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archetypes::{MorphRule, Morphology};

    fn engine(rules: Vec<MorphRule>) -> MorphologyEngine {
        MorphologyEngine::new(Morphology { rules, noun_classes: None })
    }

    #[test]
    fn test_morphology_rules() {
        let (word, noun_class) =
            engine(vec![MorphRule::Suffix("-en".to_string())]).apply_rules("root");
        assert_eq!(word, "root-en");
        assert_eq!(noun_class, None);
    }

    #[test]
    fn prefix_attaches_to_the_front() {
        let e = engine(vec![MorphRule::Prefix("ti-".to_string())]);
        assert_eq!(e.apply_affixes("kama"), "ti-kama");
    }

    #[test]
    fn rules_compose_in_declaration_order() {
        let e = engine(vec![
            MorphRule::Suffix("-a".to_string()),
            MorphRule::Prefix("ne-".to_string()),
        ]);
        assert_eq!(e.apply_affixes("kot"), "ne-kot-a");
    }

    // ── Infix boundaries ─────────────────────────────────────────────────

    #[test]
    fn infix_lands_at_the_character_midpoint() {
        let e = engine(vec![MorphRule::Infix("-ka-".to_string())]);
        assert_eq!(e.apply_affixes("mata"), "ma-ka-ta");
    }

    #[test]
    fn infix_does_not_panic_on_multibyte_roots() {
        // Byte-midpoint splitting used to panic here.
        let e = engine(vec![MorphRule::Infix("-ka-".to_string())]);
        assert_eq!(e.apply_affixes("t\u{00e4}m\u{00f6}"), "t\u{00e4}-ka-m\u{00f6}");
    }

    #[test]
    fn infix_handles_wide_ipa_symbols() {
        let e = engine(vec![MorphRule::Infix("i".to_string())]);
        assert_eq!(e.apply_affixes("\u{014b}a\u{0283}u"), "\u{014b}ai\u{0283}u");
    }

    #[test]
    fn infix_on_empty_root_is_the_infix() {
        let e = engine(vec![MorphRule::Infix("-ka-".to_string())]);
        assert_eq!(e.apply_affixes(""), "-ka-");
    }

    #[test]
    fn infix_on_single_character_root_prefixes_it() {
        let e = engine(vec![MorphRule::Infix("x".to_string())]);
        assert_eq!(e.apply_affixes("a"), "xa");
    }

    // ── Reduplication and circumfixes ────────────────────────────────────

    #[test]
    fn reduplication_doubles_the_stem() {
        assert_eq!(engine(vec![MorphRule::Reduplication]).apply_affixes("kai"), "kai-kai");
    }

    #[test]
    fn partial_reduplication_copies_a_prefix() {
        let e = engine(vec![MorphRule::PartialReduplication(2)]);
        assert_eq!(e.apply_affixes("taku"), "ta-taku");
    }

    #[test]
    fn partial_reduplication_is_char_safe() {
        let e = engine(vec![MorphRule::PartialReduplication(2)]);
        assert_eq!(e.apply_affixes("\u{00f6}\u{00e4}ku"), "\u{00f6}\u{00e4}-\u{00f6}\u{00e4}ku");
    }

    #[test]
    fn partial_reduplication_of_zero_is_a_no_op() {
        let e = engine(vec![MorphRule::PartialReduplication(0)]);
        assert_eq!(e.apply_affixes("taku"), "taku");
    }

    #[test]
    fn circumfix_wraps_the_stem() {
        let e = engine(vec![MorphRule::Circumfix("ge".to_string(), "t".to_string())]);
        assert_eq!(e.apply_affixes("mach"), "gemacht");
    }

    // ── Noun classes and determinism ─────────────────────────────────────

    #[test]
    fn noun_class_is_drawn_from_the_profile() {
        let morph = Morphology {
            rules: vec![],
            noun_classes: Some(vec!["animate".to_string(), "inanimate".to_string()]),
        };
        let (_, class) = MorphologyEngine::new(morph).apply_rules("kai");
        let class = class.expect("a class should be assigned");
        assert!(class == "animate" || class == "inanimate");
    }

    #[test]
    fn same_seed_assigns_the_same_classes() {
        let morph = Morphology {
            rules: vec![MorphRule::Suffix("-a".to_string())],
            noun_classes: Some(vec!["i".to_string(), "ii".to_string(), "iii".to_string()]),
        };
        let a = MorphologyEngine::seeded(morph.clone(), 21);
        let b = MorphologyEngine::seeded(morph, 21);
        let left: Vec<_> = (0..30).map(|_| a.apply_rules("kot")).collect();
        let right: Vec<_> = (0..30).map(|_| b.apply_rules("kot")).collect();
        assert_eq!(left, right);
    }

    #[test]
    fn every_bundled_morphology_applies_cleanly() {
        for (name, morph) in crate::archetypes::get_morphology_registry() {
            let out = MorphologyEngine::new(morph).apply_affixes("t\u{00e4}k\u{00f6}");
            assert!(!out.is_empty(), "{} produced an empty stem", name);
        }
    }
}
