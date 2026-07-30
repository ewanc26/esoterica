//! End-to-end tests over the public library surface.
//!
//! The unit tests inside each module cover that module in isolation; these
//! exercise the pipeline the CLI and WASM bindings actually drive, and pin the
//! reproducibility guarantee that seeding is supposed to provide.

use esoterica::archetypes::{Registries, WordOrder};
use esoterica::lexicon::LexiconGenerator;
use esoterica::lexicon_structs::Lexicon;
use esoterica::orthography::{GlyphStyle, OrthographyEngine, ScriptType};
use esoterica::semantic_drift::{DriftConfig, SemanticDriftEngine};
use esoterica::sound_change::SoundChangeEngine;
use esoterica::syntax::SyntaxEngine;

/// Run the whole pipeline the way the CLI does, and return the JSON it writes.
fn generate(phonology: &str, morphology: &str, sound_changes: &[&str], seed: u64) -> String {
    let registries = Registries::load().expect("registries load");
    let phono = registries.phonology(phonology).expect("phonology");
    let morph = registries.morphology(morphology).expect("morphology");
    let keys: Vec<String> = sound_changes.iter().map(|s| s.to_string()).collect();
    let changes = registries.merged_sound_changes(&keys).expect("sound changes");

    let mut generator = LexiconGenerator::try_new(phono, morph, changes, Some(seed))
        .expect("valid configuration")
        .with_syllables(2);
    let mut lexicon = generator.generate_core_lexicon(80).clone();

    let drift = SemanticDriftEngine::new(DriftConfig {
        drift_rate: 0.4,
        time_steps: 3,
        seed: Some(seed.wrapping_add(1)),
        ..Default::default()
    });
    drift.apply_to_lexicon(&mut lexicon);

    serde_json::to_string_pretty(&lexicon).expect("serialise")
}

// ── Reproducibility ──────────────────────────────────────────────────────

#[test]
fn the_same_seed_reproduces_the_same_language_byte_for_byte() {
    let first = generate("uralic_finnic", "agglutinative", &["lenition", "rhotacism"], 2024);
    let second = generate("uralic_finnic", "agglutinative", &["lenition", "rhotacism"], 2024);
    assert_eq!(first, second);
}

#[test]
fn different_seeds_reproduce_different_languages() {
    let first = generate("uralic_finnic", "agglutinative", &["lenition"], 1);
    let second = generate("uralic_finnic", "agglutinative", &["lenition"], 2);
    assert_ne!(first, second);
}

#[test]
fn seeded_output_is_stable_across_every_bundled_phonology() {
    let registries = Registries::load().unwrap();
    for name in registries.phonologies.keys() {
        let first = generate(name, "fusional", &["none"], 7);
        let second = generate(name, "fusional", &["none"], 7);
        assert_eq!(first, second, "{} was not reproducible", name);
    }
}

#[test]
fn seeded_orthography_is_stable() {
    let registries = Registries::load().unwrap();
    let phono = registries.phonology("eurasia_ie_romance").unwrap();
    let a = OrthographyEngine::seeded(ScriptType::Abugida, GlyphStyle::Curved, 99);
    let b = OrthographyEngine::seeded(ScriptType::Abugida, GlyphStyle::Curved, 99);
    assert_eq!(
        serde_json::to_string(&a.generate(phono.clone())).unwrap(),
        serde_json::to_string(&b.generate(phono)).unwrap()
    );
}

// ── Every bundled archetype is usable ────────────────────────────────────

#[test]
fn every_phonology_and_morphology_pairing_generates() {
    let registries = Registries::load().unwrap();
    for (phono_name, phono) in &registries.phonologies {
        for (morph_name, morph) in &registries.morphologies {
            let mut generator =
                LexiconGenerator::try_new(phono.clone(), morph.clone(), vec![], Some(3))
                    .unwrap_or_else(|e| panic!("{} + {}: {}", phono_name, morph_name, e))
                    .with_syllables(2);
            let lexicon = generator.generate_core_lexicon(25);
            assert!(
                !lexicon.is_empty(),
                "{} + {} produced nothing",
                phono_name,
                morph_name
            );
        }
    }
}

#[test]
fn every_sound_change_set_applies_to_every_phonology() {
    let registries = Registries::load().unwrap();
    for (phono_name, phono) in &registries.phonologies {
        for (change_name, changes) in &registries.sound_changes {
            let morph = registries.morphology("agglutinative").unwrap();
            let mut generator =
                LexiconGenerator::seeded(phono.clone(), morph, changes.clone(), 13)
                    .with_syllables(2);
            let lexicon = generator.generate_core_lexicon(20);
            assert!(
                !lexicon.is_empty(),
                "{} + {} produced nothing",
                phono_name,
                change_name
            );
        }
    }
}

#[test]
fn every_syntax_preset_produces_a_sentence() {
    let registries = Registries::load().unwrap();
    let words: Vec<String> = ["kota", "ama", "silu"].iter().map(|s| s.to_string()).collect();
    for (name, syntax) in &registries.syntaxes {
        let engine = SyntaxEngine::try_new(syntax.clone())
            .unwrap_or_else(|e| panic!("{}: {}", name, e));
        let sentence = engine.generate_sentence(&words);
        assert!(sentence.ends_with('.'), "{} produced '{}'", name, sentence);
        assert!(sentence.len() > 5, "{} produced '{}'", name, sentence);
    }
}

