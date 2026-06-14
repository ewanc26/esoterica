use crate::archetypes::SoundChange;

pub struct SoundChangeEngine {
    rules: Vec<SoundChange>,
}

impl SoundChangeEngine {
    pub fn new(rules: Vec<SoundChange>) -> Self {
        Self { rules }
    }

    pub fn apply(&self, word: &str) -> String {
        let mut result = word.to_string();
        for rule in &self.rules {
            result = self.apply_rule(&result, rule);
        }
        result
    }

    fn apply_rule(&self, word: &str, rule: &SoundChange) -> String {
        let pattern = &rule.pattern;
        let replacement = &rule.replacement;
        
        match &rule.context {
            Some(ctx) if ctx == "word_final" => {
                if word.ends_with(pattern) {
                    let new_end = word.len() - pattern.len();
                    format!("{}{}", &word[..new_end], replacement)
                } else {
                    word.to_string()
                }
            },
            Some(ctx) if ctx == "word_initial" => {
                if word.starts_with(pattern) {
                    format!("{}{}", replacement, &word[pattern.len()..])
                } else {
                    word.to_string()
                }
            },
            _ => word.replace(pattern, replacement),
        }
    }
}
