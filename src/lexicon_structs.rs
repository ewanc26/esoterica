//! Lexicon data structures for generated conlang dictionaries.
//! Each LexiconEntry stores a word's full lifecycle: root, morphed form,
//! IPA transcription, senses with citations, and noun class assignment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A citation — an in-universe reference recording the earliest known usage.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Citation {
    pub author: String,
    pub work: String,
    pub date: String,
    pub context: String,
}

/// A single sense (meaning) of a lexicon entry with supporting citations.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sense {
    pub definition: String,
    pub citations: Vec<Citation>,
}

/// A complete lexicon entry: headword, etymology, phonology, and semantics.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lexicon(pub HashMap<String, LexiconEntry>);
