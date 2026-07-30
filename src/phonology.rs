//! Syllable and word generation from phonotactic templates.
//! Walks a C/V pattern string, picks phonemes from the inventory, and
//! optionally applies tone marking and vowel harmony.
//!
//! Construct with [`PhonologyEngine::seeded`] for reproducible output, or
//! [`PhonologyEngine::new`] to seed from system entropy. Prefer
//! [`PhonologyEngine::try_new`]/[`PhonologyEngine::try_seeded`], which reject
//! inventories that cannot satisfy their own phonotactic template.

use crate::archetypes::Phonology;
use crate::ipa;
use crate::rng::SharedRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt;

// ── Validation ───────────────────────────────────────────────────────────

/// Why a phonology cannot be used for generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhonologyError {
    /// The template requires a vowel but the inventory has none.
    NoVowels,
    /// The template requires a consonant but the inventory has none.
    NoConsonants,
    /// The template contains no `C` or `V` slots, so it generates nothing.
    EmptyTemplate(String),
    /// A phoneme string is empty and would silently contribute nothing.
    EmptyPhoneme(&'static str),
    /// Tone counts above five have no superscript representation.
    TooManyTones(u8),
}

impl fmt::Display for PhonologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PhonologyError::NoVowels => {
                write!(f, "syllable structure requires vowels but the inventory is empty")
            }
            PhonologyError::NoConsonants => {
                write!(f, "syllable structure requires consonants but the inventory is empty")
            }
            PhonologyError::EmptyTemplate(t) => {
                write!(f, "syllable structure '{}' contains no C or V slots", t)
            }
            PhonologyError::EmptyPhoneme(kind) => {
                write!(f, "the {} inventory contains an empty string", kind)
            }
            PhonologyError::TooManyTones(n) => {
                write!(f, "{} tone levels exceed the supported maximum of 5", n)
            }
        }
    }
}

impl std::error::Error for PhonologyError {}

impl Phonology {
    /// Check that this phonology can actually generate syllables.
    ///
    /// Generation picks uniformly from the inventories, so an empty inventory
    /// paired with a template that needs it is unsatisfiable rather than
    /// merely degenerate.
    pub fn validate(&self) -> Result<(), PhonologyError> {
        let needs_vowel = self.syllable_structure.contains('V');
        let needs_consonant = self.syllable_structure.contains('C');

        if !needs_vowel && !needs_consonant {
            return Err(PhonologyError::EmptyTemplate(self.syllable_structure.clone()));
        }
        if needs_vowel && self.vowels.is_empty() {
            return Err(PhonologyError::NoVowels);
        }
        if needs_consonant && self.consonants.is_empty() {
            return Err(PhonologyError::NoConsonants);
        }
        if self.vowels.iter().any(|v| v.is_empty()) {
            return Err(PhonologyError::EmptyPhoneme("vowel"));
        }
        if self.consonants.iter().any(|c| c.is_empty()) {
            return Err(PhonologyError::EmptyPhoneme("consonant"));
        }
        if let Some(tones) = self.tones {
            if tones > 5 {
                return Err(PhonologyError::TooManyTones(tones));
            }
        }
        Ok(())
    }
}

// ── Vowel Harmony Classes ────────────────────────────────────────────────

/// Back/front counterparts used by harmony systems of the Finnish and
/// Hungarian type. Harmony alternates within a pair; a vowel that appears in
/// no pair is neutral and never alternates.
const HARMONY_PAIRS: &[(&str, &str)] = &[
    ("a", "\u{00e4}"),
    ("a", "\u{00e6}"),
    ("o", "\u{00f6}"),
    ("o", "\u{00f8}"),
    ("o", "\u{0151}"),
    ("u", "y"),
    ("u", "\u{00fc}"),
    ("u", "\u{0171}"),
    ("\u{0251}", "\u{00e6}"),
    ("\u{0254}", "\u{0153}"),
];

/// Which side of a harmony pair a vowel sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HarmonyClass {
    Back,
    Front,
    /// Transparent: appears in words of either class and triggers nothing.
    Neutral,
}

fn harmony_class(vowel: &str) -> HarmonyClass {
    for (back, front) in HARMONY_PAIRS {
        if vowel == *back {
            return HarmonyClass::Back;
        }
        if vowel == *front {
            return HarmonyClass::Front;
        }
    }
    HarmonyClass::Neutral
}

