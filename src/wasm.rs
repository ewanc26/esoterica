//! WebAssembly bindings for the Esoterica conlang engine.
//! Compile with: `wasm-pack build --features wasm --no-default-features`
//!
//! Every generating entry point takes an optional `seed`. Passing one makes
//! the call reproducible; passing `null`/`undefined` draws from entropy, which
//! in a browser means `crypto.getRandomValues`.

use crate::archetypes::{MorphRule, Morphology, Phonology, SoundChange, Syntax};
use crate::lexicon::LexiconGenerator;
use crate::lexicon_structs::Lexicon;
use crate::orthography::{GlyphStyle, OrthographyEngine, ScriptType};
use crate::phonology::PhonologyEngine;
use crate::semantic_drift::{DriftConfig, SemanticDriftEngine};
use crate::sound_change::SoundChangeEngine;
use crate::syntax::SyntaxEngine;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Convert any `Display` error into a JS exception value.
fn js_err(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

// ── Word Generation ─────────────────────────────────────────────────────

/// Generate a single word using the provided phonology.
///
/// Returns `{"word": …, "ipa": …}`. Rejects inventories that cannot satisfy
/// their own syllable template rather than producing malformed output.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn generate_word(
    vowels_json: &str,
    consonants_json: &str,
    syllable_structure: &str,
    tones: Option<u8>,
    vowel_harmony: Option<bool>,
    num_syllables: usize,
    seed: Option<u64>,
) -> Result<String, JsValue> {
    let vowels: Vec<String> = serde_json::from_str(vowels_json).map_err(js_err)?;
    let consonants: Vec<String> = serde_json::from_str(consonants_json).map_err(js_err)?;

    let phono = Phonology {
        vowels,
        consonants,
        syllable_structure: syllable_structure.to_string(),
        tones,
        vowel_harmony,
    };
    phono.validate().map_err(js_err)?;

    let engine = match seed {
        Some(seed) => PhonologyEngine::seeded(phono, seed),
        None => PhonologyEngine::new(phono),
    };
    let word = engine.generate_word(num_syllables);
    let ipa = engine.to_ipa(&word);

    Ok(serde_json::json!({ "word": word, "ipa": ipa }).to_string())
}

/// Generate a batch of distinct words in one call, as a JSON array of
/// `{"word": …, "ipa": …}` objects.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn generate_words(
    vowels_json: &str,
    consonants_json: &str,
    syllable_structure: &str,
    tones: Option<u8>,
    vowel_harmony: Option<bool>,
    num_syllables: usize,
    count: usize,
    seed: Option<u64>,
) -> Result<String, JsValue> {
    let vowels: Vec<String> = serde_json::from_str(vowels_json).map_err(js_err)?;
    let consonants: Vec<String> = serde_json::from_str(consonants_json).map_err(js_err)?;

    let phono = Phonology {
        vowels,
        consonants,
        syllable_structure: syllable_structure.to_string(),
        tones,
        vowel_harmony,
    };
    phono.validate().map_err(js_err)?;

    let engine = match seed {
        Some(seed) => PhonologyEngine::seeded(phono, seed),
        None => PhonologyEngine::new(phono),
    };

    let words: Vec<serde_json::Value> = engine
        .generate_distinct_words(count, num_syllables)
        .into_iter()
        .map(|word| {
            let ipa = engine.to_ipa(&word);
            serde_json::json!({ "word": word, "ipa": ipa })
        })
        .collect();

    serde_json::to_string(&words).map_err(js_err)
}

// ── Lexicon Generation ──────────────────────────────────────────────────

/// Generate a full lexicon with the provided configuration.
/// Returns a JSON object keyed by headword.
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
        sound_changes: Vec<SoundChange>,
        size: usize,
        syllables_per_word: usize,
        #[serde(default)]
        seed: Option<u64>,
    }

    let config: Config = serde_json::from_str(config_json).map_err(js_err)?;

    let phono = Phonology {
        vowels: config.vowels,
        consonants: config.consonants,
        syllable_structure: config.syllable_structure,
        tones: config.tones,
        vowel_harmony: config.vowel_harmony,
    };
    let morph = Morphology { rules: config.morph_rules, noun_classes: config.noun_classes };

    let mut generator =
        LexiconGenerator::try_new(phono, morph, config.sound_changes, config.seed)
            .map_err(js_err)?
            .with_syllables(config.syllables_per_word);
    let lexicon = generator.generate_core_lexicon(config.size);

    serde_json::to_string(lexicon).map_err(js_err)
}

// ── Semantic Drift ──────────────────────────────────────────────────────

/// Apply semantic drift to an existing lexicon.
#[wasm_bindgen]
pub fn apply_semantic_drift(
    lexicon_json: &str,
    drift_rate: f64,
    time_steps: usize,
    seed: Option<u64>,
) -> Result<String, JsValue> {
    let mut lexicon: Lexicon = serde_json::from_str(lexicon_json).map_err(js_err)?;
    let config = DriftConfig { drift_rate, time_steps, seed, ..Default::default() };
    let engine = SemanticDriftEngine::new(config);
    let history = engine.apply_to_lexicon(&mut lexicon);

    serde_json::to_string(&serde_json::json!({
        "lexicon": lexicon,
        "history": history,
    }))
    .map_err(js_err)
}

// ── Orthography ─────────────────────────────────────────────────────────

