//! Core lexicon generator. Wires together phonology, morphology, and sound
//! change engines to produce a complete dictionary of fictional words with
//! in-universe citations and semantic domains.
//!
//! Definitions and citations are generated fictional boilerplate drawn from a
//! fixed template bank — they are flavour text for a conlang, not researched
//! linguistic data.
//!
//! Entries are keyed by headword, so the generator retries collisions instead
//! of overwriting: a request for 500 entries yields 500 distinct headwords
//! whenever the phonology can supply that many.

use crate::archetypes::{Morphology, Phonology, SoundChange};
use crate::lexicon_structs::{Citation, Lexicon, LexiconEntry, Sense};
use crate::morphology::MorphologyEngine;
use crate::phonology::{PhonologyEngine, PhonologyError};
use crate::rng::SharedRng;
use anyhow::{Context, Result};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Write;

/// How many generation attempts each requested entry is allowed before the
/// generator concludes the phonology cannot supply more distinct headwords.
const ATTEMPTS_PER_ENTRY: usize = 24;

/// Orchestrates word generation across the pipeline: root → morphology → sound change.
pub struct LexiconGenerator {
    phonology: PhonologyEngine,
    morphology: MorphologyEngine,
    sound_change: crate::sound_change::SoundChangeEngine,
    lexicon: Lexicon,
    syllables_per_word: usize,
    rng: SharedRng,
}

impl LexiconGenerator {
    /// Build a generator seeded from system entropy.
    pub fn new(phonology: Phonology, morphology: Morphology, sound_changes: Vec<SoundChange>) -> Self {
        Self::build(phonology, morphology, sound_changes, None)
    }

    /// Build a fully reproducible generator.
    ///
    /// Every sub-engine is derived from `seed`, so the same seed and the same
    /// configuration produce a byte-identical lexicon.
    pub fn seeded(
        phonology: Phonology,
        morphology: Morphology,
        sound_changes: Vec<SoundChange>,
        seed: u64,
    ) -> Self {
        Self::build(phonology, morphology, sound_changes, Some(seed))
    }

    /// Validate the phonology, then build a generator with the given optional seed.
    pub fn try_new(
        phonology: Phonology,
        morphology: Morphology,
        sound_changes: Vec<SoundChange>,
        seed: Option<u64>,
    ) -> Result<Self, PhonologyError> {
        phonology.validate()?;
        Ok(Self::build(phonology, morphology, sound_changes, seed))
    }

    fn build(
        phonology: Phonology,
        morphology: Morphology,
        sound_changes: Vec<SoundChange>,
        seed: Option<u64>,
    ) -> Self {
        // Offset the per-engine seeds so the sub-engines do not share a stream.
        let (phono_engine, morph_engine, rng) = match seed {
            Some(seed) => (
                PhonologyEngine::seeded(phonology, seed),
                MorphologyEngine::seeded(morphology, seed.wrapping_add(0x9E37_79B9)),
                SharedRng::from_seed(seed.wrapping_add(0x7F4A_7C15)),
            ),
            None => (
                PhonologyEngine::new(phonology),
                MorphologyEngine::new(morphology),
                SharedRng::from_entropy(),
            ),
        };

        Self {
            phonology: phono_engine,
            morphology: morph_engine,
            sound_change: crate::sound_change::SoundChangeEngine::new(sound_changes),
            lexicon: Lexicon(BTreeMap::new()),
            syllables_per_word: 2,
            rng,
        }
    }

    pub fn with_syllables(mut self, count: usize) -> Self {
        self.syllables_per_word = count.max(1);
        self
    }

    /// The lexicon built so far.
    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    // ── Lexicon Generation ──────────────────────────────────────────────────

    /// Generate `size` lexicon entries by cycling through semantic domains and
    /// parts of speech, assigning each word in-universe citations and definitions.
    ///
    /// Headwords are unique. If the phonology cannot supply `size` distinct
    /// forms the generator stops early rather than looping forever; check
    /// [`Lexicon::len`] for what was actually produced.
    pub fn generate_core_lexicon(&mut self, size: usize) -> &Lexicon {
        let defs = definition_bank();
        let budget = size.saturating_mul(ATTEMPTS_PER_ENTRY).saturating_add(64);
        let mut attempts = 0;

        while self.lexicon.0.len() < size && attempts < budget {
            attempts += 1;
            let (headword, entry) = self.generate_entry(&defs);
            // Retry on collision instead of overwriting an existing entry.
            self.lexicon.0.entry(headword).or_insert(entry);
        }

        &self.lexicon
    }

