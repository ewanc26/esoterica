pub mod archetypes;
pub mod phonology;
pub mod morphology;
pub mod lexicon;
pub mod sound_change;
pub mod syntax;
pub mod tui;
pub mod atproto;
pub mod args;
pub mod lexicon_structs;
// ...
use args::Args;
use color_eyre::eyre::Result as EyreResult;

use color_eyre::eyre::ContextCompat;
use bsky_sdk::BskyAgent;

#[tokio::main]
async fn main() -> EyreResult<()> {
    // Install error handler
    color_eyre::install()?;

    let args = Args::parse_args();
    
    if args.interactive {
        tui::run_tui(args)?;
        return Ok(());
    }

    let phono_reg = archetypes::get_phonology_registry();
    let sc_reg = archetypes::get_sound_change_registry();
    let morph_reg = archetypes::get_morphology_registry();
    let syntax_reg = archetypes::get_syntax_registry();
    
    let phono_key = args.phonology.first().context("Phonology is required")?;
    let morph_key = args.morphology.first().context("Morphology is required")?;
    let syntax_key = args.syntax.context("Syntax is required")?;
    
    let phono = phono_reg.get(phono_key).ok_or_else(|| color_eyre::eyre::eyre!("Unknown phonology"))?.clone();
    let morph = morph_reg.get(morph_key).ok_or_else(|| color_eyre::eyre::eyre!("Unknown morphology"))?.clone();
    
    let mut merged_sc = Vec::new();
    for key in &args.sound_change {
        if let Some(sc) = sc_reg.get(key) {
            merged_sc.extend(sc.clone());
        }
    }

    let syntax = syntax_reg.get(&syntax_key).ok_or_else(|| color_eyre::eyre::eyre!("Unknown syntax: {}", syntax_key))?.clone();

    let mut generator = lexicon::LexiconGenerator::new(phono, morph, merged_sc);
    let lexicon = generator.generate_core_lexicon(100).clone();
    
    if let Some(output) = args.output {
        generator.save_to_file(output.to_str().context("Invalid output path")?)
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        println!("Lexicon saved to: {:?}", output);
    }

    // Optional ATProto Publication
    if let (Some(title), Ok(handle), Ok(pass)) = (
        args.publish_title, 
        std::env::var("ATPROTO_HANDLE"), 
        std::env::var("ATPROTO_PASSWORD")
    ) {
        let agent = BskyAgent::builder().build().await?;
        agent.login(handle, pass).await?;
        
        let publisher = atproto::AtprotoPublisher::new(agent);
        let publication_uri = args.publication_uri.context("Need --publication-uri to publish dictionary")?;
        
        let uri = publisher.publish_dictionary(&lexicon.0, &title, &publication_uri).await
            .map_err(|e| color_eyre::eyre::eyre!(e))?;
        println!("Published dictionary document to ATProto: {}", uri);
    }

    let syntax_engine = syntax::SyntaxEngine::new(syntax);
    let sentence = syntax_engine.generate_sentence(&["word1".to_string(), "word2".to_string(), "word3".to_string()]);

    println!("Example sentence: {}", sentence);
    Ok(())
}
