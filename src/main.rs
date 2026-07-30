//! Esoterica CLI entry point. Requires the `cli` feature (enabled by default).
//! Orchestrates the full generation pipeline: config loading, phonology, morphology,
//! sound change, semantic drift, orthography, ATProto publication.
//! To build for WASM: `wasm-pack build --features wasm --no-default-features`

use bsky_sdk::BskyAgent;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result as EyreResult;
use color_eyre::eyre::{Context, ContextCompat};
use esoterica::archetypes::{self, Registries};
use esoterica::args::Args;
use esoterica::lexicon;
use esoterica::lexicon_structs;
use esoterica::orthography;
use esoterica::semantic_drift;
use esoterica::sound_change::SoundChangeEngine;
use esoterica::syntax::SyntaxEngine;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> EyreResult<()> {
    color_eyre::install()?;

    let args = Args::parse_args();

    // ── Interactive Mode ────────────────────────────────────────────────────

    if args.interactive {
        esoterica::tui::run_tui(args)?;
        return Ok(());
    }

    let registries = Registries::load().map_err(|e| eyre!(e.to_string()))?;

    // ── Catalogue Listing ───────────────────────────────────────────────────

    if args.list {
        print_catalogue(&registries);
        return Ok(());
    }

    // ── Resolve Component Selections ────────────────────────────────────────

    let phono_key = args.phonology.first().context("Phonology is required (see --list)")?;
    let morph_key = args.morphology.first().context("Morphology is required (see --list)")?;
    let syntax_key = args.syntax.as_ref().context("Syntax is required (see --list)")?;

    let phono = registries.phonology(phono_key).map_err(|e| eyre!(e.to_string()))?;
    let morph = registries.morphology(morph_key).map_err(|e| eyre!(e.to_string()))?;
    let syntax = registries.syntax(syntax_key).map_err(|e| eyre!(e.to_string()))?;

    // Unknown rule-set keys are an error: silently dropping them would produce
    // a language that looks generated but skipped a whole sound change.
    let merged_sc = registries
        .merged_sound_changes(&args.sound_change)
        .map_err(|e| eyre!(e.to_string()))?;

    let syntax_engine = SyntaxEngine::try_new(syntax).map_err(|e| eyre!(e.to_string()))?;

    if let Some(seed) = args.seed {
        println!("Seed: {} (rerun with --seed {} for the same language)", seed, seed);
    }
    println!("Syntax: {}", syntax_engine.describe());

    // ── Lexicon Generation ──────────────────────────────────────────────────

    let mut generator =
        lexicon::LexiconGenerator::try_new(phono.clone(), morph, merged_sc, args.seed)
            .map_err(|e| eyre!(e.to_string()))?
            .with_syllables(args.syllables.unwrap_or(2));

    let requested = args.lexicon_size.unwrap_or(100);
    let mut lexicon = generator.generate_core_lexicon(requested).clone();

    if lexicon.len() < requested {
        eprintln!(
            "Warning: produced {} of {} requested entries; this phonology cannot supply more distinct headwords.",
            lexicon.len(),
            requested
        );
    } else {
        println!("Generated {} lexicon entries", lexicon.len());
    }

    // ── Ad-hoc Formal Sound Changes ─────────────────────────────────────────

    if !args.formal_rules.is_empty() {
        let engine = SoundChangeEngine::try_from_formal_rules(&args.formal_rules)
            .map_err(|e| eyre!(e))?;
        lexicon = apply_formal_rules(&lexicon, &engine);
        println!("Applied {} formal sound change rule(s)", args.formal_rules.len());
    }

    // ── Semantic Drift ──────────────────────────────────────────────────────

    if let Some(drift_steps) = args.drift_steps {
        let drift_config = semantic_drift::DriftConfig {
            drift_rate: args.drift_rate.unwrap_or(0.15),
            time_steps: drift_steps,
            // Offset so drift does not replay the lexicon generator's stream.
            seed: args.seed.map(|seed| seed.wrapping_add(0x51_7C_C1_B7)),
            ..Default::default()
        };
        let engine = semantic_drift::SemanticDriftEngine::new(drift_config);
        let history = engine.apply_to_lexicon(&mut lexicon);
        println!(
            "Applied semantic drift over {} steps ({} words affected)",
            drift_steps,
            history.len()
        );
    }

    // ── Orthography ─────────────────────────────────────────────────────────

    if args.generate_orthography {
        let ortho = match args.seed {
            Some(seed) => orthography::OrthographyEngine::seeded(
                args.script_type.into(),
                args.glyph_style.into(),
                // Offset so the script does not share the lexicon's stream.
                seed.wrapping_add(0x2545_F491),
            ),
            None => orthography::OrthographyEngine::new(
                args.script_type.into(),
                args.glyph_style.into(),
            ),
        };
        let mapping = ortho.generate(phono);
        println!(
            "Generated {:?} orthography with {} glyphs",
            args.script_type,
            mapping.len()
        );
        let ortho_path = make_ortho_path(args.output.as_ref());
        let json = serde_json::to_string_pretty(&mapping).map_err(|e| eyre!(e))?;
        std::fs::write(&ortho_path, &json).map_err(|e| eyre!(e))?;
        println!("Orthography saved to: {}", ortho_path.display());
    }

    // ── Preview ─────────────────────────────────────────────────────────────

    if let Some(count) = args.preview {
        print_preview(&lexicon, count);
    }

    // ── Output ──────────────────────────────────────────────────────────────

    if let Some(ref output) = args.output {
        save_lexicon(&lexicon, output)?;
        println!("Lexicon saved to: {}", output.display());
    }

    // ── ATProto Publication ─────────────────────────────────────────────────

    if let Some(title) = args.publish_title.as_ref() {
        match (std::env::var("ATPROTO_HANDLE"), std::env::var("ATPROTO_PASSWORD")) {
            (Ok(handle), Ok(pass)) => {
                let publication_uri = args
                    .publication_uri
                    .as_ref()
                    .context("Need --publication-uri to publish dictionary")?;

                let agent = BskyAgent::builder().build().await?;
                agent.login(handle, pass).await?;

                let publisher = esoterica::atproto::AtprotoPublisher::new(agent);
                let uri = publisher
                    .publish_dictionary(&lexicon.0, title, publication_uri)
                    .await
                    .map_err(|e| eyre!(e))?;
                println!("Published dictionary document to ATProto: {}", uri);
            }
            _ => {
                // Failing loudly beats pretending the publication happened.
                return Err(eyre!(
                    "--publish-title was given but ATPROTO_HANDLE and ATPROTO_PASSWORD are not both set"
                ));
            }
        }
    }

    // ── Example Sentences ───────────────────────────────────────────────────

    for sentence in example_sentences(&lexicon, &syntax_engine, args.sentences) {
        println!("Example sentence: {}", sentence);
    }

    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Print every available archetype key, grouped by kind.
fn print_catalogue(registries: &Registries) {
    println!("Phonologies ({}):", registries.phonologies.len());
    for key in archetypes::sorted_keys(&registries.phonologies) {
        let phono = &registries.phonologies[key];
        println!(
            "  {:<28} {} vowels, {} consonants, {}{}",
            key,
            phono.vowels.len(),
            phono.consonants.len(),
            phono.syllable_structure,
            match phono.tones {
                Some(t) => format!(", {} tones", t),
                None => String::new(),
            }
        );
    }

    println!("\nMorphologies ({}):", registries.morphologies.len());
    for key in archetypes::sorted_keys(&registries.morphologies) {
        let morph = &registries.morphologies[key];
        println!(
            "  {:<28} {} rule(s), {} noun class(es)",
            key,
            morph.rules.len(),
            morph.noun_classes.as_ref().map_or(0, Vec::len)
        );
    }

    println!("\nSyntaxes ({}):", registries.syntaxes.len());
    for key in archetypes::sorted_keys(&registries.syntaxes) {
        let syntax = registries.syntaxes[key].clone();
        println!("  {:<28} {}", key, SyntaxEngine::new(syntax).describe());
    }

    println!("\nSound changes ({}):", registries.sound_changes.len());
    for key in archetypes::sorted_keys(&registries.sound_changes) {
        println!("  {:<28} {} rule(s)", key, registries.sound_changes[key].len());
    }
}

/// Print the first `count` entries in headword order.
fn print_preview(lexicon: &lexicon_structs::Lexicon, count: usize) {
    println!("\nLexicon preview ({} of {}):", count.min(lexicon.len()), lexicon.len());
    for (headword, entry) in lexicon.sorted_entries().into_iter().take(count) {
        let gloss = entry
            .senses
            .first()
            .map(|s| s.definition.as_str())
            .unwrap_or("(no sense recorded)");
        println!("  {:<20} {:<18} {:<10} {}", headword, entry.ipa, entry.part_of_speech, gloss);
    }

    let counts = lexicon.part_of_speech_counts();
    if !counts.is_empty() {
        let summary: Vec<String> =
            counts.iter().map(|(pos, n)| format!("{} {}", n, pos)).collect();
        println!("  ── {}", summary.join(", "));
    }
    println!();
}

/// Rewrite every headword through a formal-rule engine, keeping the lexicon
/// keyed by its (possibly changed) headword.
fn apply_formal_rules(
    lexicon: &lexicon_structs::Lexicon,
    engine: &SoundChangeEngine,
) -> lexicon_structs::Lexicon {
    let mut result = lexicon_structs::Lexicon::new();
    for (_, entry) in lexicon.sorted_entries() {
        let mut entry = entry.clone();
        entry.headword = engine.apply(&entry.headword);
        entry.ipa = esoterica::ipa::transcribe_phonemic(&entry.headword);
        result.insert(entry);
    }
    result
}

/// Build example sentences from real lexicon entries, picking a noun as
/// subject, a verb as predicate, and another noun as object where possible.
fn example_sentences(
    lexicon: &lexicon_structs::Lexicon,
    syntax: &SyntaxEngine,
    count: usize,
) -> Vec<String> {
    if lexicon.is_empty() || count == 0 {
        return Vec::new();
    }

    let by_pos = |pos: &str| -> Vec<&str> {
        lexicon
            .sorted_entries()
            .into_iter()
            .filter(|(_, e)| e.part_of_speech == pos)
            .map(|(k, _)| k.as_str())
            .collect()
    };

    let nouns = by_pos("noun");
    let verbs = by_pos("verb");
    let all = lexicon.sorted_headwords();
    let fallback: Vec<&str> = all.iter().map(String::as_str).collect();

    let pick = |pool: &[&str], index: usize| -> String {
        let pool = if pool.is_empty() { &fallback[..] } else { pool };
        pool[index % pool.len()].to_string()
    };

    (0..count)
        .map(|i| {
            let words = vec![
                pick(&nouns, i * 2),
                pick(&verbs, i),
                pick(&nouns, i * 2 + 1),
            ];
            syntax.generate_sentence(&words)
        })
        .collect()
}

/// Derive the orthography output path from the lexicon output path.
fn make_ortho_path(output: Option<&PathBuf>) -> PathBuf {
    match output {
        Some(path) => {
            let stem = path.file_stem().unwrap_or_default().to_str().unwrap_or("lexicon");
            let parent = path.parent().unwrap_or(Path::new("."));
            parent.join(format!("{}_orthography.json", stem))
        }
        None => PathBuf::from("lexicon_orthography.json"),
    }
}

/// Write the lexicon as pretty-printed JSON.
fn save_lexicon(lexicon: &lexicon_structs::Lexicon, path: &PathBuf) -> EyreResult<()> {
    let json = serde_json::to_string_pretty(lexicon)?;
    std::fs::write(path, &json).context("Failed to write lexicon file")?;
    Ok(())
}
