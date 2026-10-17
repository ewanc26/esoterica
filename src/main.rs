//! Esoterica CLI entry point. Requires the `cli` feature (enabled by default).
//! Orchestrates the full generation pipeline: config loading, phonology, morphology,
//! sound change, semantic drift, orthography, ATProto publication.
//! To build for WASM: `wasm-pack build --features wasm --no-default-features`

use esoterica::args::Args;
use color_eyre::eyre::Result as EyreResult;
use color_eyre::eyre::{Context, ContextCompat};
use bsky_sdk::BskyAgent;
use std::path::PathBuf;
use esoterica::semantic_drift;
use esoterica::orthography;
use esoterica::lexicon_structs;

#[tokio::main]
async fn main() -> EyreResult<()> {
    color_eyre::install()?;

    let args = Args::parse_args();

    // ── Support ─────────────────────────────────────────────────────────────

    if args.support {
        println!("Support Esoterica development:");
        println!("  Ko-fi: https://ko-fi.com/ewancroft");
        println!("  GitHub Sponsors: https://github.com/sponsors/ewanc26");
        return Ok(());
    }

    // ── Interactive Mode ────────────────────────────────────────────────────

    if args.interactive {
        esoterica::tui::run_tui(args)?;
        return Ok(());
    }

    // ── Resolve Configuration ───────────────────────────────────────────────

    let registries = match &args.data_dir {
        Some(dir) => esoterica::pipeline::Registries::builtin()
            .with_user_dir(dir)
            .map_err(|e| color_eyre::eyre::eyre!("Invalid user data in {}:\n{}", dir.display(), e))?,
        None => esoterica::pipeline::Registries::builtin(),
    };

    let phono_key = args.phonology.first().context("Phonology is required")?;
    let morph_key = args.morphology.first().context("Morphology is required")?;
    let syntax_key = args.syntax.as_deref().context("Syntax is required")?;

    let mut spec = registries
        .spec(phono_key, morph_key, syntax_key, &args.sound_change)
        .map_err(|e| color_eyre::eyre::eyre!(e))?;
    spec.lexicon_size = args.lexicon_size.unwrap_or(100);
    spec.syllables = args.syllables.unwrap_or(2);

    // ── Generation ──────────────────────────────────────────────────────────

    let generated = spec.generate().map_err(|e| color_eyre::eyre::eyre!(e))?;
    let mut lexicon = generated.lexicon;

    // ── Semantic Drift ──────────────────────────────────────────────────────

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

    // ── Orthography ─────────────────────────────────────────────────────────

    if args.generate_orthography {
        let script = orthography::ScriptType::parse(args.script_type.as_deref().unwrap_or("alphabet"))
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        let style = orthography::GlyphStyle::parse(args.glyph_style.as_deref().unwrap_or("angular"))
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        let mut ortho = orthography::OrthographyEngine::new(script, style);
        let mapping = ortho.generate(spec.phonology.clone());
        println!("Generated orthography with {} glyphs", mapping.len());
        let ortho_path = make_ortho_path(args.output.as_ref());
        let json = serde_json::to_string_pretty(&mapping)
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        std::fs::write(&ortho_path, &json)
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        println!("Orthography saved to: {}", ortho_path.display());
    }

    // ── Output ──────────────────────────────────────────────────────────────

    if let Some(ref output) = args.output {
        save_lexicon(&lexicon, output)?;
        println!("Lexicon saved to: {:?}", output);
    }

    // ── ATProto Publication ─────────────────────────────────────────────────

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

    println!("Example sentence: {}", generated.example_sentence);
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Derive the orthography output path from the lexicon output path.
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

/// Write the lexicon as pretty-printed JSON.
fn save_lexicon(lexicon: &lexicon_structs::Lexicon, path: &PathBuf) -> EyreResult<()> {
    let json = serde_json::to_string_pretty(lexicon)?;
    std::fs::write(path, &json).context("Failed to write lexicon file")?;
    Ok(())
}
