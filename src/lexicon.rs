use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;
use crate::archetypes::{Phonology, Morphology, SoundChange};
use crate::lexicon_structs::{Lexicon, LexiconEntry, Sense, Citation};
use std::fs::File;
use std::io::Write;
use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use std::collections::HashMap;

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
            lexicon: Lexicon(HashMap::new()),
        }
    }

    pub fn generate_core_lexicon(&mut self, size: usize) -> &Lexicon {
        let mut rng = rand::thread_rng();
        let pos = ["n.", "v.", "adj."];
        
        for _i in 0..size {
            let root = self.phonology.generate_word(2);
            let (morphed_word, noun_class) = self.morphology.apply_rules(&root);
            let final_word = self.sound_change.apply(&morphed_word);
            
            let p_o_s = pos.choose(&mut rng).unwrap();
            
            let sense1 = Sense {
                definition: "A concept of basic existence.".to_string(),
                citations: vec![Citation {
                    author: "Ancient Bard".to_string(),
                    work: "The Proto-Songs".to_string(),
                    date: "c. 1200".to_string(),
                    context: format!("First recorded use of {}.", final_word),
                }],
            };

            let entry = LexiconEntry {
                headword: final_word.clone(),
                etymology: format!("Derived from proto-root *{}", root),
                part_of_speech: p_o_s.to_string(),
                ipa: self.phonology.to_ipa(&final_word),
                senses: vec![sense1],
                noun_class,
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
