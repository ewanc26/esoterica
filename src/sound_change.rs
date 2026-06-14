use crate::archetypes::SoundChange;

pub struct SoundChangeEngine {
    rules: Vec<SoundChange>,
    vowels: Vec<String>,
}

impl SoundChangeEngine {
    pub fn new(rules: Vec<SoundChange>, vowels: Vec<String>) -> Self {
        Self { rules, vowels }
    }

    pub fn apply(&self, word: &str) -> String {
        let mut result = word.to_string();
        for rule in &self.rules {
            result = self.apply_rule(&result, rule);
        }
        result
    }

    fn apply_rule(&self, word: &str, rule: &SoundChange) -> String {
        let mut new_word = String::new();
        let chars: Vec<char> = word.chars().collect();
        
        for i in 0..chars.len() {
            let mut matched = false;
            if chars[i].to_string() == rule.pattern {
                if let Some(context) = &rule.context {
                    if context == "V_V" 
                        && i > 0 && i < chars.len() - 1 
                        && self.is_vowel(chars[i-1]) && self.is_vowel(chars[i+1]) {
                        matched = true;
                    }
                } else {
                    matched = true;
                }
            }
            
            if matched {
                new_word.push_str(&rule.replacement);
            } else {
                new_word.push(chars[i]);
            }
        }
        new_word
    }

    fn is_vowel(&self, c: char) -> bool {
        self.vowels.iter().any(|v| v.contains(c))
    }
}
