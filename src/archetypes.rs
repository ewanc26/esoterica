//! Linguistic archetype definitions and TOML-backed registries.
//! These types define the shape of every configurable component and are
//! deserialised from the data/ directory at compile time via include_str!.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Phoneme inventory and phonotactic constraints for a language.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Phonology {
    pub vowels: Vec<String>,
    pub consonants: Vec<String>,
    /// Phonotactic template: C=consonant, V=vowel (e.g. "CVC", "CCCVCCCC")
    pub syllable_structure: String,
    /// Number of lexical tones, if tonal
    pub tones: Option<u8>,
    /// Whether vowels harmonise with the first vowel's front/back class
    pub vowel_harmony: Option<bool>,
}

/// A single sound-change rule in legacy TOML format.
/// Parser-based rules (e.g. "p > b / V_V") are handled by FormalRule in sound_change.rs.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SoundChange {
    pub pattern: String,
    pub replacement: String,
    /// Context constraint: "word_initial", "word_final", or None for unconditional
    pub context: Option<String>,
}

/// Morphological profile: affix rules and optional noun classes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Morphology {
    pub rules: Vec<MorphRule>,
    pub noun_classes: Option<Vec<String>>,
}

/// Affix operations applied in order to a root word.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MorphRule {
    Suffix(String),
    Prefix(String),
    /// Inserted at the midpoint of the root
    Infix(String),
    /// Doubles the root with a hyphen separator
    Reduplication,
}

/// Word-order configuration and case system.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Syntax {
    /// One of: SVO, SOV, VSO, VOS, OVS, OSV
    pub word_order: String,
    pub cases: Option<Vec<String>>,
}

// ── TOML Registry Loaders ────────────────────────────────────────────────

/// Load the phonology catalogue from data/phonologies.toml at compile time.
pub fn get_phonology_registry() -> HashMap<String, Phonology> {
    toml::from_str(include_str!("../data/phonologies.toml")).expect("Failed to parse phonologies.toml")
}

/// Load the sound change catalogue from data/sound_changes.toml.
pub fn get_sound_change_registry() -> HashMap<String, Vec<SoundChange>> {
    toml::from_str(include_str!("../data/sound_changes.toml")).expect("Failed to parse sound_changes.toml")
}

/// Load the morphology catalogue from data/morphologies.toml.
pub fn get_morphology_registry() -> HashMap<String, Morphology> {
    toml::from_str(include_str!("../data/morphologies.toml")).expect("Failed to parse morphologies.toml")
}

/// Load the syntax catalogue from data/syntaxes.toml.
pub fn get_syntax_registry() -> HashMap<String, Syntax> {
    toml::from_str(include_str!("../data/syntaxes.toml")).expect("Failed to parse syntaxes.toml")
}
