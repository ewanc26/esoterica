use crate::archetypes::{Morphology, MorphRule};

pub struct MorphologyEngine {
    morphology: Morphology,
}

impl MorphologyEngine {
    pub fn new(morphology: Morphology) -> Self {
        Self { morphology }
    }

    pub fn apply_rules(&self, root: &str) -> String {
        let mut word = root.to_string();
        for rule in &self.morphology.rules {
            match rule {
                MorphRule::Suffix(s) => word.push_str(s),
                MorphRule::Prefix(p) => word = format!("{}{}", p, word),
                MorphRule::Infix(i) => {
                    let mid = word.len() / 2;
                    word = format!("{}{}{}", &word[..mid], i, &word[mid..]);
                }
                MorphRule::Reduplication => word = format!("{}-{}", word, word),
            }
        }
        word
    }
}
