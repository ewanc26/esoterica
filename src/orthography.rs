//! Procedural orthography/script generator for conlangs.
//! Generates writing system mappings: phonemes → visual glyphs.
//! Supports different script types and glyph aesthetics.
//!
//! Glyph paths are random shapes in the chosen style, not designed letterforms.
//! Build the engine with [`OrthographyEngine::seeded`] to get the same script
//! back on every run.

use crate::archetypes::Phonology;
use crate::rng::SharedRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    rng: SharedRng,
}

impl OrthographyEngine {
    /// Build an engine seeded from system entropy.
    pub fn new(script_type: ScriptType, style: GlyphStyle) -> Self {
        Self { script_type, style, rng: SharedRng::from_entropy() }
    }

    /// Build a reproducible engine: the same seed, script type, style, and
    /// phonology always produce the same glyph set.
    pub fn seeded(script_type: ScriptType, style: GlyphStyle, seed: u64) -> Self {
        Self { script_type, style, rng: SharedRng::from_seed(seed) }
    }

    /// The script type this engine generates.
    pub fn script_type(&self) -> ScriptType {
        self.script_type
    }

    /// The glyph style this engine generates.
    pub fn style(&self) -> GlyphStyle {
        self.style
    }

    /// Generate a complete orthography mapping for the given phonology.
    /// Returns a mapping of phoneme/glyph-key → Glyph.
    ///
    /// Ordered by key so a seeded run serialises to the same bytes every time.
    pub fn generate(&self, phonology: Phonology) -> BTreeMap<String, Glyph> {
        self.rng.with(|rng| self.generate_with(&phonology, rng))
    }