/// Every counterpart of `vowel` on the `target` side of the pair table.
///
/// A vowel can have several (u → y, ü, ű); the caller picks the first one the
/// inventory actually contains.
fn harmony_counterparts(vowel: &str, target: HarmonyClass) -> impl Iterator<Item = &'static str> + '_ {
    HARMONY_PAIRS.iter().filter_map(move |(back, front)| match target {
        HarmonyClass::Back if vowel == *front => Some(*back),
        HarmonyClass::Front if vowel == *back => Some(*front),
        _ => None,
    })
}

/// A run of a word during harmony processing: either an inventory vowel or
/// intervening material copied verbatim.
enum Segment<'a> {
    Vowel(&'a str),
    Other(String),
}

// ── Engine ───────────────────────────────────────────────────────────────

pub struct PhonologyEngine {
    phonology: Phonology,
    rng: SharedRng,
    /// Inventory vowels sorted longest-first, so multigraphs match before
    /// their leading character does.
    vowels_by_length: Vec<String>,
}

impl PhonologyEngine {
    /// Build an engine seeded from system entropy.
    ///
    /// Accepts any phonology; template slots the inventory cannot fill are
    /// skipped rather than panicking. Use [`try_new`](Self::try_new) to reject
    /// those configurations up front instead.
    pub fn new(phonology: Phonology) -> Self {
        Self::with_rng(phonology, SharedRng::from_entropy())
    }

    /// Build a reproducible engine: the same seed and phonology always yield
    /// the same sequence of words.
    pub fn seeded(phonology: Phonology, seed: u64) -> Self {
        Self::with_rng(phonology, SharedRng::from_seed(seed))
    }

    /// Validate the phonology, then build an entropy-seeded engine.
    pub fn try_new(phonology: Phonology) -> Result<Self, PhonologyError> {
        phonology.validate()?;
        Ok(Self::new(phonology))
    }

    /// Validate the phonology, then build a reproducible engine.
    pub fn try_seeded(phonology: Phonology, seed: u64) -> Result<Self, PhonologyError> {
        phonology.validate()?;
        Ok(Self::seeded(phonology, seed))
    }

    fn with_rng(phonology: Phonology, rng: SharedRng) -> Self {
        let mut vowels_by_length = phonology.vowels.clone();
        vowels_by_length.sort_by_key(|v| std::cmp::Reverse(v.chars().count()));
        Self { phonology, rng, vowels_by_length }
    }

    /// The phonology this engine generates from.
    pub fn phonology(&self) -> &Phonology {
        &self.phonology
    }

    // ── Syllables ────────────────────────────────────────────────────────

    /// Generate one syllable by walking the phonotactic template.
    /// `C` picks a consonant, `V` picks a vowel; other characters are ignored.
    /// Appends a tone superscript if the phonology is tonal.
    pub fn generate_syllable(&self) -> String {
        self.rng.with(|rng| self.generate_syllable_with(rng))
    }

    fn generate_syllable_with(&self, rng: &mut StdRng) -> String {
        let mut syllable = String::new();

        for slot in self.phonology.syllable_structure.chars() {
            match slot {
                'C' => {
                    if let Some(consonant) = self.phonology.consonants.choose(rng) {
                        syllable.push_str(consonant);
                    }
                }
                'V' => {
                    if let Some(vowel) = self.phonology.vowels.choose(rng) {
                        syllable.push_str(vowel);
                    }
                }
                _ => {}
            }
        }

        if let Some(tones) = self.phonology.tones {
            if tones > 0 {
                syllable.push_str(&Self::generate_tone(tones, rng));
            }
        }

        syllable
    }

    // ── Tone Generation ──────────────────────────────────────────────────

    /// Generate a tone superscript. 30% chance of a contour (two different
    /// tones) when enough tone levels exist; otherwise a level tone.
    fn generate_tone(num_tones: u8, rng: &mut StdRng) -> String {
        if num_tones >= 3 && rng.gen_bool(0.3) {
            let t1 = rng.gen_range(1..=num_tones);
            let t2 = rng.gen_range(1..=num_tones);
            if t1 != t2 {
                return format!(
                    "{}{}",
                    Self::tone_to_superscript(t1),
                    Self::tone_to_superscript(t2)
                );
            }
        }
        Self::tone_to_superscript(rng.gen_range(1..=num_tones)).to_string()
    }

