#[cfg(test)]
mod tests {
    use crate::archetypes::{Phonology, Morphology, MorphRule};
    use crate::phonology::PhonologyEngine;
    use crate::morphology::MorphologyEngine;

    #[test]
    fn test_phonology_generation() {
        let phono = Phonology {
            vowels: vec!["a".to_string()],
            consonants: vec!["b".to_string()],
            syllable_structure: "CVC".to_string(),
            tones: None,
            vowel_harmony: None,
        };
        let engine = PhonologyEngine::new(phono);
        
        let syllable = engine.generate_syllable();
        // CVC -> C (b), V (a), C (b)
        assert_eq!(syllable, "bab");
    }

    #[test]
    fn test_morphology_rules() {
        let morph = Morphology {
            rules: vec![MorphRule::Suffix("-en".to_string())],
            noun_classes: None,
        };
        let engine = MorphologyEngine::new(morph);
        let word = engine.apply_rules("root");
        assert_eq!(word, "root-en");
    }
}
