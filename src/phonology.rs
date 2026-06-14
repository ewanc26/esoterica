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

    pub fn to_ipa(&self, word: &str) -> String {
        let mut ipa = word.to_string();
        
        // Map digraphs/consonants to IPA
        let mappings = [
            ("sh", "ʃ"), ("ng", "ŋ"), ("ch", "tʃ"), ("th", "θ"),
            ("p", "p"), ("b", "b"), ("t", "t"), ("d", "d"), ("k", "k"), ("g", "ɡ"),
            ("m", "m"), ("n", "n"), ("r", "ɾ"), ("l", "l"), ("s", "s"), ("h", "h"),
            ("f", "f"), ("v", "v"), ("j", "j"), ("w", "w"),
            ("a", "a"), ("e", "e"), ("i", "i"), ("o", "o"), ("u", "u"), 
            ("ä", "æ"), ("ö", "ø"), ("y", "y"),
            ("q", "q"), ("z", "z"), ("ç", "ç"), ("ʔ", "ʔ"), ("ʕ", "ʕ"),
        ];

        for (from, to) in mappings {
            ipa = ipa.replace(from, to);
        }
        
        format!("/{}/", ipa)
    }
}
