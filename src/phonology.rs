//! Syllable and word generation from phonotactic templates.
//! Walks a C/V pattern string, picks random phonemes from the inventory,
//! and optionally applies tone marking and vowel harmony.

use crate::archetypes::Phonology;
use rand::seq::SliceRandom;
use rand::Rng;

pub struct PhonologyEngine {
    phonology: Phonology,
}

impl PhonologyEngine {
    pub fn new(phonology: Phonology) -> Self {
        Self { phonology }
    }

    /// Generate one syllable by walking the phonotactic template.
    /// C picks a random consonant, V picks a random vowel; other chars are ignored.
    /// Appends a tone superscript if the phonology is tonal.
    pub fn generate_syllable(&self) -> String {
        let structure = &self.phonology.syllable_structure;
        let mut rng = rand::thread_rng();
        let mut syllable = String::new();

        for char in structure.chars() {
            match char {
                'C' => {
                    if let Some(consonant) = self.phonology.consonants.choose(&mut rng) {
                        syllable.push_str(consonant);
                    }
                }
                'V' => {
                    let vowel = self.phonology.vowels.choose(&mut rng).unwrap();
                    syllable.push_str(vowel);
                }
                _ => {}
            }
        }

        if let Some(tones) = self.phonology.tones {
            if tones > 0 {
                let tone = self.generate_tone(tones);
                syllable.push_str(&tone);
            }
        }

        syllable
    }

    // ── Tone Generation ──────────────────────────────────────────────────────

    /// Generate a tone superscript. 30% chance of a contour (two different tones)
    /// when enough tone levels exist; otherwise a level tone.
    fn generate_tone(&self, num_tones: u8) -> String {
        let mut rng = rand::thread_rng();
        if num_tones >= 3 && rng.gen_bool(0.3) {
            let t1 = rng.gen_range(1..=num_tones);
            let t2 = rng.gen_range(1..=num_tones);
            if t1 != t2 {
                return format!("{}{}", Self::tone_to_superscript(t1), Self::tone_to_superscript(t2));
            }
        }
        let level = rng.gen_range(1..=num_tones);
        Self::tone_to_superscript(level).to_string()
    }

    /// Unicode superscript numeral for tone marking (¹ ² ³ ⁴ ⁵).
    fn tone_to_superscript(tone: u8) -> char {
        match tone {
            1 => '\u{00b9}', 2 => '\u{00b2}', 3 => '\u{00b3}',
            4 => '\u{2074}', 5 => '\u{2075}', _ => '\u{00b3}',
        }
    }

    // ── Word Generation ─────────────────────────────────────────────────────

    /// Generate a word from `num_syllables` syllables, with optional vowel harmony.
    pub fn generate_word(&self, num_syllables: usize) -> String {
        let word: String = (0..num_syllables).map(|_| self.generate_syllable()).collect();
        if self.phonology.vowel_harmony.unwrap_or(false) {
            self.apply_vowel_harmony(&word)
        } else {
            word
        }
    }

    // ── Vowel Harmony ────────────────────────────────────────────────────────

    /// Force all vowels in a word to match the front/back class of the first vowel.
    /// This is a rough heuristic — real vowel harmony systems (Finnish, Hungarian)
    /// have complex rules about which vowels alternate and which are neutral.
    fn apply_vowel_harmony(&self, word: &str) -> String {
        let front_vowels: Vec<String> = self.phonology.vowels.iter()
            .filter(|v| matches!(v.as_str(), "i" | "e" | "a" | "y" | "o" | "æ"))
            .cloned().collect();
        let back_vowels: Vec<String> = self.phonology.vowels.iter()
            .filter(|v| matches!(v.as_str(), "a" | "o" | "u"))
            .cloned().collect();

        if front_vowels.is_empty() || back_vowels.is_empty() {
            return word.to_string();
        }

        let first_vowel_class: Option<bool> = word.chars()
            .find(|c| {
                let s = c.to_string();
                front_vowels.contains(&s) || back_vowels.contains(&s)
            })
            .map(|c| {
                let s = c.to_string();
                front_vowels.contains(&s)
            });

        let first_is_front = match first_vowel_class {
            Some(v) => v,
            None => return word.to_string(),
        };

        let mut rng = rand::thread_rng();
        let result: String = word.chars()
            .map(|c| {
                let s = c.to_string();
                let is_front = front_vowels.contains(&s);
                let is_back = back_vowels.contains(&s);
                if (first_is_front && is_back) || (!first_is_front && is_front) {
                    if first_is_front {
                        front_vowels.choose(&mut rng).unwrap().to_string()
                    } else {
                        back_vowels.choose(&mut rng).unwrap().to_string()
                    }
                } else {
                    s
                }
            })
            .collect();
        result
    }

    /// Convert a generated word to a rough IPA transcription via string substitution.
    /// Not phonetically rigorous — good enough for conlang lexicons.
    pub fn to_ipa(&self, word: &str) -> String {
        let mut ipa = word.to_string();
        let mappings = [
            ("sh", "\u{0283}"), ("ng", "\u{014b}"), ("ch", "t\u{0283}"), ("th", "\u{03b8}"),
            ("p", "p"), ("b", "b"), ("t", "t"), ("d", "d"), ("k", "k"), ("g", "\u{0261}"),
            ("m", "m"), ("n", "n"), ("r", "\u{027e}"), ("l", "l"), ("s", "s"), ("h", "h"),
            ("f", "f"), ("v", "v"), ("j", "j"), ("w", "w"),
            ("a", "a"), ("e", "e"), ("i", "i"), ("o", "o"), ("u", "u"),
            ("a", "æ"), ("o", "ø"), ("y", "y"),
            ("q", "q"), ("z", "z"), ("c", "c"), ("\u{0294}", "\u{0294}"), ("\u{0295}", "\u{0295}"),
        ];
        for (from, to) in mappings {
            ipa = ipa.replace(from, to);
        }
        format!("/{}/", ipa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archetypes::Phonology;

    fn make_phono() -> Phonology {
        Phonology {
            vowels: vec!["a".to_string(), "e".to_string(), "i".to_string(), "o".to_string(), "u".to_string()],
            consonants: vec!["p".to_string(), "t".to_string(), "k".to_string(), "m".to_string(), "n".to_string()],
            syllable_structure: "CV".to_string(),
            tones: None,
            vowel_harmony: None,
        }
    }

    #[test]
    fn test_syllable_generation() {
        let engine = PhonologyEngine::new(make_phono());
        let syllable = engine.generate_syllable();
        assert_eq!(syllable.len(), 2);
    }

    #[test]
    fn test_word_generation() {
        let engine = PhonologyEngine::new(make_phono());
        let word = engine.generate_word(3);
        assert_eq!(word.len(), 6);
    }

    #[test]
    fn test_tone_generation() {
        let mut phono = make_phono();
        phono.tones = Some(4);
        let engine = PhonologyEngine::new(phono);
        let syllable = engine.generate_syllable();
        assert!(syllable.len() >= 3, "Expected tone, got: {}", syllable);
    }

    #[test]
    fn test_vowel_harmony() {
        let mut phono = make_phono();
        phono.vowel_harmony = Some(true);
        let engine = PhonologyEngine::new(phono);
        let word = engine.generate_word(2);
        assert!(!word.is_empty());
    }

    #[test]
    fn test_ipa_transcription() {
        let engine = PhonologyEngine::new(make_phono());
        let ipa = engine.to_ipa("pata");
        assert!(ipa.starts_with("/"));
        assert!(ipa.ends_with("/"));
    }
}