    /// Build one entry: root, affixation, sound change, then semantics.
    fn generate_entry(
        &self,
        defs: &HashMap<(&'static str, &'static str), Vec<&'static str>>,
    ) -> (String, LexiconEntry) {
        let syl_count = self.rng.with(|rng| {
            if rng.gen_bool(0.3) {
                self.syllables_per_word + rng.gen_range(0..=1)
            } else {
                self.syllables_per_word
            }
            .max(1)
        });

        let root = self.phonology.generate_word(syl_count);
        let (morphed_word, noun_class) = self.morphology.apply_rules(&root);
        let final_word = self.sound_change.apply(&morphed_word);

        let (part_of_speech, senses) =
            self.rng.with(|rng| self.generate_semantics(&final_word, defs, rng));

        let entry = LexiconEntry {
            headword: final_word.clone(),
            etymology: format!("Derived from proto-root *{}", root),
            part_of_speech,
            ipa: self.phonology.to_ipa(&final_word),
            senses,
            root,
            noun_class,
        };
        (final_word, entry)
    }

    /// Choose a semantic domain and part of speech, then attach senses with
    /// fictional citations.
    fn generate_semantics(
        &self,
        headword: &str,
        defs: &HashMap<(&'static str, &'static str), Vec<&'static str>>,
        rng: &mut StdRng,
    ) -> (String, Vec<Sense>) {
        let domain = *DOMAINS.choose(rng).expect("DOMAINS is non-empty");
        let part_of_speech = *PARTS_OF_SPEECH.choose(rng).expect("PARTS_OF_SPEECH is non-empty");

        let fallback = vec![GENERIC_DEFINITION];
        let definitions = defs
            .get(&(domain, part_of_speech))
            .or_else(|| defs.get(&(domain, "noun")))
            .unwrap_or(&fallback);

        let num_senses = rng.gen_range(1..=2.min(definitions.len()).max(1));
        let selected: Vec<&&str> = definitions.choose_multiple(rng, num_senses).collect();

        let senses = selected
            .iter()
            .map(|definition| {
                let source = CITATION_SOURCES.choose(rng).expect("CITATION_SOURCES is non-empty");
                Sense {
                    definition: (**definition).to_string(),
                    citations: vec![Citation {
                        author: source.0.to_string(),
                        work: source.1.to_string(),
                        date: source.2.to_string(),
                        context: citation_context(headword, rng),
                    }],
                }
            })
            .collect();

        (part_of_speech.to_string(), senses)
    }

    // ── Serialisation ───────────────────────────────────────────────────────

    /// Serialise the lexicon to a JSON file.
    pub fn save_to_file(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.lexicon)?;
        let mut file = File::create(filename).context("Failed to create file")?;
        file.write_all(json.as_bytes()).context("Failed to write to file")?;
        Ok(())
    }
}

// ── Flavour Text Banks ──────────────────────────────────────────────────────

const DOMAINS: &[&str] = &[
    "nature", "action", "object", "abstract", "body", "food", "tool", "emotion", "social", "time",
    "space", "kinship", "speech", "motion", "quality",
];

const PARTS_OF_SPEECH: &[&str] = &["noun", "verb", "adjective", "adverb"];

const GENERIC_DEFINITION: &str = "A general concept of the language";

const CITATION_SOURCES: &[(&str, &str, &str)] = &[
    ("Ancient Bard", "The Proto-Songs", "c. 1200"),
    ("Elder Scribe", "The First Codex", "c. 800"),
    ("Wandering Poet", "Tales of the Ancestors", "c. 1500"),
    ("Court Linguist", "Royal Dictionary", "c. 1700"),
    ("Temple Archivist", "Inventory of Offerings", "c. 950"),
    ("Border Merchant", "Ledger of the Long Road", "c. 1620"),
    ("Anonymous", "Marginalia of the Grey Psalter", "c. 1340"),
];

