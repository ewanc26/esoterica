use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Citation {
    pub author: String,
    pub work: String,
    pub date: String,
    pub context: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sense {
    pub definition: String,
    pub citations: Vec<Citation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LexiconEntry {
    pub headword: String,
    pub etymology: String,
    pub part_of_speech: String,
    pub ipa: String,
    pub senses: Vec<Sense>,
    pub noun_class: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lexicon(pub HashMap<String, LexiconEntry>);
