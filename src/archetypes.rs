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
    serde_json::from_str(r#"{
        "afroasiatic": {
            "vowels": ["a", "i", "u"],
            "consonants": ["q", "k", "g", "t", "d", "s", "z", "sh", "h", "m", "n", "r", "l"],
            "syllable_structure": "CVC"
        },
        "sino_tibetan": {
            "vowels": ["a", "e", "i", "o", "u"],
            "consonants": ["p", "t", "k", "s", "h", "m", "n", "ng", "l", "j"],
            "syllable_structure": "CV",
            "tones": 4
        },
        "uralic_finnic": {
            "vowels": ["a", "e", "i", "o", "u", "ä", "ö", "y"],
            "consonants": ["p", "t", "k", "s", "h", "m", "n", "r", "l", "v", "j"],
            "syllable_structure": "CVC",
            "vowel_harmony": true
        },
        "celtic": {
            "vowels": ["a", "e", "i", "o", "u"],
            "consonants": ["p", "b", "t", "d", "k", "g", "m", "n", "r", "l", "s", "h"],
            "syllable_structure": "CVC"
        }
    }"#).unwrap()
}

pub fn get_sound_change_registry() -> HashMap<String, Vec<SoundChange>> {
    serde_json::from_str(r#"{
        "none": [],
        "lenition": [
            { "pattern": "p", "replacement": "b", "context": "word_final" }
        ],
        "finnic_to_estonian": [
            { "pattern": "k", "replacement": "g", "context": "word_final" },
            { "pattern": "t", "replacement": "d", "context": "word_initial" }
        ]
    }"#).unwrap()
}

pub fn get_morphology_registry() -> HashMap<String, Morphology> {
    serde_json::from_str(r#"{
        "agglutinative": {
            "rules": [{"Suffix": "-en"}, {"Suffix": "-is"}],
            "noun_classes": ["animate", "inanimate"]
        },
        "root_and_pattern": {
            "rules": [{"Infix": "a"}],
            "noun_classes": ["masculine", "feminine"]
        },
        "analytic": {
            "rules": [{"Prefix": "pre-"}],
            "noun_classes": null
        }
    }"#).unwrap()
}

pub fn get_syntax_registry() -> HashMap<String, Syntax> {
    serde_json::from_str(r#"{
        "svo": { 
            "word_order": "SVO",
            "cases": ["nominative", "accusative"]
        },
        "vso": { 
            "word_order": "VSO",
            "cases": ["nominative", "ergative"]
        },
        "sov": { 
            "word_order": "SOV",
            "cases": ["nominative", "dative"]
        }
    }"#).unwrap()
}
