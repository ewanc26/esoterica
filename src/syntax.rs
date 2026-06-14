use crate::archetypes::Syntax;

pub struct SyntaxEngine {
    syntax: Syntax,
}

impl SyntaxEngine {
    pub fn new(syntax: Syntax) -> Self {
        Self { syntax }
    }

    pub fn generate_sentence(&self, words: &[String]) -> String {
        let nom = "nominative".to_string();
        let acc = "accusative".to_string();
        
        let inflected_words: Vec<String> = words.iter().enumerate().map(|(i, word)| {
            let case = if i == 0 { 
                self.syntax.cases.first().unwrap_or(&nom) 
            } else { 
                self.syntax.cases.get(1).unwrap_or(&acc) 
            };
            format!("{}-{}", word, case)
        }).collect();

        match self.syntax.word_order.as_str() {
            "SVO" => format!("{} {} {}", inflected_words[0], inflected_words[1], inflected_words[2]),
            _ => inflected_words.join(" "),
        }
    }
}
