use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;
use crate::sound_change::SoundChangeEngine;
use crate::archetypes::{Phonology, Morphology, SoundChange};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use anyhow::{Context, Result};

pub struct LexiconGenerator {
    phonology: PhonologyEngine,
    morphology: MorphologyEngine,
    sound_change: SoundChangeEngine,
    lexicon: HashMap<String, String>,
}

impl LexiconGenerator {
    pub fn new(phonology: Phonology, morphology: Morphology, sound_changes: Vec<SoundChange>) -> Self {
        Self {
            phonology: PhonologyEngine::new(phonology),
            morphology: MorphologyEngine::new(morphology),
            sound_change: SoundChangeEngine::new(sound_changes),
            lexicon: HashMap::new(),
        }
    }

    pub fn generate_core_lexicon(&mut self, size: usize) -> &HashMap<String, String> {
        let categories = ["nature", "action", "object", "abstract"];
        
        for i in 0..size {
            let root = self.phonology.generate_word(2);
            let morphed = self.morphology.apply_rules(&root);
            let final_word = self.sound_change.apply(&morphed);
            
            self.lexicon.insert(final_word, format!("{}: definition_{}", categories[i % 4], i));
        }
        &self.lexicon
    }

    pub fn save_to_file(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.lexicon)?;
        let mut file = File::create(filename).context("Failed to create file")?;
        file.write_all(json.as_bytes()).context("Failed to write to file")?;
        Ok(())
    }
}
