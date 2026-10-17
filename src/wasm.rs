//! WebAssembly bindings for the Esoterica conlang engine.
//! Compile with: wasm-pack build --features wasm

use wasm_bindgen::prelude::*;
use crate::archetypes::{Phonology, Morphology, MorphRule, Syntax};
use crate::phonology::PhonologyEngine;
use crate::sound_change::SoundChangeEngine;
use crate::syntax::SyntaxEngine;
use crate::lexicon::LexiconGenerator;
use crate::semantic_drift::{SemanticDriftEngine, DriftConfig};
use crate::orthography::{OrthographyEngine, ScriptType, GlyphStyle};
use crate::lexicon_structs::Lexicon;
use serde::Deserialize;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// ── Word Generation ─────────────────────────────────────────────────────

/// Generate a single word using the provided phonology.
#[wasm_bindgen]
pub fn generate_word(
    vowels_json: &str,
    consonants_json: &str,
    syllable_structure: &str,
    tones: Option<u8>,
    vowel_harmony: Option<bool>,
    num_syllables: usize,
) -> Result<String, JsValue> {
    let vowels: Vec<String> = serde_json::from_str(vowels_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let consonants: Vec<String> = serde_json::from_str(consonants_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let phono = Phonology { vowels, consonants, syllable_structure: syllable_structure.to_string(), tones, vowel_harmony };
    let engine = PhonologyEngine::new(phono);
    let word = engine.generate_word(num_syllables);
    let ipa = engine.to_ipa(&word);

    let result = serde_json::json!({ "word": word, "ipa": ipa });
    Ok(result.to_string())
}

// ── Lexicon Generation ──────────────────────────────────────────────────

/// Generate a full lexicon with the provided configuration.
/// Returns JSON string with all entries.
#[wasm_bindgen]
pub fn generate_lexicon(config_json: &str) -> Result<String, JsValue> {
    #[derive(Deserialize)]
    struct Config {
        vowels: Vec<String>,
        consonants: Vec<String>,
        syllable_structure: String,
        tones: Option<u8>,
        vowel_harmony: Option<bool>,
        morph_rules: Vec<MorphRule>,
        noun_classes: Option<Vec<String>>,
        sound_changes: Vec<String>,
        size: usize,
        syllables_per_word: usize,
    }

    let config: Config = serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let phono = Phonology {
        vowels: config.vowels,
        consonants: config.consonants,
        syllable_structure: config.syllable_structure,
        tones: config.tones,
        vowel_harmony: config.vowel_harmony,
    };
    let morph = Morphology { rules: config.morph_rules, noun_classes: config.noun_classes };

    let mut gen = LexiconGenerator::new(phono, morph, &config.sound_changes)
        .with_syllables(config.syllables_per_word);
    let lexicon = gen.generate_core_lexicon(config.size);

    serde_json::to_string(lexicon).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ── Semantic Drift ──────────────────────────────────────────────────────

/// Apply semantic drift to an existing lexicon.
#[wasm_bindgen]
pub fn apply_semantic_drift(lexicon_json: &str, drift_rate: f64, time_steps: usize) -> Result<String, JsValue> {
    let mut lexicon: Lexicon = serde_json::from_str(lexicon_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let config = DriftConfig { drift_rate, time_steps, ..Default::default() };
    let engine = SemanticDriftEngine::new(config);
    let _history = engine.apply_to_lexicon(&mut lexicon);
    serde_json::to_string(&lexicon).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ── Orthography ─────────────────────────────────────────────────────────

/// Generate an orthography mapping for the given phoneme sets.
#[wasm_bindgen]
pub fn generate_orthography(vowels_json: &str, consonants_json: &str, script_type: &str, style: &str) -> Result<String, JsValue> {
    let vowels: Vec<String> = serde_json::from_str(vowels_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let consonants: Vec<String> = serde_json::from_str(consonants_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let st = match script_type {
        "alphabet" => ScriptType::Alphabet,
        "abjad" => ScriptType::Abjad,
        "abugida" => ScriptType::Abugida,
        "syllabary" => ScriptType::Syllabary,
        "logography" => ScriptType::Logography,
        _ => ScriptType::Alphabet,
    };
    let gs = match style {
        "angular" => GlyphStyle::Angular,
        "curved" => GlyphStyle::Curved,
        "minimal" => GlyphStyle::Minimal,
        "ornate" => GlyphStyle::Ornate,
        _ => GlyphStyle::Angular,
    };

    let phono = Phonology { vowels, consonants, syllable_structure: "CV".to_string(), tones: None, vowel_harmony: None };
    let mut engine = OrthographyEngine::new(st, gs);
    let mapping = engine.generate(phono);
    serde_json::to_string(&mapping).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ── Syntax ──────────────────────────────────────────────────────────────

/// Generate a sentence with the given word order.
#[wasm_bindgen]
pub fn generate_sentence(words_json: &str, word_order: &str, cases_json: &str) -> Result<String, JsValue> {
    let words: Vec<String> = serde_json::from_str(words_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let cases: Vec<String> = serde_json::from_str(cases_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let syntax = Syntax { word_order: word_order.to_string(), cases: Some(cases), alignment: None, adpositions: None };
    let engine = SyntaxEngine::new(syntax);
    Ok(engine.generate_sentence(&words))
}

// ── Catalogue Generation ───────────────────────────────────────────────

/// Generate a lexicon and example sentence from built-in catalogue keys, using the
/// same validated pipeline as the CLI. `sound_changes_json` is a JSON array of set
/// names, e.g. `["lenition"]`. Returns JSON with `lexicon` and `example_sentence`.
#[wasm_bindgen]
pub fn generate_from_catalogue(
    phonology: &str,
    morphology: &str,
    syntax: &str,
    sound_changes_json: &str,
    size: usize,
    syllables: usize,
) -> Result<String, JsValue> {
    let sound_changes: Vec<String> = serde_json::from_str(sound_changes_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut spec = crate::pipeline::Registries::builtin()
        .spec(phonology, morphology, syntax, &sound_changes)
        .map_err(|e| JsValue::from_str(&e))?;
    spec.lexicon_size = size;
    spec.syllables = syllables;
    let generated = spec.generate().map_err(|e| JsValue::from_str(&e))?;
    let result = serde_json::json!({
        "lexicon": generated.lexicon,
        "example_sentence": generated.example_sentence,
    });
    Ok(result.to_string())
}

// ── Sound Changes ──────────────────────────────────────────────────────

/// Apply sound changes using formal rule notation (e.g. "p > b / V_V").
#[wasm_bindgen]
pub fn apply_sound_changes(word: &str, rules_json: &str) -> Result<String, JsValue> {
    let rules: Vec<String> = serde_json::from_str(rules_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let engine = SoundChangeEngine::from_formal_rules(&rules);
    Ok(engine.apply(word))
}

/// Parse a formal sound change rule and return its structure.
#[wasm_bindgen]
pub fn parse_sound_rule(rule: &str) -> Result<String, JsValue> {
    match crate::sound_change::FormalRule::parse(rule) {
        Ok(rule) => serde_json::to_string(&serde_json::json!({
            "type": match rule {
                crate::sound_change::FormalRule::Unconditional { .. } => "unconditional",
                crate::sound_change::FormalRule::Contextual { .. } => "contextual",
            },
            "parsed": format!("{:?}", rule),
        })).map_err(|e| JsValue::from_str(&e.to_string())),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

// ── Preset Catalogues ───────────────────────────────────────────────────

/// Get the available phonology presets.
#[wasm_bindgen]
pub fn get_phonology_presets() -> Result<String, JsValue> {
    let registry = crate::archetypes::get_phonology_registry();
    serde_json::to_string(&registry).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get the available morphology presets.
#[wasm_bindgen]
pub fn get_morphology_presets() -> Result<String, JsValue> {
    let registry = crate::archetypes::get_morphology_registry();
    serde_json::to_string(&registry).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get the available sound change presets.
#[wasm_bindgen]
pub fn get_sound_change_presets() -> Result<String, JsValue> {
    let registry = crate::archetypes::get_sound_change_registry();
    serde_json::to_string(&registry).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get the available syntax presets.
#[wasm_bindgen]
pub fn get_syntax_presets() -> Result<String, JsValue> {
    let registry = crate::archetypes::get_syntax_registry();
    serde_json::to_string(&registry).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The success path never constructs a JsValue, so it runs on the host.
    #[test]
    fn generate_from_catalogue_returns_lexicon_and_sentence() {
        let out = generate_from_catalogue("uralic_finnic", "agglutinative", "sov", "[\"lenition\"]", 8, 2)
            .expect("generation should succeed");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["lexicon"].is_object());
        assert!(v["example_sentence"].as_str().unwrap().ends_with('.'));
    }
}