#[test]
fn all_six_word_orders_place_constituents_differently() {
    let words: Vec<String> = ["subj", "verb", "obj"].iter().map(|s| s.to_string()).collect();
    let mut seen = std::collections::HashSet::new();
    for order in WordOrder::ALL {
        let syntax = esoterica::archetypes::Syntax {
            word_order: order.to_string(),
            cases: None,
            adposition: None,
            adjective_order: None,
        };
        let sentence = SyntaxEngine::try_new(syntax).unwrap().generate_sentence(&words);
        assert!(seen.insert(sentence.clone()), "{} duplicated '{}'", order, sentence);
    }
    assert_eq!(seen.len(), 6);
}

// ── Lexicon integrity ────────────────────────────────────────────────────

#[test]
fn a_generated_lexicon_round_trips_through_json() {
    let json = generate("asia_taikadai", "polysynthetic", &["palatalization"], 555);
    let restored: Lexicon = serde_json::from_str(&json).expect("parse");
    assert_eq!(restored.len(), 80);
    for (headword, entry) in restored.iter() {
        assert_eq!(headword, &entry.headword);
        assert!(!entry.senses.is_empty());
        assert!(entry.ipa.starts_with('/') && entry.ipa.ends_with('/'));
    }
}

#[test]
fn requested_lexicon_size_is_honoured_for_realistic_inventories() {
    let registries = Registries::load().unwrap();
    let phono = registries.phonology("eurasia_ie_slavic").unwrap();
    let morph = registries.morphology("fusional").unwrap();
    let mut generator = LexiconGenerator::seeded(phono, morph, vec![], 8).with_syllables(2);
    assert_eq!(generator.generate_core_lexicon(750).len(), 750);
}

#[test]
fn multibyte_phonologies_survive_infixing_morphology() {
    let registries = Registries::load().unwrap();
    let phono = registries.phonology("uralic_finnic").unwrap();
    let morph = registries.morphology("polysynthetic").unwrap();
    let mut generator = LexiconGenerator::seeded(phono, morph, vec![], 4).with_syllables(3);
    let lexicon = generator.generate_core_lexicon(120);
    assert_eq!(lexicon.len(), 120);
    // The infix must land on a character boundary, not split a multibyte vowel.
    for entry in lexicon.values() {
        assert!(entry.headword.contains("-ka-"), "missing infix in {}", entry.headword);
    }
}

// ── Sound changes ────────────────────────────────────────────────────────

#[test]
fn formal_rules_compose_into_a_derivation() {
    let engine = SoundChangeEngine::try_from_formal_rules(&[
        "p > f".to_string(),
        "t > \u{03b8}".to_string(),
        "k > h / _#".to_string(),
    ])
    .expect("valid rules");

    assert_eq!(engine.apply("patak"), "fa\u{03b8}ah");

    let trace = engine.trace("patak");
    assert_eq!(trace.len(), 3);
    assert_eq!(trace[0].before, "patak");
    assert_eq!(trace[0].after, "fatak");
    assert_eq!(trace.last().unwrap().after, "fa\u{03b8}ah");
    assert!(trace.iter().all(|step| step.changed));
}

#[test]
fn a_rule_that_matches_nothing_is_marked_unchanged() {
    let engine = SoundChangeEngine::try_from_formal_rules(&["q > x".to_string()]).unwrap();
    let trace = engine.trace("mana");
    assert_eq!(trace.len(), 1);
    assert!(!trace[0].changed);
    assert_eq!(trace[0].before, trace[0].after);
}

#[test]
fn invalid_formal_rules_are_rejected_rather_than_dropped() {
    let err = SoundChangeEngine::try_from_formal_rules(&[
        "p > b".to_string(),
        "not a rule at all".to_string(),
    ])
    .unwrap_err();
    assert!(err.contains("not a rule"), "unhelpful error: {}", err);
}

#[test]
fn legacy_and_formal_rules_apply_in_sequence() {
    let registries = Registries::load().unwrap();
    let mut engine = SoundChangeEngine::new(registries.sound_change("grimms_law").unwrap());
    engine.add_formal_rule("h > \u{2205} / #_").expect("valid rule");
    // Grimm's Law turns k into h word-initially; the formal rule then drops it.
    assert_eq!(engine.apply("kata"), "atha");
}

// ── Drift ────────────────────────────────────────────────────────────────

#[test]
fn drift_records_a_history_entry_for_every_change() {
    let registries = Registries::load().unwrap();
    let phono = registries.phonology("oceania_austronesian").unwrap();
    let morph = registries.morphology("isolating").unwrap();
    let mut generator = LexiconGenerator::seeded(phono, morph, vec![], 21).with_syllables(2);
    let mut lexicon = generator.generate_core_lexicon(60).clone();

    let before: Vec<String> = lexicon.values().map(|e| e.etymology.clone()).collect();
    let engine = SemanticDriftEngine::new(DriftConfig {
        drift_rate: 1.0,
        time_steps: 2,
        seed: Some(21),
        ..Default::default()
    });
    let history = engine.apply_to_lexicon(&mut lexicon);

    assert_eq!(history.len(), lexicon.len());
    for records in history.values() {
        assert_eq!(records.len(), 2);
    }
    let after: Vec<String> = lexicon.values().map(|e| e.etymology.clone()).collect();
    assert_ne!(before, after);
}
