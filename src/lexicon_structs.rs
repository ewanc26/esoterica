//! Lexicon data structures for generated conlang dictionaries.
//! Each LexiconEntry stores a word's full lifecycle: root, morphed form,
//! IPA transcription, senses with citations, and noun class assignment.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// A citation — an in-universe reference recording the earliest known usage.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub author: String,
    pub work: String,
    pub date: String,
    pub context: String,
}

/// A single sense (meaning) of a lexicon entry with supporting citations.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Sense {
    pub definition: String,
    pub citations: Vec<Citation>,
}

/// A complete lexicon entry: headword, etymology, phonology, and semantics.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LexiconEntry {
    pub headword: String,
    /// Proto-root and drift history appended over the word's simulated lifetime
    pub etymology: String,
    pub part_of_speech: String,
    /// Approximate IPA transcription
    pub ipa: String,
    /// One or more definitions, each with fictional citations
    pub senses: Vec<Sense>,
    /// Noun class, if the morphology defines classes
    pub noun_class: Option<String>,
    /// The original root before morphological transformation
    pub root: String,
}

/// A map of headword → entry, serialisable as a JSON dictionary.
///
/// Backed by a `BTreeMap` rather than a `HashMap` so entries iterate and
/// serialise in headword order. That is both what a dictionary should look
/// like and what makes a seeded run byte-reproducible.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Lexicon(pub BTreeMap<String, LexiconEntry>);

impl Lexicon {
    /// An empty lexicon.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the lexicon holds no entries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Look up an entry by headword.
    pub fn get(&self, headword: &str) -> Option<&LexiconEntry> {
        self.0.get(headword)
    }

    /// Iterate over (headword, entry) pairs in headword order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &LexiconEntry)> {
        self.0.iter()
    }

    /// Iterate over entries in headword order.
    pub fn values(&self) -> impl Iterator<Item = &LexiconEntry> {
        self.0.values()
    }

    /// Headwords in sorted order.
    pub fn sorted_headwords(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    /// Entries in headword order.
    pub fn sorted_entries(&self) -> Vec<(&String, &LexiconEntry)> {
        self.0.iter().collect()
    }

    /// Insert an entry, returning any entry it displaced.
    pub fn insert(&mut self, entry: LexiconEntry) -> Option<LexiconEntry> {
        self.0.insert(entry.headword.clone(), entry)
    }

    /// Count entries by part of speech, in sorted order.
    pub fn part_of_speech_counts(&self) -> Vec<(String, usize)> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for entry in self.0.values() {
            *counts.entry(entry.part_of_speech.as_str()).or_default() += 1;
        }
        let mut counts: Vec<(String, usize)> =
            counts.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        counts.sort();
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(headword: &str, pos: &str) -> LexiconEntry {
        LexiconEntry {
            headword: headword.to_string(),
            etymology: "test".to_string(),
            part_of_speech: pos.to_string(),
            ipa: format!("/{}/", headword),
            senses: vec![Sense {
                definition: "A test concept".to_string(),
                citations: vec![Citation {
                    author: "Test".to_string(),
                    work: "Test".to_string(),
                    date: "2024".to_string(),
                    context: "test".to_string(),
                }],
            }],
            noun_class: None,
            root: headword.to_string(),
        }
    }

    fn sample() -> Lexicon {
        let mut lexicon = Lexicon::new();
        lexicon.insert(entry("kota", "noun"));
        lexicon.insert(entry("ama", "verb"));
        lexicon.insert(entry("silu", "noun"));
        lexicon
    }

    #[test]
    fn new_lexicon_is_empty() {
        assert!(Lexicon::new().is_empty());
        assert_eq!(Lexicon::new().len(), 0);
    }

    #[test]
    fn insert_and_lookup_round_trip() {
        let lexicon = sample();
        assert_eq!(lexicon.len(), 3);
        assert_eq!(lexicon.get("kota").unwrap().part_of_speech, "noun");
        assert!(lexicon.get("missing").is_none());
    }

    #[test]
    fn insert_returns_the_displaced_entry() {
        let mut lexicon = sample();
        let displaced = lexicon.insert(entry("kota", "verb"));
        assert_eq!(displaced.unwrap().part_of_speech, "noun");
        assert_eq!(lexicon.len(), 3);
    }

    #[test]
    fn sorted_headwords_are_ordered_and_stable() {
        let lexicon = sample();
        assert_eq!(lexicon.sorted_headwords(), vec!["ama", "kota", "silu"]);
        assert_eq!(lexicon.sorted_headwords(), lexicon.sorted_headwords());
    }

    #[test]
    fn sorted_entries_match_sorted_headwords() {
        let lexicon = sample();
        let keys: Vec<&String> = lexicon.sorted_entries().into_iter().map(|(k, _)| k).collect();
        assert_eq!(keys, lexicon.sorted_headwords().iter().collect::<Vec<_>>());
    }

    #[test]
    fn part_of_speech_counts_are_tallied() {
        assert_eq!(
            sample().part_of_speech_counts(),
            vec![("noun".to_string(), 2), ("verb".to_string(), 1)]
        );
    }

    #[test]
    fn serialisation_round_trips() {
        let lexicon = sample();
        let json = serde_json::to_string(&lexicon).unwrap();
        let restored: Lexicon = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.sorted_headwords(), lexicon.sorted_headwords());
        assert_eq!(restored.get("ama"), lexicon.get("ama"));
    }

    #[test]
    fn serialisation_is_in_headword_order() {
        // A `HashMap` backing would emit keys in a per-process random order,
        // so two runs of the same seed would produce different files.
        let json = serde_json::to_string(&sample()).unwrap();
        let ama = json.find("\"ama\"").unwrap();
        let kota = json.find("\"kota\"").unwrap();
        let silu = json.find("\"silu\"").unwrap();
        assert!(ama < kota && kota < silu, "keys not sorted: {}", json);
    }

    #[test]
    fn serialised_shape_is_a_flat_headword_map() {
        let lexicon = sample();
        let value: serde_json::Value = serde_json::to_value(&lexicon).unwrap();
        assert!(value.get("kota").is_some(), "expected a flat map, got {}", value);
    }
}
