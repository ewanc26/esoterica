use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LexiconEntry {
    pub definitions: Vec<String>,
    pub part_of_speech: String,
    pub domain: String,
    pub examples: Vec<String>,
    pub ipa: String,
    pub root: String,
    pub noun_class: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lexicon(pub HashMap<String, LexiconEntry>);
