use crate::archetypes::Syntax;

pub struct SyntaxEngine {
    syntax: Syntax,
}

impl SyntaxEngine {
    pub fn new(syntax: Syntax) -> Self {
        Self { syntax }
    }

    pub fn generate_sentence(&self, words: &[String]) -> String {
        match self.syntax.word_order.as_str() {
            "SVO" => format!("{} {} {}", words[0], words[1], words[2]),
            "VSO" => format!("{} {} {}", words[1], words[0], words[2]),
            "SOV" => format!("{} {} {}", words[0], words[2], words[1]),
            _ => words.join(" "),
        }
    }
}