    fn generate_with(&self, phonology: &Phonology, rng: &mut impl Rng) -> BTreeMap<String, Glyph> {
        let mut mapping = BTreeMap::new();
        let mut rng = rng;

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
                let patterns = self.generate_syllable_patterns(phonology);
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

    // ── Pattern Generation ──────────────────────────────────────────────────

    fn generate_syllable_patterns(&self, phonology: &Phonology) -> Vec<String> {
        let mut patterns = Vec::new();
        for c in &phonology.consonants {
            for v in &phonology.vowels {
                patterns.push(format!("{}{}", c, v));
            }
        }
        patterns
    }

    // ── Glyph Construction ──────────────────────────────────────────────────

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

    // ── SVG Path Generators ─────────────────────────────────────────────────
    // These produce random strokes in each style. The results are not designed
    // letterforms — they exist to give each generated script a distinct visual
    // character.
    //
    // Every generator anchors its vertices to a shared lattice and refuses to
    // repeat a point within one glyph. Free-floating coordinates produced
    // near-collinear slivers that were hard to tell apart, and a fixed shape
    // list gave a 17-phoneme alphabet only a handful of distinct letters.

    /// Lattice coordinates inside the 30x30 glyph box, leaving a stroke margin.
    const LATTICE: [i32; 4] = [5, 12, 19, 26];

    /// Pick `count` distinct lattice points, ordered as drawn.
    fn lattice_points(count: usize, rng: &mut impl Rng) -> Vec<(i32, i32)> {
        let mut points: Vec<(i32, i32)> = Vec::with_capacity(count);
        // 16 lattice positions against at most 5 vertices, so rejection
        // sampling terminates quickly.
        let mut attempts = 0;
        while points.len() < count && attempts < 64 {
            attempts += 1;
            let candidate = (
                Self::LATTICE[rng.gen_range(0..Self::LATTICE.len())],
                Self::LATTICE[rng.gen_range(0..Self::LATTICE.len())],
            );
            if !points.contains(&candidate) {
                points.push(candidate);
            }
        }
        points
    }

    /// Render points as a move-to followed by line-to segments.
    fn polyline(points: &[(i32, i32)], close: bool) -> String {
        let mut path = String::new();
        for (i, (x, y)) in points.iter().enumerate() {
            path.push_str(&format!("{}{},{}", if i == 0 { "M" } else { " L" }, x, y));
        }
        if close {
            path.push_str(" Z");
        }
        path
    }

    fn generate_angular_path(&self, rng: &mut impl Rng) -> String {
        let points = Self::lattice_points(rng.gen_range(3..=4), rng);
        Self::polyline(&points, rng.gen_bool(0.35))
    }

    fn generate_curved_path(&self, rng: &mut impl Rng) -> String {
        let points = Self::lattice_points(rng.gen_range(3..=4), rng);
        if points.len() < 2 {
            return Self::polyline(&points, false);
        }
        // Each segment bows toward a control point offset from its midpoint.
        let mut path = format!("M{},{}", points[0].0, points[0].1);
        for pair in points.windows(2) {
            let (x1, y1) = pair[0];
            let (x2, y2) = pair[1];
            let bow = rng.gen_range(-9..=9);
            let cx = (x1 + x2) / 2 + bow;
            let cy = (y1 + y2) / 2 - bow;
            path.push_str(&format!(" Q{},{} {},{}", cx.clamp(0, 30), cy.clamp(0, 30), x2, y2));
        }
        path
    }

    fn generate_minimal_path(&self, rng: &mut impl Rng) -> String {
        // Compose one to three primitive strokes so the style still yields a
        // distinct mark per phoneme rather than repeating a fixed shape list.
        let strokes = rng.gen_range(1..=3);
        let mut path = String::new();
        for _ in 0..strokes {
            let a = Self::LATTICE[rng.gen_range(0..Self::LATTICE.len())];
            let b = Self::LATTICE[rng.gen_range(0..Self::LATTICE.len())];
            let stroke = match rng.gen_range(0..4) {
                0 => format!("M{},{} L{},{}", Self::LATTICE[0], a, Self::LATTICE[3], a),
                1 => format!("M{},{} L{},{}", a, Self::LATTICE[0], a, Self::LATTICE[3]),
                2 => format!("M{},{} L{},{}", a, b, b, a),
                _ => format!("M{},{} L{},{}", a, b, (a + 7).min(30), (b + 7).min(30)),
            };
            if !path.is_empty() {
                path.push(' ');
            }
            path.push_str(&stroke);
        }
        path
    }

    fn generate_ornate_path(&self, rng: &mut impl Rng) -> String {
        let points = Self::lattice_points(rng.gen_range(4..=5), rng);
        let mut path = match points.first() {
            Some((x, y)) => format!("M{},{}", x, y),
            None => return "M5,15 L26,15".to_string(),
        };
        for pair in points.windows(2) {
            let (x1, y1) = pair[0];
            let (x2, y2) = pair[1];
            let cx1 = (x1 + rng.gen_range(-8..=8)).clamp(0, 30);
            let cy1 = (y1 + rng.gen_range(-8..=8)).clamp(0, 30);
            let cx2 = (x2 + rng.gen_range(-8..=8)).clamp(0, 30);
            let cy2 = (y2 + rng.gen_range(-8..=8)).clamp(0, 30);
            path.push_str(&format!(" C{},{} {},{} {},{}", cx1, cy1, cx2, cy2, x2, y2));
        }
        path
    }

    fn generate_diacritic_path(&self, rng: &mut impl Rng) -> String {
        // Diacritics sit above the base glyph, so they stay in the top band.
        // The centre is inset so the arms cannot reach outside the view box.
        let x = Self::LATTICE[rng.gen_range(0..Self::LATTICE.len())].clamp(10, 21);
        match rng.gen_range(0..5) {
            0 => format!("M{},7 L{},1 L{},7", x - 5, x, x + 5),
            1 => format!("M{},2 L{},2", x - 5, x + 5),
            2 => format!("M{},1 C{},6 {},9 {},9", x, x + 5, x + 8, x),
            3 => format!("M{},1 L{},7", x, x + 5),
            _ => format!("M{},4 L{},4 M{},1 L{},7", x - 5, x + 5, x, x),
        }
    }

    fn generate_abugida_path(&self, rng: &mut impl Rng) -> String {
        // A base consonant stroke plus a vowel mark attached above it.
        let base = Self::lattice_points(2, rng);
        let base_path = Self::polyline(&base, false);
        let mark_x = Self::LATTICE[rng.gen_range(0..Self::LATTICE.len())];
        format!(
            "{} M{},2 L{},6",
            base_path,
            mark_x,
            (mark_x + rng.gen_range(-4..=4)).clamp(0, 30)
        )
    }

    fn generate_complex_angular(&self, rng: &mut impl Rng) -> String {
        // Logograms are denser than phonetic glyphs: more vertices, closed.
        let points = Self::lattice_points(rng.gen_range(5..=7), rng);
        Self::polyline(&points, true)
    }

    fn generate_complex_curved(&self, rng: &mut impl Rng) -> String {
        let points = Self::lattice_points(rng.gen_range(4..=6), rng);
        let mut path = match points.first() {
            Some((x, y)) => format!("M{},{}", x, y),
            None => return "M5,15 L26,15".to_string(),
        };
        for pair in points.windows(2) {
            let (x1, y1) = pair[0];
            let (x2, y2) = pair[1];
            let bow = rng.gen_range(-10..=10);
            path.push_str(&format!(
                " C{},{} {},{} {},{}",
                (x1 + bow).clamp(0, 30),
                (y1 - bow).clamp(0, 30),
                (x2 - bow).clamp(0, 30),
                (y2 + bow).clamp(0, 30),
                x2,
                y2
            ));
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

    const ALL_SCRIPTS: [ScriptType; 5] = [
        ScriptType::Alphabet,
        ScriptType::Abjad,
        ScriptType::Abugida,
        ScriptType::Syllabary,
        ScriptType::Logography,
    ];

    const ALL_STYLES: [GlyphStyle; 4] = [
        GlyphStyle::Angular,
        GlyphStyle::Curved,
        GlyphStyle::Minimal,
        GlyphStyle::Ornate,
    ];

    /// Mapping content in a stable order, for comparing two runs.
    fn fingerprint(mapping: &BTreeMap<String, Glyph>) -> Vec<(String, String)> {
        let mut pairs: Vec<(String, String)> = mapping
            .iter()
            .map(|(k, g)| (k.clone(), g.svg_path.clone()))
            .collect();
        pairs.sort();
        pairs
    }

    #[test] fn test_alphabet_generation() {
        let engine = OrthographyEngine::new(ScriptType::Alphabet, GlyphStyle::Angular);
        let mapping = engine.generate(make_phono());
        assert!(mapping.len() >= 8); // 5 consonants + 3 vowels
        for (phoneme, glyph) in &mapping {
            assert!(!glyph.svg_path.is_empty());
            assert_eq!(glyph.phoneme, *phoneme);
        }
    }

    #[test] fn test_abjad_generation() {
        let engine = OrthographyEngine::new(ScriptType::Abjad, GlyphStyle::Curved);
        let mapping = engine.generate(make_phono());
        assert!(mapping.len() >= 8);
        // Vowels should be diacritics
        if let Some(v_glyph) = mapping.get("a") {
            assert_eq!(v_glyph.category, "diacritic");
        }
    }

    #[test] fn test_syllabary_generation() {
        let engine = OrthographyEngine::new(ScriptType::Syllabary, GlyphStyle::Minimal);
        let mapping = engine.generate(make_phono());
        // 5*3 = 15 syllable glyphs + individual phonemes
        assert!(mapping.len() >= 15);
    }

