use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;
use crate::archetypes::{Phonology, Morphology, SoundChange};
use crate::lexicon_structs::{Lexicon, LexiconEntry, Sense, Citation};
use std::fs::File;
use std::io::Write;
use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

pub struct LexiconGenerator {
    phonology: PhonologyEngine,
    morphology: MorphologyEngine,
    sound_change: crate::sound_change::SoundChangeEngine,
    lexicon: Lexicon,
    syllables_per_word: usize,
}

impl LexiconGenerator {
    pub fn new(phonology: Phonology, morphology: Morphology, sound_changes: Vec<SoundChange>) -> Self {
        Self {
            phonology: PhonologyEngine::new(phonology),
            morphology: MorphologyEngine::new(morphology),
            sound_change: crate::sound_change::SoundChangeEngine::new(sound_changes),
            lexicon: Lexicon(HashMap::new()),
            syllables_per_word: 2,
        }
    }

    pub fn with_syllables(mut self, count: usize) -> Self {
        self.syllables_per_word = count.max(1);
        self
    }

    pub fn generate_core_lexicon(&mut self, size: usize) -> &Lexicon {
        let mut rng = rand::thread_rng();
        let domains = ["nature", "action", "object", "abstract", "body", "food", "tool", "emotion", "social", "time", "space"];
        let pos = ["noun", "verb", "adjective", "adverb"];

        let defs: HashMap<(&str, &str), Vec<&str>> = HashMap::from([
            (("nature", "noun"), vec!["A natural force such as wind or water", "A living entity found in the wild", "A celestial body visible in the sky", "A geological formation shaped by time", "A weather phenomenon bringing change"]),
            (("nature", "verb"), vec!["To flow like water over stone", "To grow as plants reach toward light", "To weather and change under the elements", "To bloom or flourish in season"]),
            (("nature", "adjective"), vec!["Wild and untamed by civilization", "Growing abundantly without cultivation", "Ancient as the mountains themselves"]),
            (("action", "noun"), vec!["A swift movement through space", "An act of creation or making", "A forceful strike or impact", "A journey undertaken with purpose"]),
            (("action", "verb"), vec!["To move swiftly toward a goal", "To create something from raw materials", "To strike with precision and intent", "To carry or transport across distance", "To build up over time through effort"]),
            (("action", "adjective"), vec!["Moving quickly and decisively", "Full of energy and purpose"]),
            (("object", "noun"), vec!["A portable tool for everyday use", "A container for holding precious things", "A weapon forged for protection", "A crafted item of practical design", "A vessel used in ritual or ceremony"]),
            (("object", "adjective"), vec!["Solid and reliable in construction", "Beautifully shaped by skilled hands", "Heavy with history and significance"]),
            (("abstract", "noun"), vec!["A concept relating to mind and thought", "A hidden truth waiting to be discovered", "The essence of what makes something real", "A quality that transcends the physical"]),
            (("abstract", "adjective"), vec!["Related to the realm of thought", "Complex and difficult to grasp", "Ethereal, beyond ordinary perception", "Fundamental to understanding existence"]),
            (("body", "noun"), vec!["A part of the living form", "An organ essential for life", "A limb that enables movement", "The surface that protects within"]),
            (("body", "verb"), vec!["To feel with the senses", "To heal and restore wholeness", "To grow strong through use"]),
            (("food", "noun"), vec!["Sustenance gathered from the earth", "A prepared dish for sharing", "A sweet fruit of the season", "Nourishment that brings strength"]),
            (("food", "verb"), vec!["To prepare with fire and care", "To gather from field and forest", "To share in a communal feast"]),
            (("tool", "noun"), vec!["An implement for shaping wood or stone", "A device for measuring and marking", "A sharp edge for cutting cleanly", "A binding that holds things together"]),
            (("emotion", "noun"), vec!["A deep feeling that stirs the heart", "Joy that overflows like water", "Sorrow that settles like stone", "A fierce passion that drives action"]),
            (("emotion", "adjective"), vec!["Filled with overwhelming feeling", "Calm and at peace within", "Burning with inner fire"]),
            (("emotion", "adverb"), vec!["With great feeling and sincerity", "Passionately, without reservation"]),
            (("social", "noun"), vec!["A bond between kindred spirits", "A gathering of the community", "A leader who guides the people", "A promise made between allies"]),
            (("social", "verb"), vec!["To speak truth before witnesses", "To join together in common purpose", "To lead with wisdom and courage"]),
            (("time", "noun"), vec!["A cycle of the seasons", "A moment that changes everything", "The endless flow of days and nights", "An era remembered in stories"]),
            (("time", "adjective"), vec!["Enduring through the ages", "Brief as a heartbeat", "Returning in eternal cycles"]),
            (("time", "adverb"), vec!["At the break of dawn", "When the stars align", "In the fullness of time"]),
            (("space", "noun"), vec!["A vast expanse without boundary", "The place where earth meets sky", "A hidden hollow beneath the ground", "The highest point visible to all"]),
            (("space", "adjective"), vec!["Vast beyond measurement", "Enclosed and protected on all sides", "Elevated above the ordinary"]),
        ]);

        for _i in 0..size {
            let syl_count = if rng.gen_bool(0.3) {
                (self.syllables_per_word + rng.gen_range(0..=1)).max(1)
            } else {
                self.syllables_per_word
            };
            let root = self.phonology.generate_word(syl_count.max(1));
            let (morphed_word, noun_class) = self.morphology.apply_rules(&root);
            let final_word = self.sound_change.apply(&morphed_word);

            let domain = domains.choose(&mut rng).unwrap();
            let p_o_s = pos.choose(&mut rng).unwrap();

            let default_defs = vec!["A general concept of the language"];
            let definitions: Vec<String> = defs.get(&(domain, p_o_s))
                .or_else(|| defs.get(&(*domain, "noun")))
                .unwrap_or(&default_defs)
                .iter().map(|s| s.to_string()).collect();

            let num_senses = rng.gen_range(1..=2.min(definitions.len()));
            let selected_defs: Vec<&String> = definitions.choose_multiple(&mut rng, num_senses).collect();

            let citation_sources = [
                ("Ancient Bard", "The Proto-Songs", "c. 1200"),
                ("Elder Scribe", "The First Codex", "c. 800"),
                ("Wandering Poet", "Tales of the Ancestors", "c. 1500"),
                ("Court Linguist", "Royal Dictionary", "c. 1700"),
            ];
            let context_options = [
                format!("First recorded use of {}.", final_word),
                format!("In the dialect of the mountain peoples: {}", final_word),
                "A variant form appears in coastal settlements.".to_string(),
                "Cited in ceremonial contexts throughout the region.".to_string(),
            ];

            let senses: Vec<Sense> = selected_defs.iter().map(|def| {
                let source = citation_sources.choose(&mut rng).unwrap();
                let context = context_options.choose(&mut rng).unwrap().clone();
                Sense {
                    definition: (*def).clone(),
                    citations: vec![Citation {
                        author: source.0.to_string(), work: source.1.to_string(),
                        date: source.2.to_string(), context,
                    }],
                }
            }).collect();

            let entry = LexiconEntry {
                headword: final_word.clone(),
                etymology: format!("Derived from proto-root *{}", root),
                part_of_speech: p_o_s.to_string(),
                ipa: self.phonology.to_ipa(&final_word),
                senses,
                root,
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
