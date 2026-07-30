//! Sentence generation engine for all six major word orders.
//! Inflects subject and object with case markers and arranges constituents
//! according to the configured syntactic profile.
//!
//! Word order also implies head direction, so a profile that leaves
//! `adposition` or `adjective_order` unset inherits the value that normally
//! co-occurs with its verb–object ordering.

use crate::archetypes::{AdjectiveOrder, Adposition, Syntax, UnknownWordOrder, WordOrder};

pub struct SyntaxEngine {
    syntax: Syntax,
    word_order: WordOrder,
}

impl SyntaxEngine {
    /// Build an engine, falling back to SVO when `word_order` is unrecognised.
    pub fn new(syntax: Syntax) -> Self {
        let word_order = syntax.parsed_word_order().unwrap_or(WordOrder::Svo);
        Self { syntax, word_order }
    }

    /// Build an engine, rejecting an unrecognised `word_order` instead of
    /// silently defaulting to SVO.
    pub fn try_new(syntax: Syntax) -> Result<Self, UnknownWordOrder> {
        let word_order = syntax.parsed_word_order()?;
        Ok(Self { syntax, word_order })
    }

    /// The word order actually in effect, after any fallback.
    pub fn word_order(&self) -> WordOrder {
        self.word_order
    }

    /// Adposition placement, defaulting to the head direction implied by the
    /// word order: verb-final languages take postpositions.
    pub fn adposition(&self) -> Adposition {
        self.syntax.adposition.unwrap_or(if self.word_order.is_object_initial_of_verb() {
            Adposition::Postposition
        } else {
            Adposition::Preposition
        })
    }

    /// Attributive adjective placement, defaulted the same way. This
    /// correlation is weaker than the adposition one, so profiles that care
    /// should set `adjective_order` explicitly.
    pub fn adjective_order(&self) -> AdjectiveOrder {
        self.syntax.adjective_order.unwrap_or(
            if self.word_order.is_object_initial_of_verb() {
                AdjectiveOrder::Prenominal
            } else {
                AdjectiveOrder::Postnominal
            },
        )
    }

    /// Case labels in use, if the language marks case at all.
    fn case_markers(&self) -> Option<&[String]> {
        match self.syntax.cases.as_deref() {
            Some(cases) if !cases.is_empty() => Some(cases),
            _ => None,
        }
    }

    // ── Sentence Generation ──────────────────────────────────────────────

    /// Generate a sentence from a list of words using the configured word
    /// order and case system. The first word is treated as subject, the second
    /// as verb, the third as object; remaining words are appended as modifiers.
    ///
    /// A profile with no cases produces unmarked forms, as an isolating
    /// language would.
    pub fn generate_sentence(&self, words: &[String]) -> String {
        if words.is_empty() {
            return String::new();
        }

        let subject_case = self.case_markers().and_then(|c| c.first());
        let object_case = self.case_markers().and_then(|c| c.get(1));

        let subj = self.inflect(&words[0], subject_case);
        let verb = words.get(1).cloned().unwrap_or_default();
        let obj = words.get(2).map(|w| self.inflect(w, object_case)).unwrap_or_default();

        let mut parts: Vec<String> = self
            .word_order
            .constituents()
            .iter()
            .map(|slot| match slot {
                'S' => subj.clone(),
                'V' => verb.clone(),
                _ => obj.clone(),
            })
            .collect();
        parts.extend(words.iter().skip(3).cloned());

        let sentence = parts
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        capitalise_and_terminate(&sentence)
    }

    /// Order a noun and its attributive adjective according to the profile.
    pub fn generate_noun_phrase(&self, noun: &str, adjective: &str) -> String {
        match self.adjective_order() {
            AdjectiveOrder::Prenominal => format!("{} {}", adjective, noun),
            AdjectiveOrder::Postnominal => format!("{} {}", noun, adjective),
        }
    }

    /// Order an adposition and its complement according to the profile.
    pub fn generate_adpositional_phrase(&self, adposition: &str, complement: &str) -> String {
        match self.adposition() {
            Adposition::Preposition => format!("{} {}", adposition, complement),
            Adposition::Postposition => format!("{} {}", complement, adposition),
        }
    }

