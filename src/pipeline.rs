//! Library-level generation pipeline shared by the CLI, the TUI and embedders.
//!
//! `Registries` holds the component catalogues: the built-in data compiled into
//! the binary, optionally layered with user TOML files. `Spec` is one resolved
//! configuration, and `Spec::generate` runs the lexicon and sentence pipeline.

use crate::archetypes::{self, Morphology, Phonology, Syntax};
use crate::lexicon::LexiconGenerator;
use crate::lexicon_structs::Lexicon;
use crate::sound_change::FormalRule;
use crate::syntax::SyntaxEngine;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::Path;

/// Component catalogues, keyed by name.
#[derive(Debug, Clone, Default)]
pub struct Registries {
    pub phonologies: HashMap<String, Phonology>,
    pub morphologies: HashMap<String, Morphology>,
    pub syntaxes: HashMap<String, Syntax>,
    /// Named sound-change sets, each a list of formal rules.
    pub sound_changes: HashMap<String, Vec<String>>,
}

/// File names read from a user data directory.
pub const USER_FILES: [&str; 4] = ["phonologies.toml", "morphologies.toml", "syntaxes.toml", "sound_changes.toml"];

impl Registries {
    /// The catalogues compiled into the binary from `data/`.
    pub fn builtin() -> Self {
        Self {
            phonologies: archetypes::get_phonology_registry(),
            morphologies: archetypes::get_morphology_registry(),
            syntaxes: archetypes::get_syntax_registry(),
            sound_changes: archetypes::get_sound_change_registry(),
        }
    }

    /// Layer user TOML from `dir` over these registries. Each of the four files is
    /// optional. Entries replace built-ins with the same key, and every accepted entry
    /// passes its validator, so a bad user file is reported with its key and file.
    pub fn with_user_dir(mut self, dir: &Path) -> Result<Self, String> {
        if let Some(map) = read_optional::<Phonology>(&dir.join(USER_FILES[0]))? {
            self.phonologies.extend(map);
        }
        if let Some(map) = read_optional::<Morphology>(&dir.join(USER_FILES[1]))? {
            self.morphologies.extend(map);
        }
        if let Some(map) = read_optional::<Syntax>(&dir.join(USER_FILES[2]))? {
            self.syntaxes.extend(map);
        }
        if let Some(map) = read_optional::<Vec<String>>(&dir.join(USER_FILES[3]))? {
            self.sound_changes.extend(map);
        }
        self.validate().map_err(|errors| errors.join("\n"))
    }

    /// Check every entry, collecting all problems rather than stopping at the first.
    pub fn validate(self) -> Result<Self, Vec<String>> {
        let mut errors = Vec::new();
        for (key, p) in &self.phonologies {
            if let Err(e) = p.validate() { errors.push(format!("phonology '{}': {}", key, e)); }
        }
        for (key, m) in &self.morphologies {
            if m.rules.is_empty() { errors.push(format!("morphology '{}': no rules", key)); }
        }
        for (key, s) in &self.syntaxes {
            if let Err(e) = s.validate() { errors.push(format!("syntax '{}': {}", key, e)); }
        }
        for (key, rules) in &self.sound_changes {
            for rule in rules {
                if let Err(e) = FormalRule::parse(rule) {
                    errors.push(format!("sound change '{}': {}", key, e));
                }
            }
        }
        if errors.is_empty() { Ok(self) } else { Err(errors) }
    }

    /// Resolve a configuration from catalogue keys. Unknown keys are reported by
    /// name, with the valid choices listed.
    pub fn spec(&self, phonology: &str, morphology: &str, syntax: &str, sound_changes: &[String]) -> Result<Spec, String> {
        let phonology_cfg = lookup(&self.phonologies, "phonology", phonology)?.clone();
        let morphology_cfg = lookup(&self.morphologies, "morphology", morphology)?.clone();
        let syntax_cfg = lookup(&self.syntaxes, "syntax", syntax)?.clone();
        let mut rules = Vec::new();
        for key in sound_changes {
            rules.extend(lookup(&self.sound_changes, "sound change", key)?.iter().cloned());
        }
        Ok(Spec {
            phonology: phonology_cfg,
            morphology: morphology_cfg,
            syntax: syntax_cfg,
            sound_changes: rules,
            lexicon_size: 100,
            syllables: 2,
        })
    }
}

