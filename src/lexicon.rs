use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;
use crate::archetypes::{Phonology, Morphology, SoundChange};
use crate::lexicon_structs::{Lexicon, LexiconEntry};
use std::fs::File;
use std::io::Write;
use anyhow::{Context, Result};
use rand::seq::SliceRandom;

pub struct LexiconGenerator {
    phonology: PhonologyEngine,
    morphology: MorphologyEngine,
    sound_change: crate::sound_change::SoundChangeEngine,
    lexicon: Lexicon,
}

impl LexiconGenerator {
    pub fn new(phonology: Phonology, morphology: Morphology, sound_changes: Vec<SoundChange>) -> Self {
        Self {
            phonology: PhonologyEngine::new(phonology),
            morphology: MorphologyEngine::new(morphology),
            sound_change: crate::sound_change::SoundChangeEngine::new(sound_changes),
            lexicon: Lexicon(std::collections::HashMap::new()),
        }
    }

    pub fn generate_core_lexicon(&mut self, size: usize) -> &Lexicon {
        let mut rng = rand::thread_rng();
        let domains = ["nature", "action", "object", "abstract"];
        let pos = ["noun", "verb", "adjective"];
        
        for _i in 0..size {
            let root = self.phonology.generate_word(2);
            let morphed = self.morphology.apply_rules(&root);
            let final_word = self.sound_change.apply(&morphed);
            
            let domain = domains.choose(&mut rng).unwrap();
            let p_o_s = pos.choose(&mut rng).unwrap();
            
            let entry = LexiconEntry {
                definition: format!("Refers to a concept in the {} domain.", domain),
                part_of_speech: p_o_s.to_string(),
                domain: domain.to_string(),
                examples: vec![format!("This {} is interesting.", final_word)],
                root: root.clone(),
            };
            
            self.lexicon.0.insert(final_word, entry);
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
