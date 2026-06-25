//! Probabilistic semantic drift module for simulating how word meanings
//! evolve over time based on cultural and environmental markers.

use crate::lexicon_structs::{Lexicon, LexiconEntry};
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfig {
    pub drift_rate: f64,
    pub time_steps: usize,
    pub cultural_markers: Vec<String>,
    pub seed: Option<u64>,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            drift_rate: 0.15,
            time_steps: 3,
            cultural_markers: vec!["urbanization".to_string(), "trade_contact".to_string(), "technological_shift".to_string()],
            seed: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DriftType {
    Broadening, Narrowing, Amelioration, Pejoration, Metaphor, Metonymy, TabooReplacement,
}

impl DriftType {
    pub fn description(&self) -> &str {
        match self {
            DriftType::Broadening => "Meaning expanded to encompass a wider range of referents",
            DriftType::Narrowing => "Meaning contracted to a more specific referent",
            DriftType::Amelioration => "Meaning acquired more positive connotations",
            DriftType::Pejoration => "Meaning acquired more negative connotations",
            DriftType::Metaphor => "Meaning shifted via conceptual similarity",
            DriftType::Metonymy => "Meaning shifted via contiguity or association",
            DriftType::TabooReplacement => "Original word replaced due to cultural taboo",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftRecord {
    pub original_definition: String,
    pub new_definition: String,
    pub drift_type: DriftType,
    pub time_step: usize,
    pub trigger: String,
}

pub struct SemanticDriftEngine { config: DriftConfig }

impl SemanticDriftEngine {
    pub fn new(config: DriftConfig) -> Self { Self { config } }

    // ── Core API ───────────────────────────────────────────────────────────

    /// Apply semantic drift to every word in the lexicon across `time_steps` iterations.
    /// Returns a history map of which words changed and how.
    pub fn apply_to_lexicon(&self, lexicon: &mut Lexicon) -> HashMap<String, Vec<DriftRecord>> {
        let mut history: HashMap<String, Vec<DriftRecord>> = HashMap::new();
        let mut rng = rand::thread_rng();
        let words: Vec<String> = lexicon.0.keys().cloned().collect();
        for step in 0..self.config.time_steps {
            for word in &words {
                if rng.gen_bool(self.config.drift_rate) {
                    if let Some(entry) = lexicon.0.get_mut(word) {
                        if let Some(record) = self.drift_word(entry, step, &mut rng) {
                            history.entry(word.clone()).or_default().push(record);
                        }
                    }
                }
            }
        }
        history
    }

    // ── Per-Word Engine ─────────────────────────────────────────────────────

    /// Apply a single drift event to one lexicon entry.
    fn drift_word(&self, entry: &mut LexiconEntry, step: usize, rng: &mut impl Rng) -> Option<DriftRecord> {
        if entry.senses.is_empty() { return None; }
        let sense_idx = rng.gen_range(0..entry.senses.len());
        let original_def = entry.senses[sense_idx].definition.clone();
        let (drift_type, new_def, trigger) = self.pick_drift(&original_def, rng);
        entry.senses[sense_idx].definition = new_def.clone();
        let drift_name = match &drift_type {
            DriftType::Broadening => "broadening", DriftType::Narrowing => "narrowing",
            DriftType::Amelioration => "amelioration", DriftType::Pejoration => "pejoration",
            DriftType::Metaphor => "metaphorical", DriftType::Metonymy => "metonymic",
            DriftType::TabooReplacement => "taboo replacement",
        };
        entry.etymology.push_str(&format!(" [{} drift in period {}: {}]", drift_name, step + 1, trigger));
        Some(DriftRecord { original_definition: original_def, new_definition: new_def, drift_type, time_step: step + 1, trigger })
    }

    /// Pick a drift type probabilistically and generate a new definition.
    fn pick_drift(&self, definition: &str, rng: &mut impl Rng) -> (DriftType, String, String) {
        let cultural_marker = self.config.cultural_markers.choose(rng).cloned().unwrap_or_else(|| "general_change".to_string());
        let drift_type = match rng.gen_range(0..100) {
            0..=20 => DriftType::Broadening, 21..=35 => DriftType::Narrowing,
            36..=45 => DriftType::Amelioration, 46..=55 => DriftType::Pejoration,
            56..=70 => DriftType::Metaphor, 71..=85 => DriftType::Metonymy,
            _ => DriftType::TabooReplacement,
        };
        let new_def = match &drift_type {
            DriftType::Broadening => {
                let opts = [format!("Any kind of entity related to {}", cultural_marker), format!("Originally {}, now any similar phenomenon", definition.to_lowercase()), format!("A general class encompassing {} and related concepts", definition.to_lowercase())];
                opts.choose(rng).unwrap().clone()
            }
            DriftType::Narrowing => {
                let opts = [format!("Specifically, {} in the context of {}", definition.to_lowercase(), cultural_marker), format!("A particular type of {}, restricted to {}", definition.to_lowercase(), cultural_marker)];
                opts.choose(rng).unwrap().clone()
            }
            DriftType::Amelioration => {
                let opts = [format!("{} (elevated in status and esteem)", definition), format!("A revered form of {}", definition.to_lowercase())];
                opts.choose(rng).unwrap().clone()
            }
            DriftType::Pejoration => {
                let opts = [format!("{} (debased, now considered vulgar)", definition), format!("A crude or common form of {}", definition.to_lowercase())];
                opts.choose(rng).unwrap().clone()
            }
            DriftType::Metaphor => {
                let domains = [("water", "flowing or changing"), ("fire", "passion or destruction"), ("earth", "stability or foundation"), ("light", "knowledge or clarity")];
                let (d, q) = domains.choose(rng).unwrap();
                format!("{} (metaphorically: as {} is {})", definition, d, q)
            }
            DriftType::Metonymy => {
                let assocs = [("its physical container", "contained within"), ("the associated institution", "administered by"), ("its place of origin", "produced in")];
                let (a, r) = assocs.choose(rng).unwrap();
                format!("{} (by metonymy: referring to {} {} it)", definition, a, r)
            }
            DriftType::TabooReplacement => {
                let opts = [format!("A euphemistic term replacing the original '{}' due to {} taboos", definition, cultural_marker), format!("An indirect reference avoiding the direct name (taboo: {})", cultural_marker)];
                opts.choose(rng).unwrap().clone()
            }
        };
        (drift_type, new_def, cultural_marker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon_structs::{Citation, Sense};

    fn make_entry(word: &str, def: &str) -> LexiconEntry {
        LexiconEntry {
            headword: word.to_string(), etymology: "test".to_string(), part_of_speech: "noun".to_string(),
            ipa: "/test/".to_string(),
            senses: vec![Sense { definition: def.to_string(), citations: vec![Citation { author: "Test".to_string(), work: "Test".to_string(), date: "2024".to_string(), context: "test".to_string() }] }],
            root: word.to_string(), noun_class: None,
        }
    }

    #[test] fn test_drift_applies_to_entry() {
        let config = DriftConfig { drift_rate: 1.0, time_steps: 1, ..Default::default() };
        let engine = SemanticDriftEngine::new(config);
        let mut entry = make_entry("test", "A test concept");
        let original = entry.senses[0].definition.clone();
        let mut rng = rand::thread_rng();
        let record = engine.drift_word(&mut entry, 0, &mut rng);
        assert!(record.is_some());
        assert_ne!(entry.senses[0].definition, original);
        assert!(entry.etymology.contains("drift"));
    }

    #[test] fn test_drift_on_lexicon() {
        let config = DriftConfig { drift_rate: 1.0, time_steps: 2, ..Default::default() };
        let engine = SemanticDriftEngine::new(config);
        let mut lexicon = Lexicon(HashMap::from([("word1".to_string(), make_entry("word1", "First concept")), ("word2".to_string(), make_entry("word2", "Second concept"))]));
        let history = engine.apply_to_lexicon(&mut lexicon);
        assert!(!history.is_empty());
        for (_w, records) in &history { assert_eq!(records.len(), 2); }
    }

    #[test] fn test_no_drift_with_zero_rate() {
        let engine = SemanticDriftEngine::new(DriftConfig { drift_rate: 0.0, time_steps: 5, ..Default::default() });
        let mut lexicon = Lexicon(HashMap::from([("word1".to_string(), make_entry("word1", "Stable concept"))]));
        assert!(engine.apply_to_lexicon(&mut lexicon).is_empty());
    }
}
