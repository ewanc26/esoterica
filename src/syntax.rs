use crate::archetypes::Syntax;

pub struct SyntaxEngine {
    syntax: Syntax,
}

impl SyntaxEngine {
    pub fn new(syntax: Syntax) -> Self {
        Self { syntax }
    }

    pub fn generate_sentence(&self, words: &[String]) -> String {
        if words.is_empty() {
            return String::new();
        }

        let cases = self.syntax.cases.as_ref();
        let nom = cases.and_then(|c| c.first()).cloned().unwrap_or_else(|| "NOM".to_string());
        let acc = cases.and_then(|c| c.get(1)).cloned().unwrap_or_else(|| "ACC".to_string());

        let subj = self.inflect(&words[0], &nom);
        let verb = if words.len() > 1 { words[1].clone() } else { String::new() };
        let obj = if words.len() > 2 { self.inflect(&words[2], &acc) } else { String::new() };
        let modifiers: Vec<String> = words.iter().skip(3).cloned().collect();

        let sentence = match self.syntax.word_order.as_str() {
            "SOV" => {
                let mut parts = vec![subj, obj, verb];
                parts.extend(modifiers);
                parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
            }
            "VSO" => {
                let mut parts = vec![verb, subj, obj];
                parts.extend(modifiers);
                parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
            }
            "VOS" => {
                let mut parts = vec![verb, obj, subj];
                parts.extend(modifiers);
                parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
            }
            "OVS" => {
                let mut parts = vec![obj, verb, subj];
                parts.extend(modifiers);
                parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
            }
            "OSV" => {
                let mut parts = vec![obj, subj, verb];
                parts.extend(modifiers);
                parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
            }
            _ => {
                let mut parts = vec![subj, verb, obj];
                parts.extend(modifiers);
                parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
            }
        };

        let mut chars: Vec<char> = sentence.chars().collect();
        if let Some(first) = chars.first_mut() {
            *first = first.to_uppercase().next().unwrap_or(*first);
        }
        format!("{}.", chars.into_iter().collect::<String>())
    }

    fn inflect(&self, word: &str, case: &str) -> String {
        format!("{}-{}", word, case)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archetypes::Syntax;

    fn make_syntax(order: &str) -> Syntax {
        Syntax { word_order: order.to_string(), cases: Some(vec!["NOM".to_string(), "ACC".to_string()]) }
    }

    fn sample_words() -> Vec<String> {
        vec!["man".to_string(), "see".to_string(), "bird".to_string()]
    }

    #[test]
    fn test_svo_order() {
        let engine = SyntaxEngine::new(make_syntax("SVO"));
        let s = engine.generate_sentence(&sample_words());
        assert!(s.starts_with("Man-NOM"));
        assert!(s.contains("see"));
        assert!(s.contains("bird-ACC"));
    }

    #[test]
    fn test_sov_order() {
        let engine = SyntaxEngine::new(make_syntax("SOV"));
        let s = engine.generate_sentence(&sample_words());
        let bird_pos = s.find("bird-ACC").unwrap();
        let see_pos = s.find("see").unwrap();
        assert!(bird_pos < see_pos, "SOV: object before verb. Got: {}", s);
    }

    #[test]
    fn test_vso_order() {
        let engine = SyntaxEngine::new(make_syntax("VSO"));
        let s = engine.generate_sentence(&sample_words());
        let see_pos = s.to_lowercase().find("see").unwrap();
        let man_pos = s.to_lowercase().find("man-nom").unwrap();
        assert!(see_pos < man_pos, "VSO: verb before subject. Got: {}", s);
    }

    #[test]
    fn test_empty_words() {
        let engine = SyntaxEngine::new(make_syntax("SVO"));
        assert_eq!(engine.generate_sentence(&[]), "");
    }

    #[test]
    fn test_single_word() {
        let engine = SyntaxEngine::new(make_syntax("SVO"));
        let s = engine.generate_sentence(&["fire".to_string()]);
        assert!(s.starts_with("Fire-NOM"));
        assert!(s.ends_with('.'));
    }
}
