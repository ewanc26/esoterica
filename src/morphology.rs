use crate::archetypes::{Morphology, MorphRule};
use rand::seq::SliceRandom;

pub struct MorphologyEngine {
    morphology: Morphology,
}

impl MorphologyEngine {
    pub fn new(morphology: Morphology) -> Self {
        Self { morphology }
    }

    pub fn apply_rules(&self, root: &str) -> (String, Option<String>) {
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
        let noun_class = if let Some(classes) = &self.morphology.noun_classes {
            let mut rng = rand::thread_rng();
            classes.choose(&mut rng).cloned()
        } else {
            None
        };
        (word, noun_class)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archetypes::{Morphology, MorphRule};

    #[test]
    fn test_morphology_rules() {
        let morph = Morphology {
            rules: vec![MorphRule::Suffix("-en".to_string())],
            noun_classes: None,
        };
        let engine = MorphologyEngine::new(morph);
        let (word, noun_class) = engine.apply_rules("root");
        assert_eq!(word, "root-en");
        assert_eq!(noun_class, None);
    }
}
