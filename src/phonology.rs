use crate::archetypes::{Phonology};
use rand::seq::SliceRandom;

pub struct PhonologyEngine {
    phonology: Phonology,
}

impl PhonologyEngine {
    pub fn new(phonology: Phonology) -> Self {
        Self { phonology }
    }

    pub fn generate_syllable(&self) -> String {
        let structure = &self.phonology.syllable_structure;
        let mut rng = rand::thread_rng();
        let mut syllable = String::new();
        
        // Simple phonotactics parser: C(C)V(C)
        // We iterate through every character now to support CVC directly
        for char in structure.chars() {
            match char {
                'C' => {
                    if let Some(consonant) = self.phonology.consonants.choose(&mut rng) {
                        syllable.push_str(consonant);
                    }
                }
                'V' => {
                    let vowel = self.phonology.vowels.choose(&mut rng).unwrap();
                    syllable.push_str(vowel);
                }
                _ => {}
            }
        }
        
        syllable
    }

    pub fn generate_word(&self, num_syllables: usize) -> String {
        (0..num_syllables).map(|_| self.generate_syllable()).collect()
    }
}
