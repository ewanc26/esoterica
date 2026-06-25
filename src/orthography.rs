//! Procedural orthography/script generator for conlangs.
//! Generates writing system mappings: phonemes → visual glyphs.
//! Supports different script types and glyph aesthetics.

use crate::archetypes::Phonology;
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of writing system to generate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ScriptType {
    /// One glyph per phoneme (consonants + vowels)
    Alphabet,
    /// Glyphs only for consonants, vowels are diacritics or unwritten
    Abjad,
    /// Consonant glyphs with mandatory vowel diacritics
    Abugida,
    /// One glyph per syllable (CV, CVC, etc.)
    Syllabary,
    /// One glyph per morpheme/word (symbolic/ideographic)
    Logography,
}

/// Visual style of generated glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GlyphStyle {
    /// Straight lines, sharp angles
    Angular,
    /// Curved, flowing lines
    Curved,
    /// Dots, dashes, simple marks
    Minimal,
    /// Complex, ornate shapes
    Ornate,
}

/// A single glyph description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    /// The phoneme(s) this glyph represents
    pub phoneme: String,
    /// SVG path data for the glyph
    pub svg_path: String,
    /// Human-readable description
    pub description: String,
    /// Glyph category (consonant, vowel, syllable, logogram)
    pub category: String,
}

pub struct OrthographyEngine {
    script_type: ScriptType,
    style: GlyphStyle,
}

impl OrthographyEngine {
    pub fn new(script_type: ScriptType, style: GlyphStyle) -> Self {
        Self { script_type, style }
    }

    /// Generate a complete orthography mapping for the given phonology.
    /// Returns a mapping of phoneme → Glyph.
    pub fn generate(&mut self, phonology: Phonology) -> HashMap<String, Glyph> {
        let mut mapping = HashMap::new();
        let mut rng = rand::thread_rng();

        match self.script_type {
            ScriptType::Alphabet => {
                for c in &phonology.consonants {
                    let glyph = self.generate_glyph(c, "consonant", &mut rng);
                    mapping.insert(c.clone(), glyph);
                }
                for v in &phonology.vowels {
                    let glyph = self.generate_glyph(v, "vowel", &mut rng);
                    mapping.insert(v.clone(), glyph);
                }
            }
            ScriptType::Abjad => {
                for c in &phonology.consonants {
                    let glyph = self.generate_glyph(c, "consonant", &mut rng);
                    mapping.insert(c.clone(), glyph);
                }
                // Vowels become optional diacritics
                for v in &phonology.vowels {
                    let mut glyph = self.generate_glyph(v, "diacritic", &mut rng);
                    glyph.description = format!("Optional diacritic for vowel {}", v);
                    glyph.svg_path = self.generate_diacritic_path(&mut rng);
                    mapping.insert(v.clone(), glyph);
                }
            }
            ScriptType::Abugida => {
                // Base consonant glyphs + vowel modification marks
                for c in &phonology.consonants {
                    let base_glyph = self.generate_glyph(c, "consonant_base", &mut rng);
                    mapping.insert(c.clone(), base_glyph);
                    for v in &phonology.vowels {
                        let key = format!("{}{}", c, v);
                        let glyph = Glyph {
                            phoneme: key.clone(),
                            svg_path: self.generate_abugida_path(&mut rng),
                            description: format!("Syllable {}{} with inherent vowel modified", c, v),
                            category: "syllable".to_string(),
                        };
                        mapping.insert(key, glyph);
                    }
                }
            }
            ScriptType::Syllabary => {
                // Generate glyphs for common syllable patterns
                let patterns = self.generate_syllable_patterns(&phonology);
                for pattern in &patterns {
                    let glyph = self.generate_glyph(pattern, "syllable", &mut rng);
                    mapping.insert(pattern.clone(), glyph);
                }
                // Also map individual phonemes
                for c in &phonology.consonants {
                    if !mapping.contains_key(c) {
                        let glyph = self.generate_glyph(c, "consonant", &mut rng);
                        mapping.insert(c.clone(), glyph);
                    }
                }
                for v in &phonology.vowels {
                    if !mapping.contains_key(v) {
                        let glyph = self.generate_glyph(v, "vowel", &mut rng);
                        mapping.insert(v.clone(), glyph);
                    }
                }
            }
            ScriptType::Logography => {
                // Generate a core set of ~50 logograms for common concepts
                let base_concepts = [
                    "person", "water", "fire", "earth", "sky", "sun", "moon", "tree",
                    "animal", "house", "path", "hand", "eye", "mouth", "heart",
                    "mountain", "river", "field", "stone", "bird", "fish", "food",
                    "tool", "vessel", "cloth", "door", "child", "elder", "spirit",
                    "word", "number", "time", "good", "bad", "big", "small",
                    "life", "death", "love", "war", "peace", "power", "knowledge",
                    "light", "dark", "begin", "end", "above", "below", "within",
                ];
                for concept in &base_concepts {
                    let glyph = self.generate_logogram(concept, &mut rng);
                    mapping.insert(concept.to_string(), glyph);
                }
                // Also add basic phoneme glyphs for phonetic writing
                for c in &phonology.consonants {
                    let glyph = self.generate_glyph(c, "phonetic_consonant", &mut rng);
                    mapping.insert(format!("phon:{}", c), glyph);
                }
                for v in &phonology.vowels {
                    let glyph = self.generate_glyph(v, "phonetic_vowel", &mut rng);
                    mapping.insert(format!("phon:{}", v), glyph);
                }
            }
        }

        mapping
    }

