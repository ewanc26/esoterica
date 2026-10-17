//! Syllable and word generation from phonotactic templates.
//! Walks a C/V pattern string, picks random phonemes from the inventory,
//! and optionally applies tone marking and vowel harmony.

use crate::archetypes::Phonology;
use rand::seq::SliceRandom;
use rand::Rng;

/// Vowels that count as front for harmony purposes (romanised input forms).
const FRONT_VOWELS: [&str; 8] = ["i", "e", "y", "ä", "ö", "ü", "æ", "ø"];
/// Vowels that count as back for harmony purposes. `a` and `o` are back only.
const BACK_VOWELS: [&str; 7] = ["a", "o", "u", "ı", "ɨ", "ɯ", "ɑ"];

/// Romanised-to-IPA segment table, matched longest-first by `to_ipa`.
const IPA_MAP: &[(&str, &str)] = &[
    // Digraphs and trigraphs
    ("tsh", "tʃ"), ("ch", "tʃ"), ("dzh", "dʒ"), ("ts", "ts"), ("dz", "dz"),
    ("sh", "ʃ"), ("zh", "ʒ"), ("ng", "ŋ"), ("ny", "ɲ"), ("th", "θ"), ("dh", "ð"),
    ("kh", "x"), ("gh", "ɣ"), ("ph", "pʰ"), ("kw", "kʷ"), ("gw", "gʷ"),
    // Single romanised segments with a different IPA value; anything else passes through
    ("g", "ɡ"), ("r", "ɾ"), ("ä", "æ"), ("ö", "ø"), ("'", "ʼ"),
];

pub struct PhonologyEngine {
    phonology: Phonology,
}

impl PhonologyEngine {
    pub fn new(phonology: Phonology) -> Self {
        Self { phonology }
    }

    /// Generate one syllable by walking the phonotactic template.
    /// C picks a random consonant, V picks a random vowel. A parenthesised group
    /// such as `(C)` is an optional segment, included with probability 0.5.
    /// Appends a tone superscript if the phonology is tonal.
    pub fn generate_syllable(&self) -> String {
        let mut rng = rand::thread_rng();
        let mut syllable = String::new();
        let mut in_optional = false;
        let mut optional_included = true;

        for char in self.phonology.syllable_structure.chars() {
            match char {
                '(' => {
                    in_optional = true;
                    optional_included = rng.gen_bool(0.5);
                }
                ')' => in_optional = false,
                _ if in_optional && !optional_included => {}
                'C' => {
                    if let Some(consonant) = self.phonology.consonants.choose(&mut rng) {
                        syllable.push_str(consonant);
                    }
                }
                'V' => {
                    if let Some(vowel) = self.phonology.vowels.choose(&mut rng) {
                        syllable.push_str(vowel);
                    }
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

    /// Unicode superscript numeral for tone marking (¹ to ⁹). Tones beyond 9 are
    /// rejected by `Phonology::validate`, so the fallback is never reached from data.
    fn tone_to_superscript(tone: u8) -> char {
        match tone {
            1 => '\u{00b9}', 2 => '\u{00b2}', 3 => '\u{00b3}',
            4 => '\u{2074}', 5 => '\u{2075}', 6 => '\u{2076}',
            7 => '\u{2077}', 8 => '\u{2078}', _ => '\u{2079}',
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
            .filter(|v| FRONT_VOWELS.contains(&v.as_str()))
            .cloned().collect();
        let back_vowels: Vec<String> = self.phonology.vowels.iter()
            .filter(|v| BACK_VOWELS.contains(&v.as_str()))
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

    /// Convert a generated word to a rough IPA transcription.
    /// Scans left to right, taking the longest matching romanised segment at each
    /// position, so digraphs (`sh`, `ng`, `kw`) and multi-character inventory
    /// entries are read as single segments. Unmapped text passes through unchanged.
    /// Not phonetically rigorous — good enough for conlang lexicons.
    pub fn to_ipa(&self, word: &str) -> String {
        let mut ipa = String::new();
        let mut rest = word;
        while let Some(c) = rest.chars().next() {
            let best = IPA_MAP.iter()
                .filter(|(from, _)| rest.starts_with(from))
                .max_by_key(|(from, _)| from.len());
            match best {
                Some((from, to)) => {
                    ipa.push_str(to);
                    rest = &rest[from.len()..];
                }
                None => {
                    ipa.push(c);
                    rest = &rest[c.len_utf8()..];
                }
            }
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

    #[test]
    fn test_ipa_keeps_plain_vowels() {
        // Regression: sequential string replacement used to turn every "a" into "æ"
        let engine = PhonologyEngine::new(make_phono());
        assert_eq!(engine.to_ipa("aeiou"), "/aeiou/");
    }

    #[test]
    fn test_ipa_reads_digraphs_as_single_segments() {
        let engine = PhonologyEngine::new(make_phono());
        assert_eq!(engine.to_ipa("shang"), "/\u{0283}a\u{014b}/");
        assert_eq!(engine.to_ipa("gar"), "/\u{0261}a\u{027e}/");
    }

    #[test]
    fn test_tones_up_to_nine_have_distinct_marks() {
        let marks: std::collections::HashSet<char> = (1..=9).map(PhonologyEngine::tone_to_superscript).collect();
        assert_eq!(marks.len(), 9);
        assert!(marks.iter().all(|c| crate::sound_change::is_tone_mark(*c)));
    }

    #[test]
    fn test_optional_segment_is_sometimes_absent() {
        let mut phono = make_phono();
        phono.syllable_structure = "(C)V".to_string();
        let engine = PhonologyEngine::new(phono);
        let lengths: std::collections::HashSet<usize> =
            (0..200).map(|_| engine.generate_syllable().chars().count()).collect();
        assert!(lengths.contains(&1), "optional consonant was never omitted");
        assert!(lengths.contains(&2), "optional consonant was never included");
    }

    #[test]
    fn test_optional_segments_do_not_change_required_template() {
        let mut phono = make_phono();
        phono.syllable_structure = "CCCVCCCC".to_string();
        let engine = PhonologyEngine::new(phono);
        for _ in 0..20 {
            assert_eq!(engine.generate_syllable().chars().count(), 8);
        }
    }

    #[test]
    fn test_vowel_harmony_keeps_word_vowels_in_one_class() {
        let mut phono = make_phono();
        phono.vowels = ["a", "e", "i", "o", "u", "ö"].iter().map(|s| s.to_string()).collect();
        phono.vowel_harmony = Some(true);
        let engine = PhonologyEngine::new(phono);
        for _ in 0..200 {
            let word = engine.generate_word(3);
            let has_front = word.chars().any(|c| matches!(c, 'e' | 'i' | 'ö'));
            let has_back = word.chars().any(|c| matches!(c, 'a' | 'o' | 'u'));
            assert!(!(has_front && has_back), "mixed harmony class in {}", word);
        }
    }
}
