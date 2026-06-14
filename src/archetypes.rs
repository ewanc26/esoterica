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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArchetypeData {
    pub phonology: Phonology,
    pub morphology: Morphology,
    pub syntax: Syntax,
}

pub fn get_taxonomy() -> HashMap<String, HashMap<String, ArchetypeData>> {
    serde_json::from_str(r#"{
        "africa": {
            "afroasiatic_semitic": {
                "phonology": {
                    "vowels": ["a", "i", "u"],
                    "consonants": ["q", "k", "g", "t", "d", "s", "z", "sh", "h", "m", "n", "r", "l"],
                    "syllable_structure": "CVC"
                },
                "morphology": {"rules": [{"Infix": "a"}], "noun_classes": ["masculine", "feminine"]},
                "syntax": {"word_order": "VSO", "cases": ["nom", "acc", "gen"]}
            },
            "nigercongo_bantu": {
                "phonology": {
                    "vowels": ["a", "e", "i", "o", "u"],
                    "consonants": ["p", "b", "t", "d", "k", "g", "m", "n", "s", "z"],
                    "syllable_structure": "CV"
                },
                "morphology": {"rules": [{"Prefix": "mu-"}], "noun_classes": ["human", "plant", "abstract"]},
                "syntax": {"word_order": "SVO", "cases": ["nom", "acc"]}
            }
        },
        "eurasia": {
            "ie_germanic": {
                "phonology": {
                    "vowels": ["a", "e", "i", "o", "u"],
                    "consonants": ["p", "b", "t", "d", "k", "g", "f", "s", "h", "m", "n", "r", "l"],
                    "syllable_structure": "CCCVCCCC"
                },
                "morphology": {"rules": [{"Suffix": "-en"}], "noun_classes": ["masculine", "feminine", "neuter"]},
                "syntax": {"word_order": "SVO", "cases": ["nom", "acc", "gen", "dat"]}
            },
            "sinotibetan_sinitic": {
                "phonology": {
                    "vowels": ["a", "e", "i", "o", "u"],
                    "consonants": ["p", "t", "k", "s", "h", "m", "n", "ng", "l"],
                    "syllable_structure": "CV",
                    "tones": 4
                },
                "morphology": {"rules": [{"Prefix": "pre-"}], "noun_classes": null},
                "syntax": {"word_order": "SVO", "cases": null}
            }
        }
    }"#).unwrap()
}

// Flattened registries for backward compatibility
pub fn get_phonology_registry() -> HashMap<String, Phonology> {
    get_taxonomy()
        .values()
        .flat_map(|m| m.iter().map(|(k, v)| (k.clone(), v.phonology.clone())))
        .collect()
}

pub fn get_morphology_registry() -> HashMap<String, Morphology> {
    get_taxonomy()
        .values()
        .flat_map(|m| m.iter().map(|(k, v)| (k.clone(), v.morphology.clone())))
        .collect()
}

pub fn get_syntax_registry() -> HashMap<String, Syntax> {
    get_taxonomy()
        .values()
        .flat_map(|m| m.iter().map(|(k, v)| (k.clone(), v.syntax.clone())))
        .collect()
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
