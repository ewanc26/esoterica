use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Phonology {
    pub vowels: Vec<String>,
    pub consonants: Vec<String>,
    pub syllable_structure: String,
    pub tones: Option<u8>,
    pub vowel_harmony: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SoundChange {
    pub pattern: String,
    pub replacement: String,
    pub context: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Morphology {
    pub rules: Vec<MorphRule>,
    pub noun_classes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MorphRule {
    Suffix(String),
    Prefix(String),
    Infix(String),
    Reduplication,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Syntax {
    pub word_order: String,
    pub cases: Option<Vec<String>>,
}

pub fn get_phonology_registry() -> HashMap<String, Phonology> {
    toml::from_str(include_str!("../data/phonologies.toml")).expect("Failed to parse phonologies.toml")
}

pub fn get_sound_change_registry() -> HashMap<String, Vec<SoundChange>> {
    toml::from_str(include_str!("../data/sound_changes.toml")).expect("Failed to parse sound_changes.toml")
}

pub fn get_morphology_registry() -> HashMap<String, Morphology> {
    toml::from_str(include_str!("../data/morphologies.toml")).expect("Failed to parse morphologies.toml")
}

pub fn get_syntax_registry() -> HashMap<String, Syntax> {
    toml::from_str(include_str!("../data/syntaxes.toml")).expect("Failed to parse syntaxes.toml")
}
