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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MorphRule {
    Suffix(String),
    Prefix(String),
    Infix(String),
    Reduplication,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Syntax {
    pub word_order: String,
    pub cases: Vec<String>, // e.g., ["nominative", "accusative", "genitive"]
}

pub fn get_phonology_registry() -> HashMap<String, Phonology> {
    serde_json::from_str(r#"{
        "uralic_finnic": {
            "vowels": ["a", "e", "i", "o", "u", "ä", "ö", "y"],
            "consonants": ["p", "t", "k", "s", "h", "m", "n", "r", "l", "v", "j"],
            "syllable_structure": "CVC",
            "vowel_harmony": true
        }
    }"#).unwrap()
}

pub fn get_sound_change_registry() -> HashMap<String, Vec<SoundChange>> {
    serde_json::from_str(r#"{
        "finnic_to_estonian": [
            { "pattern": "k", "replacement": "g", "context": "word_final" },
            { "pattern": "t", "replacement": "d", "context": "word_initial" }
        ]
    }"#).unwrap()
}

pub fn get_morphology_registry() -> HashMap<String, Morphology> {
    serde_json::from_str(r#"{
        "agglutinative": {
            "rules": [{"Suffix": "-en"}, {"Suffix": "-is"}]
        }
    }"#).unwrap()
}

pub fn get_syntax_registry() -> HashMap<String, Syntax> {
    serde_json::from_str(r#"{
        "svo_nom_acc": { 
            "word_order": "SVO",
            "cases": ["nominative", "accusative"]
        }
    }"#).unwrap()
}