/// Generate an orthography mapping for the given phoneme sets.
#[wasm_bindgen]
pub fn generate_orthography(
    vowels_json: &str,
    consonants_json: &str,
    script_type: &str,
    style: &str,
    seed: Option<u64>,
) -> Result<String, JsValue> {
    let vowels: Vec<String> = serde_json::from_str(vowels_json).map_err(js_err)?;
    let consonants: Vec<String> = serde_json::from_str(consonants_json).map_err(js_err)?;

    let script = match script_type {
        "alphabet" => ScriptType::Alphabet,
        "abjad" => ScriptType::Abjad,
        "abugida" => ScriptType::Abugida,
        "syllabary" => ScriptType::Syllabary,
        "logography" => ScriptType::Logography,
        other => {
            return Err(JsValue::from_str(&format!(
                "unknown script type '{}' (expected alphabet, abjad, abugida, syllabary, or logography)",
                other
            )))
        }
    };
    let glyph_style = match style {
        "angular" => GlyphStyle::Angular,
        "curved" => GlyphStyle::Curved,
        "minimal" => GlyphStyle::Minimal,
        "ornate" => GlyphStyle::Ornate,
        other => {
            return Err(JsValue::from_str(&format!(
                "unknown glyph style '{}' (expected angular, curved, minimal, or ornate)",
                other
            )))
        }
    };

    let phono = Phonology {
        vowels,
        consonants,
        syllable_structure: "CV".to_string(),
        tones: None,
        vowel_harmony: None,
    };
    let engine = match seed {
        Some(seed) => OrthographyEngine::seeded(script, glyph_style, seed),
        None => OrthographyEngine::new(script, glyph_style),
    };
    serde_json::to_string(&engine.generate(phono)).map_err(js_err)
}

// ── Syntax ──────────────────────────────────────────────────────────────

/// Generate a sentence with the given word order.
#[wasm_bindgen]
pub fn generate_sentence(
    words_json: &str,
    word_order: &str,
    cases_json: &str,
) -> Result<String, JsValue> {
    let words: Vec<String> = serde_json::from_str(words_json).map_err(js_err)?;
    let cases: Vec<String> = serde_json::from_str(cases_json).map_err(js_err)?;
    let syntax = Syntax {
        word_order: word_order.to_string(),
        cases: Some(cases),
        adposition: None,
        adjective_order: None,
    };
    let engine = SyntaxEngine::try_new(syntax).map_err(js_err)?;
    Ok(engine.generate_sentence(&words))
}

/// Describe a syntactic profile: word order, adpositions, case, adjectives.
#[wasm_bindgen]
pub fn describe_syntax(word_order: &str, cases_json: &str) -> Result<String, JsValue> {
    let cases: Vec<String> = serde_json::from_str(cases_json).map_err(js_err)?;
    let syntax = Syntax {
        word_order: word_order.to_string(),
        cases: Some(cases),
        adposition: None,
        adjective_order: None,
    };
    Ok(SyntaxEngine::try_new(syntax).map_err(js_err)?.describe())
}

// ── Sound Changes ──────────────────────────────────────────────────────

/// Apply sound changes using formal rule notation (e.g. "p > b / V_V").
///
/// Every rule must parse; an invalid rule is reported rather than skipped.
#[wasm_bindgen]
pub fn apply_sound_changes(word: &str, rules_json: &str) -> Result<String, JsValue> {
    let rules: Vec<String> = serde_json::from_str(rules_json).map_err(js_err)?;
    let engine = SoundChangeEngine::try_from_formal_rules(&rules)
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(engine.apply(word))
}

/// Apply formal rules to a word and return each intermediate stage, so a UI
/// can show how a form evolved rule by rule.
#[wasm_bindgen]
pub fn trace_sound_changes(word: &str, rules_json: &str) -> Result<String, JsValue> {
    let rules: Vec<String> = serde_json::from_str(rules_json).map_err(js_err)?;
    let engine = SoundChangeEngine::try_from_formal_rules(&rules)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&engine.trace(word)).map_err(js_err)
}

/// Parse a formal sound change rule and return its structure.
#[wasm_bindgen]
pub fn parse_sound_rule(rule: &str) -> Result<String, JsValue> {
    let parsed = crate::sound_change::FormalRule::parse(rule)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&parsed).map_err(js_err)
}

// ── Preset Catalogues ───────────────────────────────────────────────────

/// Get the available phonology presets.
#[wasm_bindgen]
pub fn get_phonology_presets() -> Result<String, JsValue> {
    serde_json::to_string(&crate::archetypes::try_get_phonology_registry().map_err(js_err)?)
        .map_err(js_err)
}

/// Get the available morphology presets.
#[wasm_bindgen]
pub fn get_morphology_presets() -> Result<String, JsValue> {
    serde_json::to_string(&crate::archetypes::try_get_morphology_registry().map_err(js_err)?)
        .map_err(js_err)
}

/// Get the available sound change presets.
#[wasm_bindgen]
pub fn get_sound_change_presets() -> Result<String, JsValue> {
    serde_json::to_string(&crate::archetypes::try_get_sound_change_registry().map_err(js_err)?)
        .map_err(js_err)
}

/// Get the available syntax presets.
#[wasm_bindgen]
pub fn get_syntax_presets() -> Result<String, JsValue> {
    serde_json::to_string(&crate::archetypes::try_get_syntax_registry().map_err(js_err)?)
        .map_err(js_err)
}