    fn generate_syllable_patterns(&self, phonology: &Phonology) -> Vec<String> {
        let mut patterns = Vec::new();
        for c in &phonology.consonants {
            for v in &phonology.vowels {
                patterns.push(format!("{}{}", c, v));
            }
        }
        patterns
    }

    fn generate_glyph(&self, phoneme: &str, category: &str, rng: &mut impl Rng) -> Glyph {
        let svg_path = match self.style {
            GlyphStyle::Angular => self.generate_angular_path(rng),
            GlyphStyle::Curved => self.generate_curved_path(rng),
            GlyphStyle::Minimal => self.generate_minimal_path(rng),
            GlyphStyle::Ornate => self.generate_ornate_path(rng),
        };
        Glyph {
            phoneme: phoneme.to_string(),
            svg_path,
            description: format!("Glyph for {} ({}, {:?} style)", phoneme, category, self.style),
            category: category.to_string(),
        }
    }

    fn generate_logogram(&self, concept: &str, rng: &mut impl Rng) -> Glyph {
        let svg_path = match self.style {
            GlyphStyle::Angular => self.generate_complex_angular(rng),
            GlyphStyle::Curved => self.generate_complex_curved(rng),
            GlyphStyle::Minimal => self.generate_minimal_path(rng),
            GlyphStyle::Ornate => self.generate_ornate_path(rng),
        };
        Glyph {
            phoneme: concept.to_string(),
            svg_path,
            description: format!("Logogram representing '{}'", concept),
            category: "logogram".to_string(),
        }
    }

    fn generate_angular_path(&self, rng: &mut impl Rng) -> String {
        let x1 = rng.gen_range(0..20);
        let y1 = rng.gen_range(0..30);
        let x2 = rng.gen_range(10..30);
        let y2 = rng.gen_range(0..30);
        let x3 = rng.gen_range(5..25);
        let y3 = rng.gen_range(5..30);
        format!("M{},{} L{},{} L{},{} Z", x1, y1, x2, y2, x3, y3)
    }

    fn generate_curved_path(&self, rng: &mut impl Rng) -> String {
        let x1 = rng.gen_range(0..15);
        let y1 = rng.gen_range(5..25);
        let cx1 = rng.gen_range(5..25);
        let cy1 = rng.gen_range(0..30);
        let x2 = rng.gen_range(10..30);
        let y2 = rng.gen_range(5..25);
        format!("M{},{} Q{},{},{},{}", x1, y1, cx1, cy1, x2, y2)
    }

    fn generate_minimal_path(&self, rng: &mut impl Rng) -> String {
        let shapes = [
            format!("M5,15 L25,15"),
            format!("M15,5 L15,25"),
            format!("M10,10 L20,20"),
            format!("M5,5 L25,5 L25,25 L5,25 Z"),
            format!("M5,10 L25,10 M15,10 L15,20"),
        ];
        shapes.choose(rng).unwrap().clone()
    }

    fn generate_ornate_path(&self, rng: &mut impl Rng) -> String {
        let mut path = String::new();
        let segments = rng.gen_range(3..=6);
        let mut x = 5.0_f64;
        let mut y = 15.0_f64;
        path.push_str(&format!("M{:.0},{:.0}", x, y));
        for _ in 0..segments {
            let cx = x + rng.gen_range(2.0..10.0);
            let cy = y + rng.gen_range(-10.0..10.0);
            x += rng.gen_range(3.0..8.0);
            y = 5.0 + rng.gen_range(0.0..20.0);
            path.push_str(&format!(" Q{:.1},{:.1} {:.1},{:.1}", cx, cy, x, y));
        }
        path
    }