    /// Unicode superscript numeral for tone marking (¹ ² ³ ⁴ ⁵).
    fn tone_to_superscript(tone: u8) -> char {
        match tone {
            1 => '\u{00b9}',
            2 => '\u{00b2}',
            3 => '\u{00b3}',
            4 => '\u{2074}',
            5 => '\u{2075}',
            _ => '\u{00b3}',
        }
    }

    // ── Word Generation ──────────────────────────────────────────────────

    /// Generate a word from `num_syllables` syllables, applying vowel harmony
    /// afterwards when the phonology declares it.
    pub fn generate_word(&self, num_syllables: usize) -> String {
        let word = self.rng.with(|rng| {
            (0..num_syllables)
                .map(|_| self.generate_syllable_with(rng))
                .collect::<String>()
        });

        if self.phonology.vowel_harmony.unwrap_or(false) {
            self.apply_vowel_harmony(&word)
        } else {
            word
        }
    }

    /// Generate up to `count` distinct words, giving up after a bounded number
    /// of attempts when the inventory cannot supply that many forms.
    ///
    /// Returns however many distinct words it managed to produce, so callers
    /// must not assume the length matches `count`.
    pub fn generate_distinct_words(&self, count: usize, num_syllables: usize) -> Vec<String> {
        let mut seen = std::collections::HashSet::with_capacity(count);
        let mut words = Vec::with_capacity(count);
        let budget = count.saturating_mul(20).saturating_add(64);

        for _ in 0..budget {
            if words.len() >= count {
                break;
            }
            let word = self.generate_word(num_syllables);
            if seen.insert(word.clone()) {
                words.push(word);
            }
        }
        words
    }

    // ── Vowel Harmony ────────────────────────────────────────────────────

    /// Rewrite a word so every alternating vowel matches the front/back class
    /// of the first non-neutral vowel.
    ///
    /// Neutral (transparent) vowels such as Finnish *e* and *i* are left
    /// alone, and a vowel is only rewritten when a counterpart is actually
    /// present in the inventory — a language without `ä` never grows one.
    fn apply_vowel_harmony(&self, word: &str) -> String {
        let segments = self.segment_vowels(word);

        let word_class = segments
            .iter()
            .filter_map(|seg| match seg {
                Segment::Vowel(v) => Some(harmony_class(v)),
                Segment::Other(_) => None,
            })
            .find(|class| *class != HarmonyClass::Neutral);

        let Some(word_class) = word_class else {
            return word.to_string();
        };

        let mut result = String::with_capacity(word.len());
        for segment in &segments {
            match segment {
                Segment::Other(text) => result.push_str(text),
                Segment::Vowel(vowel) => {
                    let class = harmony_class(vowel);
                    if class == HarmonyClass::Neutral || class == word_class {
                        result.push_str(vowel);
                        continue;
                    }
                    let counterpart = harmony_counterparts(vowel, word_class)
                        .find(|c| self.phonology.vowels.iter().any(|v| v == c));
                    result.push_str(counterpart.unwrap_or(vowel));
                }
            }
        }
        result
    }