    #[test] fn test_logography_generation() {
        let engine = OrthographyEngine::new(ScriptType::Logography, GlyphStyle::Ornate);
        let mapping = engine.generate(make_phono());
        // 50 core logograms + phonetic glyphs
        assert!(mapping.len() >= 50);
    }

    #[test] fn test_all_styles() {
        for style in ALL_STYLES {
            let engine = OrthographyEngine::new(ScriptType::Alphabet, style);
            assert!(!engine.generate(make_phono()).is_empty());
        }
    }

    // ── Determinism ──────────────────────────────────────────────────────

    #[test] fn same_seed_produces_the_same_script() {
        for script in ALL_SCRIPTS {
            for style in ALL_STYLES {
                let a = OrthographyEngine::seeded(script, style, 4321);
                let b = OrthographyEngine::seeded(script, style, 4321);
                assert_eq!(
                    fingerprint(&a.generate(make_phono())),
                    fingerprint(&b.generate(make_phono())),
                    "{:?}/{:?} was not reproducible",
                    script,
                    style
                );
            }
        }
    }

    #[test] fn different_seeds_produce_different_scripts() {
        let a = OrthographyEngine::seeded(ScriptType::Alphabet, GlyphStyle::Ornate, 1);
        let b = OrthographyEngine::seeded(ScriptType::Alphabet, GlyphStyle::Ornate, 2);
        assert_ne!(
            fingerprint(&a.generate(make_phono())),
            fingerprint(&b.generate(make_phono()))
        );
    }

    #[test] fn repeated_generation_advances_the_stream() {
        // Two calls on one engine are independent draws, not a repeat.
        let engine = OrthographyEngine::seeded(ScriptType::Alphabet, GlyphStyle::Ornate, 7);
        assert_ne!(
            fingerprint(&engine.generate(make_phono())),
            fingerprint(&engine.generate(make_phono()))
        );
    }

    // ── Robustness ───────────────────────────────────────────────────────

    #[test] fn every_glyph_has_a_path_and_category() {
        for script in ALL_SCRIPTS {
            let engine = OrthographyEngine::seeded(script, GlyphStyle::Angular, 11);
            for (key, glyph) in engine.generate(make_phono()) {
                assert!(!glyph.svg_path.is_empty(), "{} has no path", key);
                assert!(!glyph.category.is_empty(), "{} has no category", key);
                assert!(!glyph.description.is_empty(), "{} has no description", key);
            }
        }
    }

    #[test] fn empty_inventories_produce_no_phoneme_glyphs() {
        let empty = Phonology {
            vowels: vec![],
            consonants: vec![],
            syllable_structure: "CV".to_string(),
            tones: None,
            vowel_harmony: None,
        };
        let engine = OrthographyEngine::seeded(ScriptType::Alphabet, GlyphStyle::Minimal, 1);
        assert!(engine.generate(empty).is_empty());
    }

    #[test] fn logography_still_works_without_phonemes() {
        let empty = Phonology {
            vowels: vec![],
            consonants: vec![],
            syllable_structure: "CV".to_string(),
            tones: None,
            vowel_harmony: None,
        };
        let engine = OrthographyEngine::seeded(ScriptType::Logography, GlyphStyle::Curved, 1);
        // The logogram set is concept-based, so it survives an empty inventory.
        assert!(engine.generate(empty).len() >= 50);
    }

    #[test] fn glyphs_within_a_script_are_distinct() {
        // A fixed shape list used to give every style only a handful of marks,
        // so a 17-letter alphabet had visibly duplicated letters.
        let mut phono = make_phono();
        phono.consonants = "p b t d k g q m n s z f v l r j w"
            .split(' ')
            .map(|s| s.to_string())
            .collect();
        for style in ALL_STYLES {
            let engine = OrthographyEngine::seeded(ScriptType::Alphabet, style, 5);
            let mapping = engine.generate(phono.clone());
            let paths: std::collections::HashSet<&str> =
                mapping.values().map(|g| g.svg_path.as_str()).collect();
            // Random generation can still collide; demand that the overwhelming
            // majority of glyphs are unique rather than perfection.
            assert!(
                paths.len() * 10 >= mapping.len() * 9,
                "{:?} produced only {} distinct shapes for {} phonemes",
                style,
                paths.len(),
                mapping.len()
            );
        }
    }

    #[test] fn glyph_paths_stay_inside_the_view_box() {
        for style in ALL_STYLES {
            for script in ALL_SCRIPTS {
                let engine = OrthographyEngine::seeded(script, style, 17);
                for (key, glyph) in engine.generate(make_phono()) {
                    for number in glyph
                        .svg_path
                        .split(|c: char| !c.is_ascii_digit() && c != '-')
                        .filter(|s| !s.is_empty())
                    {
                        let value: i32 = number.parse().expect("numeric path component");
                        assert!(
                            (-2..=32).contains(&value),
                            "{:?}/{:?} glyph {} leaves the box: {}",
                            script,
                            style,
                            key,
                            glyph.svg_path
                        );
                    }
                }
            }
        }
    }

    #[test] fn glyph_paths_start_with_a_move_command() {
        for style in ALL_STYLES {
            let engine = OrthographyEngine::seeded(ScriptType::Alphabet, style, 3);
            for (key, glyph) in engine.generate(make_phono()) {
                assert!(glyph.svg_path.starts_with('M'), "{} -> {}", key, glyph.svg_path);
            }
        }
    }

    #[test] fn multigraph_phonemes_get_their_own_glyphs() {
        let mut phono = make_phono();
        phono.consonants.push("ng".to_string());
        phono.consonants.push("kw".to_string());
        let engine = OrthographyEngine::seeded(ScriptType::Alphabet, GlyphStyle::Angular, 2);
        let mapping = engine.generate(phono);
        assert!(mapping.contains_key("ng"));
        assert!(mapping.contains_key("kw"));
    }
}
