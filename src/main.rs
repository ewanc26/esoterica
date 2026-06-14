pub mod archetypes;
pub mod phonology;
pub mod morphology;
pub mod lexicon;
pub mod sound_change;
pub mod syntax;
pub mod tui;
pub mod atproto;
#[cfg(test)]
mod tests;

use clap::Parser;
use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use bsky_sdk::BskyAgent;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_delimiter = ',')]
    phonology: Vec<String>,

    #[arg(short, long, value_delimiter = ',')]
    sound_change: Vec<String>,

    #[arg(short, long, value_delimiter = ',')]
    morphology: Vec<String>,

    #[arg(short, long)]
    syntax: String,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long)]
    interactive: bool,

    #[arg(long)]
    publish_title: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.interactive {
        enable_raw_mode()?;
        tui::run_tui()?;
        disable_raw_mode()?;
        return Ok(());
    }

    let phono_reg = archetypes::get_phonology_registry();
    let sc_reg = archetypes::get_sound_change_registry();
    let morph_reg = archetypes::get_morphology_registry();
    let syntax_reg = archetypes::get_syntax_registry();
    
    let phono = phono_reg.get(&args.phonology[0]).ok_or_else(|| anyhow!("Unknown phonology"))?.clone();
    let morph = morph_reg.get(&args.morphology[0]).ok_or_else(|| anyhow!("Unknown morphology"))?.clone();
    
    let mut merged_sc = Vec::new();
    for key in &args.sound_change {
        if let Some(sc) = sc_reg.get(key) {
            merged_sc.extend(sc.clone());
        }
    }

    let syntax = syntax_reg.get(&args.syntax).ok_or_else(|| anyhow!("Unknown syntax: {}", args.syntax))?.clone();

    let mut generator = lexicon::LexiconGenerator::new(phono, morph, merged_sc);
    let lexicon = generator.generate_core_lexicon(100).clone();
    
    if let Some(output) = args.output {
        generator.save_to_file(output.to_str().context("Invalid output path")?)?;
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
        let uri = publisher.publish_dictionary(&lexicon, &title).await?;
        println!("Published to ATProto: {}", uri);
    }

    let syntax_engine = syntax::SyntaxEngine::new(syntax);
    let sentence = syntax_engine.generate_sentence(&["word1".to_string(), "word2".to_string(), "word3".to_string()]);

    println!("Example sentence: {}", sentence);
    Ok(())
}