    /// Split a word into inventory vowels and everything else, matching
    /// multigraph vowels before their first character.
    fn segment_vowels<'a>(&'a self, word: &str) -> Vec<Segment<'a>> {
        let chars: Vec<char> = word.chars().collect();
        let mut segments = Vec::new();
        let mut buffer = String::new();
        let mut i = 0;

        while i < chars.len() {
            let matched = self.vowels_by_length.iter().find_map(|vowel| {
                let len = vowel.chars().count();
                if len == 0 || i + len > chars.len() {
                    return None;
                }
                (chars[i..i + len].iter().collect::<String>() == *vowel)
                    .then_some((vowel.as_str(), len))
            });

            match matched {
                Some((vowel, len)) => {
                    if !buffer.is_empty() {
                        segments.push(Segment::Other(std::mem::take(&mut buffer)));
                    }
                    segments.push(Segment::Vowel(vowel));
                    i += len;
                }
                None => {
                    buffer.push(chars[i]);
                    i += 1;
                }
            }
        }
        if !buffer.is_empty() {
            segments.push(Segment::Other(buffer));
        }
        segments
    }

    // ── Transcription ────────────────────────────────────────────────────

    /// Convert a generated word to its IPA transcription, enclosed in slashes.
    pub fn to_ipa(&self, word: &str) -> String {
        ipa::transcribe_phonemic(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archetypes::Phonology;

    fn make_phono() -> Phonology {
        Phonology {
            vowels: ["a", "e", "i", "o", "u"].iter().map(|s| s.to_string()).collect(),
            consonants: ["p", "t", "k", "m", "n"].iter().map(|s| s.to_string()).collect(),
            syllable_structure: "CV".to_string(),
            tones: None,
            vowel_harmony: None,
        }
    }

    fn harmony_phono() -> Phonology {
        Phonology {
            vowels: ["a", "o", "u", "\u{00e4}", "\u{00f6}", "y", "e", "i"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            consonants: ["p", "t", "k", "s", "m", "n"].iter().map(|s| s.to_string()).collect(),
            syllable_structure: "CV".to_string(),
            tones: None,
            vowel_harmony: Some(true),
        }
    }

    #[test]
    fn test_syllable_generation() {
        let engine = PhonologyEngine::new(make_phono());
        assert_eq!(engine.generate_syllable().chars().count(), 2);
    }

    #[test]
    fn test_word_generation() {
        let engine = PhonologyEngine::new(make_phono());
        assert_eq!(engine.generate_word(3).chars().count(), 6);
    }

    #[test]
    fn test_tone_generation() {
        let mut phono = make_phono();
        phono.tones = Some(4);
        let engine = PhonologyEngine::new(phono);
        let syllable = engine.generate_syllable();
        assert!(syllable.chars().count() >= 3, "expected tone, got: {}", syllable);
    }

    #[test]
    fn test_ipa_transcription() {
        let engine = PhonologyEngine::new(make_phono());
        let ipa = engine.to_ipa("pata");
        assert!(ipa.starts_with('/') && ipa.ends_with('/'));
    }

    // ── Determinism ──────────────────────────────────────────────────────

    #[test]
    fn same_seed_produces_identical_words() {
        let a = PhonologyEngine::seeded(make_phono(), 7);
        let b = PhonologyEngine::seeded(make_phono(), 7);
        let left: Vec<String> = (0..50).map(|_| a.generate_word(3)).collect();
        let right: Vec<String> = (0..50).map(|_| b.generate_word(3)).collect();
        assert_eq!(left, right);
    }

    #[test]
    fn different_seeds_produce_different_words() {
        let a = PhonologyEngine::seeded(make_phono(), 1);
        let b = PhonologyEngine::seeded(make_phono(), 2);
        let left: Vec<String> = (0..50).map(|_| a.generate_word(3)).collect();
        let right: Vec<String> = (0..50).map(|_| b.generate_word(3)).collect();
        assert_ne!(left, right);
    }

    #[test]
    fn seeding_is_stable_for_tonal_phonologies() {
        let mut phono = make_phono();
        phono.tones = Some(5);
        let a = PhonologyEngine::seeded(phono.clone(), 99);
        let b = PhonologyEngine::seeded(phono, 99);
        let left: Vec<String> = (0..20).map(|_| a.generate_word(4)).collect();
        let right: Vec<String> = (0..20).map(|_| b.generate_word(4)).collect();
        assert_eq!(left, right);
    }

    // ── Validation ───────────────────────────────────────────────────────

    #[test]
    fn empty_vowel_inventory_is_rejected() {
        let mut phono = make_phono();
        phono.vowels.clear();
        assert_eq!(phono.validate(), Err(PhonologyError::NoVowels));
        assert!(PhonologyEngine::try_new(phono).is_err());
    }

    #[test]
    fn empty_consonant_inventory_is_rejected() {
        let mut phono = make_phono();
        phono.consonants.clear();
        assert_eq!(phono.validate(), Err(PhonologyError::NoConsonants));
    }

    #[test]
    fn vowel_only_template_tolerates_missing_consonants() {
        let mut phono = make_phono();
        phono.consonants.clear();
        phono.syllable_structure = "V".to_string();
        assert!(phono.validate().is_ok());
    }

    #[test]
    fn template_without_slots_is_rejected() {
        let mut phono = make_phono();
        phono.syllable_structure = "xyz".to_string();
        assert!(matches!(phono.validate(), Err(PhonologyError::EmptyTemplate(_))));
    }

    #[test]
    fn empty_phoneme_string_is_rejected() {
        let mut phono = make_phono();
        phono.vowels.push(String::new());
        assert_eq!(phono.validate(), Err(PhonologyError::EmptyPhoneme("vowel")));
    }

    #[test]
    fn excessive_tone_count_is_rejected() {
        let mut phono = make_phono();
        phono.tones = Some(9);
        assert_eq!(phono.validate(), Err(PhonologyError::TooManyTones(9)));
    }

    #[test]
    fn generation_does_not_panic_on_empty_vowels() {
        // `new` stays permissive, so it must degrade rather than panic.
        let mut phono = make_phono();
        phono.vowels.clear();
        let engine = PhonologyEngine::new(phono);
        assert_eq!(engine.generate_syllable().chars().count(), 1);
    }

    #[test]
    fn every_bundled_phonology_validates() {
        for (name, phono) in crate::archetypes::get_phonology_registry() {
            assert!(phono.validate().is_ok(), "{} is invalid: {:?}", name, phono.validate());
        }
    }

    // ── Vowel Harmony ────────────────────────────────────────────────────

    #[test]
    fn harmony_fronts_following_vowels() {
        let engine = PhonologyEngine::new(harmony_phono());
        assert_eq!(engine.apply_vowel_harmony("k\u{00e4}ta"), "k\u{00e4}t\u{00e4}");
    }

    #[test]
    fn harmony_backs_following_vowels() {
        let engine = PhonologyEngine::new(harmony_phono());
        assert_eq!(engine.apply_vowel_harmony("kat\u{00e4}"), "kata");
    }

    #[test]
    fn neutral_vowels_are_transparent() {
        let engine = PhonologyEngine::new(harmony_phono());
        // e and i belong to no pair, so they survive in a back-vowel word.
        assert_eq!(engine.apply_vowel_harmony("katemi"), "katemi");
    }

    #[test]
    fn harmony_word_class_comes_from_first_non_neutral_vowel() {
        let engine = PhonologyEngine::new(harmony_phono());
        // "e" is neutral, so "ö" sets the word to front and "u" fronts to "y".
        assert_eq!(engine.apply_vowel_harmony("tek\u{00f6}mu"), "tek\u{00f6}my");
    }

    #[test]
    fn harmony_never_invents_absent_vowels() {
        let mut phono = harmony_phono();
        phono.vowels.retain(|v| v != "y");
        let engine = PhonologyEngine::new(phono);
        // No front counterpart of "u" is in the inventory, so it cannot front.
        assert_eq!(engine.apply_vowel_harmony("k\u{00f6}mu"), "k\u{00f6}mu");
    }

    #[test]
    fn all_neutral_word_is_unchanged() {
        let engine = PhonologyEngine::new(harmony_phono());
        assert_eq!(engine.apply_vowel_harmony("meti"), "meti");
    }

    #[test]
    fn generated_harmonic_words_are_internally_consistent() {
        let engine = PhonologyEngine::seeded(harmony_phono(), 12345);
        for _ in 0..200 {
            let word = engine.generate_word(4);
            let classes: Vec<HarmonyClass> = word
                .chars()
                .map(|c| harmony_class(&c.to_string()))
                .filter(|c| *c != HarmonyClass::Neutral)
                .collect();
            assert!(
                classes.windows(2).all(|w| w[0] == w[1]),
                "mixed harmony classes in {}",
                word
            );
        }
    }

    // ── Distinct words ───────────────────────────────────────────────────

    #[test]
    fn distinct_words_have_no_duplicates() {
        let engine = PhonologyEngine::seeded(make_phono(), 3);
        let words = engine.generate_distinct_words(40, 3);
        let unique: std::collections::HashSet<_> = words.iter().collect();
        assert_eq!(unique.len(), words.len());
        assert_eq!(words.len(), 40);
    }

    #[test]
    fn distinct_words_terminate_on_tiny_inventories() {
        let phono = Phonology {
            vowels: vec!["a".to_string()],
            consonants: vec!["t".to_string()],
            syllable_structure: "CV".to_string(),
            tones: None,
            vowel_harmony: None,
        };
        let engine = PhonologyEngine::seeded(phono, 1);
        // Only "ta" exists, so the request cannot be satisfied — but it must return.
        assert_eq!(engine.generate_distinct_words(10, 1), vec!["ta".to_string()]);
    }
}