    /// A one-line typological summary of the profile.
    pub fn describe(&self) -> String {
        let cases = match self.case_markers() {
            Some(cases) => cases.join("/"),
            None => "no case marking".to_string(),
        };
        format!(
            "{} word order, {}, {}, {} adjectives",
            self.word_order,
            match self.adposition() {
                Adposition::Preposition => "prepositional",
                Adposition::Postposition => "postpositional",
            },
            cases,
            match self.adjective_order() {
                AdjectiveOrder::Prenominal => "prenominal",
                AdjectiveOrder::Postnominal => "postnominal",
            },
        )
    }

    /// Apply a case suffix to a word, or leave it bare when unmarked.
    fn inflect(&self, word: &str, case: Option<&String>) -> String {
        match case {
            Some(case) => format!("{}-{}", word, case),
            None => word.to_string(),
        }
    }
}

/// Capitalise the first character and append a full stop.
fn capitalise_and_terminate(sentence: &str) -> String {
    let mut chars = sentence.chars();
    let capitalised = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    format!("{}.", capitalised)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archetypes::Syntax;

    fn make_syntax(order: &str) -> Syntax {
        Syntax {
            word_order: order.to_string(),
            cases: Some(vec!["NOM".to_string(), "ACC".to_string()]),
            adposition: None,
            adjective_order: None,
        }
    }

    fn sample_words() -> Vec<String> {
        vec!["man".to_string(), "see".to_string(), "bird".to_string()]
    }

    fn positions(sentence: &str) -> (usize, usize, usize) {
        let lower = sentence.to_lowercase();
        (
            lower.find("man-nom").expect("subject"),
            lower.find("see").expect("verb"),
            lower.find("bird-acc").expect("object"),
        )
    }

    #[test]
    fn test_svo_order() {
        let s = SyntaxEngine::new(make_syntax("SVO")).generate_sentence(&sample_words());
        let (subj, verb, obj) = positions(&s);
        assert!(subj < verb && verb < obj, "got: {}", s);
        assert!(s.starts_with("Man-NOM"));
    }

    #[test]
    fn test_sov_order() {
        let s = SyntaxEngine::new(make_syntax("SOV")).generate_sentence(&sample_words());
        let (subj, verb, obj) = positions(&s);
        assert!(subj < obj && obj < verb, "got: {}", s);
    }

    #[test]
    fn test_vso_order() {
        let s = SyntaxEngine::new(make_syntax("VSO")).generate_sentence(&sample_words());
        let (subj, verb, obj) = positions(&s);
        assert!(verb < subj && subj < obj, "got: {}", s);
    }

    #[test]
    fn test_vos_order() {
        let s = SyntaxEngine::new(make_syntax("VOS")).generate_sentence(&sample_words());
        let (subj, verb, obj) = positions(&s);
        assert!(verb < obj && obj < subj, "got: {}", s);
    }

    #[test]
    fn test_ovs_order() {
        let s = SyntaxEngine::new(make_syntax("OVS")).generate_sentence(&sample_words());
        let (subj, verb, obj) = positions(&s);
        assert!(obj < verb && verb < subj, "got: {}", s);
    }

    #[test]
    fn test_osv_order() {
        let s = SyntaxEngine::new(make_syntax("OSV")).generate_sentence(&sample_words());
        let (subj, verb, obj) = positions(&s);
        assert!(obj < subj && subj < verb, "got: {}", s);
    }

    #[test]
    fn test_empty_words() {
        assert_eq!(SyntaxEngine::new(make_syntax("SVO")).generate_sentence(&[]), "");
    }

    #[test]
    fn test_single_word() {
        let s = SyntaxEngine::new(make_syntax("SVO")).generate_sentence(&["fire".to_string()]);
        assert!(s.starts_with("Fire-NOM"));
        assert!(s.ends_with('.'));
    }

    #[test]
    fn modifiers_are_appended_after_the_core_clause() {
        let words = vec![
            "man".to_string(),
            "see".to_string(),
            "bird".to_string(),
            "today".to_string(),
        ];
        let s = SyntaxEngine::new(make_syntax("SOV")).generate_sentence(&words);
        assert!(s.ends_with("today."), "got: {}", s);
    }

    #[test]
    fn multibyte_first_letters_are_capitalised() {
        let s = SyntaxEngine::new(make_syntax("SVO"))
            .generate_sentence(&["\u{00e4}ti".to_string()]);
        assert!(s.starts_with('\u{00c4}'), "got: {}", s);
    }

    // ── Case marking ─────────────────────────────────────────────────────

    #[test]
    fn absent_cases_produce_unmarked_forms() {
        let syntax = Syntax {
            word_order: "SVO".to_string(),
            cases: None,
            adposition: None,
            adjective_order: None,
        };
        let s = SyntaxEngine::new(syntax).generate_sentence(&sample_words());
        assert_eq!(s, "Man see bird.");
    }

    #[test]
    fn empty_case_list_produces_unmarked_forms() {
        let syntax = Syntax {
            word_order: "SVO".to_string(),
            cases: Some(vec![]),
            adposition: None,
            adjective_order: None,
        };
        assert_eq!(
            SyntaxEngine::new(syntax).generate_sentence(&sample_words()),
            "Man see bird."
        );
    }

    #[test]
    fn a_single_case_marks_only_the_subject() {
        let syntax = Syntax {
            word_order: "SVO".to_string(),
            cases: Some(vec!["ERG".to_string()]),
            adposition: None,
            adjective_order: None,
        };
        assert_eq!(
            SyntaxEngine::new(syntax).generate_sentence(&sample_words()),
            "Man-ERG see bird."
        );
    }

    // ── Word order fallback ──────────────────────────────────────────────

    #[test]
    fn unknown_word_order_falls_back_to_svo() {
        let engine = SyntaxEngine::new(make_syntax("XYZ"));
        assert_eq!(engine.word_order(), WordOrder::Svo);
    }

    #[test]
    fn try_new_rejects_unknown_word_orders() {
        assert!(SyntaxEngine::try_new(make_syntax("XYZ")).is_err());
        assert!(SyntaxEngine::try_new(make_syntax("OSV")).is_ok());
    }

    // ── Head direction ───────────────────────────────────────────────────

    #[test]
    fn verb_final_orders_default_to_postpositions() {
        assert_eq!(
            SyntaxEngine::new(make_syntax("SOV")).adposition(),
            Adposition::Postposition
        );
        assert_eq!(
            SyntaxEngine::new(make_syntax("SVO")).adposition(),
            Adposition::Preposition
        );
    }

    #[test]
    fn explicit_typology_overrides_the_default() {
        let mut syntax = make_syntax("SOV");
        syntax.adposition = Some(Adposition::Preposition);
        syntax.adjective_order = Some(AdjectiveOrder::Postnominal);
        let engine = SyntaxEngine::new(syntax);
        assert_eq!(engine.adposition(), Adposition::Preposition);
        assert_eq!(engine.adjective_order(), AdjectiveOrder::Postnominal);
    }

    #[test]
    fn noun_phrases_follow_adjective_order() {
        let sov = SyntaxEngine::new(make_syntax("SOV"));
        assert_eq!(sov.generate_noun_phrase("kota", "suri"), "suri kota");
        let vso = SyntaxEngine::new(make_syntax("VSO"));
        assert_eq!(vso.generate_noun_phrase("kota", "suri"), "kota suri");
    }

    #[test]
    fn adpositional_phrases_follow_head_direction() {
        let sov = SyntaxEngine::new(make_syntax("SOV"));
        assert_eq!(sov.generate_adpositional_phrase("nin", "kota"), "kota nin");
        let svo = SyntaxEngine::new(make_syntax("SVO"));
        assert_eq!(svo.generate_adpositional_phrase("nin", "kota"), "nin kota");
    }

    #[test]
    fn description_mentions_the_word_order() {
        let description = SyntaxEngine::new(make_syntax("VOS")).describe();
        assert!(description.contains("VOS"), "got: {}", description);
        assert!(description.contains("prepositional"), "got: {}", description);
    }

    #[test]
    fn every_bundled_syntax_preset_builds_strictly() {
        for (name, syntax) in crate::archetypes::get_syntax_registry() {
            assert!(SyntaxEngine::try_new(syntax).is_ok(), "preset '{}' is invalid", name);
        }
    }
}