/// One resolved configuration, ready to generate.
#[derive(Debug, Clone)]
pub struct Spec {
    pub phonology: Phonology,
    pub morphology: Morphology,
    pub syntax: Syntax,
    /// Formal sound-change rules, applied in order.
    pub sound_changes: Vec<String>,
    pub lexicon_size: usize,
    /// Syllables per root word.
    pub syllables: usize,
}

/// The output of one generation run.
#[derive(Debug, Clone)]
pub struct Generated {
    pub lexicon: Lexicon,
    pub example_sentence: String,
}

impl Spec {
    /// Check the resolved configuration before generating.
    pub fn validate(&self) -> Result<(), String> {
        self.phonology.validate()?;
        self.syntax.validate()?;
        for rule in &self.sound_changes {
            FormalRule::parse(rule)?;
        }
        Ok(())
    }

    /// Run the lexicon and example-sentence pipeline.
    pub fn generate(&self) -> Result<Generated, String> {
        self.validate()?;
        let mut generator = LexiconGenerator::new(self.phonology.clone(), self.morphology.clone(), &self.sound_changes)
            .with_syllables(self.syllables);
        let lexicon = generator.generate_core_lexicon(self.lexicon_size).clone();
        let example_sentence = SyntaxEngine::new(self.syntax.clone())
            .generate_sentence(&["word1".to_string(), "word2".to_string(), "word3".to_string()]);
        Ok(Generated { lexicon, example_sentence })
    }
}

fn lookup<'a, T>(map: &'a HashMap<String, T>, kind: &str, key: &str) -> Result<&'a T, String> {
    map.get(key).ok_or_else(|| {
        let mut choices: Vec<&str> = map.keys().map(String::as_str).collect();
        choices.sort();
        format!("unknown {} '{}' (choose from: {})", kind, key, choices.join(", "))
    })
}

fn read_optional<T: DeserializeOwned>(path: &Path) -> Result<Option<HashMap<String, T>>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    toml::from_str(&text).map(Some).map_err(|e| format!("{}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_registries_validate() {
        assert!(Registries::builtin().validate().is_ok());
    }

    #[test]
    fn unknown_key_lists_choices() {
        let err = Registries::builtin().spec("nope", "agglutinative", "svo", &[]).unwrap_err();
        assert!(err.contains("unknown phonology 'nope'"), "{}", err);
        assert!(err.contains("uralic_finnic"), "choices missing: {}", err);
    }

    #[test]
    fn spec_generates_lexicon_and_sentence() {
        let spec = Registries::builtin()
            .spec("uralic_finnic", "agglutinative", "sov", &["lenition".to_string()])
            .unwrap();
        let generated = Spec { lexicon_size: 10, ..spec }.generate().unwrap();
        assert!(!generated.lexicon.0.is_empty());
        assert!(generated.example_sentence.ends_with('.'));
    }

    #[test]
    fn user_directory_overrides_and_extends_builtins() {
        let dir = std::env::temp_dir().join(format!("esoterica-user-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("phonologies.toml"), r#"
[my_custom_lang]
vowels = ["a", "i"]
consonants = ["p", "t", "k"]
syllable_structure = "CV"
"#).unwrap();
        std::fs::write(dir.join("sound_changes.toml"), "my_shift = [\"t > d / V_V\"]\n").unwrap();

        let regs = Registries::builtin().with_user_dir(&dir).unwrap();
        assert!(regs.phonologies.contains_key("my_custom_lang"));
        assert!(regs.sound_changes.contains_key("my_shift"));
        assert!(regs.phonologies.contains_key("uralic_finnic"), "built-ins must remain");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn invalid_user_entry_is_reported_with_its_key() {
        let dir = std::env::temp_dir().join(format!("esoterica-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("phonologies.toml"), r#"
[broken]
vowels = []
consonants = ["p"]
syllable_structure = "CV"
"#).unwrap();
        let err = Registries::builtin().with_user_dir(&dir).unwrap_err();
        assert!(err.contains("phonology 'broken'"), "{}", err);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn invalid_user_sound_change_is_rejected() {
        let dir = std::env::temp_dir().join(format!("esoterica-rule-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("sound_changes.toml"), "bad = [\"p > b / V_V extra\"]\n").unwrap();
        let err = Registries::builtin().with_user_dir(&dir).unwrap_err();
        assert!(err.contains("sound change 'bad'"), "{}", err);
        std::fs::remove_dir_all(&dir).ok();
    }
}
