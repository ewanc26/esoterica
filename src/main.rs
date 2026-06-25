//! Esoterica CLI — requires the `cli` feature (enabled by default).
//! To build for WASM: `wasm-pack build --features wasm --no-default-features`

use esoterica::args::Args;
use color_eyre::eyre::Result as EyreResult;
use color_eyre::eyre::{Context, ContextCompat};
use bsky_sdk::BskyAgent;
use std::path::PathBuf;
use esoterica::archetypes;
use esoterica::lexicon;
use esoterica::semantic_drift;
use esoterica::orthography;
use esoterica::syntax::SyntaxEngine;
use esoterica::lexicon_structs;

#[tokio::main]
async fn main() -> EyreResult<()> {
    color_eyre::install()?;

    let args = Args::parse_args();

    if args.interactive {
        esoterica::tui::run_tui(args)?;
        return Ok(());
    }

    let phono_reg = archetypes::get_phonology_registry();
    let sc_reg = archetypes::get_sound_change_registry();
    let morph_reg = archetypes::get_morphology_registry();
    let syntax_reg = archetypes::get_syntax_registry();

    let phono_key = args.phonology.first().context("Phonology is required")?;
    let morph_key = args.morphology.first().context("Morphology is required")?;
    let syntax_key = args.syntax.context("Syntax is required")?;

    let phono = phono_reg.get(phono_key)
        .ok_or_else(|| color_eyre::eyre::eyre!("Unknown phonology: {}", phono_key))?.clone();
    let morph = morph_reg.get(morph_key)
        .ok_or_else(|| color_eyre::eyre::eyre!("Unknown morphology: {}", morph_key))?.clone();

    let mut merged_sc = Vec::new();
    for key in &args.sound_change {
        if let Some(sc) = sc_reg.get(key) {
            merged_sc.extend(sc.clone());
        }
    }

    let syntax = syntax_reg.get(&syntax_key)
        .ok_or_else(|| color_eyre::eyre::eyre!("Unknown syntax: {}", syntax_key))?.clone();

    let mut generator = lexicon::LexiconGenerator::new(phono.clone(), morph, merged_sc)
        .with_syllables(args.syllables.unwrap_or(2));
    let mut lexicon = generator.generate_core_lexicon(args.lexicon_size.unwrap_or(100)).clone();

    if let Some(ref drift_steps) = args.drift_steps {
        let drift_config = semantic_drift::DriftConfig {
            drift_rate: args.drift_rate.unwrap_or(0.15),
            time_steps: *drift_steps,
            ..Default::default()
        };
        let engine = semantic_drift::SemanticDriftEngine::new(drift_config);
        let history = engine.apply_to_lexicon(&mut lexicon);
        println!("Applied semantic drift over {} steps ({} words affected)", drift_steps, history.len());
    }

    if args.generate_orthography {
        let mut ortho = orthography::OrthographyEngine::new(
            orthography::ScriptType::Alphabet,
            orthography::GlyphStyle::Angular,
        );
        let mapping = ortho.generate(phono);
        println!("Generated orthography with {} glyphs", mapping.len());
        let ortho_path = make_ortho_path(args.output.as_ref());
        let json = serde_json::to_string_pretty(&mapping)
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        std::fs::write(&ortho_path, &json)
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        println!("Orthography saved to: {}", ortho_path.display());
    }

    if let Some(ref output) = args.output {
        save_lexicon(&lexicon, output)?;
        println!("Lexicon saved to: {:?}", output);
    }

    if let (Some(title), Ok(handle), Ok(pass)) = (
        args.publish_title,
        std::env::var("ATPROTO_HANDLE"),
        std::env::var("ATPROTO_PASSWORD")
    ) {
        let agent = BskyAgent::builder().build().await?;
        agent.login(handle, pass).await?;

        let publisher = esoterica::atproto::AtprotoPublisher::new(agent);
        let publication_uri = args.publication_uri
            .context("Need --publication-uri to publish dictionary")?;

        let uri = publisher.publish_dictionary(&lexicon.0, &title, &publication_uri).await
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        println!("Published dictionary document to ATProto: {}", uri);
    }

    let syntax_engine = SyntaxEngine::new(syntax);
    let sentence = syntax_engine.generate_sentence(&[
        "word1".to_string(), "word2".to_string(), "word3".to_string()
    ]);
    println!("Example sentence: {}", sentence);
    Ok(())
}

fn make_ortho_path(output: Option<&PathBuf>) -> PathBuf {
    match output {
        Some(path) => {
            let stem = path.file_stem().unwrap_or_default().to_str().unwrap_or("lexicon");
            let parent = path.parent().unwrap_or(std::path::Path::new("."));
            parent.join(format!("{}_orthography.json", stem))
        }
        None => PathBuf::from("lexicon_orthography.json"),
    }
}

fn save_lexicon(lexicon: &lexicon_structs::Lexicon, path: &PathBuf) -> EyreResult<()> {
    let json = serde_json::to_string_pretty(lexicon)?;
    std::fs::write(path, &json).context("Failed to write lexicon file")?;
    Ok(())
}