    fn generate_diacritic_path(&self, rng: &mut impl Rng) -> String {
        let marks = [
            "M5,5 L15,0 L25,5".to_string(),
            "M10,0 L20,0".to_string(),
            "M15,0 C20,5 25,10 15,10".to_string(),
            "M5,8 L15,0 L25,8".to_string(),
        ];
        marks.choose(rng).unwrap().clone()
    }

    fn generate_abugida_path(&self, rng: &mut impl Rng) -> String {
        let base_x = rng.gen_range(0..15);
        let base_y = rng.gen_range(5..20);
        let mark_x = rng.gen_range(8..22);
        let mark_y = rng.gen_range(0..10);
        format!("M{},{} L{},{} M{},{} L{},{}",
            base_x, base_y, base_x + 10, base_y + 8,
            mark_x, mark_y, mark_x + 5, mark_y + 5)
    }

    fn generate_complex_angular(&self, rng: &mut impl Rng) -> String {
        let mut path = String::new();
        let n = rng.gen_range(4..=8);
        let mut x = rng.gen_range(2..8);
        let mut y = rng.gen_range(5..20);
        path.push_str(&format!("M{},{}", x, y));
        for _ in 0..n {
            x += rng.gen_range(2..8);
            y = rng.gen_range(2..28);
            path.push_str(&format!(" L{},{}", x, y));
        }
        path.push_str(" Z");
        path
    }

    fn generate_complex_curved(&self, rng: &mut impl Rng) -> String {
        let mut path = String::new();
        let n = rng.gen_range(3..=5);
        let mut x = 5.0_f64;
        let mut y = 15.0_f64;
        path.push_str(&format!("M{:.0},{:.0}", x, y));
        for _ in 0..n {
            let cx1 = x + rng.gen_range(2.0..8.0);
            let cy1 = y + rng.gen_range(-8.0..8.0);
            let cx2 = x + rng.gen_range(5.0..15.0);
            let cy2 = y + rng.gen_range(-6.0..6.0);
            x += rng.gen_range(4.0..12.0);
            y = 5.0 + rng.gen_range(0.0..18.0);
            path.push_str(&format!(" C{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}", cx1, cy1, cx2, cy2, x, y));
        }
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archetypes::Phonology;

    fn make_phono() -> Phonology {
        Phonology {
            vowels: vec!["a".to_string(), "i".to_string(), "u".to_string()],
            consonants: vec!["p".to_string(), "t".to_string(), "k".to_string(), "m".to_string(), "n".to_string()],
            syllable_structure: "CV".to_string(),
            tones: None,
            vowel_harmony: None,
        }
    }

    #[test] fn test_alphabet_generation() {
        let mut engine = OrthographyEngine::new(ScriptType::Alphabet, GlyphStyle::Angular);
        let mapping = engine.generate(make_phono());
        assert!(mapping.len() >= 8); // 5 consonants + 3 vowels
        for (phoneme, glyph) in &mapping {
            assert!(!glyph.svg_path.is_empty());
            assert_eq!(glyph.phoneme, *phoneme);
        }
    }

    #[test] fn test_abjad_generation() {
        let mut engine = OrthographyEngine::new(ScriptType::Abjad, GlyphStyle::Curved);
        let mapping = engine.generate(make_phono());
        assert!(mapping.len() >= 8);
        // Vowels should be diacritics
        if let Some(v_glyph) = mapping.get("a") {
            assert_eq!(v_glyph.category, "diacritic");
        }
    }

    #[test] fn test_syllabary_generation() {
        let mut engine = OrthographyEngine::new(ScriptType::Syllabary, GlyphStyle::Minimal);
        let mapping = engine.generate(make_phono());
        // 5*3 = 15 syllable glyphs + individual phonemes
        assert!(mapping.len() >= 15);
    }

    #[test] fn test_logography_generation() {
        let mut engine = OrthographyEngine::new(ScriptType::Logography, GlyphStyle::Ornate);
        let mapping = engine.generate(make_phono());
        // 50 core logograms + phonetic glyphs
        assert!(mapping.len() >= 50);
    }

    #[test] fn test_all_styles() {
        for style in &[GlyphStyle::Angular, GlyphStyle::Curved, GlyphStyle::Minimal, GlyphStyle::Ornate] {
            let mut engine = OrthographyEngine::new(ScriptType::Alphabet, *style);
            let mapping = engine.generate(make_phono());
            assert!(!mapping.is_empty());
        }
    }
}