/// A fictional attestation note for a citation.
fn citation_context(headword: &str, rng: &mut StdRng) -> String {
    let options = [
        format!("First recorded use of {}.", headword),
        format!("In the dialect of the mountain peoples: {}", headword),
        format!("Glossed in the margin beside {}.", headword),
        "A variant form appears in coastal settlements.".to_string(),
        "Cited in ceremonial contexts throughout the region.".to_string(),
        "Attested only in legal formulae after this period.".to_string(),
    ];
    options.choose(rng).expect("options is non-empty").clone()
}

/// Definition templates keyed by (semantic domain, part of speech).
fn definition_bank() -> HashMap<(&'static str, &'static str), Vec<&'static str>> {
    HashMap::from([
        (("nature", "noun"), vec!["A natural force such as wind or water", "A living entity found in the wild", "A celestial body visible in the sky", "A geological formation shaped by time", "A weather phenomenon bringing change"]),
        (("nature", "verb"), vec!["To flow like water over stone", "To grow as plants reach toward light", "To weather and change under the elements", "To bloom or flourish in season"]),
        (("nature", "adjective"), vec!["Wild and untamed by civilization", "Growing abundantly without cultivation", "Ancient as the mountains themselves"]),
        (("nature", "adverb"), vec!["As the seasons turn", "In the manner of running water"]),
        (("action", "noun"), vec!["A swift movement through space", "An act of creation or making", "A forceful strike or impact", "A journey undertaken with purpose"]),
        (("action", "verb"), vec!["To move swiftly toward a goal", "To create something from raw materials", "To strike with precision and intent", "To carry or transport across distance", "To build up over time through effort"]),
        (("action", "adjective"), vec!["Moving quickly and decisively", "Full of energy and purpose"]),
        (("action", "adverb"), vec!["Swiftly and without hesitation", "Deliberately, with measured force"]),
        (("object", "noun"), vec!["A portable tool for everyday use", "A container for holding precious things", "A weapon forged for protection", "A crafted item of practical design", "A vessel used in ritual or ceremony"]),
        (("object", "verb"), vec!["To fashion into a usable form", "To set aside for later use"]),
        (("object", "adjective"), vec!["Solid and reliable in construction", "Beautifully shaped by skilled hands", "Heavy with history and significance"]),
        (("abstract", "noun"), vec!["A concept relating to mind and thought", "A hidden truth waiting to be discovered", "The essence of what makes something real", "A quality that transcends the physical"]),
        (("abstract", "verb"), vec!["To consider at length without acting", "To hold in the mind as a possibility"]),
        (("abstract", "adjective"), vec!["Related to the realm of thought", "Complex and difficult to grasp", "Ethereal, beyond ordinary perception", "Fundamental to understanding existence"]),
        (("abstract", "adverb"), vec!["In principle, if not in practice", "Considered in itself, apart from use"]),
        (("body", "noun"), vec!["A part of the living form", "An organ essential for life", "A limb that enables movement", "The surface that protects within"]),
        (("body", "verb"), vec!["To feel with the senses", "To heal and restore wholeness", "To grow strong through use"]),
        (("body", "adjective"), vec!["Belonging to the living form", "Worn or weakened by long labour"]),
        (("food", "noun"), vec!["Sustenance gathered from the earth", "A prepared dish for sharing", "A sweet fruit of the season", "Nourishment that brings strength"]),
        (("food", "verb"), vec!["To prepare with fire and care", "To gather from field and forest", "To share in a communal feast"]),
        (("food", "adjective"), vec!["Ripe and ready for the table", "Preserved against the lean season"]),
        (("tool", "noun"), vec!["An implement for shaping wood or stone", "A device for measuring and marking", "A sharp edge for cutting cleanly", "A binding that holds things together"]),
        (("tool", "verb"), vec!["To work a material with an implement", "To sharpen or set an edge"]),
        (("tool", "adjective"), vec!["Well-balanced in the hand", "Worn smooth by generations of use"]),
        (("emotion", "noun"), vec!["A deep feeling that stirs the heart", "Joy that overflows like water", "Sorrow that settles like stone", "A fierce passion that drives action"]),
        (("emotion", "verb"), vec!["To be moved beyond speech", "To bear a feeling in silence"]),
        (("emotion", "adjective"), vec!["Filled with overwhelming feeling", "Calm and at peace within", "Burning with inner fire"]),
        (("emotion", "adverb"), vec!["With great feeling and sincerity", "Passionately, without reservation"]),
        (("social", "noun"), vec!["A bond between kindred spirits", "A gathering of the community", "A leader who guides the people", "A promise made between allies"]),
        (("social", "verb"), vec!["To speak truth before witnesses", "To join together in common purpose", "To lead with wisdom and courage"]),
        (("social", "adjective"), vec!["Bound by oath and obligation", "Welcomed into the common hearth"]),
        (("social", "adverb"), vec!["In the sight of the assembled people", "As custom requires"]),
        (("time", "noun"), vec!["A cycle of the seasons", "A moment that changes everything", "The endless flow of days and nights", "An era remembered in stories"]),
        (("time", "verb"), vec!["To wait for the proper season", "To endure beyond an expected span"]),
        (("time", "adjective"), vec!["Enduring through the ages", "Brief as a heartbeat", "Returning in eternal cycles"]),
        (("time", "adverb"), vec!["At the break of dawn", "When the stars align", "In the fullness of time"]),
        (("space", "noun"), vec!["A vast expanse without boundary", "The place where earth meets sky", "A hidden hollow beneath the ground", "The highest point visible to all"]),
        (("space", "verb"), vec!["To spread outward in all directions", "To enclose within a boundary"]),
        (("space", "adjective"), vec!["Vast beyond measurement", "Enclosed and protected on all sides", "Elevated above the ordinary"]),
        (("space", "adverb"), vec!["Far beyond the settled lands", "Close at hand, within reach"]),
        (("kinship", "noun"), vec!["A relative of the same generation", "The elder from whom a line descends", "A household counted as one hearth", "A child fostered by another family"]),
        (("kinship", "verb"), vec!["To adopt into the family line", "To trace descent through the generations"]),
        (("kinship", "adjective"), vec!["Sharing a common ancestor", "Related through marriage rather than blood"]),
        (("speech", "noun"), vec!["A formal utterance before an audience", "A name given at birth", "A story handed down unchanged", "A word whose meaning has been lost"]),
        (("speech", "verb"), vec!["To recount from memory", "To name a thing for the first time", "To argue a case before elders"]),
        (("speech", "adjective"), vec!["Spoken rather than written", "Fixed in a formula that may not vary"]),
        (("speech", "adverb"), vec!["In the old manner of speaking", "Plainly, without ornament"]),
        (("motion", "noun"), vec!["A departure with no intent to return", "The path taken by migrating herds", "A sudden turning aside"]),
        (("motion", "verb"), vec!["To travel by water", "To climb toward a high place", "To turn back along the same path", "To wander without fixed destination"]),
        (("motion", "adjective"), vec!["Not yet settled in one place", "Following a known and marked route"]),
        (("motion", "adverb"), vec!["Onward, without turning", "Back the way one came"]),
        (("quality", "noun"), vec!["The property that distinguishes one thing from another", "A degree of excellence recognised by others"]),
        (("quality", "adjective"), vec!["Fine in workmanship and material", "Rough but serviceable", "Rare enough to be counted precious", "Plain and unremarkable in every respect"]),
        (("quality", "adverb"), vec!["To a degree beyond the usual", "Barely, and only just"]),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archetypes::MorphRule;

    fn phono() -> Phonology {
        Phonology {
            vowels: ["a", "e", "i", "o", "u"].iter().map(|s| s.to_string()).collect(),
            consonants: ["p", "t", "k", "m", "n", "s", "l", "r"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            syllable_structure: "CVC".to_string(),
            tones: None,
            vowel_harmony: None,
        }
    }

    fn morph() -> Morphology {
        Morphology {
            rules: vec![MorphRule::Suffix("-en".to_string())],
            noun_classes: Some(vec!["animate".to_string(), "inanimate".to_string()]),
        }
    }

    #[test]
    fn generates_the_requested_number_of_entries() {
        let mut gen = LexiconGenerator::seeded(phono(), morph(), vec![], 5);
        let lexicon = gen.generate_core_lexicon(250);
        assert_eq!(lexicon.len(), 250);
    }

    #[test]
    fn headwords_are_unique_and_match_their_keys() {
        let mut gen = LexiconGenerator::seeded(phono(), morph(), vec![], 11);
        let lexicon = gen.generate_core_lexicon(100).clone();
        for (key, entry) in lexicon.iter() {
            assert_eq!(key, &entry.headword);
        }
        assert_eq!(lexicon.len(), 100);
    }

    #[test]
    fn same_seed_produces_an_identical_lexicon() {
        let mut a = LexiconGenerator::seeded(phono(), morph(), vec![], 1234);
        let mut b = LexiconGenerator::seeded(phono(), morph(), vec![], 1234);
        let left = serde_json::to_string(a.generate_core_lexicon(60)).unwrap();
        let right = serde_json::to_string(b.generate_core_lexicon(60)).unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn different_seeds_produce_different_lexicons() {
        let mut a = LexiconGenerator::seeded(phono(), morph(), vec![], 1);
        let mut b = LexiconGenerator::seeded(phono(), morph(), vec![], 2);
        let left: Vec<String> = a.generate_core_lexicon(40).sorted_headwords();
        let right: Vec<String> = b.generate_core_lexicon(40).sorted_headwords();
        assert_ne!(left, right);
    }

    #[test]
    fn a_tiny_phonology_stops_instead_of_looping_forever() {
        let tiny = Phonology {
            vowels: vec!["a".to_string()],
            consonants: vec!["t".to_string()],
            syllable_structure: "CV".to_string(),
            tones: None,
            vowel_harmony: None,
        };
        let plain = Morphology { rules: vec![], noun_classes: None };
        let mut gen = LexiconGenerator::seeded(tiny, plain, vec![], 3).with_syllables(1);
        // "ta" and "tata" are the only reachable forms.
        let lexicon = gen.generate_core_lexicon(500);
        assert!(lexicon.len() <= 2, "unexpectedly produced {} entries", lexicon.len());
        assert!(!lexicon.is_empty());
    }

    #[test]
    fn entries_carry_ipa_and_etymology() {
        let mut gen = LexiconGenerator::seeded(phono(), morph(), vec![], 9);
        for entry in gen.generate_core_lexicon(20).values() {
            assert!(entry.ipa.starts_with('/') && entry.ipa.ends_with('/'));
            assert!(entry.etymology.contains("proto-root"));
            assert!(!entry.senses.is_empty());
            assert!(entry.senses.iter().all(|s| !s.citations.is_empty()));
            assert!(entry.noun_class.is_some());
        }
    }

    #[test]
    fn syllable_count_is_clamped_to_at_least_one() {
        let mut gen = LexiconGenerator::seeded(phono(), morph(), vec![], 4).with_syllables(0);
        assert!(!gen.generate_core_lexicon(5).is_empty());
    }

    #[test]
    fn zero_size_yields_an_empty_lexicon() {
        let mut gen = LexiconGenerator::seeded(phono(), morph(), vec![], 4);
        assert!(gen.generate_core_lexicon(0).is_empty());
    }

    #[test]
    fn validation_rejects_an_unusable_phonology() {
        let mut broken = phono();
        broken.vowels.clear();
        assert!(LexiconGenerator::try_new(broken, morph(), vec![], Some(1)).is_err());
    }

    #[test]
    fn multibyte_inventories_with_infixes_do_not_panic() {
        let mut unicode_phono = phono();
        unicode_phono.vowels = ["\u{00e4}", "\u{00f6}", "y"].iter().map(|s| s.to_string()).collect();
        unicode_phono.consonants = ["\u{014b}", "\u{0283}", "t"].iter().map(|s| s.to_string()).collect();
        let infixing = Morphology {
            rules: vec![MorphRule::Infix("-ka-".to_string())],
            noun_classes: None,
        };
        let mut gen = LexiconGenerator::seeded(unicode_phono, infixing, vec![], 77);
        assert!(!gen.generate_core_lexicon(50).is_empty());
    }
}
