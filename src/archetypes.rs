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
        "eurasia_ie_germanic": {
            "vowels": ["a", "e", "i", "o", "u"],
            "consonants": ["p", "b", "t", "d", "k", "g", "f", "s", "h", "m", "n", "r", "l"],
            "syllable_structure": "CCCVCCCC"
        },
        "eurasia_ie_romance": {
            "vowels": ["a", "e", "i", "o", "u"],
            "consonants": ["p", "b", "t", "d", "k", "g", "f", "v", "s", "z", "m", "n", "r", "l"],
            "syllable_structure": "CV"
        },
        "africa_afroasiatic_semitic": {
            "vowels": ["a", "i", "u"],
            "consonants": ["q", "k", "g", "t", "d", "s", "z", "sh", "h", "m", "n", "r", "l"],
            "syllable_structure": "CVC"
        },
        "africa_nigercongo_bantu": {
            "vowels": ["a", "e", "i", "o", "u"],
            "consonants": ["p", "b", "t", "d", "k", "g", "m", "n", "s", "z"],
            "syllable_structure": "CV"
        },
        "asia_sinotibetan_sinitic": {
            "vowels": ["a", "e", "i", "o", "u"],
            "consonants": ["p", "t", "k", "s", "h", "m", "n", "ng", "l"],
            "syllable_structure": "CV",
            "tones": 4
        },
        "americas_utoaztecan": {
            "vowels": ["a", "i", "u", "o"],
            "consonants": ["p", "t", "k", "kw", "s", "h", "m", "n", "w", "j"],
            "syllable_structure": "CVC"
        },
        "oceania_austronesian": {
            "vowels": ["a", "i", "u"],
            "consonants": ["p", "t", "k", "s", "h", "m", "n", "ng", "l", "r"],
            "syllable_structure": "CV"
        }
    }"#).unwrap()
}

pub fn get_sound_change_registry() -> HashMap<String, Vec<SoundChange>> {
    serde_json::from_str(r#"{
        "none": [],
        "lenition": [
            { "pattern": "p", "replacement": "b", "context": "word_final" }
        ],
        "spirantization": [
            { "pattern": "k", "replacement": "h", "context": "word_final" }
        ]
    }"#).unwrap()
}

pub fn get_morphology_registry() -> HashMap<String, Morphology> {
    serde_json::from_str(r#"{
        "agglutinative": {
            "rules": [{"Suffix": "-en"}, {"Suffix": "-is"}],
            "noun_classes": ["animate", "inanimate"]
        },
        "fusional": {
            "rules": [{"Suffix": "-a"}, {"Suffix": "-o"}],
            "noun_classes": ["masculine", "feminine", "neuter"]
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
